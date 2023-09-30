//!
//! 
//! 
use crate::ffi::meta_bus::udi_bus_device_ops_t;
use crate::ffi::meta_bus::udi_bus_bind_cb_t;

pub trait BusDevice: 'static + crate::imc::ChannelHandler {
    #[allow(non_camel_case_types)]
    type Future_bind_ack<'s>: ::core::future::Future<Output=()>;
    fn bind_ack(&mut self, dma_constraints: crate::ffi::physio::udi_dma_constraints_t, preferred_endianness: bool, status: crate::ffi::udi_status_t) -> Self::Future_bind_ack<'_>;
}

//enum TaskState<T: BusDevice> {
//    BindAck(T::Future_bind_ack),
//}
//impl<T: BusDevice> crate::async_trickery::AsyncState for TaskState<T> {
//    fn get_future(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut dyn core::future::Future<Output=()>> {
//        todo!()
//    }
//}

unsafe impl crate::async_trickery::GetCb for udi_bus_bind_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}

impl udi_bus_device_ops_t {
    pub const fn scratch_requirement<T: BusDevice>() -> usize {
        crate::async_trickery::task_size::<T::Future_bind_ack<'static>>()
    }
    /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
    /// SAFETY: The scratch size must be >= value returned by [scratch_requirement]
    pub const unsafe fn for_driver<T: BusDevice>() -> Self {
        return udi_bus_device_ops_t {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T>,
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
            let job = state.bind_ack(dma_constraints, preferred_endianness != 0, status);
            // SAFE: Valid raw pointer deref, caller ensured cb scratch validity
            unsafe { crate::async_trickery::init_task(&*cb, job); }
        }
        extern "C" fn bus_unbind_ack_op<T: BusDevice>(cb: *mut udi_bus_bind_cb_t) {
        }
        extern "C" fn intr_attach_ack_op<T: BusDevice>(cb: *mut crate::ffi::meta_intr::udi_intr_attach_cb_t, status: crate::ffi::udi_status_t)
        {
        }
        extern "C" fn intr_detach_ack_op<T: BusDevice>(cb: *mut crate::ffi::meta_intr::udi_intr_detach_cb_t)
        {
        }
    }
}
