use ::udi::ffi::udi_index_t;

pub trait PioDevice
{
    fn set_interrupt_channel(&self, index: udi_index_t, channel: ::udi::imc::ChannelHandle);
    fn push_intr_cb(&self, index: udi_index_t, cb: ::udi::meta_bridge::CbHandleEvent);

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]);
    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]);
}

mod xt_serial;
mod rtl8029;
pub use xt_serial::XTSerial;
pub use rtl8029::Rtl8029;