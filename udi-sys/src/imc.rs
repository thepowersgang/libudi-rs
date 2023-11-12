use ::core::ffi::c_void;
use super::{udi_cb_t, udi_channel_t, udi_index_t};

pub type udi_channel_event_ind_op_t = unsafe extern "C" fn(*mut udi_channel_event_cb_t);
pub type udi_channel_anchor_call_t = unsafe extern "C" fn(*mut udi_cb_t, udi_channel_t);
pub type udi_channel_spawn_call_t = unsafe extern "C" fn(*mut udi_cb_t, udi_channel_t);

extern "C" {
    pub fn udi_channel_anchor(cb: udi_channel_anchor_call_t, gcb: *mut udi_cb_t, channel: udi_channel_t, ops_idx: udi_index_t, channel_context: *mut c_void);
    pub fn udi_channel_spawn(cb: udi_channel_spawn_call_t, gcb: *mut udi_cb_t, channel: udi_channel_t, spawn_idx: udi_index_t, ops_idx: udi_index_t, channel_context: *mut c_void);
    pub fn udi_channel_set_context(channel: udi_channel_t, channel_context: *mut c_void);
    pub fn udi_channel_op_abort(channel: udi_channel_t, orig_cb: *mut udi_cb_t);
    pub fn udi_channel_close(channel: udi_channel_t);
    pub fn udi_channel_event_ind(cb: *mut udi_channel_event_cb_t);
    pub fn udi_channel_event_complete(cb: *mut udi_channel_event_cb_t, status: super::udi_status_t);
}

#[repr(C)]
pub struct udi_channel_event_cb_t
{
    pub gcb: super::udi_cb_t,
    pub event: u8,
    pub params: udi_channel_event_cb_t_params,
}

#[repr(C)]
pub union udi_channel_event_cb_t_params
{
    pub internal_bound: udi_channel_event_cb_t_params_internal_bound,
    pub parent_bound: udi_channel_event_cb_t_params_parent_bound,
    pub orig_cb: *mut super::udi_cb_t,
}
#[repr(C)]
#[derive(Copy,Clone)]
pub struct udi_channel_event_cb_t_params_internal_bound
{
    pub bind_cb: *mut super::udi_cb_t,
}
#[repr(C)]
#[derive(Copy,Clone)]
pub struct udi_channel_event_cb_t_params_parent_bound
{
    pub bind_cb: *mut super::udi_cb_t,
    pub parent_id: u8,
    pub path_handles: *const super::buf::udi_buf_path_t,
}
#[repr(u8)]
pub enum ChannelEvent {
    Closed,
    Bound,
    OpAborted,
}

pub const UDI_CHANNEL_CLOSED: u8 = 0;
pub const UDI_CHANNEL_BOUND: u8 = 1;
pub const UDI_CHANNEL_OP_ABORTED: u8 = 2;