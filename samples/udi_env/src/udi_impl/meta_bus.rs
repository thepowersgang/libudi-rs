use ::udi::ffi::meta_bus::{udi_bus_device_ops_t,udi_bus_bridge_ops_t};
use ::udi::ffi::meta_intr::{udi_intr_attach_cb_t,udi_intr_detach_cb_t};
use ::udi::ffi::meta_bus::udi_bus_bind_cb_t;

impl crate::channels::MetalangOps for udi_bus_device_ops_t {}
impl crate::channels::MetalangOps for udi_bus_bridge_ops_t {}

dispatch_call!{
    fn udi_bus_bind_req(cb: *mut udi_bus_bind_cb_t)
        => udi_bus_bridge_ops_t:bus_bind_req_op;
    fn udi_bus_bind_ack(cb: *mut udi_bus_bind_cb_t,
        dma_constraints: ::udi::ffi::physio::udi_dma_constraints_t,
        preferred_endianness: ::udi::ffi::udi_ubit8_t,
        status: ::udi::ffi::udi_status_t
        )
        => udi_bus_device_ops_t:bus_bind_ack_op;
    fn udi_bus_unbind_req(cb: *mut udi_bus_bind_cb_t)
        => udi_bus_bridge_ops_t:bus_unbind_req_op;
    fn udi_bus_unbind_ack(cb: *mut udi_bus_bind_cb_t)
        => udi_bus_device_ops_t:bus_unbind_ack_op;
    fn udi_intr_attach_req(cb: *mut udi_intr_attach_cb_t)
        => udi_bus_bridge_ops_t:intr_attach_req_op;
    fn udi_intr_attach_ack(cb: *mut udi_intr_attach_cb_t, status: ::udi::ffi::udi_status_t)
        => udi_bus_device_ops_t:intr_attach_ack_op;
    fn udi_intr_detach_req(cb: *mut udi_intr_detach_cb_t)
        => udi_bus_bridge_ops_t:intr_detach_req_op;
    fn udi_intr_detach_ack(cb: *mut udi_intr_detach_cb_t)
        => udi_bus_device_ops_t:intr_detach_ack_op;
}