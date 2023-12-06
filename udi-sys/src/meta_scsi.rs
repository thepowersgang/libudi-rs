//! SCSI Metalanguage
//! 
//! See `scsi_driver_spec.pdf`

use crate::{udi_cb_t, udi_status_t, udi_buf_t};
use crate::{udi_ubit8_t, udi_ubit16_t, udi_ubit32_t};

pub macro metalang_name( $($prefix:ident::)* ) {
    $($prefix::)*udi_scsi
}

#[repr(C)]
pub struct udi_scsi_pd_ops_t
{
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub bind_ack_op: udi_scsi_bind_ack_op_t,
    pub unbind_ack_op: udi_scsi_unbind_ack_op_t,
    pub io_ack_op: udi_scsi_io_ack_op_t,
    pub io_nak_op: udi_scsi_io_nak_op_t,
    pub ctl_ack_op: udi_scsi_ctl_ack_op_t,
    pub event_ind_op: udi_scsi_event_ind_op_t,
}
pub const UDI_SCSI_PD_OPS_NUM: u8 = 1;
#[repr(C)]
pub struct udi_scsi_hd_ops_t
{
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub bind_req_op: udi_scsi_bind_req_op_t,
    pub unbind_req_op: udi_scsi_unbind_req_op_t,
    pub io_req_op: udi_scsi_io_req_op_t,
    pub ctl_req_op: udi_scsi_ctl_req_op_t,
    pub event_res_op: udi_scsi_event_res_op_t,
}
pub const UDI_SCSI_HD_OPS_NUM: u8 = 2;

#[repr(C)]
pub struct udi_scsi_bind_cb_t
{
    pub gcb: udi_cb_t,
    pub events: udi_ubit16_t,
}
pub const UDI_SCSI_BIND_CB_NUM: u8 = 1;

pub const UDI_SCSI_EVENT_AEN: udi_ubit16_t = 1 << 0;
pub const UDI_SCSI_EVENT_TGT_RESET: udi_ubit16_t = 1 << 1;
pub const UDI_SCSI_EVENT_BUS_RESET: udi_ubit16_t = 1 << 2;
pub const UDI_SCSI_EVENT_UNSOLICITED_RESELECT: udi_ubit16_t = 1 << 3;

extern "C" {
    pub fn udi_scsi_bind_req(
        cb: *mut udi_scsi_bind_cb_t,
        bind_flags: udi_ubit16_t,
        queue_depth: udi_ubit16_t,
        max_sense_len: udi_ubit16_t,
        aen_buf_size: udi_ubit16_t
    );
    pub fn udi_scsi_bind_ack (
        cb: *mut udi_scsi_bind_cb_t,
        hd_timeout_increase: udi_ubit32_t,
        status: udi_status_t,
    );
    pub fn udi_scsi_unbind_req(cb: *mut udi_scsi_bind_cb_t);
    pub fn udi_scsi_unbind_ack(cb: *mut udi_scsi_bind_cb_t);
}

pub type udi_scsi_bind_req_op_t = unsafe extern "C" fn(
    cb: *mut udi_scsi_bind_cb_t,
    bind_flags: udi_ubit16_t,
    queue_depth: udi_ubit16_t,
    max_sense_len: udi_ubit16_t,
    aen_buf_size: udi_ubit16_t
);
pub type udi_scsi_bind_ack_op_t = unsafe extern "C" fn(
    cb: *mut udi_scsi_bind_cb_t,
    hd_timeout_increase: udi_ubit32_t,
    status: udi_status_t,
);
pub type udi_scsi_unbind_req_op_t = unsafe extern "C" fn(cb: *mut udi_scsi_bind_cb_t);
pub type udi_scsi_unbind_ack_op_t = unsafe extern "C" fn(cb: *mut udi_scsi_bind_cb_t);
pub const UDI_SCSI_BIND_EXCLUSIVE: udi_ubit16_t = 1 << 0;
pub const UDI_SCSI_TEMP_BIND_EXCLUSIVE: udi_ubit16_t = 1 << 1;



#[repr(C)]
pub struct udi_scsi_io_cb_t
{
    pub gcb: udi_cb_t,
    pub data_buf: *mut udi_buf_t,
    pub timeout: udi_ubit32_t,
    pub flags: udi_ubit16_t,
    pub attribute: udi_ubit8_t,
    pub cdb_len: udi_ubit8_t,
    pub cdb_ptr: *mut udi_ubit8_t,
}
pub const UDI_SCSI_IO_CB_NUM: u8 = 2;
/* I/O Request Flags */
pub const UDI_SCSI_DATA_IN: udi_ubit16_t   = 1<<0;
pub const UDI_SCSI_DATA_OUT: udi_ubit16_t  = 1<<1;
pub const UDI_SCSI_NO_DISCONNECT: udi_ubit16_t = 1<<2;
/* SCSI Task Attributes */
pub const UDI_SCSI_SIMPLE_TASK: udi_ubit16_t = 1;
pub const UDI_SCSI_ORDERED_TASK: udi_ubit16_t = 2;
pub const UDI_SCSI_HEAD_OF_Q_TASK: udi_ubit16_t = 3;
pub const UDI_SCSI_ACA_TASK: udi_ubit16_t = 4;
pub const UDI_SCSI_UNTAGGED_TASK: udi_ubit16_t = 5;

extern "C" {
    pub fn udi_scsi_io_req(cb: *mut udi_scsi_io_cb_t);
    pub fn udi_scsi_io_ack(cb: *mut udi_scsi_io_cb_t);
    pub fn udi_scsi_io_nak(cb: *mut udi_scsi_io_cb_t, status: udi_scsi_status_t, sense_buf: *mut udi_buf_t);
}
pub type udi_scsi_io_req_op_t = unsafe extern "C" fn(cb: *mut udi_scsi_io_cb_t);
pub type udi_scsi_io_ack_op_t = unsafe extern "C" fn(cb: *mut udi_scsi_io_cb_t);
pub type udi_scsi_io_nak_op_t = unsafe extern "C" fn(cb: *mut udi_scsi_io_cb_t, status: udi_scsi_status_t, sense_buf: *mut udi_buf_t);
#[repr(C)]
pub struct udi_scsi_status_t
{
    pub req_status: udi_status_t,
    pub scsi_status: udi_ubit8_t,
    pub sense_status: udi_ubit8_t,
}

// ------ Control Operations ---------
#[repr(C)]
pub struct udi_scsi_ctl_cb_t
{
    pub gcb: udi_cb_t,
    pub ctrl_func: udi_ubit8_t,
    pub queue_depth: udi_ubit16_t,
}
pub const UDI_SCSI_CTL_CB_NUM: u8 = 3;
/* Values for ctrl_func */
pub const UDI_SCSI_CTL_ABORT_TASK_SET: udi_ubit8_t = 1;
pub const UDI_SCSI_CTL_CLEAR_TASK_SET: udi_ubit8_t = 2;
pub const UDI_SCSI_CTL_LUN_RESET: udi_ubit8_t = 3;
pub const UDI_SCSI_CTL_TGT_RESET: udi_ubit8_t = 4;
pub const UDI_SCSI_CTL_BUS_RESET: udi_ubit8_t = 5;
pub const UDI_SCSI_CTL_CLEAR_ACA: udi_ubit8_t = 6;
pub const UDI_SCSI_CTL_SET_QUEUE_DEPTH: udi_ubit8_t = 7;

extern "C" {
    pub fn udi_scsi_ctl_req(cb: *mut udi_scsi_ctl_cb_t);
    pub fn udi_scsi_ctl_ack(cb: *mut udi_scsi_ctl_cb_t, status: udi_status_t);
}
pub type udi_scsi_ctl_req_op_t = unsafe extern "C" fn(cb: *mut udi_scsi_ctl_cb_t);
pub type udi_scsi_ctl_ack_op_t = unsafe extern "C" fn(cb: *mut udi_scsi_ctl_cb_t, status: udi_status_t);


// ------- Event Operations -------
#[repr(C)]
pub struct udi_scsi_event_cb_t
{
    pub gcb: udi_cb_t,
    pub event: udi_ubit8_t,
    pub aen_data_buf: *mut udi_buf_t,
}
pub const UDI_SCSI_EVENT_CB_NUM: u8 = 4;
extern "C" {
    pub fn udi_scsi_event_ind(cb: *mut udi_scsi_event_cb_t);
    pub fn udi_scsi_event_ind_unused(cb: *mut udi_scsi_event_cb_t);
    pub fn udi_scsi_event_res(cb: *mut udi_scsi_event_cb_t);
}
pub type udi_scsi_event_ind_op_t = unsafe extern "C" fn (cb: *mut udi_scsi_event_cb_t);
pub type udi_scsi_event_res_op_t = unsafe extern "C" fn (cb: *mut udi_scsi_event_cb_t);

extern "C" {
    pub fn udi_scsi_inquiry_to_string(
        inquiry_data: *const udi_ubit8_t,
        inquiry_len: crate::udi_size_t,
        str: *const ::core::ffi::c_char
    );
}