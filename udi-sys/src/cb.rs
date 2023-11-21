use super::{c_void, udi_index_t, udi_size_t, udi_boolean_t};
use super::udi_channel_t;
use super::udi_layout_t;
use super::buf::udi_buf_path_t;

pub type udi_cb_alloc_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_cb: *mut udi_cb_t);
pub type udi_cb_alloc_batch_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, first_new_cb: *mut udi_cb_t);

extern "C" {
    pub fn udi_cb_alloc(callback: udi_cb_alloc_call_t, gcb: *mut udi_cb_t, cb_idx: udi_index_t, default_channel: udi_channel_t);
	pub fn udi_cb_alloc_dynamic(
		callback: udi_cb_alloc_call_t,
		gcb: *mut udi_cb_t,
		cb_idx: udi_index_t,
		default_channel: udi_channel_t,
		inline_size: udi_size_t,
		inline_layout: *const udi_layout_t
	);
	pub fn udi_cb_alloc_batch(
		callback: udi_cb_alloc_batch_call_t,
		gcb: *mut udi_cb_t,
		cb_idx: udi_index_t,
		count: udi_index_t,
		with_buf: udi_boolean_t,
		buf_size: udi_size_t,
		path_handle: udi_buf_path_t
	);
	pub fn udi_cb_free(cb: *mut udi_cb_t);
}

#[repr(C)]
pub struct udi_cb_t
{
	pub channel: udi_channel_t,
	pub context: *mut c_void,
	pub scratch: *mut c_void,
	pub initiator_context: *mut c_void,
	pub origin: super::udi_origin_t,
	// semi-opaque
}
