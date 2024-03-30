#![allow(non_camel_case_types)]
use crate::{udi_ubit32_t, udi_ubit8_t};
use crate::{udi_index_t, udi_status_t, udi_boolean_t};
use crate::{udi_cb_t, udi_buf_t};

pub macro metalang_name( $($prefix:ident::)* ) {
    $($prefix::)*udi_nic
}


#[repr(C)]
pub struct udi_nic_cb_t
{
    pub gcb: crate::udi_cb_t,
}

#[repr(u8)]
pub enum MediaType {
    UDI_NIC_ETHER     = 0,
    UDI_NIC_TOKEN     = 1,
    UDI_NIC_FASTETHER = 2,
    UDI_NIC_GIGETHER  = 3,
    UDI_NIC_VGANYLAN  = 4,
    UDI_NIC_FDDI      = 5,
    UDI_NIC_ATM       = 6,
    UDI_NIC_FC        = 7,
    UDI_NIC_MISCMEDIA = 0xff,
}
pub use MediaType::*;
pub const UDI_NIC_MAC_ADDRESS_SIZE: usize = 20;

#[repr(C)]
pub struct udi_nic_bind_cb_t
{
    pub gcb: udi_cb_t,
    pub media_type: udi_ubit8_t,
    pub min_pdu_size: udi_ubit32_t,
    pub max_pdu_size: udi_ubit32_t,
    pub rx_hw_threshold: udi_ubit32_t,
    pub capabilities: udi_ubit32_t,
    pub max_perfect_multicast: udi_ubit8_t,
    pub max_total_multicast: udi_ubit8_t,
    pub mac_addr_len: udi_ubit8_t,
    pub mac_addr: [udi_ubit8_t; UDI_NIC_MAC_ADDRESS_SIZE],
}

#[repr(C)]
pub struct udi_nic_ctrl_cb_t
{
    pub gcb: udi_cb_t,
    pub command: udi_ubit8_t,
    pub indicator: udi_ubit32_t,
    pub data_buf: *mut udi_buf_t,
}
/* Network Control Operation Commands */
pub const UDI_NIC_ADD_MULTI   : udi_ubit8_t =  1;
pub const UDI_NIC_DEL_MULTI   : udi_ubit8_t =  2;
pub const UDI_NIC_ALLMULTI_ON : udi_ubit8_t =  3;
pub const UDI_NIC_ALLMULTI_OFF: udi_ubit8_t =  4;
pub const UDI_NIC_GET_CURR_MAC: udi_ubit8_t =  5;
pub const UDI_NIC_SET_CURR_MAC: udi_ubit8_t =  6;
pub const UDI_NIC_GET_FACT_MAC: udi_ubit8_t =  7;
pub const UDI_NIC_PROMISC_ON  : udi_ubit8_t =  8;
pub const UDI_NIC_PROMISC_OFF : udi_ubit8_t =  9;
pub const UDI_NIC_HW_RESET    : udi_ubit8_t = 10;
pub const UDI_NIC_BAD_RXPKT   : udi_ubit8_t = 11;

#[repr(C)]
pub struct udi_nic_status_cb_t
{
    pub gcb: udi_cb_t,
    pub event: udi_ubit8_t,
}
/* Network Status Event Codes */
pub const UDI_NIC_LINK_DOWN : udi_ubit8_t = 0;
pub const UDI_NIC_LINK_UP   : udi_ubit8_t = 1;
pub const UDI_NIC_LINK_RESET: udi_ubit8_t = 2;


#[repr(C)]
pub struct udi_nic_info_cb_t
{
    pub gcb: udi_cb_t,
    pub interface_is_active: udi_boolean_t,
    pub link_is_active: udi_boolean_t,
    pub is_full_duplex: udi_boolean_t,
    pub link_mbps: udi_ubit32_t,
    pub link_bps: udi_ubit32_t,
    pub tx_packets: udi_ubit32_t,
    pub rx_packets: udi_ubit32_t,
    pub tx_errors: udi_ubit32_t,
    pub rx_errors: udi_ubit32_t,
    pub tx_discards: udi_ubit32_t,
    pub rx_discards: udi_ubit32_t,
    pub tx_underrun: udi_ubit32_t,
    pub rx_overrun: udi_ubit32_t,
    pub collisions: udi_ubit32_t,
}

#[repr(C)]
pub struct udi_nic_tx_cb_t
{
    pub gcb: udi_cb_t,
    pub chain: *mut udi_nic_tx_cb_t,
    pub tx_buf: *mut udi_buf_t,
    pub completion_urgent: udi_boolean_t,
}
#[repr(C)]
pub struct udi_nic_rx_cb_t
{
    pub gcb: udi_cb_t,
    pub chain: *mut udi_nic_rx_cb_t,
    pub rx_buf: *mut udi_buf_t,
    pub rx_status: udi_ubit8_t,
    pub addr_match: udi_ubit8_t,
    pub rx_valid: udi_ubit8_t,
}

type udi_nd_bind_req_op_t = unsafe extern "C" fn(cb: *mut udi_nic_bind_cb_t, tx_chan_index: udi_index_t, rx_chan_index: udi_index_t);
type udi_nsr_bind_ack_op_t = unsafe extern "C" fn(cb: *mut udi_nic_bind_cb_t, status: udi_status_t);
type udi_nd_unbind_req_op_t = unsafe extern "C" fn(cb: *mut udi_nic_cb_t);
type udi_nsr_unbind_ack_op_t = unsafe extern "C" fn(cb: *mut udi_nic_cb_t, status: udi_status_t);
type udi_nd_enable_req_op_t = unsafe extern "C" fn(cb: *mut udi_nic_cb_t);
type udi_nsr_enable_ack_op_t = unsafe extern "C" fn(cb: *mut udi_nic_cb_t, status: udi_status_t);
type udi_nd_disable_req_op_t = unsafe extern "C" fn(cb: *mut udi_nic_cb_t);
//type udi_nsr_disable_ack_op_t = unsafe extern "C" fn(cb: *mut udi_nic_cb_t, status: udi_status_t);
type udi_nd_ctrl_req_op_t = unsafe extern "C" fn(cb: *mut udi_nic_ctrl_cb_t);
type udi_nsr_ctrl_ack_op_t = unsafe extern "C" fn(cb: *mut udi_nic_ctrl_cb_t, status: udi_status_t);
type udi_nsr_status_ind_op_t = unsafe extern "C" fn(cb: *mut udi_nic_status_cb_t);
type udi_nd_info_req_op_t = unsafe extern "C" fn(cb: *mut udi_nic_info_cb_t, reset_statistics: udi_boolean_t);
type udi_nsr_info_ack_op_t = unsafe extern "C" fn(cb: *mut udi_nic_info_cb_t);
// - TX
type udi_nsr_tx_rdy_op_t = unsafe extern "C" fn(cb: *mut udi_nic_tx_cb_t);
type udi_nd_tx_req_op_t = unsafe extern "C" fn(cb: *mut udi_nic_tx_cb_t);
type udi_nd_exp_tx_req_op_t = unsafe extern "C" fn(cb: *mut udi_nic_tx_cb_t);
// - RX
type udi_nsr_rx_ind_op_t = unsafe extern "C" fn(cb: *mut udi_nic_rx_cb_t);
type udi_nsr_exp_rx_ind_op_t = unsafe extern "C" fn(cb: *mut udi_nic_rx_cb_t);
type udi_nd_rx_rdy_op_t = unsafe extern "C" fn(cb: *mut udi_nic_rx_cb_t);

extern "C" {
    pub fn udi_nd_bind_req(cb: *mut udi_nic_bind_cb_t, tx_chan_index: udi_index_t, rx_chan_index: udi_index_t);
    pub fn udi_nsr_bind_ack(cb: *mut udi_nic_bind_cb_t, status: udi_status_t);
    pub fn udi_nd_unbind_req(cb: *mut udi_nic_cb_t);
    pub fn udi_nsr_unbind_ack(cb: *mut udi_nic_cb_t, status: udi_status_t);
    pub fn udi_nd_enable_req(cb: *mut udi_nic_cb_t);
    pub fn udi_nsr_enable_ack(cb: *mut udi_nic_cb_t, status: udi_status_t);
    pub fn udi_nd_disable_req(cb: *mut udi_nic_cb_t);
    //pub fn udi_nsr_disable_ack(cb: *mut udi_nic_cb_t, status: udi_status_t);
    pub fn udi_nd_ctrl_req(cb: *mut udi_nic_ctrl_cb_t);
    pub fn udi_nsr_ctrl_ack(cb: *mut udi_nic_ctrl_cb_t, status: udi_status_t);
    pub fn udi_nsr_status_ind(cb: *mut udi_nic_status_cb_t);
    pub fn udi_nd_info_req(cb: *mut udi_nic_info_cb_t, reset_statistics: udi_boolean_t);
    pub fn udi_nsr_info_ack(cb: *mut udi_nic_info_cb_t);
    // - TX
    pub fn udi_nsr_tx_rdy(cb: *mut udi_nic_tx_cb_t);
    pub fn udi_nd_tx_req(cb: *mut udi_nic_tx_cb_t);
    pub fn udi_nd_exp_tx_req(cb: *mut udi_nic_tx_cb_t);
    // - RX
    pub fn udi_nsr_rx_ind(cb: *mut udi_nic_rx_cb_t);
    pub fn udi_nsr_exp_rx_ind(cb: *mut udi_nic_rx_cb_t);
    pub fn udi_nd_rx_rdy(cb: *mut udi_nic_rx_cb_t);
}


#[repr(C)]
pub struct udi_nd_ctrl_ops_t
{
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub nd_bind_req_op: udi_nd_bind_req_op_t,
    pub nd_unbind_req_op: udi_nd_unbind_req_op_t,
    pub nd_enable_req_op: udi_nd_enable_req_op_t,
    pub nd_disable_req_op: udi_nd_disable_req_op_t,
    pub nd_ctrl_req_op: udi_nd_ctrl_req_op_t,
    pub nd_info_req_op: udi_nd_info_req_op_t,
}
#[repr(C)]
pub struct udi_nd_tx_ops_t
{
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub nd_tx_req_op: udi_nd_tx_req_op_t,
    pub nd_exp_tx_req_op: udi_nd_exp_tx_req_op_t,
}
#[repr(C)]
pub struct udi_nd_rx_ops_t
{
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub nd_rx_rdy_op: udi_nd_rx_rdy_op_t,
}

#[repr(C)]
pub struct udi_nsr_ctrl_ops_t
{
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub nsr_bind_ack_op: udi_nsr_bind_ack_op_t,
    pub nsr_unbind_ack_op: udi_nsr_unbind_ack_op_t,
    pub nsr_enable_ack_op: udi_nsr_enable_ack_op_t,
    pub nsr_ctrl_ack_op: udi_nsr_ctrl_ack_op_t,
    pub nsr_info_ack_op: udi_nsr_info_ack_op_t,
    pub nsr_status_ind_op: udi_nsr_status_ind_op_t,
}
#[repr(C)]
pub struct udi_nsr_tx_ops_t
{
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub nsr_tx_rdy_op: udi_nsr_tx_rdy_op_t,
}
#[repr(C)]
pub struct udi_nsr_rx_ops_t
{
    pub channel_event_ind_op: crate::imc::udi_channel_event_ind_op_t,
    pub nsr_rx_ind_op: udi_nsr_rx_ind_op_t,
    pub nsr_exp_rx_ind_op: udi_nsr_exp_rx_ind_op_t,
}