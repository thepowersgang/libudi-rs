//! UDI memory management 

// TODO: Add a `Vec` type too, can't use `alloc` as it won't support the async allocation callbacks
// - `Vec` could have to `push` methods, one that is fallible the other being async

/// Handle to an allocation that hasn't yet been initialised
pub struct ProtoHandle<T: ?Sized>(*mut T);
impl<T> ProtoHandle<T> {
    /// Initialise
    pub fn init(self, v: impl FnOnce()->T) -> Handle<T> {
        unsafe { ::core::ptr::write(self.0, v()) }
        let rv = Handle(self.0);
        ::core::mem::forget(self);
        rv
    }
}
impl<T> ProtoHandle<[T]> {
    /// Initialise
    pub fn init(self, mut v: impl FnMut(usize)->T) -> Handle<[T]> {
        unsafe {
            for i in 0 .. (*self.0).len() {
                ::core::ptr::write((self.0 as *mut T).offset(i as isize), v(i))
            }
        }
        let rv = Handle(self.0);
        ::core::mem::forget(self);
        rv
    }
}
impl<T: ?Sized> Drop for ProtoHandle<T> {
    fn drop(&mut self) {
        unsafe { ::udi_sys::mem::udi_mem_free(self.0 as _) }
    }
}

/// An allocated and initialised block of memory
pub struct Handle<T: ?Sized>(*mut T);
impl<T: ?Sized> Drop for Handle<T> {
    fn drop(&mut self) {
        unsafe { ::udi_sys::mem::udi_mem_free(self.0 as _) }
    }
}
impl<T: ?Sized> ::core::ops::Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl<T: ?Sized> ::core::ops::DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

unsafe extern "C" fn alloc_callback(gcb: *mut ::udi_sys::udi_cb_t, mut new_mem: *mut ::udi_sys::c_void) {
    if new_mem.is_null() {
        new_mem = 0x1000 as *mut ::udi_sys::c_void;
    }
    crate::async_trickery::signal_waiter(gcb, crate::async_trickery::WaitRes::Pointer(new_mem as _))
}

/// Allocate a single instance of a type
pub fn alloc<T>(cb: super::CbRef<::udi_sys::udi_cb_t>) -> impl ::core::future::Future<Output=ProtoHandle<T>> {
    crate::async_trickery::wait_task(cb,
        |cb| unsafe { ::udi_sys::mem::udi_mem_alloc(alloc_callback, cb, ::core::mem::size_of::<T>(), 0) },
        |res| match res {
        crate::async_trickery::WaitRes::Pointer(v) => {
            let ptr = v as *mut T;
            ProtoHandle(ptr)
            },
        _ => panic!(""),
        })
}
/// Allocate a list of one instance of a type
pub fn alloc_list<T>(cb: super::CbRef<::udi_sys::udi_cb_t>, count: usize) -> impl ::core::future::Future<Output=ProtoHandle<[T]>>
{
    crate::async_trickery::wait_task(cb,
        |cb| unsafe { ::udi_sys::mem::udi_mem_alloc(alloc_callback, cb, ::core::mem::size_of::<T>(), 0) },
        move |res| match res {
        crate::async_trickery::WaitRes::Pointer(v) => unsafe {
            let ptr = v as *mut T;
            ProtoHandle(::core::slice::from_raw_parts_mut(ptr, count))
            },
        _ => panic!(""),
        })
}
