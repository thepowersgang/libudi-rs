
use ::core::pin::Pin;
use ::core::future::Future;
use ::core::task::Poll;

use crate::async_trickery;
use crate::ffi;

pub trait Driver: 'static {
	#[allow(non_camel_case_types)]
	type Future_init: Future<Output=Self> + 'static;
	fn init(resouce_level: u8) -> Self::Future_init;
	//fn enumerate(&self, level: EnumerateLevel, attr_list: &mut [()]) -> Self::Future_enumerate;
}

#[repr(C)]
pub struct RData<T> {
    pub(crate) init_context: ffi::init::udi_init_context_t,
	pub(crate) channel_cb: *mut crate::ffi::imc::udi_channel_event_cb_t,
    pub(crate) inner: T,
}

struct MgmtState<T: Driver> {
	op: Option<MgmtStateInit<T>>,
}
impl<T: 'static+Driver> async_trickery::AsyncState for MgmtState<T> {
	fn get_future(self: Pin<&mut Self>) -> Pin<&mut dyn Future<Output=()>> {
		// SAFE: Pin projection
		unsafe { Pin::new_unchecked(Pin::get_unchecked_mut(self).op.as_mut().unwrap()) }
	}
}
struct MgmtStateInit<T: Driver> {
	inner_future: T::Future_init,
}
impl<T: Driver> Future for MgmtStateInit<T> {
	type Output = ();
	fn poll(self: Pin<&mut Self>, cx: &mut ::core::task::Context<'_>) -> Poll<Self::Output> {
		let self_ = unsafe { Pin::get_unchecked_mut(self) };
		// SAFE: Pin projecting
		match unsafe { Pin::new_unchecked(&mut self_.inner_future) }.poll(cx)
		{
		Poll::Pending => Poll::Pending,
		Poll::Ready(r) => {
			let cb: &ffi::meta_mgmt::udi_usage_cb_t = async_trickery::cb_from_waker(cx.waker());
			// SAFE: This pointer should valid
            unsafe { ::core::ptr::write(&mut (*(cb.gcb.context as *mut RData<T>)).inner, r); }
            // SAFE: Correct FFI.
			unsafe { ffi::meta_mgmt::udi_usage_res(cb as *const _ as *mut _); }
			Poll::Ready( () )
			},
		}
	}
}

// TODO: Figure out where we can store state properly
// - Probably in `context`, as `scratch` is limited and not always available :(
// - But, what are the rules for `context` being updated?

impl ffi::meta_mgmt::udi_mgmt_ops_t {
    pub const fn scratch_requirement<T: Driver>() -> usize {
        async_trickery::task_size::<MgmtState<T>>()
    }
    pub const unsafe fn for_driver<T: Driver>() -> Self {
        // ENTRYPOINT: mgmt_ops.usage_ind
        unsafe extern "C" fn usage_ind<T: Driver>(cb: *mut ffi::meta_mgmt::udi_usage_cb_t, resource_level: u8)
        {
            let job = MgmtStateInit::<T> { inner_future: T::init(resource_level) };
            async_trickery::init_task(&*cb, MgmtState { op: Some(job) });
        }
        ffi::meta_mgmt::udi_mgmt_ops_t {
			usage_ind_op: usage_ind::<T>,
			}
    }
}