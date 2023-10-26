use ::udi::ffi::udi_boolean_t;
use ::udi::ffi::udi_index_t;
use ::udi::ffi::udi_status_t;
use ::udi::meta_nic::ffi::udi_nic_bind_cb_t;
use ::udi::meta_nic::ffi::udi_nic_cb_t;
use ::udi::meta_nic::ffi::udi_nic_ctrl_cb_t;
use ::udi::meta_nic::ffi::udi_nic_info_cb_t;
use ::udi::meta_nic::ffi::udi_nic_rx_cb_t;
use ::udi::meta_nic::ffi::udi_nic_status_cb_t;
use ::udi::meta_nic::ffi::udi_nic_tx_cb_t;
use ::udi::meta_nic::ffi::{udi_nd_ctrl_ops_t,udi_nsr_ctrl_ops_t};

impl crate::channels::MetalangOps for udi_nd_ctrl_ops_t {
}
impl crate::channels::MetalangOps for udi_nsr_ctrl_ops_t {
}

#[no_mangle]
unsafe extern "C" fn udi_nd_bind_req(cb: *mut udi_nic_bind_cb_t, tx_chan_index: udi_index_t, rx_chan_index: udi_index_t)
{
    let ops = crate::channels::prepare_cb_for_call::<udi_nd_ctrl_ops_t>(&mut (*cb).gcb);
    (ops.nd_bind_req_op)(cb, tx_chan_index, rx_chan_index);
}
#[no_mangle]
unsafe extern "C" fn udi_nsr_bind_ack(cb: *mut udi_nic_bind_cb_t, status: udi_status_t)
{
    let ops = crate::channels::prepare_cb_for_call::<udi_nsr_ctrl_ops_t>(&mut (*cb).gcb);
    (ops.nsr_bind_ack_op)(cb, status);
}
#[no_mangle]
unsafe extern "C" fn udi_nd_unbind_req(cb: *mut udi_nic_cb_t)
{
    let ops = crate::channels::prepare_cb_for_call::<udi_nd_ctrl_ops_t>(&mut (*cb).gcb);
    (ops.nd_unbind_req_op)(cb);
}
#[no_mangle]
unsafe extern "C" fn udi_nsr_unbind_ack(cb: *mut udi_nic_cb_t, status: udi_status_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nd_enable_req(cb: *mut udi_nic_cb_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nsr_enable_ack(cb: *mut udi_nic_cb_t, status: udi_status_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nd_disable_req(cb: *mut udi_nic_cb_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nsr_disable_ack(cb: *mut udi_nic_cb_t, status: udi_status_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nd_ctrl_req(cb: *mut udi_nic_ctrl_cb_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nsr_ctrl_ack(cb: *mut udi_nic_ctrl_cb_t, status: udi_status_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nsr_status_ind(cb: *mut udi_nic_status_cb_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nd_info_req(cb: *mut udi_nic_info_cb_t, reset_statistics: udi_boolean_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nsr_info_ack(cb: *mut udi_nic_info_cb_t)
{
    todo!()
}
#[no_mangle]
// - TX
unsafe extern "C" fn udi_nsr_tx_rdy(cb: *mut udi_nic_tx_cb_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nd_tx_req(cb: *mut udi_nic_tx_cb_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nd_exp_tx_req(cb: *mut udi_nic_tx_cb_t)
{
    todo!()
}
#[no_mangle]
// - RX
unsafe extern "C" fn udi_nsr_rx_ind(cb: *mut udi_nic_rx_cb_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nsr_exp_rx_ind(cb: *mut udi_nic_rx_cb_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_nd_rx_rdy(cb: *mut udi_nic_rx_cb_t)
{
    todo!()
}
