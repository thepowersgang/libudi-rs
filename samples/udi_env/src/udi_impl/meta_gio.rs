use ::udi::ffi::*;
use ::udi::ffi::meta_gio::*;

dispatch_call! {
    fn udi_gio_bind_req(cb: *mut udi_gio_bind_cb_t)
        => udi_gio_provider_ops_t:gio_bind_req_op;
    fn udi_gio_bind_ack(
        cb: *mut udi_gio_bind_cb_t,
        device_size_lo: udi_ubit32_t,
        device_size_hi: udi_ubit32_t,
        status: udi_status_t)
        => udi_gio_client_ops_t:gio_bind_ack_op;
    fn udi_gio_unbind_req(cb: *mut udi_gio_bind_cb_t)
        => udi_gio_provider_ops_t:gio_unbind_req_op;
    fn udi_gio_unbind_ack(cb: *mut udi_gio_bind_cb_t)
        => udi_gio_client_ops_t:gio_unbind_ack_op;

    fn udi_gio_xfer_req(cb: *mut udi_gio_xfer_cb_t)
        => udi_gio_provider_ops_t:gio_xfer_req_op;
    fn udi_gio_xfer_ack(cb: *mut udi_gio_xfer_cb_t)
        => udi_gio_client_ops_t:gio_xfer_ack_op;
    fn udi_gio_xfer_nak(cb: *mut udi_gio_xfer_cb_t, status: udi_status_t)
        => udi_gio_client_ops_t:gio_xfer_nak_op;

    fn udi_gio_event_ind(cb: *mut udi_gio_event_cb_t) => udi_gio_client_ops_t:gio_event_ind_op;
    fn udi_gio_event_res(cb: *mut udi_gio_event_cb_t) => udi_gio_provider_ops_t:gio_event_res_op;
}