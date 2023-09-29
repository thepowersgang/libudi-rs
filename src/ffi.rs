#![allow(non_camel_case_types)]
#![allow(dead_code)]

pub mod meta_mgmt;
pub mod log;
pub mod init;
pub mod pio;

pub use ::core::ffi::c_void;

pub type udi_index_t = u8;
pub type udi_size_t = usize;
pub type udi_status_t = u32;
pub type udi_channel_t = *mut c_void;
pub type udi_origin_t = *mut c_void;

#[repr(C)]
pub struct udi_cb_t
{
	pub channel:	udi_channel_t,
	pub context:	*mut c_void,
	pub scratch:	*mut c_void,
	pub initiator_context:	*mut c_void,
	pub origin:	udi_origin_t,
}

