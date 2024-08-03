//! Bus Bridge metalanguage (Phsical I/O Specification)
//! 
use ::udi_sys::meta_bridge::{udi_intr_event_cb_t, udi_intr_attach_cb_t};
use ::udi_sys::meta_bridge::udi_bus_device_ops_t;
use ::udi_sys::meta_bridge::udi_bus_bridge_ops_t;
use ::udi_sys::meta_bridge::udi_bus_bind_cb_t;

/// Reference to a `udi_bus_bind_cb_t`
pub type CbRefBind<'a> = crate::CbRef<'a, udi_bus_bind_cb_t>;
/// Reference to a `udi_intr_attach_cb_t`
pub type CbRefIntrAttach<'a> = crate::CbRef<'a, crate::ffi::meta_bridge::udi_intr_attach_cb_t>;
/// Reference to a `udi_intr_detach_cb_t`
pub type CbRefIntrDetach<'a> = crate::CbRef<'a, crate::ffi::meta_bridge::udi_intr_detach_cb_t>;
/// Owned handle to a `udi_intr_attach_cb_t`
pub type CbHandleIntrAttach<'a> = crate::cb::CbHandle<crate::ffi::meta_bridge::udi_intr_attach_cb_t>;
/// Owned handle to a `udi_intr_detach_cb_t`
pub type CbHandleIntrDetach<'a> = crate::cb::CbHandle<crate::ffi::meta_bridge::udi_intr_detach_cb_t>;


impl crate::ops_markers::ParentBind<::udi_sys::meta_bridge::udi_bus_bind_cb_t> for ::udi_sys::meta_bridge::udi_bus_device_ops_t {
    const ASSERT: () = ();
}
impl crate::ops_markers::ChildBind for ::udi_sys::meta_bridge::udi_bus_bridge_ops_t {
    const ASSERT: () = ();
}

impl_metalanguage!{
    static METALANG_SPEC;
    NAME udi_bridge;
    OPS
        1 => udi_bus_device_ops_t,
        2 => udi_bus_bridge_ops_t,
        3 => ::udi_sys::meta_bridge::udi_intr_handler_ops_t,
        4 => ::udi_sys::meta_bridge::udi_intr_dispatcher_ops_t,
        ;
    CBS
        1 => udi_bus_bind_cb_t,
        2 => ::udi_sys::meta_bridge::udi_intr_attach_cb_t,
        3 => ::udi_sys::meta_bridge::udi_intr_detach_cb_t,
        4 => ::udi_sys::meta_bridge::udi_intr_event_cb_t : BUF event_buf,
        ;
}

/// Preferred endianness of the bus, used in [BusDevice::bus_bind_ack]
///
/// "... device endianness which works most effectively with the bridges in this path."
pub enum PreferredEndianness {
    /// Any endianness will work just as well
    Any,
    /// Little endian preferred (least significant byte first)
    Little,
    /// Big endian preferred (most significant byte first)
    Big,
}

/// Trait for a device on a bus
pub trait BusDevice: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    async_method!(
        /// Acknowledge a successful binding of this device to the bus
        fn bus_bind_ack(&'a self,
            cb: crate::CbRef<'a, udi_bus_bind_cb_t>,
            dma_constraints: crate::physio::dma::DmaConstraints,
            preferred_endianness: PreferredEndianness,
            status: crate::ffi::udi_status_t
            )->crate::Result<()>
        as Future_bind_ack
    );

    async_method!(
        /// Acknowledge unbinding of this device driver from the bus
        fn bus_unbind_ack(&'a self, cb: CbRefBind<'a>) -> ()
        as Future_unbind_ack
    );
    async_method!(
        /// Acknowledge the attachment of an interrupt, with status
        fn intr_attach_ack(&'a self, cb: CbRefIntrAttach<'a>, status: crate::ffi::udi_status_t) -> ()
        as Future_intr_attach_ack
    );
    async_method!(
        /// Acknowledge successful detatchment of an interrupt
        fn intr_detach_ack(&'a self, cb: CbRefIntrDetach<'a>) -> ()
        as Future_intr_detach_ack
    );

    /// Return/release an interrupt-attach CB
    fn intr_attach_cb_ret(&mut self, cb: CbHandleIntrAttach) { let _ = cb;}
    /// Return/release an interrupt-detach CB
    fn intr_detach_cb_ret(&mut self, cb: CbHandleIntrDetach) { let _ = cb;}
}
struct MarkerBusDevice;
impl<T> crate::imc::ChannelHandler<MarkerBusDevice> for T
where
    T: BusDevice,
{
    fn channel_bound(&mut self, params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
        // SAFE: Trusting that this trait is only being used through proper config.
        unsafe {
            crate::ffi::meta_bridge::udi_bus_bind_req(params.parent_bound.bind_cb as *mut udi_bus_bind_cb_t);
        }
    }
}


/// Trait for a bus bridge (i.e. the bus itself. well, the binding between the bus and UDI for a device)
pub trait BusBridge: 'static + crate::imc::ChannelInit + crate::async_trickery::CbContext {
    async_method!(
        /// Handle a request to bind to the bus
        fn bus_bind_req(&'a self, cb: CbRefBind<'a>) -> crate::Result<(PreferredEndianness,)>
        as Future_bind_req
    );
    async_method!(
        /// Handle a request to unbind from the bus
        fn bus_unbind_req(&'a self, cb: CbRefBind<'a>) -> ()
        as Future_unbind_req
    );
    async_method!(
        /// Handle a request to attach an interrupt handler to an interrupt
        fn intr_attach_req(&'a self, cb: CbRefIntrAttach<'a>) -> crate::Result<()>
        as Future_intr_attach_req
    );
    async_method!(
        /// Handle a request to detach a previously attached interrupt handler
        fn intr_detach_req(&'a self, cb: CbRefIntrDetach<'a>) -> ()
        as Future_intr_detach_req
    );
}
struct MarkerBusBridge;
impl<T> crate::imc::ChannelHandler<MarkerBusBridge> for T
where
    T: BusBridge
{
}

future_wrapper!(bus_bind_ack_op => <T as BusDevice>(
    cb: *mut udi_bus_bind_cb_t,
    dma_constraints: crate::ffi::physio::udi_dma_constraints_t,
    preferred_endianness: u8,
    status: crate::ffi::udi_status_t
    ) val @ {
    let preferred_endianness = match preferred_endianness
        {
        crate::ffi::meta_bridge::UDI_DMA_ANY_ENDIAN => PreferredEndianness::Any,
        crate::ffi::meta_bridge::UDI_DMA_BIG_ENDIAN => PreferredEndianness::Big,
        crate::ffi::meta_bridge::UDI_DMA_LITTLE_ENDIAN => PreferredEndianness::Little,
        _ => PreferredEndianness::Any,
        };
    // SAFE: This comes from the environment, so is correct
    let dma_constraints = unsafe { crate::physio::dma::DmaConstraints::from_raw(dma_constraints) };
    val.bus_bind_ack(cb, dma_constraints, preferred_endianness, status)
} finally(res) {
    unsafe { crate::async_trickery::channel_event_complete::<T,udi_bus_bind_cb_t>(cb, crate::Error::to_status(res)) }
});
future_wrapper!(bus_unbind_ack_op => <T as BusDevice>(cb: *mut udi_bus_bind_cb_t) val @ {
    val.bus_unbind_ack(cb)
} finally( () ) {
    unsafe { crate::async_trickery::channel_event_complete::<T,udi_bus_bind_cb_t>(cb, 0 /*res*/) }
});
future_wrapper!(intr_attach_ack_op => <T as BusDevice>(cb: *mut crate::ffi::meta_bridge::udi_intr_attach_cb_t, status: crate::ffi::udi_status_t) val @ {
    val.intr_attach_ack(cb, status)
} finally( () ) {
    val.intr_attach_cb_ret(unsafe { CbHandleIntrAttach::from_raw(cb) });
});
future_wrapper!(intr_detach_ack_op => <T as BusDevice>(cb: *mut crate::ffi::meta_bridge::udi_intr_detach_cb_t) val @ {
    val.intr_detach_ack(cb)
} finally( () ) {
    val.intr_detach_cb_ret(unsafe { CbHandleIntrDetach::from_raw(cb) });
});
map_ops_structure!{
    ::udi_sys::meta_bridge::udi_bus_device_ops_t => BusDevice,MarkerBusDevice {
        bus_bind_ack_op,
        bus_unbind_ack_op,
        intr_attach_ack_op,
        intr_detach_ack_op,
    }
    CBS {
        udi_bus_bind_cb_t,
        ::udi_sys::meta_bridge::udi_intr_attach_cb_t,
        ::udi_sys::meta_bridge::udi_intr_detach_cb_t,
    }
}
// --------------------------------------------------------------------

future_wrapper!(bus_bind_req_op => <T as BusBridge>(cb: *mut udi_bus_bind_cb_t) val @ {
    val.bus_bind_req(cb)
} finally(res) {
    unsafe {
        let (status,dma,endian) = match res
            {
            Ok((endian,)) => {
                let endian = match endian
                    {
                    PreferredEndianness::Any => crate::ffi::meta_bridge::UDI_DMA_ANY_ENDIAN,
                    PreferredEndianness::Big => crate::ffi::meta_bridge::UDI_DMA_BIG_ENDIAN,
                    PreferredEndianness::Little => crate::ffi::meta_bridge::UDI_DMA_LITTLE_ENDIAN,
                    };
                (0,crate::ffi::physio::UDI_NULL_DMA_CONSTRAINTS,endian)
                },
            Err(e) => (e.into_inner(),crate::ffi::physio::UDI_NULL_DMA_CONSTRAINTS,0),
            };
        crate::ffi::meta_bridge::udi_bus_bind_ack(cb, dma, endian, status)
    }
});
future_wrapper!(bus_unbind_req_op => <T as BusBridge>(cb: *mut udi_bus_bind_cb_t) val @ {
    val.bus_unbind_req(cb)
} finally(res) {
    let () = res;
    unsafe { crate::ffi::meta_bridge::udi_bus_unbind_ack(cb) }
});
future_wrapper!(intr_attach_req_op => <T as BusBridge>(cb: *mut crate::ffi::meta_bridge::udi_intr_attach_cb_t) val @ {
    val.intr_attach_req(cb)
} finally(res) {
    unsafe { crate::ffi::meta_bridge::udi_intr_attach_ack(cb, crate::Error::to_status(res)) }
});
future_wrapper!(intr_detach_req_op => <T as BusBridge>(cb: *mut crate::ffi::meta_bridge::udi_intr_detach_cb_t) val @ {
    val.intr_detach_req(cb)
} finally( () ) {
    unsafe { crate::ffi::meta_bridge::udi_intr_detach_ack(cb) }
});
map_ops_structure!{
    ::udi_sys::meta_bridge::udi_bus_bridge_ops_t => BusBridge,MarkerBusBridge {
        bus_bind_req_op,
        bus_unbind_req_op,
        intr_attach_req_op,
        intr_detach_req_op,
    }
    CBS {
        udi_bus_bind_cb_t,
        ::udi_sys::meta_bridge::udi_intr_attach_cb_t,
        ::udi_sys::meta_bridge::udi_intr_detach_cb_t,
    }
}

/// Initiate an interrupt attach operation
pub fn intr_attach_req(cb: super::cb::CbHandle<udi_intr_attach_cb_t>) {
    unsafe { crate::ffi::meta_bridge::udi_intr_attach_req(cb.into_raw()) }
}
/// Return a handled interrupt event CB to the bus driver 
pub fn intr_event_rdy(cb: CbHandleEvent) {
    unsafe { crate::ffi::meta_bridge::udi_intr_event_rdy(cb.into_raw()) }
}

/// Reference to a `udi_intr_event_cb_t`
pub type CbRefEvent<'a> = crate::CbRef<'a, udi_intr_event_cb_t>;
/// Owned handle to a `udi_intr_event_cb_t`
pub type CbHandleEvent = crate::cb::CbHandle<udi_intr_event_cb_t>;

impl super::cb::CbHandle<udi_intr_attach_cb_t>
{
    /// Populate a `udi_intr_attach_cb_t` with a basic set of information
    pub fn init(&mut self, interrupt_index: ::udi_sys::udi_index_t, min_event_pend: u8, preprocessing_handle: crate::pio::Handle) {
        let cb = unsafe { self.get_mut() };
        cb.interrupt_index = interrupt_index;
        cb.min_event_pend = min_event_pend;
        cb.preprocessing_handle = preprocessing_handle.as_raw();
    }
}

/// Trait for an interrupt handler endpoint
pub trait IntrHandler: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit
{
    async_method!(
        /// Handle an interrupt indication
        ///
        /// TODO: Translate flags - `UDI_INTR_MASKING_NOT_REQUIRED`, `UDI_INTR_OVERRUN_OCCURRED`, `UDI_INTR_PREPROCESSED`
        fn intr_event_ind(&'a self, cb: CbRefEvent<'a>, flags: u8)->()
        as Future_intr_event_ind
    );
}
struct MarkerIntrHandler;
impl<T> crate::imc::ChannelHandler<MarkerIntrHandler> for T
where
    T: IntrHandler
{
}

future_wrapper!(intr_event_ind_op => <T as IntrHandler>(cb: *mut udi_intr_event_cb_t, flags: u8) val @ {
    val.intr_event_ind(cb, flags)
} finally( () ) {
    // Return this CB to the pool on completion
    unsafe { crate::ffi::meta_bridge::udi_intr_event_rdy(cb) }
});
map_ops_structure!{
    ::udi_sys::meta_bridge::udi_intr_handler_ops_t => IntrHandler,MarkerIntrHandler {
        intr_event_ind_op,
    }
    CBS {
        udi_intr_event_cb_t,
    }
}


/// Interrupt dispatcher trait (the endpoint that sends interrupt events)
pub trait IntrDispatcher: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit
{
    async_method!(
        /// Handle an interrupt being handled (or just a new event CB being available/returned for use)
        fn intr_event_rdy(&'a self, cb: CbRefEvent)->()
        as Future_intr_event_rdy
    );
    /// Take ownership of an incoming event CB
    fn intr_event_ret(&mut self, cb: CbHandleEvent);
}
struct MarkerIntrDispatcher;
impl<T> crate::imc::ChannelHandler<MarkerIntrDispatcher> for T
where
    T: IntrDispatcher
{
}

future_wrapper!(intr_event_rdy_op => <T as IntrDispatcher>(cb: *mut udi_intr_event_cb_t) val @ {
    val.intr_event_rdy(cb)
} finally( () ) {
    val.intr_event_ret(unsafe { crate::cb::CbHandle::from_raw(cb) })
});
map_ops_structure!{
    ::udi_sys::meta_bridge::udi_intr_dispatcher_ops_t => IntrDispatcher,MarkerIntrDispatcher {
        intr_event_rdy_op,
    }
    CBS {
        udi_intr_event_cb_t,
    }
}
