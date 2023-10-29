use crate::ffi::*;

extern "C" {
    pub fn udi_bus_bind_req(cb: *mut udi_bus_bind_cb_t);
    pub fn udi_bus_unbind_req(cb: *mut udi_bus_bind_cb_t);
}

impl_metalanguage!{
    static METALANG_SPEC;
    OPS
        1 => udi_bus_device_ops_t,
        2 => udi_bus_bridge_ops_t,
        3 => super::meta_intr::udi_intr_handler_ops_t,
        4 => super::meta_intr::udi_intr_dispatcher_ops_t,
        ;
    CBS
        1 => udi_bus_bind_cb_t,
        2 => super::meta_intr::udi_intr_attach_cb_t,
        3 => super::meta_intr::udi_intr_detach_cb_t,
        4 => super::meta_intr::udi_intr_event_cb_t,
        ;
}

pub struct udi_bus_device_ops_t
{
    pub channel_event_ind_op: imc::udi_channel_event_ind_op_t,
    pub bus_bind_ack_op: unsafe extern "C" fn(*mut udi_bus_bind_cb_t, physio::udi_dma_constraints_t, u8, udi_status_t),
    pub bus_unbind_ack_op: unsafe extern "C" fn(*mut udi_bus_bind_cb_t),
    pub intr_attach_ack_op: meta_intr::udi_intr_attach_ack_op_t,
    pub intr_detach_ack_op: meta_intr::udi_intr_detach_ack_op_t,
}
unsafe impl const crate::Ops for udi_bus_device_ops_t
{
    const OPS_NUM: crate::ffi::udi_index_t = 1;
}

pub struct udi_bus_bridge_ops_t
{
    pub channel_event_ind_op: imc::udi_channel_event_ind_op_t,
    pub bus_bind_req_op: unsafe extern "C" fn(*mut udi_bus_bind_cb_t),
    pub bus_unbind_req_op: unsafe extern "C" fn(*mut udi_bus_bind_cb_t),
    pub intr_attach_req_op: unsafe extern "C" fn(*mut meta_intr::udi_intr_attach_cb_t),
    pub intr_detach_req_op: unsafe extern "C" fn(*mut meta_intr::udi_intr_detach_cb_t),
}
unsafe impl const crate::Ops for udi_bus_bridge_ops_t
{
    const OPS_NUM: crate::ffi::udi_index_t = 2;
}

#[repr(C)]
pub struct udi_bus_bind_cb_t
{
    pub gcb: udi_cb_t,
}