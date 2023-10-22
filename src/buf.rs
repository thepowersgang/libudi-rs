use crate::ffi::udi_buf_t;

pub struct Handle(*mut udi_buf_t);
impl Handle
{
    /// UNSAFE: Caller must ensure either ownership or mutable access to `raw` (it can be null)
    pub unsafe fn from_raw(raw: *mut udi_buf_t) -> Self {
        Self(raw)
    }
    pub unsafe fn update_from_raw(&mut self, raw: *mut udi_buf_t) {
        self.0 = raw;
    }
    pub fn to_raw(&mut self) -> *mut udi_buf_t {
        self.0
    }

    pub fn len(&self) -> usize {
        if self.0.is_null() {
            0
        }
        else {
            // SAFE: Non-null and owned
            unsafe { (*self.0).buf_size }
        }
    }

    /// Construct a new buffer
    pub fn new<'d>(
    	cb: crate::CbRef<crate::ffi::udi_cb_t>,
        init_data: &'d [u8],
        path_handle: crate::ffi::buf::udi_buf_path_t
    ) -> impl ::core::future::Future<Output=Self> + 'd {
        crate::async_trickery::wait_task::<_, _,_,_>(
            cb,
            move |cb| unsafe {
                crate::ffi::buf::UDI_BUF_ALLOC(Self::callback, cb as *const _ as *mut _, init_data.as_ptr() as *const _, init_data.len(), path_handle)
                },
            |res| {
                let crate::WaitRes::Pointer(p) = res else { panic!(""); };
                // SAFE: Trusting the environemnt to have given us a valid pointer
                unsafe { Self::from_raw(p as *mut _) }
                }
            )
    }
    /// Ensure that this buffer has at least `size` bytes allocated within it
    /// 
    /// If the current size is smaller than `size`, then extra uninitialied bytes are added to the end
    pub fn ensure_size(&mut self, cb: crate::CbRef<crate::ffi::udi_cb_t>, size: usize) -> impl ::core::future::Future<Output=()> + '_
    {
        let self_buf = self.0;
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            cb,
            move |gcb| unsafe {
                let cur_size = if self_buf.is_null() { 0 } else { (*self_buf).buf_size };
                if cur_size >= size {
                    // Sufficient size
                    Self::callback(gcb, self_buf)
                }
                else {
                    let dst_off = cur_size;
                    let size = size - dst_off;
                    crate::ffi::buf::udi_buf_write(
                        Self::callback, gcb,
                        ::core::ptr::null(), size as _,
                        self_buf,
                        dst_off, 0,
                        crate::ffi::buf::UDI_NULL_PATH_BUF
                        );
                }
                },
            |res| {
                let crate::WaitRes::Pointer(p) = res else { panic!(""); };
                // SAFE: Trusting the environemnt to have given us a valid pointer
                unsafe { self.update_from_raw(p as *mut _); }
                }
            )
    }
    /// Truncate the buffer's length to the specified value
    /// 
    /// NOTE: This doesn't do any environment calls, just sets `buf_size`
    pub fn truncate(&mut self, len: usize)
    {
        // SAFE: Logically owned
        unsafe {
            assert!(len <= self.len());
            if len != self.len() {
                (*self.0).buf_size = len;
            }
        }
    }
    /// Write data into a buffer
    pub fn write<'a>(&'a mut self,
    	cb: crate::CbRef<crate::ffi::udi_cb_t>,
        dst: ::core::ops::Range<usize>,
        data: &'a [u8]
    ) -> impl ::core::future::Future<Output=()> + 'a {
        let self_buf = self.0;
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            cb,
            move |gcb| unsafe {
                crate::ffi::buf::udi_buf_write(
                    Self::callback, gcb,
                    data.as_ptr() as *const _, data.len(),
                    self_buf,
                    dst.start, dst.end - dst.start,
                    crate::ffi::buf::UDI_NULL_PATH_BUF
                    );
                },
            |res| {
                let crate::WaitRes::Pointer(p) = res else { panic!(""); };
                // SAFE: Trusting the environemnt to have given us a valid pointer
                unsafe { self.update_from_raw(p as *mut _); }
                }
            )
    }


    extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, handle: *mut udi_buf_t) {
        unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
    }
}