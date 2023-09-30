use crate::ffi::*;

pub struct udi_bus_device_ops_t
{
    pub channel_event_ind_op: imc::udi_channel_event_ind_op_t,
    pub bus_bind_ack_op: extern "C" fn(*mut udi_bus_bind_cb_t, physio::udi_dma_constraints_t, u8, udi_status_t),
    pub bus_unbind_ack_op: extern "C" fn(*mut udi_bus_bind_cb_t),
    pub intr_attach_ack_op: meta_intr::udi_intr_attach_ack_op_t,
    pub intr_detach_ack_op: meta_intr::udi_intr_detach_ack_op_t,
}
impl const crate::Ops for udi_bus_device_ops_t
{
    fn get_num() -> crate::ffi::udi_index_t {
        1
    }
    fn to_ops_vector(&self) -> crate::ffi::udi_ops_vector_t {
        self as *const _ as *const _
    }
}

#[repr(C)]
pub struct udi_bus_bind_cb_t
{
    pub gcb: udi_cb_t,
}