

pub struct Device {
    regs: ::std::sync::Mutex<Regs>,
    interrupt_channel: ::std::sync::Mutex<::udi::imc::ChannelHandle>,
    irq_cbs: ::std::sync::Mutex< ::std::collections::VecDeque<::udi::meta_bridge::CbHandleEvent> >,
    dma: super::DmaPool,
}
impl Device {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {
            regs: Default::default(),
            interrupt_channel: Default::default(),
            irq_cbs: Default::default(),
            dma: Default::default(),
        })
    }
}
impl super::PioDevice for Device {
    fn set_interrupt_channel(&self, index: ::udi::ffi::udi_index_t, channel: ::udi::imc::ChannelHandle) {
        if index.0 != 0 {
            panic!("Bad IRQ index");
        }
        *self.interrupt_channel.lock().unwrap() = channel;
    }
    fn push_intr_cb(&self, index: ::udi::ffi::udi_index_t, cb: ::udi::meta_bridge::CbHandleEvent) {
        assert!(index.0 == 0, "Bad IRQ index");
        self.irq_cbs.lock().unwrap()
            .push_back(cb);
    }

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]) {
        assert!(regset_idx == 0);
        let regs = self.regs.lock().unwrap();
        fn encode(dst: &mut [u8], name: &str, val: &[u8]) {
            assert!(dst.len() == val.len(), "Accessing {} with wrong size: {} != {}", name, dst.len(), val.len());
            dst.copy_from_slice(val);
        }
        fn encode_u32(dst: &mut [u8], name: &str, val: u32) {
            encode(dst, name, &val.to_le_bytes())
        }
        fn encode_u16(dst: &mut [u8], name: &str, val: u16) {
            encode(dst, name, &val.to_le_bytes())
        }
        fn encode_u8(dst: &mut [u8], name: &str, val: u8) {
            encode(dst, name, &val.to_le_bytes())
        }
        match reg {
        // MAC address
        0..=5 => dst[0] = [0xAB,0xCD,0xEF,0x00,0x01,0x23][reg as usize],
        regs::TSD0 ..=regs::TSD3  => encode_u32(dst, "TSDn" , regs.tsd [ (reg >> 2) as usize & 3 ]),
        regs::TSAD0..=regs::TSAD3 => encode_u32(dst, "TSADn", regs.tsad[ (reg >> 2) as usize & 3 ]),
        regs::RBSTART => encode_u32(dst, "RBSTART", regs.rbstart),
        regs::CMD => encode_u8(dst, "CMD", regs.cmd),
        regs::CAPR => encode_u16(dst, "CAPR", regs.capr),
        regs::CBR => encode_u16(dst, "CBR", regs.cba),
        regs::IMR => encode_u16(dst, "IMR", regs.imr),
        regs::ISR => encode_u16(dst, "ISR", regs.isr),
        regs::TCR => encode_u32(dst, "TCR", regs.tcr),
        regs::RCR => encode_u32(dst, "RCR", regs.rcr),
        regs::CONFIG0 => encode_u8(dst, "CONFIG0", 0x00),
        regs::CONFIG1 => encode_u8(dst, "CONFIG1", regs.config1),
        _ => todo!("Handle reg {:#X}", reg),
        }
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        assert!(regset_idx == 0);
        assert!(src.len() == 1);
        let mut regs = self.regs.lock().unwrap();
        fn try_into_or<U>(src: &[u8], name: &'static str) -> U
        where
            for<'a> &'a [u8]: TryInto<U>,
            for<'a> <&'a [u8] as TryInto<U>>::Error: ::core::fmt::Debug,
        {
            match src.try_into()
            {
            Ok(v) => v,
            Err(e) => panic!("Accessing {} with wrong size: {:?}", name, e)
            }
        }
        fn read_u32(src: &[u8], name: &'static str) -> u32 {
            u32::from_le_bytes(try_into_or(src, name))
        }
        fn read_u16(src: &[u8], name: &'static str) -> u16 {
            u16::from_le_bytes(try_into_or(src, name))
        }
        fn read_u8(src: &[u8], name: &'static str) -> u8 {
            u8::from_le_bytes(try_into_or(src, name))
        }
        fn check_reserved<T>(slot: &mut T, new: T, mask_rsvd: T, mask_ro: T, name: &str) -> T
        where
            T: Copy,
            T: Eq,
            T: ::core::fmt::LowerHex,
            T: ::core::ops::BitAnd<Output=T>,
            T: ::core::ops::BitOr<Output=T>,
            T: ::core::ops::Not<Output=T>,
        {
            let prev = *slot;
            assert!(new & mask_rsvd == prev & mask_rsvd,
                "Reserved bits changed in {name} {:#x} != {:#x}", new & mask_rsvd, prev & mask_rsvd);
            *slot = (new & !mask_ro) | (prev & mask_ro);
            prev
        }
        match reg
        {
        regs::TSD0 ..=regs::TSD3  => regs.tsd [ (reg >> 2) as usize & 3 ] = read_u32(src, "TSDn" ),
        regs::TSAD0..=regs::TSAD3 => regs.tsad[ (reg >> 2) as usize & 3 ] = read_u32(src, "TSADn"),
        regs::RBSTART => regs.rbstart = read_u32(src, "RBSTART"),
        regs::CMD => {
            let new_val = read_u8(src, "CMD");
            let prev_val = check_reserved(&mut regs.cmd, new_val, 0xE2, 0x01, "CMD");
            // Reset requested
            if regs.cmd & 0x10 != 0 && prev_val & 0x10 == 0 {
                regs.reset();
            }
            },
        regs::CAPR => regs.capr = read_u16(src, "CAPR"),
        regs::CBR => panic!("Invalid write to CBA"),
        regs::IMR => regs.imr = read_u16(src, "IMR"),
        regs::ISR => regs.isr &= !read_u16(src, "ISR"),
        regs::TCR => {
            let new_val = read_u32(src, "TCR");
            let _ = check_reserved(&mut regs.tcr, new_val, 0x8038_F80E, 0x7C0C_0000, "TCR");
        }
        regs::RCR => {
            let new_val = read_u32(src, "RCR");
            let _ = check_reserved(&mut regs.rcr, new_val, 0xF0FC_0040, 0x0000_0000, "RCR");
        }
        regs::CONFIG0 => panic!("Invalid write to CONFIG0"),
        regs::CONFIG1 => regs.config1 = read_u8(src, "CONFIG1"),
        _ => todo!("Write reg {:#X}", reg),
        }
    }

    fn dma(&self) -> &super::DmaPool { &self.dma }
}

#[derive(Default)]
struct Regs
{
    tsd: [u32; 4],
    tsad: [u32; 4],
    rbstart: u32,

    cmd: u8,
    capr: u16,
    cba: u16,

    imr: u16,
    isr: u16,
    tcr: u32,
    rcr: u32,

    config1: u8,
}
impl Regs
{
    fn reset(&mut self) {
        self.cmd &= !0x10;
    }
}
mod regs {
    pub const TSD0  : u32 = 0x10;
    pub const TSD3  : u32 = 0x1C;
    /// Transmit Start Address(es)
    pub const TSAD0 : u32 = 0x20;
    pub const TSAD3 : u32 = 0x2C;
    /// Recieve Buffer Start (DWord)
    pub const RBSTART: u32 = 0x30;
    /// Command
    pub const CMD   : u32 = 0x37;
    /// Current address of packet read (u16)
    pub const CAPR  : u32 = 0x38;
    /// Current Buffer Address - Total byte count in RX buffer (u16)
    pub const CBR   : u32 = 0x3A;
    /// Interrupt Mask Register (u16)
    pub const IMR   : u32 = 0x3C;
    /// Interrupt Status Register (u16)
    pub const ISR   : u32 = 0x3E;
    /// Transmit Configuration Register (u32)
    pub const TCR   : u32 = 0x40;
    /// Recieve Configuration Register
    pub const RCR   : u32 = 0x44;
    /// EEPROM Configuration Resgister 0:
    /// - `2:0` = BS (Read-only)
    ///   - Bootrom Size: `8K << BS`
    pub const CONFIG0: u32 = 0x51;
    /// EEPROM Configuration Resgister 1:
    /// - 0 = PMEn
    ///   - Power management enable
    /// - 1 = VPD
    /// - 2 = IOMAP (RO)
    ///   - Registers are present in IO space
    /// - 3 = MEMMAP (RO)
    ///   - Registers are present in memory-mapped space
    /// - 4 = LWACT
    /// - 5 = DVRLOAD
    ///   - Indicates that the driver is loaded
    /// - 6 = LEDS0
    /// - 7 = LEDS1
    pub const CONFIG1: u32 = 0x52;
}