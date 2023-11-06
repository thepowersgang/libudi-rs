//! Bus Bridge metalanguage (Phsical I/O Specification)
//! 
//! 
use ::udi_sys::meta_bus::udi_bus_device_ops_t;
use ::udi_sys::meta_bus::udi_bus_bridge_ops_t;
use ::udi_sys::meta_bus::udi_bus_bind_cb_t;

pub type CbRefBind<'a> = crate::CbRef<'a, udi_bus_bind_cb_t>;
pub type CbRefIntrAttach<'a> = crate::CbRef<'a, crate::ffi::meta_intr::udi_intr_attach_cb_t>;
pub type CbRefIntrDetach<'a> = crate::CbRef<'a, crate::ffi::meta_intr::udi_intr_detach_cb_t>;

impl_metalanguage!{
    static METALANG_SPEC;
    NAME udi_bridge;
    OPS
        1 => udi_bus_device_ops_t,
        2 => udi_bus_bridge_ops_t,
        3 => ::udi_sys::meta_intr::udi_intr_handler_ops_t,
        4 => ::udi_sys::meta_intr::udi_intr_dispatcher_ops_t,
        ;
    CBS
        1 => udi_bus_bind_cb_t,
        2 => ::udi_sys::meta_intr::udi_intr_attach_cb_t,
        3 => ::udi_sys::meta_intr::udi_intr_detach_cb_t,
        4 => ::udi_sys::meta_intr::udi_intr_event_cb_t : BUF event_buf,
        ;
}

pub enum PreferredEndianness {
    Any,
    Little,
    Big,
}

/// Trait for a device on a bus
pub trait BusDevice: 'static {
    async_method!(fn bus_bind_ack(&'a mut self,
        cb: crate::CbRef<'a, udi_bus_bind_cb_t>,
        dma_constraints: crate::ffi::physio::udi_dma_constraints_t,
        preferred_endianness: PreferredEndianness,
        status: crate::ffi::udi_status_t
        )->crate::Result<()> as Future_bind_ack
    );

    async_method!(fn bus_unbind_ack(&'a mut self, cb: CbRefBind<'a>) -> () as Future_unbind_ack);
    async_method!(fn intr_attach_ack(&'a mut self, cb: CbRefIntrAttach<'a>, status: crate::ffi::udi_status_t) -> () as Future_intr_attach_ack);
    async_method!(fn intr_detach_ack(&'a mut self, cb: CbRefIntrDetach<'a>) -> () as Future_intr_detach_ack);
}
struct MarkerBusDevice;
impl<T> crate::imc::ChannelHandler<MarkerBusDevice> for T
where
    T: BusDevice
{
    fn channel_closed(&mut self) {
    }
    fn channel_bound(&mut self, params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
        // SAFE: Trusting that this trait is only being used through proper config.
        unsafe {
            crate::ffi::meta_bus::udi_bus_bind_req(params.parent_bound.bind_cb as *mut udi_bus_bind_cb_t);
        }
    }
}

pub trait BusBridge: 'static {
    async_method!(fn bus_bind_req(&'a mut self, cb: CbRefBind<'a>) -> crate::Result<(PreferredEndianness,)> as Future_bind_req);
    async_method!(fn bus_unbind_req(&'a mut self, cb: CbRefBind<'a>) -> () as Future_unbind_req);
    async_method!(fn intr_attach_req(&'a mut self, cb: CbRefIntrAttach<'a>) -> crate::Result<()> as Future_intr_attach_req);
    async_method!(fn intr_detach_req(&'a mut self, cb: CbRefIntrDetach<'a>) -> () as Future_intr_detach_req);
}
struct MarkerBusBridge;
impl<T> crate::imc::ChannelHandler<MarkerBusBridge> for T
where
    T: BusBridge
{
    fn channel_closed(&mut self) {
    }
    fn channel_bound(&mut self, _params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
        // SAFE: Trusting that this trait is only being used through proper config.
        //unsafe {
        //    crate::ffi::meta_bus::udi_bus_bind_req(params.parent_bound.bind_cb as *mut udi_bus_bind_cb_t);
        //}
    }
}

future_wrapper!(bus_bind_ack_op => <T as BusDevice>(
    cb: *mut udi_bus_bind_cb_t,
    dma_constraints: crate::ffi::physio::udi_dma_constraints_t,
    preferred_endianness: u8,
    status: crate::ffi::udi_status_t
    ) val @ {
    let preferred_endianness = match preferred_endianness
        {
        crate::ffi::meta_bus::UDI_DMA_ANY_ENDIAN => PreferredEndianness::Any,
        crate::ffi::meta_bus::UDI_DMA_BIG_ENDIAN => PreferredEndianness::Big,
        crate::ffi::meta_bus::UDI_DMA_LITTLE_ENDIAN => PreferredEndianness::Little,
        _ => PreferredEndianness::Any,
        };
    crate::async_trickery::with_ack(
        val.bus_bind_ack(cb, dma_constraints, preferred_endianness, status),
        |cb,res| unsafe { crate::async_trickery::channel_event_complete::<udi_bus_bind_cb_t>(cb, crate::Error::to_status(res)) }
        )
});
future_wrapper!(bus_unbind_ack_op => <T as BusDevice>(cb: *mut udi_bus_bind_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.bus_unbind_ack(cb),
        |cb,_res| unsafe { crate::async_trickery::channel_event_complete::<udi_bus_bind_cb_t>(cb, 0 /*res*/) }
        )
});
future_wrapper!(intr_attach_ack_op => <T as BusDevice>(cb: *mut crate::ffi::meta_intr::udi_intr_attach_cb_t, status: crate::ffi::udi_status_t) val @ {
    val.intr_attach_ack(cb, status)
});
future_wrapper!(intr_detach_ack_op => <T as BusDevice>(cb: *mut crate::ffi::meta_intr::udi_intr_detach_cb_t) val @ {
    val.intr_detach_ack(cb)
});

impl<T,CbList> crate::OpsStructure<::udi_sys::meta_bus::udi_bus_device_ops_t, T,CbList>
where
	T: BusDevice,
    CbList: crate::HasCb<udi_bus_bind_cb_t>,
    CbList: crate::HasCb<crate::ffi::meta_intr::udi_intr_attach_cb_t>,
    CbList: crate::HasCb<crate::ffi::meta_intr::udi_intr_detach_cb_t>,
{
    pub const fn scratch_requirement() -> usize {
        let rv = crate::imc::task_size::<T,MarkerBusDevice>();
        let rv = crate::const_max(rv, bus_bind_ack_op::task_size::<T>());
        let rv = crate::const_max(rv, bus_unbind_ack_op::task_size::<T>());
        let rv = crate::const_max(rv, intr_attach_ack_op::task_size::<T>());
        let rv = crate::const_max(rv, intr_detach_ack_op::task_size::<T>());
        rv
    }
    /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
    /// SAFETY: The scratch size must be >= value returned by [Self::scratch_requirement]
    pub const unsafe fn for_driver() -> udi_bus_device_ops_t {
        return udi_bus_device_ops_t {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T, MarkerBusDevice>,
            bus_bind_ack_op: bus_bind_ack_op::<T>,
            bus_unbind_ack_op: bus_unbind_ack_op::<T>,
            intr_attach_ack_op: intr_attach_ack_op::<T>,
            intr_detach_ack_op: intr_detach_ack_op::<T>,
        };
    }
}


future_wrapper!(bus_bind_req_op => <T as BusBridge>(
    cb: *mut udi_bus_bind_cb_t
    ) val @ {
    crate::async_trickery::with_ack(
        val.bus_bind_req(cb),
        |cb,res| unsafe {
            let (status,dma,endian) = match res
                {
                Ok((endian,)) => {
                    let endian = match endian
                        {
                        PreferredEndianness::Any => crate::ffi::meta_bus::UDI_DMA_ANY_ENDIAN,
                        PreferredEndianness::Big => crate::ffi::meta_bus::UDI_DMA_BIG_ENDIAN,
                        PreferredEndianness::Little => crate::ffi::meta_bus::UDI_DMA_LITTLE_ENDIAN,
                        };
                    (0,crate::ffi::physio::UDI_NULL_DMA_CONSTRAINTS,endian)
                    },
                Err(e) => (e.into_inner(),crate::ffi::physio::UDI_NULL_DMA_CONSTRAINTS,0),
                };
            crate::ffi::meta_bus::udi_bus_bind_ack(cb, dma, endian, status)
        }
        )
});
future_wrapper!(bus_unbind_req_op => <T as BusBridge>(cb: *mut udi_bus_bind_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.bus_unbind_req(cb),
        |cb,_res| unsafe { crate::ffi::meta_bus::udi_bus_unbind_ack(cb) }
        )
});
future_wrapper!(intr_attach_req_op => <T as BusBridge>(cb: *mut crate::ffi::meta_intr::udi_intr_attach_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.intr_attach_req(cb),
        |cb,res| unsafe { crate::ffi::meta_intr::udi_intr_attach_ack(cb, crate::Error::to_status(res)) }
        )
});
future_wrapper!(intr_detach_req_op => <T as BusBridge>(cb: *mut crate::ffi::meta_intr::udi_intr_detach_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.intr_detach_req(cb),
        |cb,_res| unsafe { crate::ffi::meta_intr::udi_intr_detach_ack(cb) }
        )
});

impl<T,CbList> crate::OpsStructure<::udi_sys::meta_bus::udi_bus_bridge_ops_t, T,CbList>
where
	T: BusBridge,
    CbList: crate::HasCb<udi_bus_bind_cb_t>,
    CbList: crate::HasCb<crate::ffi::meta_intr::udi_intr_attach_cb_t>,
    CbList: crate::HasCb<crate::ffi::meta_intr::udi_intr_detach_cb_t>,
{
    pub const fn scratch_requirement() -> usize {
        let rv = crate::imc::task_size::<T,MarkerBusBridge>();
        let rv = crate::const_max(rv, bus_bind_req_op::task_size::<T>());
        let rv = crate::const_max(rv, bus_unbind_req_op::task_size::<T>());
        let rv = crate::const_max(rv, intr_attach_req_op::task_size::<T>());
        let rv = crate::const_max(rv, intr_detach_req_op::task_size::<T>());
        rv
    }
    /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
    /// SAFETY: The scratch size must be >= value returned by [Self::scratch_requirement]
    pub const unsafe fn for_driver() -> udi_bus_bridge_ops_t {
        return udi_bus_bridge_ops_t {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T, MarkerBusBridge>,
            bus_bind_req_op: bus_bind_req_op::<T>,
            bus_unbind_req_op: bus_unbind_req_op::<T>,
            intr_attach_req_op: intr_attach_req_op::<T>,
            intr_detach_req_op: intr_detach_req_op::<T>,
        };
    }
}
