use super::{c_void, udi_index_t, udi_channel_t};

pub type udi_cb_alloc_call_t = unsafe extern "C" fn(*mut udi_cb_t, *mut udi_cb_t);

extern "C" {
    pub fn udi_cb_alloc(callback: udi_cb_alloc_call_t, gcb: *mut udi_cb_t, cb_idx: udi_index_t, default_channel: udi_channel_t);
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
