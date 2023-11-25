//use ::udi::ffi::udi_index_t;

pub trait PioDevice
{
    fn poll(&self, actions: &mut Actions);

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]);
    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]);

    fn dma(&self) -> &DmaPool { panic!("DMA unsupported"); }
    fn irq(&self, index: u8) -> &Interrupt { let _ = index; panic!("Interrupts unsupported"); }
}

pub use self::helpers::Actions;
pub use self::helpers::{DmaPool,DmaHandle};
pub use self::helpers::{Interrupt, InterruptHandler};

mod helpers;
mod xt_serial;
mod rtl8029;
pub mod rtl8139;

pub use xt_serial::XTSerial;
pub use rtl8029::Rtl8029;