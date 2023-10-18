use ::core::ffi::c_void;
use super::{udi_cb_t, udi_channel_t, udi_index_t};

pub type udi_channel_event_ind_op_t = unsafe extern "C" fn(*mut udi_channel_event_cb_t);
pub type udi_channel_anchor_call_t = unsafe extern "C" fn(*mut udi_cb_t, udi_channel_t);
pub type udi_channel_spawn_call_t = unsafe extern "C" fn(*mut udi_cb_t, udi_channel_t);

extern "C" {
    pub fn udi_channel_anchor(cb: udi_channel_anchor_call_t, gcb: *mut udi_cb_t, channel: udi_channel_t, ops_idx: udi_index_t, channel_context: *mut c_void);
    pub fn udi_channel_spawn(cb: udi_channel_spawn_call_t, gcb: *mut udi_cb_t, channel: udi_channel_t, spawn_idx: udi_index_t, ops_idx: udi_index_t, channel_context: *mut c_void);
    pub fn udi_channel_event_complete(cb: *mut udi_channel_event_cb_t, status: super::udi_status_t);
}

#[repr(C)]
pub struct udi_channel_event_cb_t
{
    pub gcb: super::udi_cb_t,
    pub event: u8,
    pub params: udi_channel_event_cb_t_params,
}
unsafe impl crate::async_trickery::GetCb for udi_channel_event_cb_t {
    fn get_gcb(&self) -> &super::udi_cb_t {
        &self.gcb
    }
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