use ::udi::ffi::udi_index_t;

pub trait PioDevice
{
    fn set_interrupt_channel(&self, index: udi_index_t, channel: ::udi::imc::ChannelHandle);
    fn push_intr_cb(&self, index: udi_index_t, cb: ::udi::meta_bridge::CbHandleEvent);

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]);
    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]);

    fn dma_alloc(&self, size: usize) -> DmaHandle { let _ = size; panic!("DMA unsupported"); }
    fn dma_free(&self, handle: DmaHandle) { let _ = handle; }
}

pub struct DmaHandle
{
    base: u32,
    len: u32,
    data_ptr: *mut (),
}
impl DmaHandle {
    pub fn addr(&self) -> u32 { self.base }
    pub fn len(&self) -> u32 { self.len }
    pub fn write(&mut self, ofs: usize, src: &[u8]) {
        assert!(ofs + src.len() <= self.len as usize);
        // SAFE: Pointer is valid for this range, and aliasing won't matter
        unsafe {
            ::core::ptr::copy_nonoverlapping(src.as_ptr(), (self.data_ptr as *mut u8).offset(ofs as isize), src.len());
        }
    }
    pub fn read(&self, ofs: usize, dst: &mut [u8]) {
        assert!(ofs + dst.len() <= self.len as usize);
        // SAFE: Pointer is valid for this range, and aliasing won't matter
        unsafe {
            ::core::ptr::copy_nonoverlapping((self.data_ptr as *const u8).offset(ofs as isize), dst.as_mut_ptr(), dst.len());
        }
    }
}

mod xt_serial;
mod rtl8029;
pub mod rtl8139;

pub use xt_serial::XTSerial;
pub use rtl8029::Rtl8029;