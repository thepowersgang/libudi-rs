use crate::ffi::meta_intr::{udi_intr_event_cb_t, udi_intr_attach_cb_t};


pub fn attach_req(cb: super::cb::CbHandle<udi_intr_attach_cb_t>) {
    unsafe { crate::ffi::meta_intr::udi_intr_attach_req(cb.into_raw()) }
}
pub fn event_rdy(cb: super::cb::CbHandle<udi_intr_event_cb_t>) {
    unsafe { crate::ffi::meta_intr::udi_intr_event_rdy(cb.into_raw()) }
}

pub trait IntrHandler: 'static + crate::imc::ChannelHandler {
    #[allow(non_camel_case_types)]
    type Future_intr_event_ind<'s>: ::core::future::Future<Output=()>;
    fn intr_event_ind(&mut self, flags: u8) -> Self::Future_intr_event_ind<'_>;
}

impl crate::ffi::meta_intr::udi_intr_handler_ops_t {
    pub const fn scratch_requirement<T: IntrHandler>() -> usize {
        crate::async_trickery::task_size::<T::Future_intr_event_ind<'static>>()
    }
    pub const unsafe fn for_driver<T: IntrHandler>() -> Self {
        // ENTRYPOINT: mgmt_ops.usage_ind
        unsafe extern "C" fn intr_event_ind_op<T: IntrHandler>(cb: *mut udi_intr_event_cb_t, flags: u8)
        {
            // TODO: Wrap such that `udi_intr_event_rdy` is re-called once completed
            let job = (*((*cb).gcb.context as *mut T)).intr_event_ind(flags);
            crate::async_trickery::init_task(&*cb, job);
        }
        Self {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T>,
            intr_event_ind_op: intr_event_ind_op::<T>,
        }
    }
}