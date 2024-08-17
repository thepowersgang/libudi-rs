//! Control blocks
//!
//! Control blocks are the core context for UDI calls between drivers and the environment or each other
use crate::ffi::udi_channel_t;

mod collections;

pub use self::collections::Chain;
pub use self::collections::SharedQueue;

/// A reference to a Cb (for async calls)
pub struct CbRef<'a, T: 'static>(*mut T, ::core::marker::PhantomData<&'a T>);
impl<'a, T: 'static> Copy for CbRef<'a, T> {
}
impl<'a, T: 'static> Clone for CbRef<'a, T> {
    fn clone(&self) -> Self { *self }
}
/// Methods common to all CbRef instances - additional methods can be present on specific CB types
impl<'a, T: 'static> CbRef<'a, T>
where
    T: crate::async_trickery::GetCb
{
    /// Create a new reference from a shared pointer to a CB
    ///
    /// SAFETY: The CB must be fully valid, but doesn't need to be uniquely owned
    pub unsafe fn new(p: *mut T) -> Self {
        CbRef(p, ::core::marker::PhantomData)
    }
    /// Get the raw pointer from this reference
    pub fn to_raw(&self) -> *mut T {
        self.0
    }
    /// Convert this shared handle into an owned handle
    ///
    /// UNSAFE: Caller must ensure that this is the only reference
    pub unsafe fn into_owned(self) -> CbHandle<T> {
        CbHandle(self.0)
    }
    /// Obtain a `udi_cb_t` reference from any other CB type
    pub fn gcb(&self) -> CbRef<'a, crate::ffi::udi_cb_t>
    where
        T: crate::async_trickery::GetCb,
    {
        CbRef(self.0 as *mut _, ::core::marker::PhantomData)
    }
}
impl<'a, T: 'static> ::core::ops::Deref for CbRef<'a,T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFE: Pointer is valid... and it shouldn't change while this handle is open?
        unsafe { &*self.0 }
    }
}

/// An owning handle to a CB
pub struct CbHandle<T>(*mut T)
where
    T: crate::async_trickery::GetCb
    ;
impl<T> Drop for CbHandle<T>
where
    T: crate::async_trickery::GetCb
{
    fn drop(&mut self) {
        unsafe { ::udi_sys::cb::udi_cb_free(self.0 as *mut ::udi_sys::udi_cb_t); }
    }
}
/// Methods common to all CbHandle instances - additional methods can be present on specific CB types
impl<T> CbHandle<T>
where
    T: crate::async_trickery::GetCb
{
    /// SAFETY: Caller must ensure that this is the only reference to the CB, to allow mutation
    pub unsafe fn from_raw(v: *mut T) -> Self {
        Self(v)
    }
    /// Convert into a raw handle
    pub fn into_raw(self) -> *mut T {
        let CbHandle(rv) = self;
        ::core::mem::forget(self);
        rv
    }
    /// Get a reference to the Generic Control Block
    pub fn gcb(&self) -> CbRef<'_, crate::ffi::udi_cb_t>
    where
        T: crate::async_trickery::GetCb,
    {
        CbRef(self.0 as *mut _, ::core::marker::PhantomData)
    }

    /// SAFETY: The caller must ensure that all internal pointers stay valid
    pub unsafe fn get_mut(&mut self) -> &mut T {
        // SAFE: Owned
        unsafe { &mut *self.0 }
    }

    /// Set the channel field of the CB from a channel handle
    // TODO - Is there a safety requirement for the channel to be matched to the CB
    // - Might be the same as ensuring the right channel endpoint matching - environment can detect and crash on it.
    pub fn set_channel(&mut self, channel: &crate::imc::ChannelHandle) {
        // SAFE: Valid data, and correct pointer values being written
        unsafe {
            (*(self.0 as *mut crate::ffi::udi_cb_t)).channel = channel.raw();
        }
    }
}
impl<T> ::core::ops::Deref for CbHandle<T>
where
    T: crate::async_trickery::GetCb
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFE: Owned
        unsafe { &*self.0 }
    }
}

/// Trait covering the definition of a Control Block (in [crate::define_driver])
pub trait CbDefinition {
    /// Control block index, matching udiprops
    const INDEX: crate::ffi::udi_index_t;
    /// Metalanguage control block data type
    type Cb: crate::metalang_trait::MetalangCb;
}

/// Allocate a new control block for the nominated channel
pub fn alloc<CbDef>(
	cb: crate::CbRef<crate::ffi::udi_cb_t>,
    default_channel: udi_channel_t
    ) -> impl ::core::future::Future<Output=CbHandle<CbDef::Cb>>
where
    CbDef: CbDefinition,
    CbDef::Cb: crate::async_trickery::GetCb
{
	unsafe extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, new_cb: *mut crate::ffi::udi_cb_t) {
		unsafe { crate::async_trickery::signal_waiter(gcb, crate::WaitRes::Pointer(new_cb as *mut ())); }
	}
	crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
        cb,
		move |cb| unsafe {
            crate::ffi::cb::udi_cb_alloc(callback, cb as *const _ as *mut _, CbDef::INDEX, default_channel)
			},
		|res| {
			let crate::WaitRes::Pointer(p) = res else { panic!(""); };
			CbHandle(p as *mut _)
			}
		)
}

/// Allocate a collection of CBs
pub fn alloc_batch<CbDef>(
	cb: crate::CbRef<crate::ffi::udi_cb_t>,
    count: u8,
    buffer: Option<(usize, crate::ffi::buf::udi_buf_path_t)>,
    ) -> impl ::core::future::Future<Output=Chain<CbDef::Cb>>
where
    CbDef: CbDefinition,
{
	extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, new_cb: *mut crate::ffi::udi_cb_t) {
		unsafe { crate::async_trickery::signal_waiter(gcb, crate::WaitRes::Pointer(new_cb as *mut ())); }
	}
    let (with_buf,buf_size,path_handle,) = match buffer {
        None => (::udi_sys::FALSE, 0, ::udi_sys::buf::UDI_NULL_PATH_BUF),
        Some((size, path)) => (::udi_sys::TRUE, size, path)
    };
	crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
        cb,
		move |cb| unsafe {
            crate::ffi::cb::udi_cb_alloc_batch(callback, cb as *const _ as *mut _, CbDef::INDEX, count.into(), with_buf, buf_size, path_handle)
			},
		|res| {
			let crate::WaitRes::Pointer(p) = res else { panic!(""); };
            // SAFE: This input pointer is from `alloc_batch` - so is a valid chain of CBs
			unsafe { Chain::from_raw(p as *mut _) }
			}
		)
}