//!
//!
//!
#![no_std]
#![feature(waker_getters)]	// For evil with contexts
#![feature(const_trait_impl)]
#![feature(concat_idents)]	// Very evil macros

use self::async_trickery::WaitRes;
//use self::future_ext::FutureExt;
mod future_ext;
mod async_trickery;

pub mod ffi;

pub mod init;
pub mod imc;
pub mod pio;
pub mod log;
pub mod meta_mgmt;
pub mod meta_bus;
pub mod meta_intr;
pub mod meta_nic;

pub fn get_gcb_channel() -> impl ::core::future::Future<Output=ffi::udi_channel_t> {
	async_trickery::with_cb::<ffi::udi_cb_t,_,_>(|cb| cb.channel)
}
pub fn get_gcb_context() -> impl ::core::future::Future<Output=*mut ::core::ffi::c_void> {
	async_trickery::with_cb::<ffi::udi_cb_t,_,_>(|cb| cb.context)
}

// A "region" is a thread
// - rdata is the thread's data, i.e. the drive instance
// - scratch is where the future should go


#[const_trait]
pub trait Ops {
	fn get_num() -> ffi::udi_index_t;
	//fn context_size() -> ffi::udi_size_t;
	fn to_ops_vector(&self) -> ffi::udi_ops_vector_t;
}
#[doc(hidden)]
pub const fn make_ops_init<T: ~const Ops>(ops_idx: ffi::udi_index_t, meta_idx: ffi::udi_index_t, ops: &'static T) -> crate::ffi::init::udi_ops_init_t {
	crate::ffi::init::udi_ops_init_t {
		ops_idx,
		meta_idx,
		meta_ops_num: T::get_num(),
		chan_context_size: 0,
		ops_vector: ops.to_ops_vector(),
		op_flags: ::core::ptr::null(),
	}
}

#[macro_export]
macro_rules! define_driver
{
	(
		$driver:path;
		ops: {
			$($op_name:ident: Meta=$op_meta:expr, $op_op:path),*$(,)?
		},
		cbs: { $($cbs:tt)* }
	) => {
		#[repr(u8)]
		enum OpsList {
			_Zero,
			$($op_name,)*
		}
		const _STATE_SIZE: usize = {
			let v = $crate::ffi::meta_mgmt::udi_mgmt_ops_t::scratch_requirement::<Driver>();
			$(let a = <$op_op>::scratch_requirement::<$driver>(); let v = if v > a { v } else { a };)*
			v
			};
		#[no_mangle]
		pub static udi_init_info: $crate::ffi::init::udi_init_t = $crate::ffi::init::udi_init_t {
			primary_init_info: Some(&$crate::ffi::init::udi_primary_init_t {
					mgmt_ops: unsafe { &$crate::ffi::meta_mgmt::udi_mgmt_ops_t::for_driver::<Driver>() },
					mgmt_op_flags: [0,0,0,0].as_ptr(),
					mgmt_scratch_requirement: _STATE_SIZE,
					rdata_size: ::core::mem::size_of::<$crate::init::RData<Driver>>(),
					child_data_size: 0,
					enumeration_attr_list_length: 0,
					per_parent_paths: 0,
				}),
			secondary_init_list: ::core::ptr::null(),
			ops_init_list: [
				$(
				$crate::make_ops_init(OpsList::$op_name as _, $op_meta, unsafe { &<$op_op>::for_driver::<$driver>() }),
				)*
				$crate::ffi::init::udi_ops_init_t::end_of_list()
			].as_ptr(),
			};
	}
}

