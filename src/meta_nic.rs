///! Network Interface Card Metalanguage
use crate::ffi::udi_index_t;

use crate::ffi::meta_nic as ffi;

pub fn nsr_rx_ind(rx_cb: CbHandleNicRx) {
    unsafe { ffi::udi_nsr_rx_ind(rx_cb.into_raw()) }
}
pub fn nd_rx_rdy(cb: CbHandleNicRx) {
    unsafe { ffi::udi_nd_rx_rdy(cb.into_raw()) }
}
pub fn nd_tx_req(tx_cb: CbHandleNicTx) {
    unsafe { ffi::udi_nd_tx_req(tx_cb.into_raw()) }
}
pub fn nd_exp_tx_req(tx_cb: CbHandleNicTx) {
    unsafe { ffi::udi_nd_exp_tx_req(tx_cb.into_raw()) }
}
pub fn nsr_tx_rdy(cb: CbHandleNicTx) {
    unsafe { ffi::udi_nsr_tx_rdy(cb.into_raw()) }
}

macro_rules! def_cb {
    (unsafe $ref_name:ident => $t:ty : $cb_num:expr) => {
        pub type $ref_name<'a> = crate::CbRef<'a, $t>;
    }
}

impl_metalanguage!{
    static METALANG_SPEC;
    NAME udi_nic;
    OPS
        1 => ffi::udi_nd_ctrl_ops_t,
        2 => ffi::udi_nd_tx_ops_t,
        3 => ffi::udi_nd_rx_ops_t,
        4 => ffi::udi_nsr_ctrl_ops_t,
        5 => ffi::udi_nsr_tx_ops_t,
        6 => ffi::udi_nsr_rx_ops_t,
        ;
    CBS
        1 => ffi::udi_nic_cb_t,
        2 => ffi::udi_nic_bind_cb_t,
        3 => ffi::udi_nic_ctrl_cb_t,
        4 => ffi::udi_nic_status_cb_t,
        5 => ffi::udi_nic_info_cb_t,
        6 => ffi::udi_nic_tx_cb_t : BUF tx_buf : CHAIN chain,
        7 => ffi::udi_nic_rx_cb_t : BUF rx_buf : CHAIN chain,
        ;
}

impl crate::ops_markers::ParentBind<::udi_sys::meta_nic::udi_nic_bind_cb_t> for ::udi_sys::meta_nic::udi_nsr_ctrl_ops_t {
    const ASSERT: () = ();
}
impl crate::ops_markers::ChildBind for ::udi_sys::meta_nic::udi_nd_ctrl_ops_t {
    const ASSERT: () = ();
}

// SAFE: Follows the contract, gcb is first field
def_cb!(unsafe CbRefNic => ffi::udi_nic_cb_t : 1);
// SAFE: Follows the contract, gcb is first field
def_cb!(unsafe CbRefNicBind => ffi::udi_nic_bind_cb_t : 2);
// SAFE: Follows the contract, gcb is first field
def_cb!(unsafe CbRefNicCtrl => ffi::udi_nic_ctrl_cb_t : 3);
// SAFE: Follows the contract, gcb is first field
def_cb!(unsafe CbRefNicStatus => ffi::udi_nic_status_cb_t : 4);
// SAFE: Follows the contract, gcb is first field
def_cb!(unsafe CbRefNicInfo => ffi::udi_nic_info_cb_t : 5);
// SAFE: Follows the contract, gcb is first field
def_cb!(unsafe CbRefNicTx => ffi::udi_nic_tx_cb_t : 6);
// SAFE: Follows the contract, gcb is first field
def_cb!(unsafe CbRefNicRx => ffi::udi_nic_rx_cb_t : 7);
pub type CbHandleNicTx = crate::cb::CbHandle<ffi::udi_nic_tx_cb_t>;
pub type CbHandleNicRx = crate::cb::CbHandle<ffi::udi_nic_rx_cb_t>;

impl crate::cb::CbHandle<ffi::udi_nic_tx_cb_t>
{
    /// Put this cb onto the end of the chain formed by `other` (i.e. `push_front`)
    pub fn link_front(&mut self, other: Self) {
        unsafe {
            let other = other.into_raw();
            let mut cursor = other;
            while ! (*cursor).chain.is_null() {
                cursor = (*cursor).chain;
            }
            (*cursor).chain = ::core::mem::replace(self, Self::from_raw(other)).into_raw();
        }
    }
    pub fn unlink(mut self) -> (Self,Option<Self>) {
        unsafe {
            let chain = &mut self.get_mut().chain;
            if chain.is_null() {
                (self, None)
            }
            else {
                let next = CbHandleNicTx::from_raw(::core::mem::replace(chain, ::core::ptr::null_mut()));
                (self, Some(next))
            }
        }
    }

    pub fn tx_buf_ref(&self) -> &crate::buf::Handle {
        unsafe { crate::buf::Handle::from_ref( &self.tx_buf ) }
    }
    pub fn tx_buf_mut(&mut self) -> &mut crate::buf::Handle {
        unsafe { crate::buf::Handle::from_mut( &mut self.get_mut().tx_buf ) }
    }
}

impl crate::cb::CbHandle<ffi::udi_nic_rx_cb_t>
{
    /// Put this cb onto the end of the chain formed by `other` (i.e. `push_front`)
    pub fn link_front(&mut self, other: Self) {
        unsafe {
            let other = other.into_raw();
            let mut cursor = other;
            while ! (*cursor).chain.is_null() {
                cursor = (*cursor).chain;
            }
            (*cursor).chain = ::core::mem::replace(self, Self::from_raw(other)).into_raw();
        }
    }
    pub fn unlink(mut self) -> (Self,Option<Self>) {
        unsafe {
            let chain = &mut self.get_mut().chain;
            if chain.is_null() {
                (self, None)
            }
            else {
                let next = CbHandleNicRx::from_raw(::core::mem::replace(chain, ::core::ptr::null_mut()));
                (self, Some(next))
            }
        }
    }

    pub fn rx_buf_ref(&self) -> &crate::buf::Handle {
        unsafe { crate::buf::Handle::from_ref( &self.rx_buf ) }
    }
    pub fn rx_buf_mut(&mut self) -> &mut crate::buf::Handle {
        unsafe { crate::buf::Handle::from_mut( &mut self.get_mut().rx_buf ) }
    }

    //pub fn set_rx_status(&mut self, 
}

// TODO: `CbHandleNicTx`/`CbHandleNicRx` could be a wrapper type to provide safe access to fields

/// A queue of RX CBs
pub struct ReadCbQueue
{
    head: *mut ffi::udi_nic_rx_cb_t,
    tail: *mut ffi::udi_nic_rx_cb_t,
}
impl Default for ReadCbQueue {
    fn default() -> Self {
        Self::new()
    }
}
impl ReadCbQueue
{
    pub const fn new() -> Self {
        Self {
            head: ::core::ptr::null_mut(),
            tail: ::core::ptr::null_mut(),
        }
    }
    pub fn push(&mut self, cb: CbHandleNicRx) {
        let cb = cb.into_raw();
        if self.head.is_null() {
            self.head = cb;
            self.tail = self.head;
        }
        else {
            // SAFE: This type logically owns these pointers (so they're non-NULL)
            // SAFE: Trusting the `chain` on incoming cbs to be a valid single-linked list
            unsafe {
                (*self.tail).chain = cb;
                while !(*self.tail).chain.is_null() {
                    self.tail = (*self.tail).chain;
                }
            }
        }
    }
    pub fn pop(&mut self) -> Option< CbHandleNicRx > {
        if self.head.is_null() {
            None
        }
        else {
            let rv = self.head;
            // SAFE: The chain is a valid singularly-linked list of owned pointers
            unsafe {
                self.head = (*rv).chain;
                if self.head.is_null() {
                    // Defensive measure.
                    self.tail = ::core::ptr::null_mut();
                }
                Some( CbHandleNicRx::from_raw(rv) )
            }
        }
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

pub trait Control: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(fn bind_req(&'a mut self, cb: CbRefNicBind<'a>, tx_chan_index: udi_index_t, rx_chan_index: udi_index_t)->crate::Result<NicInfo> as Future_bind_req);
    async_method!(fn unbind_req(&'a mut self, cb: CbRefNic<'a>)->() as Future_unbind_req);
    async_method!(fn enable_req(&'a mut self, cb: CbRefNic<'a>)->crate::Result<()> as Future_enable_req);
    async_method!(fn disable_req(&'a mut self, cb: CbRefNic<'a>)->() as Future_disable_req);
    async_method!(fn ctrl_req(&'a mut self, cb: CbRefNicCtrl<'a>)->() as Future_ctrl_req);
    async_method!(fn info_req(&'a mut self, cb: CbRefNicInfo<'a>, reset_statistics: bool)->() as Future_info_req);
}
struct MarkerControl;
impl<T> crate::imc::ChannelHandler<MarkerControl> for T
where
    T: Control
{
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
    val.disable_req(cb)
});
future_wrapper!(nd_ctrl_req_op => <T as Control>(cb: *mut ffi::udi_nic_ctrl_cb_t) val @ {
    val.ctrl_req(cb)
});
future_wrapper!(nd_info_req_op => <T as Control>(cb: *mut ffi::udi_nic_info_cb_t, reset_statistics: crate::ffi::udi_boolean_t) val @ {
    val.info_req(cb, reset_statistics.to_bool())
});

map_ops_structure!{
    ffi::udi_nd_ctrl_ops_t => Control,MarkerControl {
        nd_bind_req_op,
        nd_unbind_req_op,
        nd_enable_req_op,
        nd_disable_req_op,
        nd_ctrl_req_op,
        nd_info_req_op,
    }
    CBS {
        ffi::udi_nic_cb_t,
        ffi::udi_nic_bind_cb_t,
        ffi::udi_nic_ctrl_cb_t,
        ffi::udi_nic_info_cb_t,
    }
}

// --------------------------------------------------------------------

pub struct BindChannels {
    pub tx: udi_index_t,
    pub rx: udi_index_t,
}

pub trait NsrControl: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(fn get_bind_channels(&'a mut self, cb: CbRefNicBind<'a>)->BindChannels as Future_gbc);
    async_method!(fn bind_ack(&'a mut self, cb: CbRefNicBind<'a>, res: crate::Result<()>)->() as Future_bind_ack);
    async_method!(fn unbind_ack(&'a mut self, cb: CbRefNic<'a>, res: crate::Result<()>)->() as Future_unbind_ack);
    async_method!(fn enable_ack(&'a mut self, cb: CbRefNic<'a>, res: crate::Result<()>)->() as Future_enable_ack);
    async_method!(fn ctrl_ack(&'a mut self, cb: CbRefNicCtrl<'a>, res: crate::Result<()>)->() as Future_ctrl_ack);
    async_method!(fn info_ack(&'a mut self, cb: CbRefNicInfo<'a>)->() as Future_info_ack);
    async_method!(fn status_ind(&'a mut self, cb: CbRefNicStatus<'a>)->() as Future_status_ind);
}
future_wrapper!(nsr_channel_bound => <T as NsrControl>(cb: *mut ffi::udi_nic_bind_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.get_bind_channels(cb),
        |cb,chans| unsafe { ffi::udi_nd_bind_req(cb, chans.tx, chans.rx) },
    )
});
struct MarkerNsrControl;
impl<T> crate::imc::ChannelHandler<MarkerNsrControl> for T
where
    T: NsrControl
{
    fn channel_bound(&mut self, params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
        // Start a UDI async using the bind CB
        unsafe {
            let cb = params.parent_bound.bind_cb as *mut ffi::udi_nic_bind_cb_t;
            nsr_channel_bound::<T>(cb)
        }
    }
}

future_wrapper!(nsr_bind_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_bind_cb_t, status: ::udi_sys::udi_status_t) val @ {
    let res = crate::Error::from_status(status);
    crate::async_trickery::with_ack(
        val.bind_ack(cb, res),
        |cb,()| unsafe { crate::async_trickery::channel_event_complete::<T,ffi::udi_nic_bind_cb_t>(cb, ::udi_sys::UDI_OK as _) }
        )
});
future_wrapper!(nsr_unbind_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_cb_t, status: ::udi_sys::udi_status_t) val @ {
    val.unbind_ack(cb, crate::Error::from_status(status))
});
future_wrapper!(nsr_enable_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_cb_t, status: ::udi_sys::udi_status_t) val @ {
    val.enable_ack(cb, crate::Error::from_status(status))
});
future_wrapper!(nsr_ctrl_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_ctrl_cb_t, status: ::udi_sys::udi_status_t) val @ {
    val.ctrl_ack(cb, crate::Error::from_status(status))
});
future_wrapper!(nsr_info_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_info_cb_t) val @ {
    val.info_ack(cb)
});
future_wrapper!(nsr_status_ind_op => <T as NsrControl>(cb: *mut ffi::udi_nic_status_cb_t) val @ {
    val.status_ind(cb)
});
map_ops_structure!{
    ffi::udi_nsr_ctrl_ops_t => NsrControl,MarkerNsrControl {
        nsr_bind_ack_op,
        nsr_unbind_ack_op,
        nsr_enable_ack_op,
        nsr_ctrl_ack_op,
        nsr_info_ack_op,
        nsr_status_ind_op,
    }
    CBS {
        ffi::udi_nic_cb_t,
        ffi::udi_nic_bind_cb_t,
        ffi::udi_nic_ctrl_cb_t,
        ffi::udi_nic_info_cb_t,
    }
    EXTRA_OP nsr_channel_bound
}

// --------------------------------------------------------------------

pub trait NdTx: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(fn tx_req(&'a mut self, cb: CbHandleNicTx)->() as Future_tx_req);
    async_method!(fn exp_tx_req(&'a mut self, cb: CbHandleNicTx)->() as Future_exp_tx_req);
}
struct MarkerNdTx;
impl<T> crate::imc::ChannelHandler<MarkerNdTx> for T
where
    T: NdTx
{
}

future_wrapper!(nd_tx_req_op => <T as NdTx>(cb: *mut ffi::udi_nic_tx_cb_t) val @ {
    val.tx_req(unsafe { cb.into_owned() })
});
future_wrapper!(nd_exp_tx_req_op => <T as NdTx>(cb: *mut ffi::udi_nic_tx_cb_t) val @ {
    val.exp_tx_req(unsafe { cb.into_owned() })
});
map_ops_structure!{
    ffi::udi_nd_tx_ops_t => NdTx,MarkerNdTx {
        nd_tx_req_op,
        nd_exp_tx_req_op,
    }
    CBS {
        ffi::udi_nic_tx_cb_t,
    }
}
// --------------------------------------------------------------------

pub trait NsrTx: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(fn tx_rdy(&'a mut self, cb: CbHandleNicTx)->() as Future_tx_rdy);
}
struct MarkerNsrTx;
impl<T> crate::imc::ChannelHandler<MarkerNsrTx> for T
where
    T: NsrTx
{
}

future_wrapper!(nsr_tx_rdy_op => <T as NsrTx>(cb: *mut ffi::udi_nic_tx_cb_t) val @ {
    val.tx_rdy(unsafe { cb.into_owned() })
});
map_ops_structure!{
    ffi::udi_nsr_tx_ops_t => NsrTx,MarkerNsrTx {
        nsr_tx_rdy_op,
    }
    CBS {
        ffi::udi_nic_tx_cb_t,
    }
}

// --------------------------------------------------------------------

pub trait NdRx: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(fn rx_rdy(&'a mut self, cb: CbHandleNicRx)->() as Future_rx_rdy);
}
struct MarkerNdRx;
impl<T> crate::imc::ChannelHandler<MarkerNdRx> for T
where
    T: NdRx
{
}
future_wrapper!(nd_rx_rdy_op => <T as NdRx>(cb: *mut ffi::udi_nic_rx_cb_t) val @ {
    val.rx_rdy( unsafe { cb.into_owned() } )
});
map_ops_structure!{
    ffi::udi_nd_rx_ops_t => NdRx,MarkerNdRx {
        nd_rx_rdy_op,
    }
    CBS {
        ffi::udi_nic_rx_cb_t,
    }
}
// --------------------------------------------------------------------

pub trait NsrRx: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(fn rx_ind(&'a mut self, cb: CbHandleNicRx)->() as Future_rx_ind);
    async_method!(fn exp_rx_ind(&'a mut self, cb: CbHandleNicRx)->() as Future_exp_rx_ind);
}
struct MarkerNsrRx;
impl<T> crate::imc::ChannelHandler<MarkerNsrRx> for T
where
    T: NsrRx
{
}
future_wrapper!(nsr_rx_ind_op => <T as NsrRx>(cb: *mut ffi::udi_nic_rx_cb_t) val @ {
    val.rx_ind( unsafe { cb.into_owned() } )
});
future_wrapper!(nsr_exp_rx_ind_op => <T as NsrRx>(cb: *mut ffi::udi_nic_rx_cb_t) val @ {
    val.exp_rx_ind( unsafe { cb.into_owned() } )
});
map_ops_structure!{
    ffi::udi_nsr_rx_ops_t => NsrRx,MarkerNsrRx {
        nsr_rx_ind_op,
        nsr_exp_rx_ind_op,
    }
    CBS {
        ffi::udi_nic_rx_cb_t,
    }
}

// --------------------------------------------------------------------

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
