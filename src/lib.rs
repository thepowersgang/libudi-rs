//!
//!
//!
#![no_std]
#![feature(waker_getters)]	// For evil with contexts
#![feature(const_trait_impl)]
#![feature(concat_idents)]	// Very evil macros

use self::async_trickery::WaitRes;
//use self::future_ext::FutureExt;
#[macro_use]
mod future_ext;
mod async_trickery;

pub mod ffi;

pub mod buf;
pub mod init;
pub mod cb;
pub mod imc;
pub mod pio;
pub mod log;
pub mod meta_mgmt;
pub mod meta_bus;
pub mod meta_intr;
pub mod meta_nic;

pub type Result = ::core::result::Result<(),ffi::udi_status_t>;

pub fn get_gcb_channel() -> impl ::core::future::Future<Output=ffi::udi_channel_t> {
	async_trickery::with_cb::<ffi::udi_cb_t,_,_>(|cb| cb.channel)
}
pub fn get_gcb_context() -> impl ::core::future::Future<Output=*mut ::core::ffi::c_void> {
	async_trickery::with_cb::<ffi::udi_cb_t,_,_>(|cb| cb.context)
}

// A "region" is a thread
// - rdata is the thread's data, i.e. the drive instance
// - scratch is where the future should go


/// SAFETY: The pointed-to data must be valid as [udi_ops_init_t]
pub unsafe trait Ops {
	const OPS_NUM: ffi::udi_index_t;
}
/// Indicates that this type is just a wrapper around `Inner`, and thus it's valid to
/// pointer cast between the two
pub unsafe trait Wrapper<Inner> {
}

#[macro_export]
macro_rules! define_wrappers {
	($root_type:ty : $($name:ident)+) => {
		$(
		#[repr(transparent)]
		struct $name($root_type);
		unsafe impl $crate::Wrapper<$root_type> for $name {}
		)+
	}
}

#[doc(hidden)]
pub const fn make_ops_init<T: Ops>(ops_idx: ffi::udi_index_t, meta_idx: ffi::udi_index_t, ops: &'static T) -> crate::ffi::init::udi_ops_init_t {
	crate::ffi::init::udi_ops_init_t {
		ops_idx,
		meta_idx,
		meta_ops_num: T::OPS_NUM,
		chan_context_size: 0,
		ops_vector: ops as *const _ as *const _,
		op_flags: ::core::ptr::null(),
	}
}
#[doc(hidden)]
pub const fn enforce_is_wrapper_for<T, U>()
where
	T: Wrapper<U>
{
}

#[macro_export]
macro_rules! define_driver
{
	(
		$driver:path;
		ops: {
			$($op_name:ident: Meta=$op_meta:expr, $op_op:path $(: $wrapper:ty)?),*$(,)?
		},
		cbs: {
			$($cb_name:ident: Meta=$cb_meta:expr, $cb_ty:path),*$(,)?
		}
	) => {
		#[repr(u8)]
		enum OpsList {
			_Zero,
			$($op_name,)*
		}
		#[repr(u8)]
		enum CbList {
			_Zero,
			$($cb_name,)*
		}
		mod Cbs {
			$(
				pub struct $cb_name(());
				impl $crate::cb::CbDefinition for $cb_name {
					const INDEX: u8 = super::CbList::$cb_name as _;
					type Cb = $cb_ty;
				}
			)*
		}
		const _STATE_SIZE: usize = {
			let v = $crate::ffi::meta_mgmt::udi_mgmt_ops_t::scratch_requirement::<$driver>();
			$(
				let a = <$op_op>::scratch_requirement::<$crate::define_driver!(@get_wrapper $driver $(: $wrapper)?)>();
				let v = if v > a { v } else { a };
			)*
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
				{
					$( $crate::enforce_is_wrapper_for::<$wrapper, $driver>(); )?
					$crate::make_ops_init(OpsList::$op_name as _, $op_meta, unsafe { &<$op_op>::for_driver::<$crate::define_driver!(@get_wrapper $driver $(: $wrapper)?)>() })
				},
				)*
				$crate::ffi::init::udi_ops_init_t::end_of_list()
			].as_ptr(),
			cb_init_list: [
				//$( $crate::make_cb_init(CbList::$cb_name as _, $cb_meta, Cbs::$cb_name::Cb::META_CB_NUM, _STATE_SIZE, None), )*
				$crate::ffi::init::udi_cb_init_t::end_of_list()
			].as_ptr(),
			};
	};

	(@get_wrapper $driver:ty: $wrapper:ty ) => { $wrapper };
	(@get_wrapper $driver:ty ) => { $driver };
}

