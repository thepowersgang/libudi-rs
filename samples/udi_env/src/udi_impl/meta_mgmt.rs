use ::udi::ffi::meta_mgmt::udi_mgmt_ops_t;
use ::udi::ffi::meta_mgmt::{udi_usage_cb_t, udi_enumerate_cb_t, udi_mgmt_cb_t};
use ::udi::ffi::{udi_index_t, udi_status_t};
use ::udi::ffi::udi_ubit8_t;

pub struct ManagementAgentOps {
    pub usage_res_op        : unsafe extern "C" fn(cb: *mut ::udi::ffi::meta_mgmt::udi_usage_cb_t),
	pub enumerate_ack_op    : unsafe extern "C" fn (cb: *mut udi_enumerate_cb_t, enumeration_result: udi_ubit8_t, ops_idx: udi_index_t),
	pub devmgmt_ack_op      : unsafe extern "C" fn (cb: *mut udi_mgmt_cb_t, flags: udi_ubit8_t, status: udi_status_t),
	pub final_cleanup_ack_op: unsafe extern "C" fn (cb: *mut udi_mgmt_cb_t),
}

impl crate::channels::MetalangOps for udi_mgmt_ops_t {
}
impl crate::channels::MetalangOps for ManagementAgentOps {
}

dispatch_call!{
    pub fn udi_usage_ind(cb: *mut udi_usage_cb_t, resource_level: u8)
        => udi_mgmt_ops_t:usage_ind_op;
}
#[no_mangle]
pub unsafe extern "C" fn udi_usage_res(cb: *mut udi_usage_cb_t)
{
    let ops = crate::channels::prepare_cb_for_call::<ManagementAgentOps>(&mut (*cb).gcb);
    (ops.usage_res_op)(cb);
}
#[no_mangle]
pub unsafe extern "C" fn udi_enumerate_ack(cb: *mut udi_enumerate_cb_t, enumeration_result: udi_ubit8_t, ops_idx: udi_index_t)
{
    let ops = crate::channels::prepare_cb_for_call::<ManagementAgentOps>(&mut (*cb).gcb);
    (ops.enumerate_ack_op)(cb, enumeration_result, ops_idx);
}
#[no_mangle]
pub unsafe extern "C" fn udi_devmgmt_ack(cb: *mut udi_mgmt_cb_t, flags: udi_ubit8_t, status: udi_status_t)
{
    let ops = crate::channels::prepare_cb_for_call::<ManagementAgentOps>(&mut (*cb).gcb);
    (ops.devmgmt_ack_op)(cb, flags, status);
}
#[no_mangle]
pub unsafe extern "C" fn udi_final_cleanup_ack(cb: *mut udi_mgmt_cb_t)
{
    let ops = crate::channels::prepare_cb_for_call::<ManagementAgentOps>(&mut (*cb).gcb);
    (ops.final_cleanup_ack_op)(cb);
}