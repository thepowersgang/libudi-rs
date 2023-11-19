use ::udi::ffi::udi_index_t;

pub trait PioDevice
{
    fn set_interrupt_channel(&self, index: udi_index_t, channel: ::udi::imc::ChannelHandle);
    fn push_intr_cb(&self, index: udi_index_t, cb: ::udi::meta_bridge::CbHandleEvent);

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]);
    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]);

    fn dma(&self) -> &DmaPool { panic!("DMA unsupported"); }
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

    fn new_pair(base: u32, len: usize) -> (DmaHandle, DmaHandle) {
        let data_ptr = unsafe { ::libc::malloc(len) as *mut () };
        (
            DmaHandle { base, len: len as u32, data_ptr },
            DmaHandle { base, len: len as u32, data_ptr },
        )
    }
}

#[derive(Default)]
pub struct DmaPool
{
    buffers: ::std::sync::RwLock<Vec<DmaHandle>>,
}
impl DmaPool {
    pub fn allocate(&self, size: usize) -> Option<DmaHandle> {
        let mut buffers = self.buffers.write().unwrap();
        let mut cur = 0;
        for (i,ent) in buffers.iter().enumerate() {
            assert!(cur <= ent.base);
            let space = ent.base - cur;
            if space as usize >= size {
                // Insert here

                let (rv, ent) = DmaHandle::new_pair(cur, size);
                buffers.insert(i, ent);
                return Some(rv);
            }
            cur = ent.base + ent.len;
        }
        let (rv, ent) = DmaHandle::new_pair(cur, size);
        buffers.push(ent);
        Some(rv)
    }
    pub fn free(&self, handle: DmaHandle) {
        let mut buffers = self.buffers.write().unwrap();
        let opt_pos = match buffers.binary_search_by_key(&handle.base, |v| v.base)
            {
            Ok(i) if buffers[i].len == handle.len && buffers[i].data_ptr == handle.data_ptr => Some(i),
            Ok(_) => None,
            Err(_) => None,
            };
        match opt_pos {
        None => panic!("Failed to find matching element for `{:#x}+{:#x} {:p}`", handle.base, handle.len, handle.data_ptr),
        Some(i) => { buffers.remove(i); },
        }
    }
}

mod xt_serial;
mod rtl8029;
pub mod rtl8139;

pub use xt_serial::XTSerial;
pub use rtl8029::Rtl8029;