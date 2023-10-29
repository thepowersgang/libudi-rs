//! UDI - Uniform Driver Interface
//!
//! An absolutely evil attempt at making bindings for the various UDI interfaces 
#![no_std]
#![feature(waker_getters)]	// For evil with contexts
#![feature(const_trait_impl)]
#![feature(concat_idents)]	// Very evil macros
#![feature(const_mut_refs)]	// Used for getting size of taskss

// A "region" is a thread
// - rdata is the thread's data, i.e. the drive instance
// - scratch is where the future should go

use self::async_trickery::WaitRes;
//use self::future_ext::FutureExt;
#[macro_use]
mod future_ext;
#[macro_use]
mod async_trickery;

#[macro_use]
pub mod metalang_trait;

pub use cb::CbRef;

pub mod ffi;

pub mod buf;
pub mod init;
pub mod cb;
#[macro_use]
pub mod imc;
pub mod pio;
pub mod log;
pub mod meta_mgmt;
pub mod meta_bus;
pub mod meta_intr;
pub mod meta_nic;

pub type Result<T> = ::core::result::Result<T,Error>;

/// A wrapper around `udi_status_t` that cannot be `UDI_OK`
pub struct Error(::core::num::NonZeroU32);
impl Error {
	pub fn into_inner(self) -> ffi::udi_status_t {
		self.0.get()
	}
	pub fn from_status(s: ffi::udi_status_t) -> Result<()> {
		match ::core::num::NonZeroU32::new(s) {
		Some(v) => Err(Error(v)),
		None => Ok( () ),
		}
	}
	pub fn to_status(r: Result<()>) -> ffi::udi_status_t {
		match r {
		Ok(()) => ffi::UDI_OK as _,
		Err(e) => e.into_inner(),
		}
	}
}

#[cfg(false_)]
pub fn get_gcb() -> impl ::core::future::Future<Output=&'static ffi::udi_cb_t> {
	async_trickery::with_cb::<ffi::udi_cb_t,_,_>(|cb| unsafe { &*(cb as *const _) })
}
pub fn get_gcb_channel() -> impl ::core::future::Future<Output=ffi::udi_channel_t> {
	async_trickery::with_cb::<ffi::udi_cb_t,_,_>(|cb| cb.channel)
}
pub fn get_gcb_context() -> impl ::core::future::Future<Output=*mut ::core::ffi::c_void> {
	async_trickery::with_cb::<ffi::udi_cb_t,_,_>(|cb| cb.context)
}

/// HELPER: A constant `max` operation on `usize`
pub const fn const_max(a: usize, b: usize) -> usize {
	if a > b { a } else { b }
}

/// Trait for `udi_*_ops_t` types
/// 
/// SAFETY: The pointed-to data must be valid as [ffi::init::udi_ops_init_t]
pub unsafe trait Ops {
	const OPS_NUM: ffi::udi_index_t;
}
/// Indicates that this type is just a wrapper around `Inner`, and thus it's valid to
/// pointer cast between the two
pub unsafe trait Wrapper<Inner> {
}

/// Define a set of wrapper types for another type, to separate trait impls
/// 
/// ```rust
/// struct MyType;
/// define_wrappers!(MyType: MyType1 MyType2)
/// ```
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
pub const fn make_cb_init<T: cb::CbDefinition>(meta_idx: ffi::udi_index_t, scratch_requirement: ffi::udi_size_t, inline: Option<()>) -> crate::ffi::init::udi_cb_init_t {
	let (inline_size,inline_layout) = if let Some(_) = inline {
		todo!()
	}
	else {
		(0, ::core::ptr::null())
	};
	crate::ffi::init::udi_cb_init_t {
		cb_idx: T::INDEX,
		meta_idx,
		meta_cb_num: <T::Cb as metalang_trait::MetalangCb>::META_CB_NUM,
		inline_size,
		inline_layout,
		scratch_requirement,
	}
}
#[doc(hidden)]
pub const fn enforce_is_wrapper_for<T, U>()
where
	T: Wrapper<U>
{
}

/// Define a UDI driver
/// 
/// ```rust
/// struct Driver;
/// define_driver!{Driver;
/// ops: {
/// 	},
/// cbs: {
///		}
/// }
/// ```
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
		#[allow(non_snake_case)]
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
					enumeration_attr_list_length: <$driver as $crate::init::Driver>::MAX_ATTRS,
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
				$( $crate::make_cb_init::<Cbs::$cb_name>($cb_meta, _STATE_SIZE, None), )*
				$crate::ffi::init::udi_cb_init_t::end_of_list()
			].as_ptr(),
			gcb_init_list: ::core::ptr::null(),
			cb_select_list: ::core::ptr::null(),
			};
	};

	(@get_wrapper $driver:ty: $wrapper:ty ) => { $wrapper };
	(@get_wrapper $driver:ty ) => { $driver };
}

