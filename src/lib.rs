//! UDI - Uniform Driver Interface
//!
//! An absolutely evil attempt at making bindings for the various UDI interfaces 
#![no_std]
#![feature(waker_getters)]	// For evil with contexts
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]	// Used for getting size of tasks
#![feature(extern_types)]	// Handle types
#![feature(fundamental)]

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
pub mod channel_context;

mod error;
pub mod buf;
pub mod init;
pub mod cb;
pub mod mem;
pub mod layout;
pub mod libc;
#[macro_use]
pub mod imc;
pub mod pio;
pub mod log;
pub mod meta_mgmt;
pub mod meta_bridge;
pub mod meta_gio;
pub mod meta_nic;


pub use ::udi_macros::{debug_printf,/*GetLayout,*/};
pub use ::udi_sys as ffi;

pub use self::cb::CbRef;
pub use self::channel_context::ChildBind;

pub use self::error::{Result,Error};

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

/// Marker: Implemented on `CbList` by [define_driver] to indicate that a CB is present in the list
pub trait HasCb<T: metalang_trait::MetalangCb> {
}

/// Indicates that this type is just a wrapper around `Inner`, and thus it's valid to
/// pointer cast between the two
pub unsafe trait Wrapper<Inner> {
}

/// Define a set of wrapper types for another type, to separate trait impls
/// 
/// ```rust
/// struct MyType;
/// ::udi::define_wrappers!{MyType: MyType1 MyType2}
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
pub const fn make_ops_init<T: metalang_trait::MetalangOps>(ops_idx: ffi::udi_index_t, meta_idx: ffi::udi_index_t, chan_context_size: ffi::udi_size_t, ops: &'static T) -> crate::ffi::init::udi_ops_init_t {
	crate::ffi::init::udi_ops_init_t {
		ops_idx,
		meta_idx,
		meta_ops_num: T::META_OPS_NUM,
		chan_context_size,
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

/// Hacky structure used in place of a const trait for methods on `udi_*_ops_t` structures
pub struct OpsStructure<Ops, T, CbList>
{
	pd: ::core::marker::PhantomData<(Ops,T,CbList)>,
}

pub mod ops_wrapper_markers {
	/// Indicates that the ops entry is a valid ChildBind
	pub trait ChildBind: super::ops_markers::Ops {
		const IDX: crate::ffi::udi_index_t;
	}
}
/// Marker traits for checking ops entries
pub mod ops_markers {
	/// Trait for an entry in `OpsList` created by [super::define_driver]
	pub trait Ops {
		type OpsTy;
	}
	/// Trait for `udi_*_ops_t` structures indicating that they expect to be a parent binding with the given
	/// cb.
	pub trait ParentBind<Cb> {
		const ASSERT: ();
	}
	/// Trait for `udi_*_ops_t` structures indicating that they expect to be a child binding
	pub trait ChildBind {
		const ASSERT: ();
	}
}

/// Define a UDI driver
/// 
/// ```
/// # #[derive(Default)]
/// struct Driver;
/// # impl ::udi::init::Driver for ::udi::init::RData<Driver> {
/// #  const MAX_ATTRS: u8 = 0;
/// #  type Future_init<'s> = ::core::future::Pending<()>;
/// #  fn usage_ind<'s>(&'s mut self, _cb: ::udi::meta_mgmt::CbRefUsage<'s>, _resouce_level: u8) -> Self::Future_init<'s> {
/// #    ::core::future::pending()
/// #  }
/// #  type Future_enumerate<'s> = ::core::future::Pending<(::udi::init::EnumerateResult,::udi::init::AttrSink<'s>)>;
/// #  fn enumerate_req<'s>(&'s mut self, _cb: ::udi::init::CbRefEnumerate<'s>, level: ::udi::init::EnumerateLevel, mut attrs_out: ::udi::init::AttrSink<'s> ) -> Self::Future_enumerate<'s> {
/// #    ::core::future::pending()
/// #  }
/// #  type Future_devmgmt<'s> = ::core::future::Pending<::udi::Result<u8>>;
/// #  fn devmgmt_req<'s>(&'s mut self,  _cb: ::udi::init::CbRefMgmt<'s>, mgmt_op: udi::init::MgmtOp, _parent_id: ::udi::ffi::udi_ubit8_t) -> Self::Future_devmgmt<'s> {
/// #    ::core::future::pending()
/// #  }
/// # }
/// ::udi::define_driver!{Driver;
/// ops: {
/// 	},
/// cbs: {
///		}
/// }
/// ```
#[macro_export]
macro_rules! define_driver
{
	// TODOs:
	// - How to ensure that the metalang listed here is also correctly in the udiprops?
	//   > Could require that the path is a udiprops path.
	//   > But it still needs to be listed, and must match the op path
	//   > How to get the metalang name from the op path? Maybe use path trickery and macros?
	// `::$($op_path:ident ::)*$op_ty_name` and then `::$($op_path:ident ::)*metalang_name!(udiprops::meta)`
	(
		$driver:path;
		ops: {
			$($op_name:ident: ::$($op_op_mod:ident)::* @ $op_op_name:ident $(: $wrapper:ident<_$(,$wrapper_arg:ty)*>)?),*$(,)?
		},
		cbs: {
			$($cb_name:ident: $(::$cb_ty_mod:ident)* @ $cb_ty_name:ident),*$(,)?
		}
	) => {
		$crate::define_driver!{
			$driver as #[no_mangle] udi_init_info;
			ops: { $($op_name: Meta=::$($op_op_mod)::*::metalang_name!(udiprops::meta::), ::$($op_op_mod::)*$op_op_name $(: $wrapper<_$(,$wrapper_arg)*>)? ),* },
			cbs: { $($cb_name: Meta=$(::$cb_ty_mod)*::metalang_name!(udiprops::meta::), $(::$cb_ty_mod)*::$cb_ty_name ),* }
		}
	};
	(
		$driver:path as $(#[$a:meta])* $symname:ident;
		ops: {
			$($op_name:ident: Meta=$op_meta:expr, $op_op:path $(: $wrapper:ident<_$(,$wrapper_arg:ty)*>)?),*$(,)?
		},
		cbs: {
			$($cb_name:ident: Meta=$cb_meta:expr, $cb_ty:path),*$(,)?
		}
	) => {
		/// Indexes for the Ops list
		#[repr(u8)]
		enum RawOpsList {
			_Zero,
			$($op_name,)*
		}
		#[allow(non_snake_case, non_upper_case_globals)]
		mod OpsList {
			$crate::define_driver!(@indexes $($op_name)*);
			$(
				pub const $op_name: $crate::ffi::udi_index_t = $crate::ffi::udi_index_t(super::RawOpsList::$op_name as _);
				pub struct $op_name { _inner: () }
				impl $crate::ops_markers::Ops for $op_name {
					type OpsTy = $op_op;
				}
				$(impl $crate::ops_wrapper_markers::$wrapper for $op_name { const IDX: $crate::ffi::udi_index_t = $op_name; })?
			)*
		}
		/// Indexes for the CB list
		#[repr(u8)]
		enum RawCbList {
			_Zero,
			$($cb_name,)*
		}
		#[allow(non_snake_case)]
		mod CbList {
			$crate::define_driver!(@indexes $($cb_name)*);
			$(
				pub struct $cb_name { _inner: () }
				impl $crate::cb::CbDefinition for $cb_name {
					const INDEX: $crate::ffi::udi_index_t = $crate::ffi::udi_index_t(super::RawCbList::$cb_name as _);
					type Cb = $cb_ty;
				}
			)*
			pub struct List {}
			$(impl $crate::HasCb<$cb_ty> for List {})*
		}
		const _STATE_SIZE: usize = {
			let v = $crate::define_driver!(@ops_structrure_call $crate::ffi::meta_mgmt::udi_mgmt_ops_t, $driver, scratch_requirement)();
			$(
				let a = $crate::define_driver!(@ops_structrure_call $op_op, $driver $(: $wrapper<_$(,$wrapper_arg)*>)?, scratch_requirement)();
				let v = if v > a { v } else { a };
			)*
			v
			};
		$(#[$a])*
		pub static $symname: $crate::ffi::init::udi_init_t = $crate::ffi::init::udi_init_t {
			primary_init_info: Some(&$crate::ffi::init::udi_primary_init_t {
					mgmt_ops: unsafe { &$crate::define_driver!(@ops_structrure_call $crate::ffi::meta_mgmt::udi_mgmt_ops_t, $driver, for_driver)() },
					mgmt_op_flags: [0,0,0,0].as_ptr(),
					mgmt_scratch_requirement: _STATE_SIZE,
					rdata_size: ::core::mem::size_of::<$crate::init::RData<Driver>>(),
					child_data_size: 0,
					enumeration_attr_list_length: <$crate::init::RData<$driver> as $crate::init::Driver>::MAX_ATTRS,
					per_parent_paths: 0,
				}),
			secondary_init_list: ::core::ptr::null(),
			ops_init_list: [
				$(
				{
					$crate::make_ops_init(
						OpsList::$op_name as _,
						$op_meta,
						0 $(+ ::core::mem::size_of::< $crate::$wrapper<$driver$(,$wrapper_arg)*> >())?,
						unsafe { &$crate::define_driver!(@ops_structrure_call $op_op, $driver $(: $wrapper<_$(,$wrapper_arg)*>)?, for_driver)() }
						)
				},
				)*
				$crate::ffi::init::udi_ops_init_t::end_of_list()
			].as_ptr(),
			cb_init_list: [
				$( $crate::make_cb_init::<CbList::$cb_name>($cb_meta, _STATE_SIZE, None), )*
				$crate::ffi::init::udi_cb_init_t::end_of_list()
			].as_ptr(),
			gcb_init_list: ::core::ptr::null(),
			cb_select_list: ::core::ptr::null(),
			};
	};

	(@ops_structrure_call $op_ty:ty, $driver:ty $(: $wrapper:ident<_$(,$wrapper_arg:ty)*>)?, $call:ident) => {
		<$crate::OpsStructure<$op_ty,$crate::define_driver!(@get_wrapper $driver $(: $wrapper<_$(,$wrapper_arg)*>)?),CbList::List>>::$call
	};
	(@get_wrapper $driver:ty: $wrapper:ident<_$(,$wrapper_arg:ty)*> ) => { $crate::$wrapper<$driver$(,$wrapper_arg)*> };
	(@get_wrapper $driver:ty ) => { $crate::init::RData<$driver> };
	(@indexes $($name:ident)*) => { $crate::define_driver!(@indexes_inner $($name)* = _1); };
	(@indexes_inner $this_name:ident $($name:ident)* =_1 ) => { pub type _1 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _2); };
	(@indexes_inner $this_name:ident $($name:ident)* =_2 ) => { pub type _2 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _3); };
	(@indexes_inner $this_name:ident $($name:ident)* =_3 ) => { pub type _3 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _4); };
	(@indexes_inner $this_name:ident $($name:ident)* =_4 ) => { pub type _4 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _5); };
	(@indexes_inner $this_name:ident $($name:ident)* =_5 ) => { pub type _5 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _6); };
	(@indexes_inner $this_name:ident $($name:ident)* =_6 ) => { pub type _6 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _7); };
	(@indexes_inner $this_name:ident $($name:ident)* =_7 ) => { pub type _7 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _8); };
	(@indexes_inner $this_name:ident $($name:ident)* =_8 ) => { pub type _8 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _9); };
	(@indexes_inner $this_name:ident $($name:ident)* =_9 ) => { pub type _9 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _10); };
	(@indexes_inner $this_name:ident $($name:ident)* =_10 ) => { pub type _10 = $this_name; $crate::define_driver!(@indexes_inner $($name)* = _11); };
	(@indexes_inner =$out:ident ) => { };
}

