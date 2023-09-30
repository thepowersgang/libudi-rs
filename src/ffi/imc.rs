pub type udi_channel_event_ind_op_t = unsafe extern "C" fn(*mut udi_channel_event_cb_t);

#[repr(C)]
pub struct udi_channel_event_cb_t
{
    gcb: super::udi_cb_t,
    event: u8,
    params: udi_channel_event_cb_t_params,
}
#[repr(C)]
pub union udi_channel_event_cb_t_params
{
    internal_bound: udi_channel_event_cb_t_params_internal_bound,
    parent_bound: udi_channel_event_cb_t_params_parent_bound,
    orig_cb: *mut super::udi_cb_t,
}
#[repr(C)]
#[derive(Copy,Clone)]
pub struct udi_channel_event_cb_t_params_internal_bound
{
    bind_cb: *mut super::udi_cb_t,
}
#[repr(C)]
#[derive(Copy,Clone)]
pub struct udi_channel_event_cb_t_params_parent_bound
{
    bind_cb: *mut super::udi_cb_t,
    parent_id: u8,
    //path_handles: *const udi_buf_path_t,
}
#[repr(u8)]
pub enum ChannelEvent {
    Closed,
    Bound,
    OpAborted,
}