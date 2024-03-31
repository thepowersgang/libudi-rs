use ::udi::ffi::meta_mgmt::{udi_usage_cb_t, udi_enumerate_cb_t, udi_mgmt_cb_t};
use ::udi::ffi::{udi_index_t, udi_status_t};
use ::udi::ffi::udi_ubit8_t;

#[no_mangle]
pub unsafe extern "C" fn udi_usage_res(cb: *mut udi_usage_cb_t)
{
    let cb = ::udi::cb::CbHandle::from_raw(cb);
    let instance = &*(cb.gcb.initiator_context as *mut crate::DriverInstance);
    instance.management_state.usage_res(&instance, cb);
}
#[no_mangle]
pub unsafe extern "C" fn udi_enumerate_ack(cb: *mut udi_enumerate_cb_t, enumeration_result: udi_ubit8_t, ops_idx: udi_index_t)
{
    let cb = ::udi::cb::CbHandle::from_raw(cb);
    let instance = &*(cb.gcb.initiator_context as *mut crate::DriverInstance);
    let enumeration_result = match enumeration_result
        {
        0 => ::udi::init::EnumerateResult::Ok(::udi::init::EnumerateResultOk::from_raw(ops_idx, (*cb).child_id)),
        1 => ::udi::init::EnumerateResult::Leaf,
        2 => ::udi::init::EnumerateResult::Done,
        3 => ::udi::init::EnumerateResult::Rescan,
        4 => ::udi::init::EnumerateResult::Removed((*cb).child_id),
        5 => ::udi::init::EnumerateResult::RemovedSelf,
        6 => ::udi::init::EnumerateResult::Released,
        255 => ::udi::init::EnumerateResult::Failed,
        _ => panic!("Unexpected value for enumeration_result {}", enumeration_result),
        };
    instance.management_state.enumerate_ack(&instance, cb, enumeration_result)
}
#[no_mangle]
pub unsafe extern "C" fn udi_devmgmt_ack(cb: *mut udi_mgmt_cb_t, flags: udi_ubit8_t, status: udi_status_t)
{
    let cb = ::udi::cb::CbHandle::from_raw(cb);
    let instance = &*(cb.gcb.initiator_context as *mut crate::DriverInstance);
    let status = ::udi::Error::from_status(status);
    instance.management_state.devmgmt_ack(&instance, cb, flags, status);
}
#[no_mangle]
pub unsafe extern "C" fn udi_final_cleanup_ack(cb: *mut udi_mgmt_cb_t)
{
    let cb = ::udi::cb::CbHandle::from_raw(cb);
    let instance = &*(cb.gcb.initiator_context as *mut crate::DriverInstance);
    instance.management_state.final_cleanup_ack(instance, cb);
}