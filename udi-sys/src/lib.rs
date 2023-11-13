//! Raw UDI C API definitions
//! 
#![feature(extern_types)]	// For opaque handle types
#![feature(c_variadic)]	// For va_list in MEI
#![no_std]

#![allow(non_camel_case_types)]
#![allow(dead_code)]

pub mod cb;
pub mod buf;
pub mod attr;
pub mod meta_mgmt;
pub mod log;
pub mod imc;
pub mod init;
pub mod pio;
pub mod physio;
pub mod meta_bridge;
pub mod meta_gio;
pub mod meta_nic;
pub mod meta_usb;
pub mod libc;
pub mod layout;
pub mod mem;
pub mod time;
pub mod queue;
pub mod endian;
pub mod mei;

pub use ::core::ffi::c_void;

#[repr(transparent)]
#[derive(Copy,Clone,Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Default)]
pub struct udi_index_t(pub u8);
impl udi_index_t {
	pub fn to_usize(&self) -> usize { self.0 as usize }
}
impl From<u8> for udi_index_t {
	fn from(v: u8) -> udi_index_t { udi_index_t(v) }
}
impl ::core::fmt::Display for udi_index_t {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        self.0.fmt(f)
    }
}

mod arch {
	pub type udi_size_t = usize;
	// IA-32 defines it this way.
	pub type udi_timestamp_t = u32;
}
pub use arch::*;

pub type udi_status_t = u32;
pub type _udi_handle_t = *mut c_void;
#[doc(hidden)] #[repr(C)] pub struct _udi_channel_s { inner: () }
pub type udi_channel_t = *mut _udi_channel_s;
#[doc(hidden)] #[repr(C)] pub struct _udi_origin_s { inner: () }
pub type udi_origin_t = *mut _udi_origin_s;

#[repr(transparent)]
#[derive(Copy,Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct udi_boolean_t(pub u8);
impl udi_boolean_t {
	pub fn to_bool(&self) -> bool { self.0 != 0 }
}
pub const TRUE: udi_boolean_t = udi_boolean_t(1);
pub const FALSE: udi_boolean_t = udi_boolean_t(0);
pub type udi_ubit8_t  = u8;
pub type udi_ubit16_t = u16;
pub type udi_ubit32_t = u32;
// NOTE: No 64-bit, by design
pub type udi_sbit8_t  = i8;
pub type udi_sbit16_t = i16;
pub type udi_sbit32_t = i32;

pub use layout::udi_layout_t;
pub use cb::udi_cb_t;

pub type udi_op_t = unsafe extern "C" fn();
pub type udi_ops_vector_t = *const udi_op_t;

#[repr(C)]
pub struct udi_buf_t
{
	pub buf_size: udi_size_t,
	// semi-opaque
}

#[repr(u32)]
pub enum StatusValues
{
	UDI_OK = 0,
	UDI_STAT_NOT_SUPPORTED        	= 1,
	UDI_STAT_NOT_UNDERSTOOD         = 2,
	UDI_STAT_INVALID_STATE          = 3,
	UDI_STAT_MISTAKEN_IDENTITY      = 4,
	UDI_STAT_ABORTED                = 5,
	UDI_STAT_TIMEOUT                = 6,
	UDI_STAT_BUSY                   = 7,
	UDI_STAT_RESOURCE_UNAVAIL       = 8,
	UDI_STAT_HW_PROBLEM             = 9,
	UDI_STAT_NOT_RESPONDING         = 10,
	UDI_STAT_DATA_UNDERRUN          = 11,
	UDI_STAT_DATA_OVERRUN           = 12,
	UDI_STAT_DATA_ERROR             = 13,
	UDI_STAT_PARENT_DRV_ERROR       = 14,
	UDI_STAT_CANNOT_BIND            = 15,
	UDI_STAT_CANNOT_BIND_EXCL       = 16,
	UDI_STAT_TOO_MANY_PARENTS       = 17,
	UDI_STAT_BAD_PARENT_TYPE        = 18,
	UDI_STAT_TERMINATED             = 19,
	UDI_STAT_ATTR_MISMATCH          = 20,
}
pub use StatusValues::*;
