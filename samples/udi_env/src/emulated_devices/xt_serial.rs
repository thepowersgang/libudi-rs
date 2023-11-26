#[derive(Default)]
pub struct XTSerial {
    regs: ::std::sync::Mutex<Regs>,
    irq: super::Interrupt,
}
impl XTSerial {
    pub fn new_boxed() -> Box<Self> {
        let mut rv = Box::new(Self::default());
        rv.regs.get_mut().unwrap().lsr = vals::LSR_TEMT|vals::LSR_THRE;
        rv
    }
}
impl super::PioDevice for XTSerial {
    fn poll(&self, actions: &mut super::Actions) {
        let is_int = {
            let mut regs = self.regs.lock().unwrap();
            while let Some(bytes) = actions.pull("uart_rx") {
                for b in bytes {
                    regs.hacky_fifo.push_back(b);
                    // Mark the interrupt
                    regs.isr |= vals::IER_ERBI;
                    // Set DataReady
                    regs.lsr |= vals::LSR_DR;
                }
            }
            regs.isr & regs.ier != 0
        };
        if is_int {
            self.irq.raise()
        }
    }
    
    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]) {
        assert!(regset_idx == 0);
        assert!(dst.len() == 1, "Does this device support non-byte IO? ({})", dst.len());
        let mut regs = self.regs.lock().unwrap();
        dst[0] = match reg {
            0..=1 if regs.lcr & vals::LCR_DLAB != 0 => {
                regs.divisor.to_le_bytes()[reg as usize]
                },
            0 => {
                regs.isr &= !vals::IER_ERBI;
                let v = match regs.hacky_fifo.pop_front() {
                    Some(v) => v,
                    None => 0xFF
                    };
                if regs.hacky_fifo.is_empty() {
                    regs.lsr &= !vals::LSR_DR;
                }
                v
                },
            1 => regs.ier,
            2 => {
                if regs.iir & 0xF == 0x2 {
                    regs.isr &= !vals::IER_ETBEI;
                }
                regs.iir
                },
            3 => regs.lcr,
            4 => regs.mcr,  //regs.lcr,
            5 => {
                regs.isr &= !vals::IER_ELSI;  // Clear a pending interrupt
                regs.lsr
                },
            6 => {
                regs.isr &= !vals::IER_EDSSI;  // Clear a pending interrupt
                regs.msr
                },
            7 => regs.scratch,
            _ => todo!("pio_read({})", reg)
            };
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        assert!(regset_idx == 0);
        assert!(src.len() == 1, "Does this device support non-byte IO? ({})", src.len());
        let mut regs = self.regs.lock().unwrap();
        match reg {
        0..=1 if regs.lcr & vals::LCR_DLAB != 0 => {
            let mut v = regs.divisor.to_le_bytes();
            v[reg as usize] = src[0];
            regs.divisor = u16::from_le_bytes(v);
            },
        0 => {  // TX holding register
            regs.isr &= !vals::IER_ETBEI;
            println!("> TX {:#x} {:?}", src[0], src[0] as char);
            },
        1 => { check_reserved(&mut regs.ier, src, "IER", 0xF0, 0x00); },
        2 => { check_reserved(&mut regs.fcr, src, "FCR", 0x30, 0x00); },
        3 => { check_reserved(&mut regs.lcr, src, "LCR", 0x00, 0x00); },
        4 => { check_reserved(&mut regs.mcr, src, "MCR", 0xC0, 0x00); },
        5 => { check_reserved(&mut regs.lsr, src, "LSR", 0x00, 0xFF); },
        6 => { check_reserved(&mut regs.msr, src, "MSR", 0x00, 0xFF); },
        7 => regs.scratch = src[0],
        _ => todo!("pio_write({reg})"),
        }
    }

    fn irq(&self, index: u8) -> &super::Interrupt {
        assert!(index == 0);
        &self.irq
    }
}

#[derive(Default)]
struct Regs {
    hacky_fifo: ::std::collections::VecDeque<u8>,

    divisor: u16,

    // Pending interrupts
    isr: u8,

    /// Interrupt Enable Register
    ier: u8,
    /// Interrupt Identification Register
    iir: u8,
    /// FIFO Control Register
    fcr: u8,
    /// Line Control Register
    lcr: u8,
    /// Modem Control Register
    mcr: u8,
    /// Line Status Register
    lsr: u8,
    /// Modem Status Register
    msr: u8,
    /// Scratch Register
    scratch: u8,
}

#[allow(dead_code)]
mod vals {

    /// Enable Received Data Available Interrupt
    pub const IER_ERBI: u8 = 0x01;
    /// Enable Transmitter Holding Register Empty Interrupt
    pub const IER_ETBEI: u8 = 0x02;
    /// Emable Line Status Interrupt
    pub const IER_ELSI: u8 = 0x04;
    /// Emable Modem Status Interrupt
    pub const IER_EDSSI: u8 = 0x08;

    pub const LCR_DLAB: u8 = 0x80;

    /// Line Status Register: Data Ready
    pub const LSR_DR: u8 = 0x01;
    /// Line Status Register: Overflow Error
    pub const LSR_OE: u8 = 0x02;
    /// Line Status Register: Pairity Error
    pub const LSR_PE: u8 = 0x04;
    /// Line Status Register: Framing Error
    pub const LSR_FE: u8 = 0x08;
    /// Line Status Register: Break interrupt
    pub const LSR_BR: u8 = 0x10;
    /// Line Status Register: Transmitter Holding Register (?Empty)
    pub const LSR_THRE: u8 = 0x20;
    /// Line Status Register: Transmitter Empty
    pub const LSR_TEMT: u8 = 0x40;
    /// Line Status Register: Error in RCVR FIFO
    pub const LSR_ERR: u8 = 0x80;
}

fn check_reserved(slot: &mut u8, src: &[u8], name: &'static str, mask_rsvd: u8, mask_ro: u8) -> u8
{
    let new = src[0];
    let prev = *slot;
    assert!(new & mask_rsvd == prev & mask_rsvd,
        "Reserved bits changed in {name} {:#x} != {:#x}", new & mask_rsvd, prev & mask_rsvd);
    *slot = (new & !mask_ro) | (prev & mask_ro);
    prev
}