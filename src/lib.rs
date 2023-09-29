//!
//!
//!
#![no_std]
#![feature(waker_getters)]	// For evil with contexts

use ::core::pin::Pin;
use ::core::future::Future;
use ::core::task::Poll;

use self::async_trickery::WaitRes;
//use self::future_ext::FutureExt;
mod future_ext;
mod async_trickery;

pub mod ffi;

pub mod pio;
pub mod log;

#[repr(transparent)]
pub struct Gcb(());

// A "region" is a thread
// - rdata is the thread's data, i.e. the drive instance
// - scratch is where the future should go

pub trait Driver: 'static {
	#[allow(non_camel_case_types)]
	type Future_init: Future<Output=Self> + 'static;
	fn init(cb: Gcb, resouce_level: u8) -> Self::Future_init;
	//fn enumerate(&self, level: EnumerateLevel, attr_list: &mut [()]) -> Self::Future_enumerate;
}

struct RData<T> {
	inner: T,
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
	rdata: *mut RData<T>,
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
			// SAFE: This pointer should the valid
			unsafe { ::core::ptr::write(self_.rdata, RData { inner: r }); }
			unsafe {
				let mut v = async_trickery::wait_task::<ffi::udi_cb_t/*meta_mgmt::udi_usage_cb_t*/, _,_,()>(|cb| ffi::meta_mgmt::udi_usage_res(cb as *const _ as *mut _), |_| panic!(""));
				let _ = Pin::new_unchecked(&mut v).poll(cx);
			}
			Poll::Ready( () )
			},
		}
	}
}


// ENTRYPOINT: mgmt_ops.usage_ind
unsafe extern "C" fn usage_ind<T: Driver>(cb: *mut ffi::meta_mgmt::udi_usage_cb_t, resource_level: u8)
{
	let rdata_ptr = (*cb).gcb.context as *mut RData<T>;

	let job = MgmtStateInit { inner_future: T::init(Gcb(()), resource_level), rdata: rdata_ptr };
	async_trickery::init_task(&mut (*cb).gcb, MgmtState { op: Some(job) });
	async_trickery::run(&mut (*cb).gcb);
}

/// Create a `udi_primary_init_t` instance
pub fn make_pri_init<T: Driver>() -> ffi::init::udi_primary_init_t {
	ffi::init::udi_primary_init_t {
		mgmt_ops: &ffi::meta_mgmt::udi_mgmt_ops_t {
			usage_ind_op: usage_ind::<T>,
			},
		mgmt_op_flags: [0,0,0,0].as_ptr(),
		mgmt_scratch_requirement: async_trickery::task_size::<MgmtState<T>>(),
		rdata_size: ::core::mem::size_of::<RData<T>>(),
		child_data_size: 0,
		enumeration_attr_list_length: 0,
		per_parent_paths: 0,
	}
}

