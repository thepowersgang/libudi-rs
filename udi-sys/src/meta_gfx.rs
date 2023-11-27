//! UDI GFX Bindings (draft)
//! 
// From public domain header by "Marcel Sondaar"
// See https://mysticos.combuster.nl/downloads/mos-nightly-source.tar.gz
// - mos/include/common/udi_gfx.h

use crate::{udi_cb_t, udi_status_t, udi_buf_t};
use crate::{udi_index_t, udi_ubit32_t};

pub macro metalang_name( $($prefix:ident::)* ) {
    $($prefix::)*udi_gfx
}

macro_rules! a {
    ( $( $(#[$a:meta])* $op_name:ident = fn $name:ident($($a_name:ident: $a_ty:ty),* $(,)?); )+ ) => {
        extern "C" {
            $( $(#[$a])* pub fn $name( $($a_name: $a_ty),* ); )+
        }
        $( pub type $op_name = unsafe extern "C" fn ( $($a_name: $a_ty),* ); )+
    };
}

#[repr(C)]
pub struct udi_gfx_provider_ops_t {
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub gfx_bind_req_op: udi_gfx_bind_req_op_t,
    pub gfx_unbind_req_op: udi_gfx_unbind_req_op_t,
    pub gfx_set_connector_req_op: udi_gfx_set_connector_req_op_t,
    pub gfx_set_engine_req_op: udi_gfx_set_engine_req_op_t,
    pub gfx_get_connector_req_op: udi_gfx_get_connector_req_op_t,
    pub gfx_get_engine_req_op: udi_gfx_get_engine_req_op_t,
    pub gfx_range_connector_req_op: udi_gfx_range_connector_req_op_t,
    pub gfx_range_engine_req_op: udi_gfx_range_engine_req_op_t,
    pub gfx_get_engine_operator_req_op: udi_gfx_get_engine_operator_req_op_t,
    pub gfx_connector_command_req_op: udi_gfx_connector_command_req_op_t,
    pub gfx_engine_command_req_op: udi_gfx_engine_command_req_op_t,
    pub gfx_buffer_info_req_op: udi_gfx_buffer_info_req_op_t,
    pub gfx_buffer_read_req_op: udi_gfx_buffer_read_req_op_t,
    pub gfx_buffer_write_req_op: udi_gfx_buffer_write_req_op_t,
}
pub const UDI_GFX_PROVIDER_OPS_NUM: u8 = 1;

#[repr(C)]
pub struct udi_gfx_client_ops_t {
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub gfx_bind_ack_op: udi_gfx_bind_ack_op_t,
    pub gfx_unbind_ack_op: udi_gfx_unbind_ack_op_t,
    pub gfx_set_connector_ack_op: udi_gfx_set_connector_ack_op_t,
    pub gfx_set_engine_ack_op: udi_gfx_set_engine_ack_op_t,
    pub gfx_set_connector_nak_op: udi_gfx_set_connector_nak_op_t,
    pub gfx_set_engine_nak_op: udi_gfx_set_engine_nak_op_t,
    pub gfx_get_connector_ack_op: udi_gfx_get_connector_ack_op_t,
    pub gfx_get_engine_ack_op: udi_gfx_get_engine_ack_op_t,
    pub gfx_range_connector_ack_op: udi_gfx_range_connector_ack_op_t,
    pub gfx_range_engine_ack_op: udi_gfx_range_engine_ack_op_t,
    pub gfx_get_engine_operator_ack_op: udi_gfx_get_engine_operator_req_op_t,
    pub gfx_connector_command_ack_op: udi_gfx_connector_command_ack_op_t,
    pub gfx_engine_command_ack_op: udi_gfx_engine_command_ack_op_t,
    pub gfx_buffer_info_ack_op: udi_gfx_buffer_info_ack_op_t,
    pub gfx_buffer_read_ack_op: udi_gfx_buffer_read_ack_op_t,
    pub gfx_buffer_write_ack_op: udi_gfx_buffer_write_ack_op_t,
    pub gfx_buffer_read_nak_op: udi_gfx_buffer_read_nak_op_t,
    pub gfx_buffer_write_nak_op: udi_gfx_buffer_write_nak_op_t,
}
pub const UDI_GFX_CLIENT_OPS_NUM: u8 = 1;

#[repr(C)]
pub struct udi_gfx_bind_cb_t {
    pub gcb: udi_cb_t,
}
pub const UDI_GFX_BIND_CB_NUM: u8 = 1;
a!{
    udi_gfx_bind_req_op_t = fn udi_gfx_bind_req(cb: *mut udi_gfx_bind_cb_t);
    udi_gfx_bind_ack_op_t = fn udi_gfx_bind_ack(
        cb: *mut udi_gfx_bind_cb_t,
        sockets: udi_index_t,
        engines: udi_index_t,
        status: udi_status_t,
    );
    udi_gfx_unbind_req_op_t = fn udi_gfx_unbind_req(cb: *mut udi_gfx_bind_cb_t);
    udi_gfx_unbind_ack_op_t = fn udi_gfx_unbind_ack(cb: *mut udi_gfx_bind_cb_t);
}


#[repr(C)]
pub struct udi_gfx_state_cb_t {
    pub gcb: udi_cb_t,
    pub subsystem: udi_ubit32_t,
    pub attribute: udi_ubit32_t,
}
pub const UDI_GFX_STATE_CB_NUM: u8 = 2;
a!{
    udi_gfx_set_engine_req_op_t     = fn udi_gfx_set_engine_req(cb: *mut udi_gfx_state_cb_t, value: udi_ubit32_t);
    udi_gfx_set_connector_req_op_t  = fn udi_gfx_set_connector_req (cb: *mut udi_gfx_state_cb_t, value: udi_ubit32_t);
    udi_gfx_set_engine_ack_op_t     = fn udi_gfx_set_engine_ack(cb: *mut udi_gfx_state_cb_t);
    udi_gfx_set_connector_ack_op_t  = fn udi_gfx_set_connector_ack (cb: *mut udi_gfx_state_cb_t);
    udi_gfx_get_engine_req_op_t     = fn udi_gfx_get_engine_req(cb: *mut udi_gfx_state_cb_t);
    udi_gfx_get_connector_req_op_t  = fn udi_gfx_get_connector_req (cb: *mut udi_gfx_state_cb_t);
    udi_gfx_get_engine_ack_op_t     = fn udi_gfx_get_engine_ack(cb: *mut udi_gfx_state_cb_t, value: udi_ubit32_t);
    udi_gfx_get_connector_ack_op_t  = fn udi_gfx_get_connector_ack (cb: *mut udi_gfx_state_cb_t, value: udi_ubit32_t);
    udi_gfx_set_engine_nak_op_t     = fn udi_gfx_set_engine_nak(cb: *mut udi_gfx_state_cb_t, status: udi_status_t);
    udi_gfx_get_connector_nak_op_t  = fn udi_gfx_get_connector_nak (cb: *mut udi_gfx_state_cb_t, status: udi_status_t);
    udi_gfx_set_connector_nak_op_t  = fn udi_gfx_set_connector_nak(cb: *mut udi_gfx_state_cb_t, status: udi_status_t);
}

#[repr(C)]
pub struct udi_gfx_range_cb_t {
    pub gcb: udi_cb_t,
    pub subsystem: udi_ubit32_t,
    pub attribute: udi_ubit32_t,
    pub rangedata: *mut udi_buf_t,
}
pub const UDI_GFX_RANGE_CB_NUM: u8 = 3;
a!{
    udi_gfx_range_engine_req_op_t    = fn udi_gfx_range_engine_req(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_range_connector_req_op_t = fn udi_gfx_range_connector_req(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_range_engine_ack_op_t    = fn udi_gfx_range_engine_ack(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_range_connector_ack_op_t = fn udi_gfx_range_connector_ack(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_get_engine_operator_req_op_t = fn udi_gfx_get_engine_operator_req(cb: *mut udi_gfx_range_cb_t);
    udi_gfx_get_engine_operator_ack_op_t = fn udi_gfx_get_engine_operator_ack(
        cb: *mut udi_gfx_range_cb_t,
        op: udi_ubit32_t,
        arg1: udi_ubit32_t,
        arg2: udi_ubit32_t,
        arg3: udi_ubit32_t,
    );
}


#[repr(C)]
pub struct udi_gfx_command_cb_t {
    pub gcb: udi_cb_t,
    pub commanddata: *mut udi_buf_t,
}
pub const UDI_GFX_COMMAND_CB_NUM: u8 = 4;
a!{
    udi_gfx_connector_command_req_op_t  = fn udi_gfx_connector_command_req(cb: *mut udi_gfx_command_cb_t);
    udi_gfx_engine_command_req_op_t     = fn udi_gfx_engine_command_req(cb: *mut udi_gfx_command_cb_t);
    udi_gfx_connector_command_ack_op_t  = fn udi_gfx_connector_command_ack(cb: *mut udi_gfx_command_cb_t);
    udi_gfx_engine_command_ack_op_t     = fn udi_gfx_engine_command_ack(cb: *mut udi_gfx_command_cb_t);
}

#[repr(C)]
pub struct udi_gfx_buffer_info_cb_t {
    pub gcb: udi_cb_t,
    pub buffer_index: udi_ubit32_t,
}
//pub const UDI_GFX_BUFFER_INFO_CB_NUM: u8 = 5;
a!{
    udi_gfx_buffer_info_req_op_t = fn udi_gfx_buffer_info_req(cb: *mut udi_gfx_buffer_info_cb_t);
    udi_gfx_buffer_info_ack_op_t = fn udi_gfx_buffer_info_ack(
        cb: *mut udi_gfx_buffer_info_cb_t,
        width: udi_ubit32_t,
        height: udi_ubit32_t,
        bitsper: udi_ubit32_t,
        flags: udi_ubit32_t,
    );
}


#[repr(C)]
pub struct udi_gfx_buffer_cb_t {
    pub gcb: udi_cb_t,
    pub buffer_index: udi_ubit32_t,
    pub x: udi_ubit32_t,
    pub y: udi_ubit32_t,
    pub width: udi_ubit32_t,
    pub height: udi_ubit32_t,
    pub buffer: *mut udi_buf_t,
}
//pub const UDI_GFX_BUFFER_CB_NUM: u8 = 6;
a!{
    udi_gfx_buffer_write_req_op_t = fn udi_gfx_buffer_write_req(cb: *mut udi_gfx_buffer_cb_t);
    udi_gfx_buffer_read_req_op_t  = fn udi_gfx_buffer_read_req (cb: *mut udi_gfx_buffer_cb_t);
    udi_gfx_buffer_write_ack_op_t = fn udi_gfx_buffer_write_ack(cb: *mut udi_gfx_buffer_cb_t);
    udi_gfx_buffer_read_ack_op_t  = fn udi_gfx_buffer_read_ack (cb: *mut udi_gfx_buffer_cb_t);
    udi_gfx_buffer_write_nak_op_t = fn udi_gfx_buffer_write_nak(cb: *mut udi_gfx_buffer_cb_t, status: udi_status_t);
    udi_gfx_buffer_read_nak_op_t  = fn udi_gfx_buffer_read_nak (cb: *mut udi_gfx_buffer_cb_t, status: udi_status_t);
}

