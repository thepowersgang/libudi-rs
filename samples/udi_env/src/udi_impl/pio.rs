use ::udi::ffi::pio::udi_pio_handle_t;
use ::udi::ffi::pio::udi_pio_map_call_t;
use ::udi::ffi::pio::udi_pio_trans_call_t;
use ::udi::ffi::pio::udi_pio_trans_t;
use ::udi::ffi::udi_buf_t;
use ::udi::ffi::udi_cb_t;
use ::udi::ffi::udi_size_t;
use ::udi::ffi::udi_index_t;
use ::udi::ffi::c_void;

pub type Handle = Box<PioTransReal>;

pub struct PioTransReal {
    // TODO: Get a handle/reference to the device too
    instance: ::std::sync::Arc<crate::DriverInstance>,

    regset_idx: u32,
    base_offset: u32,
    length: u32,
    trans_list: Vec<udi_pio_trans_t>,
    data_translation: DataTranslation,
    data_ordering: DataOrdering,
    #[allow(dead_code)]
    unaligned: bool,
    #[allow(dead_code)]
    serialization_domain: udi_index_t
}
/// Translation applied to values sent to the device
#[derive(Copy,Clone)]
enum DataTranslation {
    /// Never swap, only byte IO accesses allowed
    NeverSwap,
    /// Use big-endian ordering
    BigEndian,
    /// Use little-endian ordering
    LittleEndian,
}
#[derive(Copy,Clone)]
enum DataOrdering {
    Paced(u32),
    StrictOrder,
    UnorderedOk,
    MergingOk,
    LoadCaching,
    StoreCaching,
}

#[no_mangle]
unsafe extern "C" fn udi_pio_map(
    callback: udi_pio_map_call_t,
    gcb: *mut udi_cb_t,
    regset_idx: u32, base_offset: u32, length: u32,
    trans_list: *const udi_pio_trans_t, list_length: u16,
    pio_attributes: u16, pace: u32, serialization_domain: udi_index_t
    )
{
    // TODO: This needs to communicate with the PCI bridge (or other bridges)
    let instance = crate::channels::get_driver_instance( &(*gcb).channel );

    let trans_list = ::core::slice::from_raw_parts(trans_list, list_length as usize);
    let data_translation = match pio_attributes & (7 << 5)
        {
        ::udi::ffi::pio::UDI_PIO_BIG_ENDIAN => DataTranslation::BigEndian,
        ::udi::ffi::pio::UDI_PIO_LITTLE_ENDIAN => DataTranslation::LittleEndian,
        0|::udi::ffi::pio::UDI_PIO_NEVERSWAP => DataTranslation::NeverSwap,
        v => {
            println!("Error: Multiple data translation attributes provided : {:#x}", v);
            DataTranslation::NeverSwap
        },
        };
    let data_ordering = if pace != 0 {
            if (pio_attributes & 0x1F) != ::udi::ffi::pio::UDI_PIO_STRICTORDER {
                // Not allowed, must have StrictOrder when pace is non-zero
            }
            DataOrdering::Paced(pace)
        } else if pio_attributes & ::udi::ffi::pio::UDI_PIO_STORECACHING_OK != 0 {
            DataOrdering::StoreCaching
        }
        else  if pio_attributes & ::udi::ffi::pio::UDI_PIO_LOADCACHING_OK != 0 {
            DataOrdering::LoadCaching
        }
        else if pio_attributes & ::udi::ffi::pio::UDI_PIO_MERGING_OK != 0 {
            DataOrdering::MergingOk
        }
        else if pio_attributes & ::udi::ffi::pio::UDI_PIO_UNORDERED_OK != 0 {
            DataOrdering::UnorderedOk
        }
        else {
            DataOrdering::StrictOrder
        };
    let unaligned = pio_attributes & ::udi::ffi::pio::UDI_PIO_UNALIGNED != 0;

    let rv = Box::new(PioTransReal {
        instance: instance.clone(),
        regset_idx,
        base_offset,
        length,
        trans_list: trans_list.to_vec(),
        data_translation,
        data_ordering,
        unaligned,
        serialization_domain,
    });
    let rv = Box::into_raw(rv) as udi_pio_handle_t;
    crate::async_call(gcb, move |gcb| callback(gcb, rv))
}
#[no_mangle]
unsafe extern "C" fn udi_pio_unmap(pio_handle: udi_pio_handle_t)
{
    drop(Box::from_raw(pio_handle as *mut PioTransReal));
}
#[no_mangle]
unsafe extern "C" fn udi_pio_atmic_sizes(_pio_handle: udi_pio_handle_t) -> u32
{
    4
}
// Register a handle as used to abort the device
#[no_mangle]
unsafe extern "C" fn udi_pio_abort_sequence(pio_handle: udi_pio_handle_t, scratch_requirement: udi_size_t)
{
    let handle = Box::from_raw(pio_handle as *mut PioTransReal);
    let instance = handle.instance.clone();
    *instance.pio_abort_sequence.lock().unwrap() = Some((handle, scratch_requirement));
}

#[no_mangle]
unsafe extern "C" fn udi_pio_trans(
    callback: udi_pio_trans_call_t, gcb: *mut udi_cb_t,
    pio_handle: udi_pio_handle_t,
    start_label: udi_index_t,
    mut buf: *mut udi_buf_t,
    mem_ptr: *mut c_void
    )
{
    let pio_handle = &*(pio_handle as *const PioTransReal);
    let mut state = PioMemState {
        buf: &mut buf,
        mem_ptr,
        scratch: (*gcb).scratch,
        registers: Default::default()
    };
    let mut io_state = PioDevState {
        dev: &**pio_handle.instance.device.get().expect("udi_pio_trans with no bound device"),
        regset_idx: pio_handle.regset_idx,
        base_offset: pio_handle.base_offset,
        length: pio_handle.length,
        data_translation: pio_handle.data_translation,
        _data_ordering: pio_handle.data_ordering,
    };
    let (status, retval) = match pio_trans_inner(&mut state, &mut io_state, &pio_handle.trans_list, start_label.0)
        {
        Err(e) => {
            (e.into_inner(), 0)
            },
        Ok(v) => (::udi::ffi::UDI_OK as _, v),
        };
    if status != 0 {
        println!("PIO Error {:?}", status);
    }
    crate::async_call(gcb, move |gcb| callback(gcb, buf, status, retval))
}

#[derive(Default,Copy,Clone)]
struct RegVal {
    bytes: [u8; 32],
}
impl RegVal {
    fn from_bytes(v: &[u8]) -> RegVal {
        let mut bytes = [0; 32];
        bytes[..v.len()].copy_from_slice(v);
        RegVal { bytes }
    }
    fn from_u8(v: u8) -> RegVal {
        Self::from_bytes(&[v])
    }
    fn from_u16(v: u16) -> RegVal {
        Self::from_bytes(&v.to_le_bytes())
    }
    fn from_u16_signed(v: u16) -> RegVal {
        let mut rv = Self::from_bytes(&v.to_le_bytes());
        if rv.bytes[1] & 0x80 != 0 {
            rv.bytes[2..32].fill(0xFF);
        }
        rv
    }
    fn to_u16(&self) -> u16 {
        u16::from_le_bytes(self.bytes[..2].try_into().unwrap())
    }
    fn to_u32(&self) -> u32 {
        u32::from_le_bytes(self.bytes[..4].try_into().unwrap())
    }
    fn masked(&self, size: u8) -> RegVal {
        assert!(size <= 5);
        let len = 1 << size;
        Self::from_bytes(&self.bytes[..len])
    }

    fn is_zero(&self) -> bool {
        self.bytes.iter().all(|v| *v == 0)
    }
    fn is_neg(&self, size: u8) -> bool {
        assert!(size <= 5);
        let final_byte = (1 << size) - 1;
        self.bytes[final_byte] & 0x80 != 0
    }
    fn display(&self, size: u8) -> impl ::core::fmt::Display + '_ {
        return Display(self, size);
        struct Display<'a>(&'a RegVal, u8);
        impl ::core::fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("0x")?;
                for byte in (0 .. 1 << self.1).rev() {
                    f.write_fmt(format_args!("{:02x}", self.0.bytes[byte]))?;
                    if byte != 0 && byte % 4 == 0 {
                        f.write_str("_")?;
                    }
                }
                Ok( () )
            }
        }
    }
}
impl ::core::ops::BitOr for RegVal {
    type Output = RegVal;
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut rv = RegVal::default();
        for (d,(a,b)) in rv.bytes.iter_mut().zip( self.bytes.iter().zip(rhs.bytes.iter()) ) {
            *d = *a | *b;
        }
        rv
    }
}
impl ::core::ops::BitAnd for RegVal {
    type Output = RegVal;
    fn bitand(self, rhs: Self) -> Self::Output {
        let mut rv = RegVal::default();
        for (d,(a,b)) in rv.bytes.iter_mut().zip( self.bytes.iter().zip(rhs.bytes.iter()) ) {
            *d = *a & *b;
        }
        rv
    }
}
impl ::core::ops::BitXor for RegVal {
    type Output = RegVal;
    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut rv = RegVal::default();
        for (d,(a,b)) in rv.bytes.iter_mut().zip( self.bytes.iter().zip(rhs.bytes.iter()) ) {
            *d = *a ^ *b;
        }
        rv
    }
}
fn carrying_add(a: u8, b: u8, carry: bool) -> (u8, bool) {
    let new_carry2;
    let (mut rv,new_carry) = a.overflowing_add(b);
    (rv,new_carry2) = rv.overflowing_add(carry as u8);
    (rv, new_carry | new_carry2)

    //a.carrying_add(b, carry)
}
impl ::core::ops::Add for RegVal {
    type Output = RegVal;
    fn add(self, rhs: Self) -> Self::Output {
        let mut rv = RegVal::default();
        let mut carry = false;
        for (d,(&a,&b)) in rv.bytes.iter_mut().zip( self.bytes.iter().zip(rhs.bytes.iter()) ) {
            (*d,carry) = carrying_add(a, b, carry);
            //(*d,carry) = a.carrying_add(*b, carry);
        }
        rv
    }
}
impl ::core::ops::Sub for RegVal {
    type Output = RegVal;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut rv = RegVal::default();
        let mut carry = true;
        for (d,(&a,&b)) in rv.bytes.iter_mut().zip( self.bytes.iter().zip(rhs.bytes.iter()) ) {
            (*d,carry) = carrying_add(a, !b, carry);
            //(*d,carry) = a.carrying_add(!b, carry);
        }
        rv
    }
}
impl ::core::ops::Shl<u8> for RegVal {
    type Output = RegVal;
    fn shl(self, rhs: u8) -> Self::Output {
        let mut rv = RegVal::default();
        let bytes = rhs / 8;
        let bits = rhs % 8;
        // Start at the LSB, since we're shifting left/up
        let src = self.bytes.iter().copied()
            .chain(std::iter::repeat(0))
            .skip(bytes as _)
            .take(self.bytes.len());
        let mut prev = 0;
        for (d, v) in rv.bytes.iter_mut().zip( src ) {
            if bits == 0 {
                *d = v;
            }
            else {
                *d = (v << bits) | (prev >> (8-bits));
            }
            prev = v;
        }
        rv
    }
}
impl ::core::ops::Shr<u8> for RegVal {
    type Output = RegVal;
    fn shr(self, rhs: u8) -> Self::Output {
        let mut rv = RegVal::default();
        let bytes = rhs / 8;
        let bits = rhs % 8;
        // Start at the MSB, since we're shifting right/down
        let src = self.bytes.iter().copied().rev()
            .chain(std::iter::repeat(0))
            .skip(bytes as _)
            .take(self.bytes.len());
        let mut prev = 0;
        for (d, v) in rv.bytes.iter_mut().rev().zip( src ) {
            if bits == 0 {
                *d = v;
            }
            else {
                *d = (v >> bits) | (prev >> (8-bits));
            }
            prev = v;
        }
        rv
    }
}
struct PioMemState<'a> {
    buf: &'a mut *mut udi_buf_t,
    scratch: *mut c_void,
    mem_ptr: *mut c_void,
    registers: [RegVal; 8],
}
impl PioMemState<'_> {
    fn little_to_native(val: &mut RegVal, size: u8) -> usize {
        let len = 1 << size;
        if cfg!(target_endian = "big") {
            val.bytes[..len].reverse();
        }
        len
    }
    fn write(&mut self, location_spec: u8, mut val: RegVal, size: u8) {
        let len = Self::little_to_native(&mut val, size);
        let reg = &mut self.registers[ (location_spec & 7) as usize ];
        let ptr = match location_spec & 0x18
            {
            ::udi::ffi::pio::UDI_PIO_DIRECT => {
                Self::little_to_native(&mut val, size); // Undo the endian flip
                *reg = val.masked(size);
                println!("> R{} = {}", location_spec, reg.display(size));
                return
                },
            ::udi::ffi::pio::UDI_PIO_SCRATCH => {
                let addr = reg.to_u32();
                // SAFE: We're trusting the caller to have provided a valid pointer
                unsafe { (self.scratch as *mut u8).offset(addr as _) }
                }
            ::udi::ffi::pio::UDI_PIO_BUF => {
                let addr = reg.to_u32() as usize;
                //println!("write(buf {:#x}, {})", addr, val.display(size));
                // SAFE: We're trusting the caller to have provided a valid pointer
                unsafe { crate::udi_impl::buf::write(self.buf, addr..addr+len, &val.bytes[..len]) };
                return ;
            },
            ::udi::ffi::pio::UDI_PIO_MEM => {
                let addr = reg.to_u32();
                // SAFE: We're trusting the caller to have provided a valid pointer
                unsafe { (self.mem_ptr as *mut u8).offset(addr as _) }
                },
            _ => unreachable!(),
            };
        unsafe {
            for i in 0 .. len {
                *ptr.offset(i as _) = val.bytes[i];
            }
        }
    }
    fn read(&self, location_spec: u8, size: u8) -> RegVal {
        let reg = &self.registers[ (location_spec & 7) as usize ];
        let ptr = match location_spec & 0x18
            {
            ::udi::ffi::pio::UDI_PIO_DIRECT => return reg.masked(size),
            ::udi::ffi::pio::UDI_PIO_SCRATCH => {
                let addr = reg.to_u32();
                // SAFE: We're trusting the caller to have provided a valid pointer
                unsafe { (self.scratch as *const u8).offset(addr as _) }
                },
            ::udi::ffi::pio::UDI_PIO_BUF => {
                let addr = reg.to_u32();
                let mut val = RegVal::default();
                // TODO: Error handling.
                // SAFE: We're trusting the caller to have provided a valid pointer
                unsafe { crate::udi_impl::buf::read(*self.buf, addr as usize, &mut val.bytes[..1 << size]) };
                Self::little_to_native(&mut val, size);
                return val;
            },
            ::udi::ffi::pio::UDI_PIO_MEM => {
                let addr = reg.to_u32();
                // SAFE: We're trusting the caller to have provided a valid pointer
                unsafe { (self.mem_ptr as *const u8).offset(addr as _) }
                },
            _ => unreachable!(),
            };
        let mut val = RegVal::default();
        unsafe {
            let len = 1 << size;
            for i in 0 .. len {
                val.bytes[i] = *ptr.offset(i as _);
            }
        }
        Self::little_to_native(&mut val, size);
        val
    }
}
struct MemRef(u8);
impl ::core::fmt::Display for MemRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 & 0x18
        {
        ::udi::ffi::pio::UDI_PIO_DIRECT => write!(f, "R{}", self.0 & 0x7),
        ::udi::ffi::pio::UDI_PIO_SCRATCH => write!(f, "[scratch R{}]", self.0 & 0x7),
        ::udi::ffi::pio::UDI_PIO_BUF => write!(f, "[buf R{}]", self.0 & 0x7),
        ::udi::ffi::pio::UDI_PIO_MEM => write!(f, "[mem R{}]", self.0 & 0x7),
        _ => unreachable!(),
        }
    }
}
struct Size(u8);
impl ::core::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0
        {
        0 => f.write_str("B"),
        1 => f.write_str("S"),
        2 => f.write_str("L"),
        _ => write!(f, "{}", 1 << self.0),
        }
    }
}

struct PioDevState<'a> {
    dev: &'a dyn crate::emulated_devices::PioDevice,
    regset_idx: u32,
    base_offset: u32,
    length: u32,

    data_translation: DataTranslation,
    _data_ordering: DataOrdering,
}
impl PioDevState<'_> {
    fn read(&self, reg: u32, size: u8) -> RegVal {
        assert!(size <= 5);
        assert!(reg + (1 << size) <= self.length);
        let mut rv = RegVal::default();
        {
            let dst = &mut rv.bytes[..1 << size];
            self.dev.pio_read(self.regset_idx, self.base_offset + reg, dst);
            match self.data_translation {
            DataTranslation::NeverSwap => assert!(size == 0, "NeverSwap with non-byte IO access"),
            DataTranslation::BigEndian => dst.reverse(),
            DataTranslation::LittleEndian => {},
            }
        }
        println!("PIO Read {:#x}+{:#x},l={} - {}", self.base_offset, reg, 1<<size, rv.display(size));
        rv
    }
    fn write(&mut self, reg: u32, mut val: RegVal, size: u8) {
        assert!(size <= 5);
        println!("PIO Write {:#x}+{} = {}", reg, 1<<size, val.display(size));
        assert!(reg + (1 << size) <= self.length);

        {
            let src = &mut val.bytes[..1 << size];
            match self.data_translation {
            DataTranslation::NeverSwap => assert!(size == 0, "NeverSwap with non-byte IO access"),
            DataTranslation::BigEndian => src.reverse(),
            DataTranslation::LittleEndian => {},
            }
            self.dev.pio_write(self.regset_idx, self.base_offset + reg, src);
        }
    }
}

const MAX_OPERATIONS: usize = 1000;

fn pio_trans_inner(state: &mut PioMemState, io_state: &mut PioDevState, trans_list: &[udi_pio_trans_t], start_label: u8) -> Result<u16,::udi::Error>
{
    fn find_label(trans_list: &[udi_pio_trans_t], label: u8) -> Result<usize,::udi::Error> {
        if label == 0 {
            return Ok(0);
        }
        match trans_list.iter()
            .position(move |op| op.pio_op == ::udi::pio::vals::ops_group_c::LABEL && op.operand == label as _)
        {
        Some(v) => Ok(v),
        None => {
            eprintln!("pio_trans_inner: Unable to find label #{}", label);
            Err(::udi::Error::from_status(::udi::ffi::UDI_STAT_NOT_UNDERSTOOD as _).unwrap_err())
            },
        }
    }
    let mut ofs = find_label(trans_list, start_label)?;
    println!("pio_trans_inner: Start at +{} of {}", ofs, trans_list.len());
    for _ in 0 .. MAX_OPERATIONS {
        let op = &trans_list[ofs];
        let s = Size(op.tran_size);
        print!("pio_trans_inner: +{} OP 0x{:02x} 0x{:04x}: ", ofs, op.pio_op, op.operand);

        if op.pio_op < 0x80 {
            // Group A
            use ::udi::pio::vals::ops_group_a::*;
            match op.pio_op & 0xE0
            {
            IN => {
                println!("IN.{s} {}, #{:#x}", MemRef(op.pio_op & 0x1F), op.operand);
                let val = io_state.read(op.operand as u32, op.tran_size);
                state.write(op.pio_op & 0x1F, val, op.tran_size);
                },
            OUT => {
                println!("OUT.{s} #{:#x}, {}", op.operand, MemRef(op.pio_op & 0x1F));
                let val = state.read(op.pio_op & 0x1F, op.tran_size);
                io_state.write(op.operand as u32, val, op.tran_size);
                },
            LOAD => {
                println!("LOAD.{s} R{}, {}", op.operand & 7, MemRef(op.pio_op & 0x1F));
                let val = state.read(op.pio_op & 0x1F, op.tran_size);
                state.write(op.operand as u8 & 7, val, op.tran_size);
                },
            STORE => {
                println!("STORE.{s} {}, R{}", MemRef(op.pio_op & 0x1F), op.operand & 7);
                let val = state.read(op.operand as u8 & 7, op.tran_size);
                state.write(op.pio_op & 0x1F, val, op.tran_size);
                },
            |0x01..=0x1F
            |0x21..=0x3F
            |0x41..=0x5F
            |0x61..=0x7F
                => unreachable!(),
            0x00 ..= 0xFF => unreachable!(),
            }
        }
        else if op.pio_op < 0xF0 {
            // Group B
            use ::udi::pio::vals::ops_group_b::*;
            match op.pio_op & 0xF8
            {
            0x00 ..= 0x7F => unreachable!(),
            LOAD_IMM => {
                println!("LOAD_IMM.{s} R{} {:#x}", op.pio_op&7, op.operand);
                state.write(op.pio_op & 7, RegVal::from_u16(op.operand), op.tran_size);
                },
            CSKIP => {
                let val = state.read(op.pio_op & 7, op.tran_size);
                let (msg,cnd) = match op.operand
                    {
                    0 => ("Z"   ,val.is_zero()),
                    1 => ("NZ"  , !val.is_zero()),
                    2 => ("Neg ", val.is_neg(op.tran_size)),
                    3 => ("NNeg", !val.is_neg(op.tran_size)),
                    _ => panic!("Unknwon CSKIP operand {:#x}", op.operand),
                    };
                println!("CSKIP.{s} R{} {}", op.pio_op&7, msg);
                if cnd {
                    ofs += 1;
                }
                },
            IN_IND => {
                println!("IN_IND.{s} R{}, R{}", op.pio_op & 7, op.operand & 7);
                let reg = state.read(op.operand as u8 & 7, op.tran_size).to_u32();
                let val = io_state.read(reg, op.tran_size);
                state.write(op.pio_op & 7, val, op.tran_size);
            }
            OUT_IND => {
                println!("OUT_IND.{s} R{}, R{}", op.operand & 7, op.pio_op & 7);
                let val = state.read(op.pio_op & 7, op.tran_size);
                let reg = state.read(op.operand as u8 & 7, op.tran_size).to_u32();
                io_state.write(reg, val, op.tran_size)
            },
            SHIFT_LEFT => {
                println!("SHIFT_LEFT.{s} R{}, {}", op.pio_op & 7, op.operand);
                let val = state.read(op.pio_op & 7, op.tran_size);
                assert!(op.operand <= 8 << op.tran_size);
                state.write(op.pio_op & 7, val << op.operand as u8, op.tran_size);
            },
            SHIFT_RIGHT => {
                println!("SHIFT_RIGHT.{s} R{}, {}", op.pio_op & 7, op.operand);
                let val = state.read(op.pio_op & 7, op.tran_size);
                assert!(op.operand <= 8 << op.tran_size);
                state.write(op.pio_op & 7, val >> op.operand as u8, op.tran_size);
            },
            AND => {
                println!("AND.{s} R{}, R{}", op.pio_op & 7, op.operand & 7);
                let val_l = state.read(op.pio_op & 7, op.tran_size);
                let val_r = state.read(op.operand as u8 & 7, op.tran_size);
                state.write(op.pio_op & 7, val_l & val_r, op.tran_size);
                },
            AND_IMM => {
                println!("AND_IMM.{s} R{}, {:#x}", op.pio_op&7, op.operand);
                let val = state.read(op.pio_op & 7, op.tran_size);
                state.write(op.pio_op & 7, val & RegVal::from_u16(op.operand), op.tran_size);
                },
            OR => {
                println!("OR.{s} R{}, R{}", op.pio_op & 7, op.operand & 7);
                let val_l = state.read(op.pio_op & 7, op.tran_size);
                let val_r = state.read(op.operand as u8 & 7, op.tran_size);
                state.write(op.pio_op & 7, val_l | val_r, op.tran_size);
                },
            OR_IMM  => {
                println!("OR_IMM.{s} R{}, {:#x}", op.pio_op&7, op.operand);
                let val = state.read(op.pio_op & 7, op.tran_size);
                state.write(op.pio_op & 7, val | RegVal::from_u16(op.operand), op.tran_size);
                },
            XOR => {
                println!("ADD.{s} R{}, R{}", op.pio_op & 7, op.operand & 7);
                let val_l = state.read(op.pio_op & 7, op.tran_size);
                let val_r = state.read(op.operand as u8 & 7, op.tran_size);
                state.write(op.pio_op & 7, val_l ^ val_r, op.tran_size);
                },
            ADD => {
                println!("ADD.{s} R{}, R{}", op.pio_op & 7, op.operand & 7);
                let val_l = state.read(op.pio_op & 7, op.tran_size);
                let val_r = state.read(op.operand as u8 & 7, op.tran_size);
                state.write(op.pio_op & 7, val_l + val_r, op.tran_size);
                },
            ADD_IMM => {
                println!("ADD_IMM.{s} R{}, {:#x}", op.pio_op & 7, op.operand);
                let val = state.read(op.pio_op & 7, op.tran_size);
                state.write(op.pio_op & 7, val + RegVal::from_u16_signed(op.operand), op.tran_size);
                },
            SUB => {
                println!("SUB.{s} R{}, R{}", op.pio_op & 7, op.operand & 7);
                let val_l = state.read(op.pio_op & 7, op.tran_size);
                let val_r = state.read(op.operand as u8 & 7, op.tran_size);
                state.write(op.pio_op & 7, val_l - val_r, op.tran_size);
                },
            0xF0 ..= 0xFF => unreachable!(),
            |0x81..=0x87|0x89..=0x8F
            |0x91..=0x97|0x99..=0x9F
            |0xA1..=0xA7|0xA9..=0xAF
            |0xB1..=0xB7|0xB9..=0xBF
            |0xC1..=0xC7|0xC9..=0xCF
            |0xD1..=0xD7|0xD9..=0xDF
            |0xE1..=0xE7|0xE9..=0xEF
                => unreachable!(),
            }
        }
        else {
            use ::udi::pio::vals::ops_group_c::*;
            match op.pio_op
            {
            0x00 ..= 0xEF => unreachable!(),
            // Group C
            LABEL => {
                println!("LABEL {}", op.operand);
                },
            BRANCH => {
                println!("BRANCH {}", op.operand);
                ofs = find_label(trans_list, op.operand as _)?;
                // Explicitly skip the `ofs += 1`, so this can branch to label 0 (which doesn't have a label instruction)
                continue ;
                },
            REP_IN_IND|REP_OUT_IND => {
                fn get_stride(v: u16) -> u8 {
                    let stride = v & 3;
                    if stride == 0 { 0 } else { 1 << (stride-1) }
                }
                let mem_ref = (op.operand & 0x1F) as u8;
                let mem_stride = get_stride(op.operand >> 5);
                let pio_reg = ((op.operand >> 7) & 7) as u8;
                let pio_stride = get_stride(op.operand >> 10);
                let count_reg = ((op.operand >> 12) & 7) as u8;
                println!("{}.{s} {}+{} R{}+{} *R{}", if op.pio_op == REP_OUT_IND { "REP_OUT_IND" } else { "REP_IN_IND" }, MemRef(mem_ref), mem_stride, pio_reg, pio_stride, count_reg);

                let orig_mem_val = state.read(mem_ref & 7, 5);

                let count = state.read(count_reg, 1).to_u32();
                let mut reg = state.read(pio_reg, 1).to_u32();
                println!("> [{}++{}] {}+{} *{}", orig_mem_val.to_u32(), mem_stride, reg, pio_stride, count);
                for _ in 0..count {
                    if op.pio_op == REP_OUT_IND {
                        io_state.write(reg, state.read(mem_ref, op.tran_size), op.tran_size);
                    }
                    else {
                        state.write(mem_ref, io_state.read(reg, op.tran_size), op.tran_size);
                    }
                    if mem_ref >= 0x8 {
                        let v = state.read(mem_ref & 7, 5);
                        state.write(mem_ref & 7, v + RegVal::from_u8(mem_stride), 5);
                    }
                    reg += pio_stride as u32;
                }

                state.write(mem_ref & 7, orig_mem_val, 5);
                },
            DELAY   => println!("DELAY"),
            BARRIER => println!("BARRIER"),
            SYNC    => println!("SYNC"),
            SYNC_OUT=> println!("SYNC_OUT"),
            DEBUG   => println!("DEBUG"),
            // Unallocated
            0xF9..=0xFD => {
                println!("unallocated - errror");
                return Err(::udi::Error::from_status(::udi::ffi::UDI_STAT_NOT_UNDERSTOOD as _).unwrap_err())
                },
            END    => {
                println!("END.{s} R{}", op.operand);
                return Ok(state.read(op.operand as u8 & 7, op.tran_size).to_u16());
                },
            END_IMM => {
                println!("END_IMM.{s} {:#x}", op.operand);
                return Ok(op.operand)
                },
            }
        }
        ofs += 1;
    }
    todo!("Inifinite loop? Ran out of iterations")
}