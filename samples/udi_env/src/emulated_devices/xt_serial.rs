#[derive(Default)]
pub struct XTSerial {
    regs: ::std::sync::Mutex<Regs>,
    irq: super::Interrupt,
}
impl XTSerial {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Default::default())
    }
}
impl super::PioDevice for XTSerial {
    fn poll(&self) {
    }
    
    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]) {
        assert!(regset_idx == 0);
        assert!(dst.len() == 1, "Does this device support non-byte IO? ({})", dst.len());
        let regs = self.regs.lock().unwrap();
        dst[0] = match reg {
            0 => 0x99,
            4 => regs.lcr,
            5 => {  // Line Status Register
                let is_data_ready = false;
                let transmitter_holding_empty = true;
                let transmitter_empty = false;
                0
                    | 0x01 * (is_data_ready as u8)
                    | 0x20 * (transmitter_holding_empty as u8)
                    | 0x40 * (transmitter_empty as u8)
                },
            _ => todo!("pio_read({})", reg)
            };
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        assert!(regset_idx == 0);
        assert!(src.len() == 1, "Does this device support non-byte IO? ({})", src.len());
        let mut regs = self.regs.lock().unwrap();
        match reg {
        0 if regs.lcr & LCR_DLAB == 0 => {  // TX holding register
            println!("> TX {:#x} {:?}", src[0], src[0] as char);
            },
        0 if regs.lcr & LCR_DLAB != 0 => {  // Interrupt enable register
            regs.ier = src[0];
            },
        4 => regs.lcr = src[0],
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
    ier: u8,
    lcr: u8,
}
const LCR_DLAB: u8 = 0x80;