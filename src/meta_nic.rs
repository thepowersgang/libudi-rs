use crate::ffi::{udi_index_t, udi_status_t};

// SAFE: Follows the contract, gcb is first field
unsafe impl crate::async_trickery::GetCb for ffi::udi_nic_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}
// SAFE: Follows the contract, gcb is first field
unsafe impl crate::async_trickery::GetCb for ffi::udi_nic_bind_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}
// SAFE: Follows the contract, gcb is first field
unsafe impl crate::async_trickery::GetCb for ffi::udi_nic_ctrl_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}
// SAFE: Follows the contract, gcb is first field
unsafe impl crate::async_trickery::GetCb for ffi::udi_nic_info_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}
// SAFE: Follows the contract, gcb is first field
unsafe impl crate::async_trickery::GetCb for ffi::udi_nic_tx_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}
// SAFE: Follows the contract, gcb is first field
unsafe impl crate::async_trickery::GetCb for ffi::udi_nic_rx_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}

#[repr(u8)]
pub enum OpsNum
{
    NdCtrl = 1,
    NdTx,
    NdRx,
    NsrCtrl,
    NsrTx,
    NsrRx,
}

pub type CbRefNic<'a> = crate::CbRef<'a, crate::meta_nic::ffi::udi_nic_cb_t>;
pub type CbRefNicBind<'a> = crate::CbRef<'a, crate::meta_nic::ffi::udi_nic_bind_cb_t>;
pub type CbRefNicTx<'a> = crate::CbRef<'a, crate::meta_nic::ffi::udi_nic_tx_cb_t>;

pub trait Control: 'static {
    async_method!(fn bind_req(&'a mut self, cb: CbRefNicBind<'a>, tx_chan_index: udi_index_t, rx_chan_index: udi_index_t)->crate::Result<NicInfo> as Future_bind_req);
    async_method!(fn unbind_req(&'a mut self, cb: CbRefNic<'a>)->() as Future_unbind_req);
    async_method!(fn enable_req(&'a mut self, cb: CbRefNic<'a>)->crate::Result<()> as Future_enable_req);
    async_method!(fn disable_req(&mut self)->() as Future_disable_req);
    async_method!(fn ctrl_req(&mut self)->() as Future_ctrl_req);
    async_method!(fn info_req(&mut self, reset_statistics: bool)->() as Future_info_req);
}
struct MarkerControl;
impl<T> crate::imc::ChannelHandler<MarkerControl> for T
where
    T: Control
{
    fn channel_closed(&mut self) {
    }
    fn channel_bound(&mut self, _params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
    }
}

future_wrapper!(nd_bind_req_op => <T as Control>(cb: *mut ffi::udi_nic_bind_cb_t, tx_chan_index: udi_index_t, rx_chan_index: udi_index_t)
    val @ {
        crate::async_trickery::with_ack(
            val.bind_req(cb, tx_chan_index, rx_chan_index),
            |cb: *mut ffi::udi_nic_bind_cb_t, res| unsafe {
                let status = match res {
                    Ok(v) => {
                        let cb = &mut *cb;
                        cb.media_type = v.media_type as _;
                        cb.min_pdu_size = v.min_pdu_size;
                        cb.max_pdu_size = v.max_pdu_size;
                        cb.rx_hw_threshold = v.rx_hw_threshold;
                        cb.capabilities = v.capabilities;
                        cb.max_perfect_multicast = v.max_perfect_multicast;
                        cb.max_total_multicast = v.max_total_multicast;
                        cb.mac_addr_len = v.mac_addr_len;
                        cb.mac_addr = v.mac_addr;
                        0
                        },
                    Err(s) => s.into_inner(),
                    };
                ffi::udi_nsr_bind_ack(cb, status)
                }
            )
        }
    );
future_wrapper!(nd_unbind_req_op => <T as Control>(cb: *mut ffi::udi_nic_cb_t)
    val @ {
        val.unbind_req(cb)
        }
    );
future_wrapper!(nd_enable_req_op => <T as Control>(cb: *mut ffi::udi_nic_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.enable_req(cb),
        |cb, res| unsafe { ffi::udi_nsr_enable_ack(cb, crate::Error::to_status(res)) }
        )
});
future_wrapper!(nd_disable_req_op => <T as Control>(cb: *mut ffi::udi_nic_cb_t) val @ {
    val.disable_req()
});
future_wrapper!(nd_ctrl_req_op => <T as Control>(cb: *mut ffi::udi_nic_ctrl_cb_t) val @ {
    val.ctrl_req()
});
future_wrapper!(nd_info_req_op => <T as Control>(cb: *mut ffi::udi_nic_info_cb_t, reset_statistics: crate::ffi::udi_boolean_t) val @ {
    val.info_req(reset_statistics != 0)
});
impl ffi::udi_nd_ctrl_ops_t
{
    pub const fn scratch_requirement<T: Control>() -> usize {
        let v = crate::imc::task_size::<T, MarkerControl>();
        let v = crate::const_max(v, nd_bind_req_op::task_size::<T>());
        let v = crate::const_max(v, nd_unbind_req_op::task_size::<T>());
        let v = crate::const_max(v, nd_enable_req_op::task_size::<T>());
        let v = crate::const_max(v, nd_disable_req_op::task_size::<T>());
        let v = crate::const_max(v, nd_ctrl_req_op::task_size::<T>());
        let v = crate::const_max(v, nd_info_req_op::task_size::<T>());
        v
    }
    /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
    /// SAFETY: The scratch size must be >= value returned by [scratch_requirement]
    pub const unsafe fn for_driver<T: Control>() -> Self {
        Self {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T, MarkerControl>,
            nd_bind_req_op: nd_bind_req_op::<T>,
            nd_unbind_req_op: nd_unbind_req_op::<T>,
            nd_enable_req_op: nd_enable_req_op::<T>,
            nd_disable_req_op: nd_disable_req_op::<T>,
            nd_ctrl_req_op: nd_ctrl_req_op::<T>,
            nd_info_req_op: nd_info_req_op::<T>,
        }
    }
}
unsafe impl crate::Ops for ffi::udi_nd_ctrl_ops_t
{
    const OPS_NUM: crate::ffi::udi_index_t = OpsNum::NdCtrl as _;
}



pub trait NdTx: 'static {
    async_method!(fn tx_req(&'a mut self, cb: CbRefNicTx<'a>)->() as Future_tx_req);
    async_method!(fn exp_tx_req(&'a mut self, cb: CbRefNicTx<'a>)->() as Future_exp_tx_req);
}
struct MarkerNdTx;
impl<T> crate::imc::ChannelHandler<MarkerNdTx> for T
where
    T: NdTx
{
    fn channel_closed(&mut self) {
    }
    fn channel_bound(&mut self, _params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
    }
}

future_wrapper!(nd_tx_req_op => <T as NdTx>(cb: *mut ffi::udi_nic_tx_cb_t) val @ {
    val.tx_req(cb)
});
future_wrapper!(nd_exp_tx_req_op => <T as NdTx>(cb: *mut ffi::udi_nic_tx_cb_t) val @ {
    val.exp_tx_req(cb)
});
impl ffi::udi_nd_tx_ops_t
{
    pub const fn scratch_requirement<T: NdTx>() -> usize {
        let v = crate::imc::task_size::<T, MarkerNdTx>();
        let v = crate::const_max(v, nd_tx_req_op::task_size::<T>());
        let v = crate::const_max(v, nd_exp_tx_req_op::task_size::<T>());
        v
    }
    /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
    /// SAFETY: The scratch size must be >= value returned by [scratch_requirement]
    pub const unsafe fn for_driver<T: NdTx>() -> Self {
        Self {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T, MarkerNdTx>,
            nd_tx_req_op: nd_tx_req_op::<T>,
            nd_exp_tx_req_op: nd_exp_tx_req_op::<T>,
        }
    }
}
unsafe impl crate::Ops for ffi::udi_nd_tx_ops_t
{
    const OPS_NUM: crate::ffi::udi_index_t = OpsNum::NdTx as _;
}

/// Result type from a bind
pub struct NicInfo {
    pub media_type: ffi::MediaType,
    pub min_pdu_size: u32,
    pub max_pdu_size: u32,
    pub rx_hw_threshold: u32,
    pub capabilities: u32,
    pub max_perfect_multicast: u8,
    pub max_total_multicast: u8,
    pub mac_addr_len: u8,
    pub mac_addr: [u8; ffi::UDI_NIC_MAC_ADDRESS_SIZE],
}

#[allow(non_camel_case_types)]
pub mod ffi {
    use crate::ffi::{udi_ubit32_t, udi_ubit8_t};
    use crate::ffi::{udi_index_t, udi_status_t, udi_boolean_t};
    use crate::ffi::{udi_cb_t, udi_buf_t};


    #[repr(C)]
    pub struct udi_nic_cb_t
    {
        pub gcb: crate::ffi::udi_cb_t,
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

    #[repr(C)]
    pub struct udi_nic_status_cb_t
    {
        pub gcb: udi_cb_t,
        pub event: udi_ubit8_t,
    }

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
        pub fn udi_nsr_disable_ack(cb: *mut udi_nic_cb_t, status: udi_status_t);
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
        pub channel_event_ind_op: crate::ffi::imc::udi_channel_event_ind_op_t,
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
        pub channel_event_ind_op: crate::ffi::imc::udi_channel_event_ind_op_t,
        pub nd_tx_req_op: udi_nd_tx_req_op_t,
        pub nd_exp_tx_req_op: udi_nd_exp_tx_req_op_t,
    }
    #[repr(C)]
    pub struct udi_nd_rx_ops_t
    {
        pub channel_event_ind_op: crate::ffi::imc::udi_channel_event_ind_op_t,
        pub nd_rx_rdy_op: udi_nd_rx_rdy_op_t,
    }

    #[repr(C)]
    pub struct udi_nsr_ctrl_ops_t
    {
        pub channel_event_ind_op: crate::ffi::imc::udi_channel_event_ind_op_t,
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
        pub channel_event_ind_op: crate::ffi::imc::udi_channel_event_ind_op_t,
        pub nsr_tx_rdy_op: udi_nsr_tx_rdy_op_t,
    }
    #[repr(C)]
    pub struct udi_nsr_rx_ops_t
    {
        pub channel_event_ind_op: crate::ffi::imc::udi_channel_event_ind_op_t,
        pub nsr_rx_ind_op: udi_nsr_rx_ind_op_t,
        pub nsr_exp_rx_ind_op: udi_nsr_exp_rx_ind_op_t,
    }
}