//! Buffers (`udi_buf_t`)
//! 
//! 
use ::core::future::Future;
use crate::ffi::udi_buf_t;

/// An owning buffer handle
#[repr(transparent)]
pub struct Handle(*mut udi_buf_t);
impl Default for Handle {
    fn default() -> Self {
        Self(::core::ptr::null_mut())
    }
}
impl Handle
{
    pub unsafe fn from_ref(raw: &*mut udi_buf_t) -> &Self {
        &*(raw as *const _ as *const Self)
    }
    pub unsafe fn from_mut(raw: &mut *mut udi_buf_t) -> &mut Self {
        &mut *(raw as *mut _ as *mut Self)
    }
    /// Construct a buffer handle from a raw pointer
    /// 
    /// UNSAFE: Caller must ensure either ownership or mutable access to `raw` (it can be null)
    pub unsafe fn from_raw(raw: *mut udi_buf_t) -> Self {
        Self(raw)
    }
    /// Steal a buffer pointer from some other location
    /// 
    /// UNSAFE: Caller must ensure either ownership or mutable access to `raw` (it can be null)
    pub unsafe fn take_raw(raw: &mut *mut udi_buf_t) -> Self {
        Self(::core::ptr::replace(raw, ::core::ptr::null_mut()))
    }
    /// Update this handle from a raw pointer
    /// 
    /// UNSAFE: Caller must ensure either ownership or mutable access to `raw` (it can be null)
    pub unsafe fn update_from_raw(&mut self, raw: *mut udi_buf_t) {
        self.0 = raw;
    }
    /// Obtain the raw pointer
    pub fn to_raw(&mut self) -> *mut udi_buf_t {
        self.0
    }
    /// Obtain the raw pointer (moving)
    pub fn into_raw(self) -> *mut udi_buf_t {
        let Handle(rv) = self;
        rv
    }

    /// Get the buffer length
    pub fn len(&self) -> usize {
        if self.0.is_null() {
            0
        }
        else {
            // SAFE: Non-null and owned
            unsafe { (*self.0).buf_size }
        }
    }

    /// Construct a new buffer using provided data
    pub fn new<'d>(
    	cb: crate::CbRef<crate::ffi::udi_cb_t>,
        init_data: &'d [u8],
        path_handle: crate::ffi::buf::udi_buf_path_t
    ) -> impl Future<Output=Self> + 'd {
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
    pub fn ensure_size(&mut self, cb: crate::CbRef<crate::ffi::udi_cb_t>, size: usize) -> impl Future<Output=()> + '_
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

    #[cfg(false_)]
    pub fn copy_from<'a>(
        &'a mut self,
        cb: crate::CbRef<crate::ffi::udi_cb_t>,
        src: &'a Handle,
        src_range: ::core::ops::Range<usize>,
        dst_range: ::core::ops::Range<usize>,
    )
    {
        let self_buf = self.0;
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            cb,
            move |gcb| unsafe {
                crate::ffi::buf::udi_buf_copy(
                    Self::callback, gcb,
                    src.0, src_range.start, src_range.end - src_range.start,
                    self_buf, dst_range.start, dst_range.end - dst_range.start,
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

    /// Write data into a buffer
    pub fn write<'a>(&'a mut self,
    	cb: crate::CbRef<crate::ffi::udi_cb_t>,
        dst: ::core::ops::Range<usize>,
        data: &'a [u8]
    ) -> impl Future<Output=()> + 'a {
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

    pub fn read(&self, ofs: usize, dst: &mut [u8]) {
        assert!(ofs <= self.len());
        assert!(ofs + dst.len() <= self.len());
        // SAFE: Correct FFI inputs
        unsafe {
            crate::ffi::buf::udi_buf_read(self.0, ofs, dst.len(), dst.as_mut_ptr() as _)
        }
    }

    /// Consume and free this buffer
    pub fn free(mut self)
    {
        if !self.0.is_null() {
            unsafe { crate::ffi::buf::udi_buf_free(self.0); }
            self.0 = ::core::ptr::null_mut();
        }
    }

    extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, handle: *mut udi_buf_t) {
        unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
    }
}