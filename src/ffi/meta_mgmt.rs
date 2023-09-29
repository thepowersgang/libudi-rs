
use super::*;

#[repr(C)]
pub struct udi_mgmt_ops_t
{
	pub usage_ind_op: unsafe extern "C" fn(cb: *mut udi_usage_cb_t, resource_level: u8),
}
#[repr(C)]
pub struct udi_usage_cb_t
{
	pub gcb:	udi_cb_t,
	pub trace_mask:	super::log::udi_trevent_t,
	pub meta_idx:	udi_index_t,
}

extern "C" {
	pub fn udi_usage_res(cb: *mut udi_usage_cb_t);
}
