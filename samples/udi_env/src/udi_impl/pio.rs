use ::udi::ffi::pio::udi_pio_handle_t;
use ::udi::ffi::pio::udi_pio_map_call_t;
use ::udi::ffi::pio::udi_pio_trans_call_t;
use ::udi::ffi::pio::udi_pio_trans_t;
use ::udi::ffi::udi_buf_t;
use ::udi::ffi::udi_cb_t;
use ::udi::ffi::udi_size_t;
use ::udi::ffi::udi_index_t;
use ::udi::ffi::c_void;

struct PioTransReal {
    // TODO: Get a handle/reference to the device too
    instance: ::std::sync::Arc<crate::DriverInstance>,

    regset_idx: u32,
    base_offset: u32,
    length: u32,
    trans_list: Vec<udi_pio_trans_t>,
    data_translation: DataTranslation,
    data_ordering: DataOrdering,
    unaligned: bool,
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
    callback(gcb, rv)
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
    todo!("udi_pio_abort_sequence");
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
    callback(gcb, buf, status, retval);
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
        Self::from_bytes(&v.to_ne_bytes())
    }
    fn to_u32(&self) -> u32 {
        u32::from_ne_bytes(self.bytes[..4].try_into().unwrap())
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
        return self.bytes[final_byte] & 0x80 != 0;
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
impl ::core::ops::Add for RegVal {
    type Output = RegVal;
    fn add(self, rhs: Self) -> Self::Output {
        let mut rv = RegVal::default();
        let mut carry = false;
        for (d,(a,b)) in rv.bytes.iter_mut().zip( self.bytes.iter().zip(rhs.bytes.iter()) ) {
            let new_carry;
            let new_carry2;
            (*d,new_carry) = a.overflowing_add(*b);
            (*d,new_carry2) = d.overflowing_add(carry as u8);
            carry = new_carry | new_carry2;
            //(*d,carry) = a.carrying_add(*b, carry);
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
    fn write(&mut self, location_spec: u8, val: RegVal, size: u8) {
        let reg = &mut self.registers[ (location_spec & 7) as usize ];
        let ptr = match location_spec & 0x18
            {
            ::udi::ffi::pio::UDI_PIO_DIRECT => {
                *reg = val.masked(size);
                return
                },
            ::udi::ffi::pio::UDI_PIO_SCRATCH => {
                let addr = reg.to_u32();
                unsafe { (self.scratch as *mut u8).offset(addr as _) }
                }
            ::udi::ffi::pio::UDI_PIO_BUF => todo!("write buf"),
            ::udi::ffi::pio::UDI_PIO_MEM => {
                let addr = reg.to_u32();
                unsafe { (self.mem_ptr as *mut u8).offset(addr as _) }
                },
            _ => unreachable!(),
            };
        unsafe {
            let len = 1 << size;
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
                unsafe { (self.scratch as *const u8).offset(addr as _) }
                },
            ::udi::ffi::pio::UDI_PIO_BUF => todo!("read buf"),
            ::udi::ffi::pio::UDI_PIO_MEM => {
                let addr = reg.to_u32();
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
        val
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
    // PILE OF HACK:
    // - This is set up to semi-emulate a NE2000
    fn read(&self, reg: u32, size: u8) -> RegVal {
        println!("PIO Read {:#x}+{:#x},l={}", self.base_offset, reg, 1<<size);
        assert!(size <= 5);
        assert!(self.base_offset + reg + 1 << size <= self.length);
        let mut rv = RegVal::default();
        {
            let dst = &mut rv.bytes[..1 << size];
            self.dev.pio_read(self.regset_idx, self.base_offset + reg, dst);
            match self.data_translation {
            DataTranslation::NeverSwap => assert!(size == 0),
            DataTranslation::BigEndian => dst.reverse(),
            DataTranslation::LittleEndian => {},
            }
        }
        rv
    }
    fn write(&mut self, reg: u32, mut val: RegVal, size: u8) {
        assert!(size <= 5);
        println!("PIO Write {:#x}+{} = {:?}", reg, 1<<size, &val.bytes[..1<<size]);
        assert!(self.base_offset + reg + 1 << size <= self.length);

        {
            let src = &mut val.bytes[..1 << size];
            match self.data_translation {
            DataTranslation::NeverSwap => assert!(size == 0),
            DataTranslation::BigEndian => src.reverse(),
            DataTranslation::LittleEndian => {},
            }
            self.dev.pio_write(self.regset_idx, self.base_offset + reg, src);
        }
    }
}

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
    for _ in 0 .. 1000 {
        let op = &trans_list[ofs];
        println!("pio_trans_inner: OP 0x{:02x} 0x{:04x}", op.pio_op, op.operand);

        if op.pio_op < 0x80 {
            // Group A
            use ::udi::pio::vals::ops_group_a::*;
            match op.pio_op & 0xE0
            {
            IN => {
                let val = io_state.read(op.operand as u32, op.tran_size);
                state.write(op.pio_op & 0x1F, val, op.tran_size);
                },
            OUT => {
                let val = state.read(op.pio_op & 0x1F, op.tran_size);
                io_state.write(op.operand as u32, val, op.tran_size);
                },
            LOAD => todo!("LOAD"),
            STORE => todo!("STORE"),
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
                state.write(op.pio_op & 7, RegVal::from_u16(op.operand), op.tran_size);
                },
            CSKIP => {
                let val = state.read(op.pio_op & 7, op.tran_size);
                let cnd = match op.operand
                    {
                    0 /*Z*/ => val.is_zero(),
                    1 /*NZ*/ => !val.is_zero(),
                    2 /*Neg*/ => val.is_neg(op.tran_size),
                    3 /*NNeg*/ => !val.is_neg(op.tran_size),
                    _ => todo!("CSKIP"),
                    };
                if cnd {
                    ofs += 1;
                }
                },
            IN_IND   => todo!("IN_IND"),
            OUT_IND  => todo!("OUT_IND"),
            SHIFT_LEFT => todo!("SHIFT_LEFT"),
            SHIFT_RIGHT => todo!("SHIFT_RIGHT"),
            AND     => todo!("AND"),
            AND_IMM => {
                let val = state.read(op.pio_op & 7, op.tran_size);
                state.write(op.pio_op & 7, val & RegVal::from_u16(op.operand), op.tran_size);
                },
            OR      => todo!("OR"),
            OR_IMM  => todo!("OR_IMM"),
            XOR         => todo!("XOR"),
            ADD         => todo!("ADD"),
            ADD_IMM     => todo!("ADD_IMM"),
            SUB         => todo!("SUB"),
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
            LABEL => {},
            BRANCH => {
                ofs = find_label(trans_list, op.operand as _)?;
                // Explicitly skip the `ofs += 1`
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
                let count_reg = ((op.operand >> 13) & 7) as u8;

                let orig_mem_val = state.read(mem_ref & 7, 5);

                let count = state.read(count_reg, 1).to_u32();
                let mut reg = state.read(pio_reg, 1).to_u32();
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
            DELAY   => {},
            BARRIER => {},
            SYNC    => {},
            SYNC_OUT=> {},
            DEBUG   => {},
            // Unallocated
            0xF9..=0xFD => return Err(::udi::Error::from_status(::udi::ffi::UDI_STAT_NOT_UNDERSTOOD as _).unwrap_err()),
            END    => todo!("END"),
            END_IMM => return Ok(op.operand),
            }
        }
        ofs += 1;
    }
    todo!("Inifinite loop? Ran out of iterations")
}