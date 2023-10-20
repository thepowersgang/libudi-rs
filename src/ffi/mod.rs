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

pub use cb::udi_cb_t;

pub type udi_ops_vector_t = *const extern "C" fn();

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
