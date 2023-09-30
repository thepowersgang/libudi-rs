use super::*;

pub type udi_intr_attach_ack_op_t = unsafe extern "C" fn(cb: *mut udi_intr_attach_cb_t, status: super::udi_status_t);
pub type udi_intr_detach_ack_op_t = unsafe extern "C" fn(cb: *mut udi_intr_detach_cb_t);
pub type udi_intr_event_ind_op_t = unsafe extern "C" fn(cb: *mut udi_intr_event_cb_t, flags: u8);

#[repr(C)]
pub struct udi_intr_handler_ops_t
{
    pub channel_event_ind_op: imc::udi_channel_event_ind_op_t,
    pub intr_event_ind_op: udi_intr_event_ind_op_t,
}
impl const crate::Ops for udi_intr_handler_ops_t {
    fn get_num() -> super::udi_index_t {
        // TODO: This is the number for the bus metalang, does that always apply?
        // - It's defined in `meta_intr.h` for acess2's impl
        3
    }
    fn to_ops_vector(&self) -> super::udi_ops_vector_t {
        self as *const _ as *const _
    }
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