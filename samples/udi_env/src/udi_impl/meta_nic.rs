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
use ::udi::meta_nic::ffi::{udi_nd_tx_ops_t,udi_nsr_tx_ops_t};
use ::udi::meta_nic::ffi::{udi_nd_rx_ops_t,udi_nsr_rx_ops_t};

impl_metalang_ops_for!{udi_nd_ctrl_ops_t,udi_nsr_ctrl_ops_t}
impl_metalang_ops_for!{udi_nd_tx_ops_t, udi_nsr_tx_ops_t}
impl_metalang_ops_for!{udi_nd_rx_ops_t, udi_nsr_rx_ops_t}
impl_metalang_cbs!{
    1 = udi_nic_cb_t,
    2 = udi_nic_bind_cb_t,
    3 = udi_nic_ctrl_cb_t,
    4 = udi_nic_status_cb_t,
    5 = udi_nic_info_cb_t,
    6 = udi_nic_tx_cb_t,
    7 = udi_nic_rx_cb_t,
}

dispatch_call! {
    fn udi_nd_bind_req(cb: *mut udi_nic_bind_cb_t, tx_chan_index: udi_index_t, rx_chan_index: udi_index_t)
        => udi_nd_ctrl_ops_t:nd_bind_req_op;
    fn udi_nd_unbind_req(cb: *mut udi_nic_cb_t)
        => udi_nd_ctrl_ops_t:nd_unbind_req_op;
    fn udi_nd_enable_req(cb: *mut udi_nic_cb_t)
        => udi_nd_ctrl_ops_t:nd_enable_req_op;
    fn udi_nd_disable_req(cb: *mut udi_nic_cb_t)
        => udi_nd_ctrl_ops_t:nd_disable_req_op;
    fn udi_nd_ctrl_req(cb: *mut udi_nic_ctrl_cb_t)
        => udi_nd_ctrl_ops_t:nd_ctrl_req_op;
    fn udi_nd_info_req(cb: *mut udi_nic_info_cb_t, reset_statistics: udi_boolean_t)
        => udi_nd_ctrl_ops_t:nd_info_req_op;
}
dispatch_call! {
    fn udi_nsr_bind_ack(cb: *mut udi_nic_bind_cb_t, status: udi_status_t)
        => udi_nsr_ctrl_ops_t:nsr_bind_ack_op;
    fn udi_nsr_unbind_ack(cb: *mut udi_nic_cb_t, status: udi_status_t)
        => udi_nsr_ctrl_ops_t:nsr_unbind_ack_op;
    fn udi_nsr_enable_ack(cb: *mut udi_nic_cb_t, status: udi_status_t)
        => udi_nsr_ctrl_ops_t:nsr_enable_ack_op;
    //fn udi_nsr_disable_ack(cb: *mut udi_nic_cb_t, status: udi_status_t)
    //    => udi_nsr_ctrl_ops_t:nsr_disable_ack_op;
    fn udi_nsr_ctrl_ack(cb: *mut udi_nic_ctrl_cb_t, status: udi_status_t)
        => udi_nsr_ctrl_ops_t:nsr_ctrl_ack_op;
    fn udi_nsr_status_ind(cb: *mut udi_nic_status_cb_t)
        => udi_nsr_ctrl_ops_t:nsr_status_ind_op;
    fn udi_nsr_info_ack(cb: *mut udi_nic_info_cb_t)
        => udi_nsr_ctrl_ops_t:nsr_info_ack_op;
}
// - TX
dispatch_call! {
    fn udi_nd_tx_req(cb: *mut udi_nic_tx_cb_t)
        => udi_nd_tx_ops_t:nd_tx_req_op;
    fn udi_nd_exp_tx_req(cb: *mut udi_nic_tx_cb_t)
        => udi_nd_tx_ops_t:nd_exp_tx_req_op;
}
dispatch_call! {
    fn udi_nsr_tx_rdy(cb: *mut udi_nic_tx_cb_t)
        => udi_nsr_tx_ops_t:nsr_tx_rdy_op;
}
// - RX
dispatch_call! {
    fn udi_nd_rx_rdy(cb: *mut udi_nic_rx_cb_t)
        => udi_nd_rx_ops_t:nd_rx_rdy_op;
}
dispatch_call! {
    fn udi_nsr_rx_ind(cb: *mut udi_nic_rx_cb_t)
        => udi_nsr_rx_ops_t:nsr_rx_ind_op;
    fn udi_nsr_exp_rx_ind(cb: *mut udi_nic_rx_cb_t)
        => udi_nsr_rx_ops_t:nsr_exp_rx_ind_op;
}
