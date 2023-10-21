//!
//! 
//! 
use crate::ffi::meta_bus::udi_bus_device_ops_t;
use crate::ffi::meta_bus::udi_bus_bind_cb_t;

pub trait BusDevice: 'static {
    async_method!(fn bus_bind_ack(&mut self,
        dma_constraints: crate::ffi::physio::udi_dma_constraints_t,
        preferred_endianness: bool,
        status: crate::ffi::udi_status_t
        )->() as Future_bind_ack
    );

    async_method!(fn bus_unbind_ack(&mut self) -> () as Future_unbind_ack);
    async_method!(fn intr_attach_ack(&mut self, status: crate::ffi::udi_status_t) -> () as Future_intr_attach_ack);
    async_method!(fn intr_detach_ack(&mut self) -> () as Future_intr_detach_ack);
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

unsafe impl crate::async_trickery::GetCb for udi_bus_bind_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}

impl udi_bus_device_ops_t {
    pub const fn scratch_requirement<T: BusDevice>() -> usize {
        let rv = crate::imc::task_size::<T,MarkerBusDevice>();
        let rv = crate::const_max(rv, crate::async_trickery::task_size::<T::Future_bind_ack<'static>>());
        let rv = crate::const_max(rv, crate::async_trickery::task_size::<T::Future_unbind_ack<'static>>());
        let rv = crate::const_max(rv, crate::async_trickery::task_size::<T::Future_intr_attach_ack<'static>>());
        rv
    }
    /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
    /// SAFETY: The scratch size must be >= value returned by [scratch_requirement]
    pub const unsafe fn for_driver<T: BusDevice>() -> Self {
        return udi_bus_device_ops_t {
            // TODO: Maybe have `channel_event_ind` be fully defined here? Handling the bind call
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T, MarkerBusDevice>,
            bus_bind_ack_op: bus_bind_ack_op::<T>,
            bus_unbind_ack_op: bus_unbind_ack_op::<T>,
            intr_attach_ack_op: intr_attach_ack_op::<T>,
            intr_detach_ack_op: intr_detach_ack_op::<T>,
        };
        extern "C" fn bus_bind_ack_op<T: BusDevice>(
            cb: *mut udi_bus_bind_cb_t,
            dma_constraints: crate::ffi::physio::udi_dma_constraints_t,
            preferred_endianness: u8,
            status: crate::ffi::udi_status_t
        )
        {
            // SAFE: Caller has ensured that the context is valid for this type
            let state: &mut T = unsafe { &mut *((*cb).gcb.context as *mut T) };
            let job = state.bus_bind_ack(dma_constraints, preferred_endianness != 0, status);
            // SAFE: Valid raw pointer deref, caller ensured cb scratch validity
            unsafe { crate::async_trickery::init_task(&*cb, job); }
        }
        extern "C" fn bus_unbind_ack_op<T: BusDevice>(cb: *mut udi_bus_bind_cb_t)
        {
            // SAFE: Caller has ensured that the context is valid for this type
            let state: &mut T = unsafe { &mut *((*cb).gcb.context as *mut T) };
            let job = state.bus_unbind_ack();
            // SAFE: Valid raw pointer deref, caller ensured cb scratch validity
            unsafe { crate::async_trickery::init_task(&*cb, job); }
        }
        extern "C" fn intr_attach_ack_op<T: BusDevice>(cb: *mut crate::ffi::meta_intr::udi_intr_attach_cb_t, status: crate::ffi::udi_status_t)
        {
            // SAFE: Caller has ensured that the context is valid for this type
            let state: &mut T = unsafe { &mut *((*cb).gcb.context as *mut T) };
            let job = state.intr_attach_ack(status);
            // SAFE: Valid raw pointer deref, caller ensured cb scratch validity
            unsafe { crate::async_trickery::init_task(&*cb, job); }
        }
        extern "C" fn intr_detach_ack_op<T: BusDevice>(cb: *mut crate::ffi::meta_intr::udi_intr_detach_cb_t)
        {
            // SAFE: Caller has ensured that the context is valid for this type
            let state: &mut T = unsafe { &mut *((*cb).gcb.context as *mut T) };
            let job = state.intr_detach_ack();
            // SAFE: Valid raw pointer deref, caller ensured cb scratch validity
            unsafe { crate::async_trickery::init_task(&*cb, job); }
        }
    }
}
