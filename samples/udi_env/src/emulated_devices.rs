//use ::udi::ffi::udi_index_t;

pub trait PioDevice
{
    fn poll(&self);

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]);
    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]);

    fn dma(&self) -> &DmaPool { panic!("DMA unsupported"); }
    fn irq(&self, index: u8) -> &Interrupt { let _ = index; panic!("Interrupts unsupported"); }
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

    fn read(&self, addr: u32, len: u32) -> Vec<u8> {
        let buffers = self.buffers.read().unwrap();
        let idx = match buffers.binary_search_by_key(&addr, |v| v.base)
            {
            Ok(i) => i,
            Err(i) => i,
            };
        assert!(idx < buffers.len());
        let buf = &buffers[idx];
        assert!(addr <= buf.base + buf.len);
        let ofs = addr - buf.base;
        let space = buf.len - ofs;
        assert!(len <= space);
        let mut rv = vec![0; len as usize];
        // SAFE: Pointer is valid for this length/offset
        unsafe {
            ::core::ptr::copy_nonoverlapping(buf.data_ptr.offset(ofs as isize) as _, rv.as_mut_ptr(), len as usize);
        }
        rv
    }
}

#[derive(Default)]
pub struct Interrupt {
    inner: ::std::sync::Mutex<InterruptInner>,
}
pub trait InterruptHandler {
    fn raise(&mut self);
}
#[derive(Default)]
pub struct InterruptInner
{
    handler: Option< Box<dyn InterruptHandler> >,
}
impl Interrupt
{
    pub fn bind(&self, handler: Box<dyn InterruptHandler>) {
        self.inner.lock().unwrap().handler = Some(handler);
    }
    pub fn unbind(&self) {
        self.inner.lock().unwrap().handler = None;
    }
    fn raise(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(ref mut h) = inner.handler {
            h.raise();
        }
    }
}

mod xt_serial;
mod rtl8029;
pub mod rtl8139;

pub use xt_serial::XTSerial;
pub use rtl8029::Rtl8029;