//! Generic IO metalanguage

use crate::ffi::*;
use super::imc::udi_channel_event_ind_op_t;

macro_rules! a {
    ( $( $(#[$a:meta])* $op_name:ident = fn $name:ident($($a_name:ident: $a_ty:ty),*); )+ ) => {
        extern "C" {
            $( $(#[$a])* pub fn $name( $($a_name: $a_ty),* ); )+
        }
        $( pub type $op_name = unsafe extern "C" fn ( $($a_name: $a_ty),* ); )+
    };
}

a! {
    udi_gio_bind_req_op_t = fn udi_gio_bind_req(cb: *mut udi_gio_bind_cb_t);
    udi_gio_bind_ack_op_t = fn udi_gio_bind_ack(
        cb: *mut udi_gio_bind_cb_t,
        device_size_lo: udi_ubit32_t,
        device_size_hi: udi_ubit32_t,
        status: udi_status_t);
    
    udi_gio_unbind_req_op_t = fn udi_gio_unbind_req(cb: *mut udi_gio_bind_cb_t);
    udi_gio_unbind_ack_op_t = fn udi_gio_unbind_ack(cb: *mut udi_gio_bind_cb_t);

    udi_gio_xfer_req_op_t = fn udi_gio_xfer_req(cb: *mut udi_gio_xfer_cb_t);
    udi_gio_xfer_ack_op_t = fn udi_gio_xfer_ack(cb: *mut udi_gio_xfer_cb_t);
    udi_gio_xfer_nak_op_t = fn udi_gio_xfer_nak(cb: *mut udi_gio_xfer_cb_t, status: udi_status_t);

    udi_gio_event_ind_op_t = fn udi_gio_event_ind(cb: *mut udi_gio_event_cb_t);
    udi_gio_event_res_op_t = fn udi_gio_event_res(cb: *mut udi_gio_event_cb_t);
}
extern "C" {
    fn udi_gio_event_ind_unused(cb: *mut udi_gio_event_cb_t);
    fn udi_gio_event_res_unused(cb: *mut udi_gio_event_cb_t);
}

impl_metalanguage!{
    static METALANG_SPEC;
    NAME udi_gio;
    OPS
        1 => udi_gio_provider_ops_t,
        2 => udi_gio_client_ops_t,
        ;
    CBS
        1 => udi_gio_bind_cb_t,
        2 => udi_gio_xfer_cb_t,
        3 => udi_gio_event_cb_t,
        ;
}

#[repr(C)]
pub struct udi_gio_provider_ops_t
{
    pub channel_event_ind_op: udi_channel_event_ind_op_t,
    pub gio_bind_req_op: udi_gio_bind_req_op_t,
    pub gio_unbind_req_op: udi_gio_unbind_req_op_t,
    pub gio_xfer_req_op: udi_gio_xfer_req_op_t,
    pub gio_event_res_op: udi_gio_event_res_op_t,
}
#[repr(C)]
pub struct udi_gio_client_ops_t
{
    pub channel_event_ind_op: udi_channel_event_ind_op_t,
    pub gio_bind_ack_op: udi_gio_bind_ack_op_t,
    pub gio_unbind_ack_op: udi_gio_unbind_ack_op_t,
    pub gio_xfer_ack_op: udi_gio_xfer_ack_op_t,
    pub gio_xfer_nak_op: udi_gio_xfer_nak_op_t,
    pub gio_event_ind_op: udi_gio_event_ind_op_t,
}

#[repr(C)]
pub struct udi_gio_bind_cb_t
{
    pub gcb: udi_cb_t,
    pub xfer_constraints: crate::ffi::buf::udi_xfer_constraints_t,
}
#[repr(C)]
pub struct udi_gio_xfer_cb_t
{
    pub gcb: udi_cb_t,
    pub op: udi_gio_op_t,
    pub tr_params: *mut c_void,
    pub data_buf: *mut udi_buf_t,
}

pub type udi_gio_op_t = udi_ubit8_t;
/* Limit values for udi_gio_op_t */
pub const UDI_GIO_OP_CUSTOM: udi_gio_op_t = 16;
pub const UDI_GIO_OP_MAX:  udi_gio_op_t = 64;
/* Direction flag values for op */
pub const UDI_GIO_DIR_READ : udi_gio_op_t = 1<<6;
pub const UDI_GIO_DIR_WRITE: udi_gio_op_t = 1<<7;
/* Standard Operation Codes */
pub const UDI_GIO_OP_READ : udi_gio_op_t = UDI_GIO_DIR_READ;
pub const UDI_GIO_OP_WRITE: udi_gio_op_t = UDI_GIO_DIR_WRITE;

#[repr(C)]
pub struct udi_gio_rw_params_t
{
    pub offset_lo: udi_ubit32_t,
    pub offset_hi: udi_ubit32_t,
}

#[repr(C)]
pub struct udi_gio_event_cb_t
{
    pub gcb: udi_cb_t,
    pub event_code: udi_ubit8_t,
    pub event_params: *mut c_void,
}