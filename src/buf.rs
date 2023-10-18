use crate::ffi::udi_buf_t;

pub struct Handle(*mut udi_buf_t);
impl Handle
{
    pub unsafe fn from_raw(raw: *mut udi_buf_t) -> Self {
        Self(raw)
    }
    pub unsafe fn update_from_raw(&mut self, raw: *mut udi_buf_t) {
        self.0 = raw;
    }
    pub fn to_raw(&mut self) -> *mut udi_buf_t {
        self.0
    }

    pub fn new<'d>(init_data: &'d [u8], path_handle: crate::ffi::buf::udi_buf_path_t) -> impl ::core::future::Future<Output=Self> + 'd {
        extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, handle: *mut udi_buf_t) {
            unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
        }
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            move |cb| unsafe {
                crate::ffi::buf::UDI_BUF_ALLOC(callback, cb as *const _ as *mut _, init_data.as_ptr() as *const _, init_data.len(), path_handle)
                },
            |res| {
                let crate::WaitRes::Pointer(p) = res else { panic!(""); };
                // SAFE: Trusting the environemnt to have given us a valid pointer
                unsafe { Self::from_raw(p as *mut _) }
                }
            )
    }
    pub fn write<'a>(&'a mut self, dst: ::core::ops::Range<usize>, data: &'a [u8]) -> impl ::core::future::Future<Output=()> + 'a {
        extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, handle: *mut udi_buf_t) {
            unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
        }
        let self_buf = self.0;
        crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
            move |cb| unsafe {
                crate::ffi::buf::udi_buf_write(
                    callback, cb as *const _ as *mut _,
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
}