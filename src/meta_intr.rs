use crate::ffi::meta_intr::{udi_intr_event_cb_t, udi_intr_attach_cb_t};


pub fn attach_req(cb: super::cb::CbHandle<udi_intr_attach_cb_t>) {
    unsafe { crate::ffi::meta_intr::udi_intr_attach_req(cb.into_raw()) }
}
pub fn event_rdy(cb: super::cb::CbHandle<udi_intr_event_cb_t>) {
    unsafe { crate::ffi::meta_intr::udi_intr_event_rdy(cb.into_raw()) }
}

pub trait IntrHandler: 'static
{
    channel_handler_method!();

    async_method!(fn intr_event_ind(&mut self, flags: u8)->() as Future_intr_event_ind);
}
channel_handler_forward!(MarkerIntrHandler, IntrHandler);


future_wrapper!(intr_event_ind_op => <T as IntrHandler>(cb: *mut udi_intr_event_cb_t, flags: u8) val @ {
    crate::async_trickery::with_ack(
        val.intr_event_ind(flags),
        // Return this CB to the pool on completion
        |cb,_res| unsafe { crate::ffi::meta_intr::udi_intr_event_rdy(cb) }
        )
});

impl crate::ffi::meta_intr::udi_intr_handler_ops_t {
    pub const fn scratch_requirement<T: IntrHandler>() -> usize {
        let v = crate::imc::task_size::<T, MarkerIntrHandler>();
        let v = crate::const_max(v, intr_event_ind_op::task_size::<T>());
        v
    }
    pub const unsafe fn for_driver<T: IntrHandler>() -> Self {
        Self {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T, MarkerIntrHandler>,
            intr_event_ind_op: intr_event_ind_op::<T>,
        }
    }
}