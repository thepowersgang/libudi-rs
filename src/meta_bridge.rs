//! Bus Bridge metalanguage (Phsical I/O Specification)
//! 
use ::udi_sys::meta_bridge::{udi_intr_event_cb_t, udi_intr_attach_cb_t};
use ::udi_sys::meta_bridge::udi_bus_device_ops_t;
use ::udi_sys::meta_bridge::udi_bus_bridge_ops_t;
use ::udi_sys::meta_bridge::udi_bus_bind_cb_t;

pub type CbRefBind<'a> = crate::CbRef<'a, udi_bus_bind_cb_t>;
pub type CbRefIntrAttach<'a> = crate::CbRef<'a, crate::ffi::meta_bridge::udi_intr_attach_cb_t>;
pub type CbRefIntrDetach<'a> = crate::CbRef<'a, crate::ffi::meta_bridge::udi_intr_detach_cb_t>;


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

pub enum PreferredEndianness {
    Any,
    Little,
    Big,
}

/// Trait for a device on a bus
pub trait BusDevice: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
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
    T: BusDevice,
{
    fn channel_bound(&mut self, params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
        // SAFE: Trusting that this trait is only being used through proper config.
        unsafe {
            crate::ffi::meta_bridge::udi_bus_bind_req(params.parent_bound.bind_cb as *mut udi_bus_bind_cb_t);
        }
    }
}

pub trait BusBridge: 'static + crate::imc::ChannelInit + crate::async_trickery::CbContext {
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
    crate::async_trickery::with_ack(
        val.bus_bind_ack(cb, dma_constraints, preferred_endianness, status),
        |cb,res| unsafe { crate::async_trickery::channel_event_complete::<T,udi_bus_bind_cb_t>(cb, crate::Error::to_status(res)) }
        )
});
future_wrapper!(bus_unbind_ack_op => <T as BusDevice>(cb: *mut udi_bus_bind_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.bus_unbind_ack(cb),
        |cb,_res| unsafe { crate::async_trickery::channel_event_complete::<T,udi_bus_bind_cb_t>(cb, 0 /*res*/) }
        )
});
future_wrapper!(intr_attach_ack_op => <T as BusDevice>(cb: *mut crate::ffi::meta_bridge::udi_intr_attach_cb_t, status: crate::ffi::udi_status_t) val @ {
    val.intr_attach_ack(cb, status)
});
future_wrapper!(intr_detach_ack_op => <T as BusDevice>(cb: *mut crate::ffi::meta_bridge::udi_intr_detach_cb_t) val @ {
    val.intr_detach_ack(cb)
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
        )
});
future_wrapper!(bus_unbind_req_op => <T as BusBridge>(cb: *mut udi_bus_bind_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.bus_unbind_req(cb),
        |cb,_res| unsafe { crate::ffi::meta_bridge::udi_bus_unbind_ack(cb) }
        )
});
future_wrapper!(intr_attach_req_op => <T as BusBridge>(cb: *mut crate::ffi::meta_bridge::udi_intr_attach_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.intr_attach_req(cb),
        |cb,res| unsafe { crate::ffi::meta_bridge::udi_intr_attach_ack(cb, crate::Error::to_status(res)) }
        )
});
future_wrapper!(intr_detach_req_op => <T as BusBridge>(cb: *mut crate::ffi::meta_bridge::udi_intr_detach_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.intr_detach_req(cb),
        |cb,_res| unsafe { crate::ffi::meta_bridge::udi_intr_detach_ack(cb) }
        )
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


pub fn attach_req(cb: super::cb::CbHandle<udi_intr_attach_cb_t>) {
    unsafe { crate::ffi::meta_bridge::udi_intr_attach_req(cb.into_raw()) }
}
pub fn event_rdy(cb: super::cb::CbHandle<udi_intr_event_cb_t>) {
    unsafe { crate::ffi::meta_bridge::udi_intr_event_rdy(cb.into_raw()) }
}

pub type CbRefEvent<'a> = crate::CbRef<'a, udi_intr_event_cb_t>;
pub type CbHandleEvent = crate::cb::CbHandle<udi_intr_event_cb_t>;

pub trait IntrHandler: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit
{
    async_method!(fn intr_event_ind(&'a mut self, cb: CbRefEvent<'a>, flags: u8)->() as Future_intr_event_ind);
}
struct MarkerIntrHandler;
impl<T> crate::imc::ChannelHandler<MarkerIntrHandler> for T
where
    T: IntrHandler
{
}

future_wrapper!(intr_event_ind_op => <T as IntrHandler>(cb: *mut udi_intr_event_cb_t, flags: u8) val @ {
    crate::async_trickery::with_ack(
        val.intr_event_ind(cb, flags),
        // Return this CB to the pool on completion
        |cb,_res| unsafe { crate::ffi::meta_bridge::udi_intr_event_rdy(cb) }
        )
});
map_ops_structure!{
    ::udi_sys::meta_bridge::udi_intr_handler_ops_t => IntrHandler,MarkerIntrHandler {
        intr_event_ind_op,
    }
    CBS {
        udi_intr_event_cb_t,
    }
}


/// Interrupte dispatcher trait
pub trait IntrDispatcher: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit
{
    async_method!(fn intr_event_rdy(&'a mut self, cb: CbHandleEvent)->() as Future_intr_event_rdy);
}
struct MarkerIntrDispatcher;
impl<T> crate::imc::ChannelHandler<MarkerIntrDispatcher> for T
where
    T: IntrDispatcher
{
}

future_wrapper!(intr_event_rdy_op => <T as IntrDispatcher>(cb: *mut udi_intr_event_cb_t) val @ {
    val.intr_event_rdy(unsafe { cb.into_owned() })
});
map_ops_structure!{
    ::udi_sys::meta_bridge::udi_intr_dispatcher_ops_t => IntrDispatcher,MarkerIntrDispatcher {
        intr_event_rdy_op,
    }
    CBS {
        udi_intr_event_cb_t,
    }
}
