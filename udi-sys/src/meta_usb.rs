//! USB Metalanguage, as defined in `udbdi10.pdf`
//! 
//! <https://www.usb.org/document-library/openusbdi-specification-10>
//! 
//! NOTE: This spec seems incomplete, there's at least one bug, and it was written for UDI 0x095, not 0x101
use super::*;
use super::imc::udi_channel_event_ind_op_t;

macro_rules! define_calls {
    ( $( $(#[$a:meta])* $op_name:ident = fn $name:ident($($a_name:ident: $a_ty:ty),*); )+ ) => {
        extern "C" {
            $( $(#[$a])* pub fn $name( $($a_name: $a_ty),* ); )+
        }
        $( pub type $op_name = unsafe extern "C" fn ( $($a_name: $a_ty),* ); )+
    };
}
define_calls!{
    usbdi_bind_req_op_t = fn usbdi_bind_req(cb: *mut usbdi_misc_cb_t);
    /// `n_intfc` is the number of interfaces the LDD is in control of
    usbdi_bind_ack_op_t = fn usbdi_bind_ack(cb: *mut usbdi_misc_cb_t, n_intfc: udi_index_t, status: udi_status_t);
    usbdi_unbind_req_op_t = fn usbdi_unbind_req(cb: *mut usbdi_misc_cb_t);
    usbdi_unbind_ack_op_t = fn usbdi_unbind_ack(cb: *mut usbdi_misc_cb_t, status: udi_status_t);
    usbdi_intfc_open_req_op_t = fn usbdi_intfc_open_req(cb: *mut usbdi_misc_cb_t, alternate_intfc: udi_ubit8_t, open_flag: udi_ubit8_t);
    usbdi_intfc_open_ack_op_t = fn usbdi_intfc_open_ack(cb: *mut usbdi_misc_cb_t, n_edpt: udi_index_t, status: udi_status_t);
    usbdi_intfc_close_req_op_t = fn usbdi_intfc_close_req(cb: *mut usbdi_misc_cb_t);
    usbdi_intfc_close_ack_op_t = fn usbdi_intfc_close_ack(cb: *mut usbdi_misc_cb_t, status: udi_status_t);

    usbdi_intr_bulk_xfer_req_op_t = fn usbdi_intr_bulk_xfer_req(cb: *mut usbdi_intr_bulk_xfer_cb_t);
    usbdi_intr_bulk_xfer_ack_op_t = fn usbdi_intr_bulk_xfer_ack(cb: *mut usbdi_intr_bulk_xfer_cb_t);
    usbdi_intr_bulk_xfer_nak_op_t = fn usbdi_intr_bulk_xfer_nak(cb: *mut usbdi_intr_bulk_xfer_cb_t, status: udi_status_t);

    usbdi_control_xfer_req_op_t = fn usbdi_control_xfer_req(cb: *mut usbdi_control_xfer_cb_t);
    usbdi_control_xfer_ack_op_t = fn usbdi_control_xfer_ack(cb: *mut usbdi_control_xfer_cb_t, status: udi_status_t);

    usbdi_isoc_xfer_req_op_t = fn usbdi_isoc_xfer_req(cb: *mut usbdi_isoc_xfer_cb_t);
    usbdi_isoc_xfer_ack_op_t = fn usbdi_isoc_xfer_ack(cb: *mut usbdi_isoc_xfer_cb_t);
    usbdi_isoc_xfer_nak_op_t = fn usbdi_isoc_xfer_nak(cb: *mut usbdi_isoc_xfer_cb_t, status: udi_status_t);

    usbdi_frame_number_req_op_t = fn usbdi_frame_number_req(cb: *mut usbdi_misc_cb_t);
    usbdi_frame_number_ack_op_t = fn usbdi_frame_number_ack(cb: *mut usbdi_misc_cb_t, frame_number: udi_ubit32_t);

    usbdi_device_speed_req_op_t = fn usbdi_device_speed_req(cb: *mut usbdi_misc_cb_t);
    usbdi_device_speed_ack_op_t = fn usbdi_device_speed_ack(cb: *mut usbdi_misc_cb_t, device_speed: udi_ubit8_t);

    usbdi_reset_device_req_op_t = fn usbdi_reset_device_req(cb: *mut usbdi_misc_cb_t);
    usbdi_reset_device_ack_op_t = fn usbdi_reset_device_ack(cb: *mut usbdi_misc_cb_t);

    usbdi_pipe_abort_req_op_t = fn usbdi_pipe_abort_req(cb: *mut usbdi_misc_cb_t);
    usbdi_pipe_abort_ack_op_t = fn usbdi_pipe_abort_ack(cb: *mut usbdi_misc_cb_t);

    usbdi_intfc_abort_req_op_t = fn usbdi_intfc_abort_req(cb: *mut usbdi_misc_cb_t);
    usbdi_intfc_abort_ack_op_t = fn usbdi_intfc_abort_ack(cb: *mut usbdi_misc_cb_t);

    usbdi_pipe_state_set_req_op_t = fn usbdi_pipe_state_set_req(cb: *mut usbdi_state_cb_t);
    usbdi_pipe_state_set_ack_op_t = fn usbdi_pipe_state_set_ack(cb: *mut usbdi_state_cb_t, status: udi_status_t);
    usbdi_pipe_state_get_req_op_t = fn usbdi_pipe_state_get_req(cb: *mut usbdi_state_cb_t);
    usbdi_pipe_state_get_ack_op_t = fn usbdi_pipe_state_get_ack(cb: *mut usbdi_state_cb_t);

    usbdi_edpt_state_set_req_op_t = fn usbdi_edpt_state_set_req(cb: *mut usbdi_state_cb_t);
    usbdi_edpt_state_set_ack_op_t = fn usbdi_edpt_state_set_ack(cb: *mut usbdi_state_cb_t, status: udi_status_t);
    usbdi_edpt_state_get_req_op_t = fn usbdi_edpt_state_get_req(cb: *mut usbdi_state_cb_t);
    usbdi_edpt_state_get_ack_op_t = fn usbdi_edpt_state_get_ack(cb: *mut usbdi_state_cb_t, status: udi_status_t);

    usbdi_intfc_state_set_req_op_t = fn usbdi_intfc_state_set_req(cb: *mut usbdi_state_cb_t);
    usbdi_intfc_state_set_ack_op_t = fn usbdi_intfc_state_set_ack(cb: *mut usbdi_state_cb_t, status: udi_status_t);
    usbdi_intfc_state_get_req_op_t = fn usbdi_intfc_state_get_req(cb: *mut usbdi_state_cb_t);
    usbdi_intfc_state_get_ack_op_t = fn usbdi_intfc_state_get_ack(cb: *mut usbdi_state_cb_t);

    usbdi_device_state_get_req_op_t = fn usbdi_device_state_get_req(cb: *mut usbdi_state_cb_t);
    usbdi_device_state_get_ack_op_t = fn usbdi_device_state_get_ack(cb: *mut usbdi_state_cb_t);

    usbdi_desc_req_op_t = fn usbdi_desc_req(cb: *mut usbdi_desc_cb_t);
    usbdi_desc_ack_op_t = fn usbdi_desc_ack(cb: *mut usbdi_desc_cb_t, status: udi_status_t);

    usbdi_config_set_req_op_t = fn usbdi_config_set_req(cb: *mut usbdi_misc_cb_t, config_value: udi_ubit16_t);
    usbdi_config_set_ack_op_t = fn usbdi_config_set_ack(cb: *mut usbdi_misc_cb_t, status: udi_status_t);

    usbdi_async_event_ind_op_t = fn usbdi_async_event_ind(cb: *mut usbdi_misc_cb_t, async_event: udi_ubit16_t);
    usbdi_async_event_res_op_t = fn usbdi_async_event_res(cb: *mut usbdi_misc_cb_t);
}
extern "C" {
    pub fn usbdi_intr_bulk_xfer_ack_unused(cb: *mut usbdi_intr_bulk_xfer_cb_t);
    pub fn usbdi_intr_bulk_xfer_nak_unused(cb: *mut usbdi_intr_bulk_xfer_cb_t, status: udi_status_t);
    pub fn usbdi_control_xfer_ack_unused(cb: *mut usbdi_control_xfer_cb_t, status: udi_status_t);
    pub fn usbdi_isoc_xfer_ack_unused(cb: *mut usbdi_isoc_xfer_cb_t);
    pub fn usbdi_isoc_xfer_nak_unused(cb: *mut usbdi_isoc_xfer_cb_t, status: udi_status_t);
    pub fn usbdi_frame_number_ack_unused(cb: *mut usbdi_misc_cb_t, frame_number: udi_ubit32_t);
    pub fn usbdi_device_speed_ack_unused(cb: *mut usbdi_misc_cb_t, device_speed: udi_ubit8_t);
    pub fn usbdi_reset_device_ack_unused(cb: *mut usbdi_misc_cb_t);
    pub fn usbdi_pipe_abort_ack_unused(cb: *mut usbdi_misc_cb_t);
    pub fn usbdi_intfc_abort_ack_unused(cb: *mut usbdi_misc_cb_t);
    pub fn usbdi_pipe_state_set_ack_unused(cb: *mut usbdi_state_cb_t, status: udi_status_t);
    pub fn usbdi_pipe_state_get_ack_unused(cb: *mut usbdi_state_cb_t);
    pub fn usbdi_endpt_state_set_ack_unused(cb: *mut usbdi_state_cb_t, status: udi_status_t);
    pub fn usbdi_endpt_state_get_ack_unused(cb: *mut usbdi_state_cb_t, status: udi_status_t);
    pub fn usbdi_intfc_state_set_ack_unused(cb: *mut usbdi_state_cb_t, status: udi_status_t);
    pub fn usbdi_intfc_state_get_ack_unused(cb: *mut usbdi_state_cb_t);
    pub fn usbdi_device_state_get_req_unused(cb: *mut usbdi_state_cb_t);
    pub fn usbdi_desc_ack_unused(cb: *mut usbdi_desc_cb_t, status: udi_status_t);
    pub fn usbdi_config_set_ack_unused(cb: *mut usbdi_misc_cb_t, status: udi_status_t);
    pub fn usbdi_async_event_ind_unused(cb: *mut usbdi_misc_cb_t, async_event: udi_ubit16_t);

    pub fn usbdi_isoc_frame_distrib(cb: *mut usbdi_isoc_xfer_cb_t) -> udi_status_t;
}

#[repr(C)]
pub struct usbdi_ldd_intfc_ops_t
{
    pub /*udi_channel_*/event_ind_op:   udi_channel_event_ind_op_t,
    pub /*usbdi_*/bind_ack_op:  usbdi_bind_ack_op_t,
    pub /*usbdi_*/unbind_ack_op:    usbdi_unbind_ack_op_t,
    pub /*usbdi_*/intfc_open_ack_op:    usbdi_intfc_open_ack_op_t,
    pub /*usbdi_*/intfc_close_ack_op:   usbdi_intfc_close_ack_op_t,
    pub /*usbdi_*/frame_number_ack_op:  usbdi_frame_number_ack_op_t,
    pub /*usbdi_*/device_speed_ack_op:  usbdi_device_speed_ack_op_t,
    pub /*usbdi_*/reset_device_ack_op:  usbdi_reset_device_ack_op_t,
    pub /*usbdi_*/intfc_abort_ack_op:   usbdi_intfc_abort_ack_op_t,
    pub /*usbdi_*/intfc_state_set_ack_op:   usbdi_intfc_state_set_ack_op_t,
    pub /*usbdi_*/intfc_state_get_ack_op:   usbdi_intfc_state_get_ack_op_t,
    pub /*usbdi_*/desc_ack_op:  usbdi_desc_ack_op_t,
    pub /*usbdi_*/device_state_get_ack_op:  usbdi_device_state_get_ack_op_t,
    pub /*usbdi_*/config_set_ack_op:    usbdi_config_set_ack_op_t,
    pub /*usbdi_*/async_event_ind_op:   usbdi_async_event_ind_op_t,
}
pub const USBDI_LDD_INTFC_OPS_NUM: u8 = 1;

#[repr(C)]
pub struct usbdi_ldd_pipe_ops_t
{
    pub /*udi_channel_*/event_ind_op:   udi_channel_event_ind_op_t,
    pub /*usbdi_*/intr_bulk_xfer_ack_op: usbdi_intr_bulk_xfer_ack_op_t,
    pub /*usbdi_*/intr_bulk_xfer_nak_op: usbdi_intr_bulk_xfer_nak_op_t,
    pub /*usbdi_*/control_xfer_ack_op: usbdi_control_xfer_ack_op_t,
    pub /*usbdi_*/isoc_xfer_ack_op: usbdi_isoc_xfer_ack_op_t,
    pub /*usbdi_*/isoc_xfer_nak_op: usbdi_isoc_xfer_nak_op_t,
    pub /*usbdi_*/pipe_abort_ack_op: usbdi_pipe_abort_ack_op_t,
    pub /*usbdi_*/pipe_state_set_ack_op: usbdi_pipe_state_set_ack_op_t,
    pub /*usbdi_*/pipe_state_get_ack_op: usbdi_pipe_state_get_ack_op_t,
    pub /*usbdi_*/edpt_state_set_ack_op: usbdi_edpt_state_set_ack_op_t,
    pub /*usbdi_*/edpt_state_get_ack_op: usbdi_edpt_state_get_ack_op_t,
}
pub const USBDI_LDD_PIPE_OPS_NUM: u8 = 2;

#[repr(C)]
pub struct usbdi_usbd_intfc_ops_t
{
    pub /*udi_channel_*/event_ind_op: udi_channel_event_ind_op_t,
    pub /*usbdi_*/bind_req_op: usbdi_bind_req_op_t,
    pub /*usbdi_*/unbind_req_op: usbdi_unbind_req_op_t,
    pub /*usbdi_*/intfc_open_req_op: usbdi_intfc_open_req_op_t,
    pub /*usbdi_*/intfc_close_req_op: usbdi_intfc_close_req_op_t,
    pub /*usbdi_*/frame_number_req_op: usbdi_frame_number_req_op_t,
    pub /*usbdi_*/reset_device_req_op: usbdi_reset_device_req_op_t,
    pub /*usbdi_*/intfc_abort_req_op: usbdi_intfc_abort_req_op_t,
    pub /*usbdi_*/intfc_state_set_req_op: usbdi_intfc_state_set_req_op_t,
    pub /*usbdi_*/intfc_state_get_req_op: usbdi_intfc_state_get_req_op_t,
    pub /*usbdi_*/desc_req_op: usbdi_desc_req_op_t,
    pub /*usbdi_*/device_state_get_req_op: usbdi_device_state_get_req_op_t,
    pub /*usbdi_*/config_set_req_op: usbdi_config_set_req_op_t,
    pub /*usbdi_*/async_event_res_op: usbdi_async_event_res_op_t,
}
//pub const USBDI_USBD_INTFC_OPS_NUM: u8 = 3;
#[repr(C)]
pub struct usbdi_usbd_pipe_ops_t
{
    pub /*udi_channel_*/event_ind_op: udi_channel_event_ind_op_t,
    pub /*usbdi_*/intr_bulk_xfer_req_op: usbdi_intr_bulk_xfer_req_op_t,
    pub /*usbdi_*/control_xfer_req_op: usbdi_control_xfer_req_op_t,
    pub /*usbdi_*/isoc_xfer_req_op: usbdi_isoc_xfer_req_op_t,
    pub /*usbdi_*/pipe_abort_req_op: usbdi_pipe_abort_req_op_t,
    pub /*usbdi_*/pipe_state_set_req_op: usbdi_pipe_state_set_req_op_t,
    pub /*usbdi_*/pipe_state_get_req_op: usbdi_pipe_state_get_req_op_t,
    pub /*usbdi_*/edpt_state_set_req_op: usbdi_edpt_state_set_req_op_t,
    pub /*usbdi_*/edpt_state_get_req_op: usbdi_edpt_state_get_req_op_t,
}
//pub const USBDI_USBD_PIPE_OPS_NUM: u8 = 4;


#[repr(C)]
pub struct usbdi_misc_cb_t
{
    pub gcb: udi_cb_t,
}
pub const USBDI_MISC_CB_NUM: u8 = 1;

#[repr(C)]
pub struct usbdi_intr_bulk_xfer_cb_t
{
    pub gcb: udi_cb_t,
    pub data_buf: *mut udi_buf_t,
    pub timeout: udi_ubit32_t,
    pub xfer_flags: u8,
}
pub const USBDI_INTR_BULK_XFER_CB_NUM: u8 = 2;

pub const USBDI_XFER_SHORT_OK: u8 = 1 << 0;
pub const USBDI_XFER_IN : u8 = 1 << 2;
pub const USBDI_XFER_OUT: u8 = 1 << 3;

#[repr(C)]
pub struct usbdi_control_xfer_cb_t
{
    pub gcb: udi_cb_t,
    pub request: usbdi_control_xfer_cb_t__request,
    pub data_buf: *mut udi_buf_t,
    pub timeout: udi_ubit32_t,
    pub xfer_flags: u8,
}
pub const USBDI_CONTROL_XFER_CB_NUM: u8 = 3;

#[repr(C)]
pub union usbdi_control_xfer_cb_t__request
{
    pub device_request: usb_device_request_t,
    pub request: [u8; 8],
}
#[repr(C)]
#[derive(Copy,Clone)]
#[allow(non_snake_case)]
pub struct usb_device_request_t
{
    pub bmRequestType: udi_ubit8_t,
    pub bRequest: udi_ubit8_t,
    pub wValue0: udi_ubit8_t,
    pub wValue1: udi_ubit8_t,
    pub wIndex0: udi_ubit8_t,
    pub wIndex1: udi_ubit8_t,
    pub wLength0: udi_ubit8_t,
    pub wLength1: udi_ubit8_t,
}


#[repr(C)]
pub struct usbdi_isoc_frame_request_t
{
    pub frame_len: udi_ubit32_t,
    pub frame_status: udi_status_t,
}
#[repr(C)]
pub struct usbdi_isoc_xfer_cb_t
{
    pub gcb: udi_cb_t,
    pub data_buf: *mut udi_buf_t,
    // Inline data
    pub frame_array: *mut usbdi_isoc_frame_request_t,
    pub frame_count: u8,
    pub timeout: udi_ubit32_t,
    pub xfer_flags: u8,
    pub frame_number: udi_ubit32_t,
}
pub const USBDI_ISOC_XFER_CB_NUM: u8 = 4;   // DUPLICATE! with `USBDI_STATE_CB_NUM`

pub const USBDI_XFER_ASAP: u8 = 1 << 4;

#[repr(C)]
pub struct usbdi_state_cb_t
{
    pub gcb: udi_cb_t,
    pub state: udi_ubit8_t,
}
pub const USBDI_STATE_CB_NUM: u8 = 4;   // DUPLICATE! with `USBDI_ISOC_XFER_CB_NUM`

pub const USBDI_STATE_ACTIVE : udi_ubit8_t = 1;
pub const USBDI_STATE_STALLED: udi_ubit8_t = 2;
pub const USBDI_STATE_IDLE   : udi_ubit8_t = 3;
pub const USBDI_STATE_HALTED : udi_ubit8_t = 4;

pub const USBDI_STATE_CONFIGURED: udi_ubit8_t = 1 << 1;
pub const USBDI_STATE_SUSPENDED: udi_ubit8_t = 1 << 2;

#[repr(C)]
pub struct usbdi_desc_cb_t
{
    pub gcb: udi_cb_t,
    pub desc_type: udi_ubit8_t,
    pub desc_index: udi_ubit8_t,
    pub desc_id: udi_ubit16_t,
    pub desc_length: udi_ubit16_t,
    pub desc_buf: *mut udi_buf_t,
}
pub const USBDI_DESC_CB_NUM: u8 = 5;

pub const USB_DESC_TYPE_DEVICE: udi_ubit8_t = 0x01;
pub const USB_DESC_TYPE_CONFIG: udi_ubit8_t = 0x02;
pub const USB_DESC_TYPE_STRING: udi_ubit8_t = 0x03;
pub const USB_DESC_TYPE_INTFC : udi_ubit8_t = 0x04;
pub const USB_DESC_TYPE_EDPT  : udi_ubit8_t = 0x05;
