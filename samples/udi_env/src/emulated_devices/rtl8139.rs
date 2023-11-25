

pub struct Device {
    regs: ::std::sync::Mutex<Regs>,
    dma: super::DmaPool,
    irq: super::Interrupt,
}
impl Device {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {
            regs: Default::default(),
            dma: Default::default(),
            irq: Default::default(),
        })
    }
}
impl super::PioDevice for Device {

    fn poll(&self) {
        println!("rtl8139 - poll: {:#x}", self.regs.lock().unwrap().isr);
        if self.regs.lock().unwrap().isr != 0 {
            self.irq.raise();
        }
    }
    
    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]) {
        assert!(regset_idx == 0);
        let regs = self.regs.lock().unwrap();
        match reg {
        // MAC address
        0..=5 => dst[0] = [0xAB,0xCD,0xEF,0x00,0x01,0x23][reg as usize],
        regs::TSD0 ..=regs::TSD3  => u32::encode(dst, "TSDn" , regs.tsd [ (reg >> 2) as usize & 3 ]),
        regs::TSAD0..=regs::TSAD3 => u32::encode(dst, "TSADn", regs.tsad[ (reg >> 2) as usize & 3 ]),
        regs::RBSTART => u32::encode(dst, "RBSTART", regs.rbstart),
        regs::CMD => u8::encode(dst, "CMD", regs.cmd),
        regs::CAPR => u16::encode(dst, "CAPR", regs.capr),
        regs::CBR => u16::encode(dst, "CBR", regs.cba),
        regs::IMR => u16::encode(dst, "IMR", regs.imr),
        regs::ISR => u16::encode(dst, "ISR", regs.isr),
        regs::TCR => u32::encode(dst, "TCR", regs.tcr),
        regs::RCR => u32::encode(dst, "RCR", regs.rcr),
        regs::CONFIG0 => u8::encode(dst, "CONFIG0", 0x00),
        regs::CONFIG1 => u8::encode(dst, "CONFIG1", regs.config1),
        _ => todo!("Handle reg {:#X}", reg),
        }
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        assert!(regset_idx == 0);
        fn check_reserved<T>(slot: &mut T, src: &[u8], name: &'static str, mask_rsvd: T, mask_ro: T) -> T
        where
            T: RegVal,
            T: ::core::ops::BitAnd<Output=T>,
            T: ::core::ops::BitOr<Output=T>,
            T: ::core::ops::Not<Output=T>,
        {
            let new = T::decode(src, name);
            let prev = *slot;
            assert!(new & mask_rsvd == prev & mask_rsvd,
                "Reserved bits changed in {name} {:#x} != {:#x}", new & mask_rsvd, prev & mask_rsvd);
            *slot = (new & !mask_ro) | (prev & mask_ro);
            prev
        }

        let mut regs = self.regs.lock().unwrap();
        match reg
        {
        regs::TSD0 ..=regs::TSD3  => {
            let idx = (reg >> 2) as usize & 3;
            let slot = &mut regs.tsd [idx];
            let prev_val = check_reserved(slot, src, "TSDn", 0x0000_0000, 0xFF00_C000);
            if prev_val & 0x1FFF != *slot & 0x1FFF {
                // SIZE was written
                assert!(*slot & 0x1000 == 0, "Size changed, but OWN didn't clear");
                let size = *slot & 0xFFF;
                *slot &= !0xFFF;

                let tsad = regs.tsad[idx];
                let data = self.dma.read(tsad, size);
                println!("RTL8139 TX {} {:02x?}", idx, data);

                regs.isr |= 1 << 2; // TOK
            }
            },
        regs::TSAD0..=regs::TSAD3 => regs.tsad[ (reg >> 2) as usize & 3 ] = u32::decode(src, "TSADn"),
        regs::RBSTART => regs.rbstart = u32::decode(src, "RBSTART"),
        regs::CMD => {
            let prev_val = check_reserved(&mut regs.cmd, src, "CMD", 0xE2, 0x01);
            // Reset requested
            if regs.cmd & 0x10 != 0 && prev_val & 0x10 == 0 {
                regs.reset();
            }
            },
        regs::CAPR => regs.capr = u16::decode(src, "CAPR"),
        regs::CBR => panic!("Invalid write to CBR"),
        regs::IMR => regs.imr = u16::decode(src, "IMR"),
        regs::ISR => regs.isr &= !u16::decode(src, "ISR"),
        regs::TCR => {
            let _ = check_reserved(&mut regs.tcr, src, "TCR", 0x8038_F80E, 0x7C0C_0000);
        }
        regs::RCR => {
            let _ = check_reserved(&mut regs.rcr, src, "RCR", 0xF0FC_0040, 0x0000_0000);
        }
        regs::CONFIG0 => panic!("Invalid write to CONFIG0"),
        regs::CONFIG1 => regs.config1 = u8::decode(src, "CONFIG1"),
        _ => todo!("Write reg {:#X}", reg),
        }
    }

    fn dma(&self) -> &super::DmaPool { &self.dma }
    fn irq(&self, index: u8) -> &super::Interrupt {
        assert!(index == 0, "RTL8139 only has one interrupt - requested {}", index);
        &self.irq
    }
}

fn try_into_or<const N: usize>(src: &[u8], name: &'static str) -> [u8; N] {
    match src.try_into()
    {
    Ok(v) => v,
    Err(_) => panic!("Accessing {} with wrong size: {} != {}", name, src.len(), N),
    }
}
fn encode(dst: &mut [u8], name: &str, val: &[u8]) {
    assert!(dst.len() == val.len(), "Accessing {} with wrong size: {} != {}", name, dst.len(), val.len());
    dst.copy_from_slice(val);
}
trait RegVal: Copy + Eq + ::core::fmt::LowerHex {
    fn decode(src: &[u8], name: &'static str) -> Self;
    fn encode(dst: &mut [u8], name: &str, val: Self);
}
impl RegVal for u8 {
    fn decode(src: &[u8], name: &'static str) -> Self {
        Self::from_le_bytes(try_into_or(src, name))
    }
    fn encode(dst: &mut [u8], name: &str, val: Self) {
        encode(dst, name, &val.to_le_bytes())
    }
}
impl RegVal for u16 {
    fn decode(src: &[u8], name: &'static str) -> Self {
        Self::from_le_bytes(try_into_or(src, name))
    }
    fn encode(dst: &mut [u8], name: &str, val: Self) {
        encode(dst, name, &val.to_le_bytes())
    }
}
impl RegVal for u32 {
    fn decode(src: &[u8], name: &'static str) -> Self {
        Self::from_le_bytes(try_into_or(src, name))
    }
    fn encode(dst: &mut [u8], name: &str, val: Self) {
        encode(dst, name, &val.to_le_bytes())
    }
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
        self.cba = 0;
        self.capr = 0;
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