use super::*;

pub type udi_intr_attach_ack_op_t = unsafe extern "C" fn(cb: *mut udi_intr_attach_cb_t, status: super::udi_status_t);
pub type udi_intr_detach_ack_op_t = unsafe extern "C" fn(cb: *mut udi_intr_detach_cb_t);
pub type udi_intr_event_ind_op_t = unsafe extern "C" fn(cb: *mut udi_intr_event_cb_t, flags: u8);
pub type udi_intr_event_rdy_op_t = unsafe extern "C" fn(cb: *mut udi_intr_event_cb_t);

extern "C" {
    pub fn udi_intr_attach_req(cb: *mut udi_intr_attach_cb_t);
    pub fn udi_intr_attach_ack(cb: *mut udi_intr_attach_cb_t, status: udi_status_t);
    pub fn udi_intr_detach_req(cb: *mut udi_intr_detach_cb_t);
    pub fn udi_intr_detach_ack(cb: *mut udi_intr_detach_cb_t);
    pub fn udi_intr_event_rdy(cb: *mut udi_intr_event_cb_t);
}

#[repr(C)]
pub struct udi_intr_handler_ops_t
{
    pub channel_event_ind_op: imc::udi_channel_event_ind_op_t,
    pub intr_event_ind_op: udi_intr_event_ind_op_t,
}
unsafe impl crate::Ops for udi_intr_handler_ops_t {
    const OPS_NUM: super::udi_index_t = 3;
}
#[repr(C)]
pub struct udi_intr_dispatcher_ops_t
{
    pub channel_event_ind_op: imc::udi_channel_event_ind_op_t,
    pub intr_event_rdy_op: udi_intr_event_rdy_op_t,
}
unsafe impl crate::Ops for udi_intr_dispatcher_ops_t {
    const OPS_NUM: super::udi_index_t = 4;
}

#[repr(C)]
pub struct udi_intr_attach_cb_t
{
    pub gcb: udi_cb_t,
    pub interrupt_index: udi_index_t,
    pub min_event_pend: u8,
    pub preprocessing_handle: pio::udi_pio_handle_t,
}
unsafe impl crate::async_trickery::GetCb for udi_intr_attach_cb_t {
    fn get_gcb(&self) -> &udi_cb_t {
        &self.gcb
    }
}
#[repr(C)]
pub struct udi_intr_detach_cb_t
{
    pub gcb: udi_cb_t,
    pub interrupt_idx: udi_index_t,
}
unsafe impl crate::async_trickery::GetCb for udi_intr_detach_cb_t {
    fn get_gcb(&self) -> &udi_cb_t {
        &self.gcb
    }
}

#[repr(C)]
pub struct udi_intr_event_cb_t
{
    pub gcb: udi_cb_t,
    pub event_buf: *mut udi_buf_t,
    pub intr_result: u16,
}
unsafe impl crate::async_trickery::GetCb for udi_intr_event_cb_t {
    fn get_gcb(&self) -> &udi_cb_t {
        &self.gcb
    }
}
pub const UDI_INTR_UNCLAIMED: u16 = 1 << 0;
pub const UDI_INTR_NO_EVENT: u16 = 1 << 1;