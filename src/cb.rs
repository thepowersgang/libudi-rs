use crate::ffi::udi_channel_t;

/// A reference to a Cb (for async calls)
pub struct CbRef<'a, T: 'static>(*mut T, ::core::marker::PhantomData<&'a T>);
impl<'a, T: 'static> Copy for CbRef<'a, T> {
}
impl<'a, T: 'static> Clone for CbRef<'a, T> {
    fn clone(&self) -> Self { *self }
}
impl<'a, T: 'static> CbRef<'a, T> {
    pub unsafe fn new(p: *mut T) -> Self {
        CbRef(p, ::core::marker::PhantomData)
    }
    pub fn to_raw(&self) -> *mut T {
        self.0
    }
    pub fn gcb(&self) -> CbRef<'a, crate::ffi::udi_cb_t>
    where
        T: crate::async_trickery::GetCb,
    {
        CbRef(self.0 as *mut _, ::core::marker::PhantomData)
    }
}

/// An owning handle to a CB
pub struct CbHandle<T>(*mut T);
impl<T> CbHandle<T> {
    pub fn into_raw(self) -> *mut T {
        self.0
    }
}
impl<T> ::core::ops::Deref for CbHandle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFE: Owned
        unsafe { &*self.0 }
    }
}
impl<T> ::core::ops::DerefMut for CbHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFE: Owned
        unsafe { &mut *self.0 }
    }
}

pub trait CbDefinition {
    const INDEX: u8;
    type Cb;
}

pub fn alloc<CbDef>(
	cb: crate::CbRef<crate::ffi::udi_cb_t>,
    default_channel: udi_channel_t
    ) -> impl ::core::future::Future<Output=CbHandle<CbDef::Cb>>
where
    CbDef: CbDefinition,
{
	extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, new_cb: *mut crate::ffi::udi_cb_t) {
		unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(new_cb as *mut ())); }
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
