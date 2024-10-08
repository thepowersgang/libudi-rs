/*!
 * Network Interface Card Metalanguage
 * 
 * # Safety of Rx/Tx traits
 * The [NdTx]/[NdRx]/[NsrTx]/[NsrRx] traits are all marked as `unsafe` as they
 * pass an owning handle to the control block to the method. If this control
 * block is dropped before the future completes, the future will read freed
 * memory.
 * 
 * The implementation of these methods can avoid UB by storing the cb passed in the
 * region data after the last async call. This ensures that it will not be reused
 * until after the future has been cleaned up.
 * 
 * # Terms
 * - ND: "Network Device" - the network card driver
 * - NSR: "Network Service Requestor" - The UDI network stack interface
 * - expedited - There are operations that request high-priority handling of packets.
 *               It is expected that these packets are handled (sent or processed) before
 *               any lower-priority packets are.
 */
use crate::ffi::udi_index_t;
use crate::ffi::meta_nic as ffi;

/// Request enabling of a network device
pub fn nd_enable_req(cb: crate::cb::CbHandle<ffi::udi_nic_cb_t>) {
    unsafe { ffi::udi_nd_enable_req(cb.into_raw()) }
}
/// Request disabling of a network device
pub fn nd_disable_req(cb: crate::cb::CbHandle<ffi::udi_nic_cb_t>) {
    unsafe { ffi::udi_nd_disable_req(cb.into_raw()) }
}
/// Send a control request to a network device
pub fn nd_ctrl_req(cb: crate::cb::CbHandle<ffi::udi_nic_ctrl_cb_t>) {
    unsafe { ffi::udi_nd_ctrl_req(cb.into_raw()) }
}
/// Send a information request to a network device
pub fn nd_info_req(cb: crate::cb::CbHandle<ffi::udi_nic_info_cb_t>, reset_statistics: bool) {
    unsafe { ffi::udi_nd_info_req(cb.into_raw(), reset_statistics.into()) }
}
/// Inform the NSR that a device's status has changed
pub fn nsr_status_ind(cb: crate::cb::CbHandle<ffi::udi_nic_status_cb_t>) {
    unsafe { ffi::udi_nsr_status_ind(cb.into_raw()) }
}

/// Inform the NSR that a packet has been recived
pub fn nsr_rx_ind(rx_cb: CbHandleNicRx) {
    unsafe { ffi::udi_nsr_rx_ind(rx_cb.into_raw()) }
}
/// Hand/return the network device a CB to use for incoming packets
pub fn nd_rx_rdy(cb: CbHandleNicRx) {
    unsafe { ffi::udi_nd_rx_rdy(cb.into_raw()) }
}
/// Request the network device send a packet
pub fn nd_tx_req(tx_cb: CbHandleNicTx) {
    unsafe { ffi::udi_nd_tx_req(tx_cb.into_raw()) }
}
/// Request the network device send a packet (expedited)
pub fn nd_exp_tx_req(tx_cb: CbHandleNicTx) {
    unsafe { ffi::udi_nd_exp_tx_req(tx_cb.into_raw()) }
}
/// Hand the NSR a CB to use for outgoing packets
pub fn nsr_tx_rdy(cb: CbHandleNicTx) {
    unsafe { ffi::udi_nsr_tx_rdy(cb.into_raw()) }
}

macro_rules! def_cb {
    (unsafe $ref_name:ident => $t:ty : $cb_num:expr) => {
        #[doc=concat!("Reference to a [", stringify!($t), "]")]
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
/// An owned handle to a [ffi::udi_nic_tx_cb_t]
pub type CbHandleNicTx = crate::cb::CbHandle<ffi::udi_nic_tx_cb_t>;
/// An owned handle to a [ffi::udi_nic_rx_cb_t]
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
    /// Remove this CB from its current chain (i.e. `pop_front`)
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

    /// Get a reference to the buffer
    pub fn tx_buf_ref(&self) -> &crate::buf::Handle {
        unsafe { crate::buf::Handle::from_ref( &self.tx_buf ) }
    }
    /// Get a mutable reference to the buffer
    pub fn tx_buf_mut(&mut self) -> &mut crate::buf::Handle {
        unsafe { crate::buf::Handle::from_mut( &mut self.get_mut().tx_buf ) }
    }
}
impl crate::cb::CbRef<'_, ffi::udi_nic_tx_cb_t>
{
    /// Get a reference to the buffer
    pub fn tx_buf_ref(&self) -> &crate::buf::Handle {
        unsafe { crate::buf::Handle::from_ref( &self.tx_buf ) }
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
    /// Remove this CB from its current chain (i.e. `pop_front`)
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

    /// Get a reference to the buffer
    pub fn rx_buf_ref(&self) -> &crate::buf::Handle {
        unsafe { crate::buf::Handle::from_ref( &self.rx_buf ) }
    }
    /// Get a mutable reference to the buffer
    pub fn rx_buf_mut(&mut self) -> &mut crate::buf::Handle {
        unsafe { crate::buf::Handle::from_mut( &mut self.get_mut().rx_buf ) }
    }

    //pub fn set_rx_status(&mut self, 
}

impl CbRefNicRx<'_>
{
    /// Get a reference to the buffer
    pub fn rx_buf_ref(&self) -> &crate::buf::Handle {
        unsafe { crate::buf::Handle::from_ref( &self.rx_buf ) }
    }
}

/// A FIFO queue of RX CBs
#[derive(Default)]
pub struct ReadCbQueue( crate::cb::SharedQueue<ffi::udi_nic_rx_cb_t> );
impl ReadCbQueue
{
    /// Create a new (empty) queue
    pub const fn new() -> Self {
        Self(crate::cb::SharedQueue::new())
    }
    /// Push a CB onto the back of the queue
    pub fn push(&self, cb: CbHandleNicRx) {
        self.0.push_back(cb)
    }
    /// Pop a CB from the front of the queue
    pub fn pop(&self) -> Option< CbHandleNicRx > {
        self.0.pop_front()
    }
}

#[cfg(any())]
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

/// Trait for the control endpoint on a network device
pub trait Control: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(
        /// Handle a binding request with a NSR
        ///
        /// The `tx_chan_index` and `rx_chan_index` values are channel indexes used to match created channels
        /// with the channel halves already created by the NSR
        fn bind_req(&'a self, cb: CbRefNicBind<'a>, tx_chan_index: udi_index_t, rx_chan_index: udi_index_t)->crate::Result<NicInfo>
        as Future_bind_req
    );
    async_method!(
        /// Handle a request to unbind from the NSR
        fn unbind_req(&'a self, cb: CbRefNic<'a>)->crate::Result<()>
        as Future_unbind_req
    );
    async_method!(
        /// Handle a request to enable full operation (e.g. RX) of the device
        fn enable_req(&'a self, cb: CbRefNic<'a>)->crate::Result<()>
        as Future_enable_req
    );
    async_method!(
        /// Handle a request to disable operation of the device
        fn disable_req(&'a self, cb: CbRefNic<'a>)->()
        as Future_disable_req
    );
    async_method!(
        /// Handle a control request
        fn ctrl_req(&'a self, cb: CbRefNicCtrl<'a>)->crate::Result<()>
        as Future_ctrl_req
    );
    async_method!(
        /// Handle an information request
        ///
        /// - `reset_statistics` is a request to reset the internal statistics counters
        fn info_req(&'a self, cb: CbRefNicInfo<'a>, reset_statistics: bool)->()
        as Future_info_req
    );
}
struct MarkerControl;
impl<T> crate::imc::ChannelHandler<MarkerControl> for T
where
    T: Control
{
}

future_wrapper!(nd_bind_req_op => <T as Control>(cb: *mut ffi::udi_nic_bind_cb_t, tx_chan_index: udi_index_t, rx_chan_index: udi_index_t)
    val @ {
        val.bind_req(cb, tx_chan_index, rx_chan_index)
    } finally(res) {
        // SAFE: Correct FFI and CB access
        unsafe {
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
    });
future_wrapper!(nd_unbind_req_op => <T as Control>(cb: *mut ffi::udi_nic_cb_t) val @ {
    val.unbind_req(cb)
});
future_wrapper!(nd_enable_req_op => <T as Control>(cb: *mut ffi::udi_nic_cb_t) val @ {
    val.enable_req(cb)
} finally(res) {
    // SAFE: Correct FFIs
    unsafe { ffi::udi_nsr_enable_ack(cb, crate::Error::to_status(res)) }
});
future_wrapper!(nd_disable_req_op => <T as Control>(cb: *mut ffi::udi_nic_cb_t) val @ {
    val.disable_req(cb)
});
future_wrapper!(nd_ctrl_req_op => <T as Control>(cb: *mut ffi::udi_nic_ctrl_cb_t) val @ {
    val.ctrl_req(cb)
} finally(res) {
    // SAFE: Correct FFIs
    unsafe { ffi::udi_nsr_ctrl_ack(cb, crate::Error::to_status(res)) }
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
/// Bind channel indexes between `bind_ack` and `bind_req`
pub struct BindChannels {
    /// Transmit channel index
    pub tx: udi_index_t,
    /// Recieve channel index
    pub rx: udi_index_t,
}

/// Trait for the NSR's side of the control channel
pub trait NsrControl: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(
        /// Get the bind channel indexes pass to the ND when binding (called when the control channel is bound)
        fn get_bind_channels(&'a self, cb: CbRefNicBind<'a>)->BindChannels
        as Future_gbc
    );
    async_method!(
        /// Acknowlegement of binding to the ND, with result
        fn bind_ack(&'a self, cb: CbRefNicBind<'a>, res: crate::Result<()>)->()
        as Future_bind_ack
    );
    async_method!(
        /// Acknowledgement of unbinding
        fn unbind_ack(&'a self, cb: CbRefNic<'a>, res: crate::Result<()>)->()
        as Future_unbind_ack
    );
    async_method!(
        /// Acknowledgemnet of the device being enabled, with result
        fn enable_ack(&'a self, cb: CbRefNic<'a>, res: crate::Result<()>)->()
        as Future_enable_ack
    );
    async_method!(
        /// Handle the response to an control request (see [nd_ctrl_req])
        fn ctrl_ack(&'a self, cb: CbRefNicCtrl<'a>, res: crate::Result<()>)->()
        as Future_ctrl_ack
    );
    async_method!(
        /// Handle the response to an information request (see [nd_info_req])
        fn info_ack(&'a self, cb: CbRefNicInfo<'a>)->()
        as Future_info_ack
    );
    async_method!(
        /// Handle a change in status from the device
        fn status_ind(&'a self, cb: CbRefNicStatus<'a>)->()
        as Future_status_ind
    );
    /// Return/release a generic NIC CB
    fn ret_cb_nic(&self, cb: crate::cb::CbHandle<ffi::udi_nic_cb_t>) { let _ = cb; }
    /// Return/release a control CB
    fn ret_cb_nic_ctrl(&self, cb: crate::cb::CbHandle<ffi::udi_nic_ctrl_cb_t>) { let _ = cb; }
    /// Return/release an info CB
    fn ret_cb_nic_info(&self, cb: crate::cb::CbHandle<ffi::udi_nic_info_cb_t>) { let _ = cb; }
}
future_wrapper!(nsr_channel_bound => <T as NsrControl>(cb: *mut ffi::udi_nic_bind_cb_t) val @ {
    val.get_bind_channels(cb)
} finally(chans) {
    // SAFE: Correct FFI
    unsafe { ffi::udi_nd_bind_req(cb, chans.tx, chans.rx) }
});
struct MarkerNsrControl;
impl<T> crate::imc::ChannelHandler<MarkerNsrControl> for T
where
    T: NsrControl
{
    fn channel_bound(&self, params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
        // SAFE: We're assuming that the channel is being correctly bound to a parent
        unsafe {
            // Start a UDI async using the bind CB
            let cb = params.parent_bound.bind_cb as *mut ffi::udi_nic_bind_cb_t;
            nsr_channel_bound::<T>(cb)
        }
    }
}

future_wrapper!(nsr_bind_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_bind_cb_t, status: ::udi_sys::udi_status_t) val @ {
    let res = crate::Error::from_status(status);
    val.bind_ack(cb, res)
} finally( () ) {
    // SAFE: Owns the cb, correct FFI
    unsafe { crate::async_trickery::channel_event_complete::<T,ffi::udi_nic_bind_cb_t>(cb, ::udi_sys::UDI_OK as _) }
});
future_wrapper!(nsr_unbind_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_cb_t, status: ::udi_sys::udi_status_t) val @ {
    val.unbind_ack(cb, crate::Error::from_status(status))
} finally( () ) {
    // SAFE: Owns the CB
    val.ret_cb_nic(unsafe { crate::cb::CbHandle::from_raw(cb) });
});
future_wrapper!(nsr_enable_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_cb_t, status: ::udi_sys::udi_status_t) val @ {
    val.enable_ack(cb, crate::Error::from_status(status))
} finally( () ) {
    // SAFE: Owns the CB
    val.ret_cb_nic(unsafe { crate::cb::CbHandle::from_raw(cb) });
});
future_wrapper!(nsr_ctrl_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_ctrl_cb_t, status: ::udi_sys::udi_status_t) val @ {
    val.ctrl_ack(cb, crate::Error::from_status(status))
} finally( () ) {
    // SAFE: Owns the CB
    val.ret_cb_nic_ctrl(unsafe { crate::cb::CbHandle::from_raw(cb) });
});
future_wrapper!(nsr_info_ack_op => <T as NsrControl>(cb: *mut ffi::udi_nic_info_cb_t) val @ {
    val.info_ack(cb)
} finally( () ) {
    // SAFE: Owns the CB
    val.ret_cb_nic_info(unsafe { crate::cb::CbHandle::from_raw(cb) });
});
future_wrapper!(nsr_status_ind_op => <T as NsrControl>(cb: *mut ffi::udi_nic_status_cb_t) val @ {
    val.status_ind(cb)
} finally( () ) {
    // See the docs for `udi_nsr_status_ind` - Expected to deallocate the CB
    // SAFE: Owns the CB
    drop(unsafe { crate::cb::CbHandle::from_raw(cb) });
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

/// SAFETY:
/// The implementations of `tx_req`/`exp_tx_req` shall ensure that the
/// cb is not dropped until after the future completes.
/// 
/// Failure to do so can lead to crashes. See the module documentation for more details
pub unsafe trait NdTx: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(
        /// Schedule transmission of a packet, using normal priority rules
        fn tx_req(&'a self, cb: CbHandleNicTx)->()
        as Future_tx_req
    );
    async_method!(
        /// Schedule transmission of a packet, expediting the transmission
        fn exp_tx_req(&'a self, cb: CbHandleNicTx)->()
        as Future_exp_tx_req
    );
}
struct MarkerNdTx;
impl<T> crate::imc::ChannelHandler<MarkerNdTx> for T
where
    T: NdTx
{
}

future_wrapper!(nd_tx_req_op => <T as NdTx>(cb: *mut ffi::udi_nic_tx_cb_t) val @ {
    // SAFE: Trait is unsafe
    val.tx_req(unsafe { cb.into_owned() })
});
future_wrapper!(nd_exp_tx_req_op => <T as NdTx>(cb: *mut ffi::udi_nic_tx_cb_t) val @ {
    // SAFE: Trait is unsafe
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

/// SAFETY:
/// The implementations of `tx_rdy` shall ensure that the cb is not
/// dropped until after the future completes.
/// 
/// Failure to do so can lead to crashes. See the module documentation for more details
pub unsafe trait NsrTx: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(
        /// Return a TX CB to the pool
        fn tx_rdy(&'a self, cb: CbHandleNicTx)->()
        as Future_tx_rdy
    );
}
struct MarkerNsrTx;
impl<T> crate::imc::ChannelHandler<MarkerNsrTx> for T
where
    T: NsrTx
{
}

future_wrapper!(nsr_tx_rdy_op => <T as NsrTx>(cb: *mut ffi::udi_nic_tx_cb_t) val @ {
    // SAFE: Trait is unsafe
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

/// SAFETY:
/// The implementations of `rx_rdy` shall ensure that the cb is not
/// dropped until after the future completes.
/// 
/// Failure to do so can lead to crashes. See the module documentation for more details
pub unsafe trait NdRx: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(
        /// Add a RX CB to the pool of available CBs for incoming packets
        fn rx_rdy(&'a self, cb: CbHandleNicRx)->()
        as Future_rx_rdy
    );
}
struct MarkerNdRx;
impl<T> crate::imc::ChannelHandler<MarkerNdRx> for T
where
    T: NdRx
{
}
future_wrapper!(nd_rx_rdy_op => <T as NdRx>(cb: *mut ffi::udi_nic_rx_cb_t) val @ {
    // SAFE: The trait is unsafe
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
/// Network Service Requester - Receive operations
/// 
/// SAFETY:
/// The implementations of `rx_ind`/`exp_rx_ind` shall ensure that the
/// cb is not dropped until after the future completes.
/// 
/// Failure to do so can lead to crashes. See the module documentation for more details
pub unsafe trait NsrRx: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!{
        /// Indication of a received packet
        fn rx_ind(&'a self, cb: CbRefNicRx<'a>)->() as Future_rx_ind
    }
    async_method!(
        /// Indication of a newly recivied packet that should be processed in an expedited manner
        fn exp_rx_ind(&'a self, cb: CbRefNicRx<'a>)->()
        as Future_exp_rx_ind
    );
    /// Return the CB to either the ND or release it
    fn rx_cb_ret(&self, cb: CbHandleNicRx);
}
struct MarkerNsrRx;
impl<T> crate::imc::ChannelHandler<MarkerNsrRx> for T
where
    T: NsrRx
{
}
future_wrapper!(nsr_rx_ind_op => <T as NsrRx>(cb: *mut ffi::udi_nic_rx_cb_t) val @ {
    val.rx_ind(cb)
} finally( () ) {
    // SAFE: CB is valid
    val.rx_cb_ret(unsafe { CbHandleNicRx::from_raw(cb) });
});
future_wrapper!(nsr_exp_rx_ind_op => <T as NsrRx>(cb: *mut ffi::udi_nic_rx_cb_t) val @ {
    val.exp_rx_ind(cb)
} finally( () ) {
    // SAFE: CB is valid
    val.rx_cb_ret(unsafe { CbHandleNicRx::from_raw(cb) });
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
    /// Type of media used for the network device
    pub media_type: ffi::MediaType,
    /// Minimum packet size the device can send - if zero then a value is assumed from `media_type`
    pub min_pdu_size: u32,
    /// Maximum packet size the device can send - if zero then a value is assumed from `media_type`
    pub max_pdu_size: u32,
    /// A hint to the NSR as to how many RX slots the device has (thus how many RX CBs to allocate)
    pub rx_hw_threshold: u32,
    /// Bitset listing device capabilities
    pub capabilities: u32,
    /// Number of multicast addresses that can be perfectly handled (i.e. not using fuzzy matching)
    pub max_perfect_multicast: u8,
    /// Total number of multicast addresses that can be handled
    pub max_total_multicast: u8,
    /// Length of the MAC (physical layer) address
    pub mac_addr_len: u8,
    /// MAC (physical layer) address of the network device
    pub mac_addr: [u8; ffi::UDI_NIC_MAC_ADDRESS_SIZE],
}
