//! Bus Bridge metalanguage (Phsical I/O Specification)
//! 
//! 
use crate::ffi::meta_bus::udi_bus_device_ops_t;
use crate::ffi::meta_bus::udi_bus_bind_cb_t;

pub type CbRefBind<'a> = crate::CbRef<'a, udi_bus_bind_cb_t>;
pub type CbRefIntrAttach<'a> = crate::CbRef<'a, crate::ffi::meta_intr::udi_intr_attach_cb_t>;
pub type CbRefIntrDetach<'a> = crate::CbRef<'a, crate::ffi::meta_intr::udi_intr_detach_cb_t>;

/// Trait for a device on a bus
pub trait BusDevice: 'static {
    async_method!(fn bus_bind_ack(&'a mut self,
        cb: crate::CbRef<'a, udi_bus_bind_cb_t>,
        dma_constraints: crate::ffi::physio::udi_dma_constraints_t,
        preferred_endianness: bool,
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
    async_method!(fn bus_bind_req(&'a mut self, cb: CbRefBind<'a>) -> () as Future_bind_req);
    async_method!(fn bus_unbind_req(&'a mut self, cb: CbRefBind<'a>) -> () as Future_unbind_req);
    async_method!(fn intr_attach_req(&'a mut self, cb: CbRefIntrAttach<'a>) -> () as Future_intr_attach_req);
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
    crate::async_trickery::with_ack(
        val.bus_bind_ack(cb, dma_constraints, preferred_endianness != 0, status),
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

impl udi_bus_device_ops_t {
    pub const fn scratch_requirement<T: BusDevice>() -> usize {
        let rv = crate::imc::task_size::<T,MarkerBusDevice>();
        let rv = crate::const_max(rv, bus_bind_ack_op::task_size::<T>());
        let rv = crate::const_max(rv, bus_unbind_ack_op::task_size::<T>());
        let rv = crate::const_max(rv, intr_attach_ack_op::task_size::<T>());
        let rv = crate::const_max(rv, intr_detach_ack_op::task_size::<T>());
        rv
    }
    /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
    /// SAFETY: The scratch size must be >= value returned by [Self::scratch_requirement]
    pub const unsafe fn for_driver<T: BusDevice>() -> Self {
        return udi_bus_device_ops_t {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T, MarkerBusDevice>,
            bus_bind_ack_op: bus_bind_ack_op::<T>,
            bus_unbind_ack_op: bus_unbind_ack_op::<T>,
            intr_attach_ack_op: intr_attach_ack_op::<T>,
            intr_detach_ack_op: intr_detach_ack_op::<T>,
        };
    }
}
