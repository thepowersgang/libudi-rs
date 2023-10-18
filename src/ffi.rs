#![allow(non_camel_case_types)]
#![allow(dead_code)]

pub mod cb;
pub mod buf;
pub mod meta_mgmt;
pub mod log;
pub mod imc;
pub mod init;
pub mod pio;
pub mod physio;
pub mod meta_bus;
pub mod meta_intr;

pub use ::core::ffi::c_void;

pub type udi_index_t = u8;
pub type udi_size_t = usize;
pub type udi_status_t = u32;
pub type _udi_handle_t = *mut c_void;
pub type udi_channel_t = _udi_handle_t;
pub type udi_origin_t = _udi_handle_t;

pub type udi_boolean_t = u8;
pub type udi_ubit8_t = u8;
pub type udi_ubit16_t = u16;
pub type udi_ubit32_t = u32;

pub type udi_layout_t = u8;

pub type udi_ops_vector_t = *const extern "C" fn();

#[repr(C)]
pub struct udi_buf_t
{
	pub buf_size: udi_size_t,
	// semi-opaque
}

#[repr(C)]
pub struct udi_cb_t
{
	pub channel:	udi_channel_t,
	pub context:	*mut c_void,
	pub scratch:	*mut c_void,
	pub initiator_context:	*mut c_void,
	pub origin:	udi_origin_t,
	// semi-opaque
}

