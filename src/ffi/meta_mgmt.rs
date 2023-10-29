
use super::*;

extern "C" {
	pub fn udi_usage_res(cb: *mut udi_usage_cb_t);
	pub fn udi_enumerate_ack(cb: *mut udi_enumerate_cb_t, enumeration_result: udi_ubit8_t, ops_idx: udi_index_t);
	pub fn udi_devmgmt_ack(cb: *mut udi_mgmt_cb_t, flags: udi_ubit8_t, status: udi_status_t);
	pub fn udi_final_cleanup_ack(cb: *mut udi_mgmt_cb_t);
}

#[repr(C)]
pub struct udi_mgmt_ops_t
{
	pub usage_ind_op: unsafe extern "C" fn(cb: *mut udi_usage_cb_t, resource_level: u8),
	pub enumerate_req_op: unsafe extern "C" fn(cb: *mut udi_enumerate_cb_t, enumeration_level: u8),
	pub devmgmt_req_op: unsafe extern "C" fn(cb: *mut udi_mgmt_cb_t, mgmt_op: udi_ubit8_t, parent_ID: udi_ubit8_t),
	pub final_cleanup_req_op: unsafe extern "C" fn(cb: *mut udi_mgmt_cb_t),
}
#[repr(C)]
pub struct udi_usage_cb_t
{
	pub gcb: udi_cb_t,
	pub trace_mask:	super::log::udi_trevent_t,
	pub meta_idx: udi_index_t,
}
#[repr(C)]
pub struct udi_enumerate_cb_t
{
	pub gcb: udi_cb_t,
	pub child_id: udi_ubit32_t,
	pub child_data: *mut c_void,
    pub attr_list: *mut super::attr::udi_instance_attr_list_t,
    pub attr_valid_length: udi_ubit8_t,
    pub filter_list: *const udi_filter_element_t,
    pub filter_list_length: udi_ubit8_t,
    pub parent_id: udi_ubit8_t,
}
#[repr(C)]
pub struct udi_mgmt_cb_t
{
	pub gcb: udi_cb_t,
}

#[repr(C)]
pub struct udi_filter_element_t
{
	pub attr_name: [::core::ffi::c_char; super::attr::UDI_MAX_ATTR_NAMELEN],
	pub attr_min: [udi_ubit8_t; super::attr::UDI_MAX_ATTR_SIZE],
	pub attr_min_len: udi_ubit8_t,
	pub attr_max: [udi_ubit8_t; super::attr::UDI_MAX_ATTR_SIZE],
	pub attr_max_len: udi_ubit8_t,
	pub attr_type: super::attr::udi_instance_attr_type_t,
	pub attr_stride: udi_ubit32_t,
}
