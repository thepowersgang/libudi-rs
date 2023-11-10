//! Interrupt metalanguage (Phsical I/O Specification)
use crate::ffi::meta_intr::{udi_intr_event_cb_t, udi_intr_attach_cb_t};


pub fn attach_req(cb: super::cb::CbHandle<udi_intr_attach_cb_t>) {
    unsafe { crate::ffi::meta_intr::udi_intr_attach_req(cb.into_raw()) }
}
pub fn event_rdy(cb: super::cb::CbHandle<udi_intr_event_cb_t>) {
    unsafe { crate::ffi::meta_intr::udi_intr_event_rdy(cb.into_raw()) }
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
        |cb,_res| unsafe { crate::ffi::meta_intr::udi_intr_event_rdy(cb) }
        )
});
map_ops_structure!{
    ::udi_sys::meta_intr::udi_intr_handler_ops_t => IntrHandler,MarkerIntrHandler {
        intr_event_ind_op,
    }
    CBS {
        udi_intr_event_cb_t,
    }
}


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
    ::udi_sys::meta_intr::udi_intr_dispatcher_ops_t => IntrDispatcher,MarkerIntrDispatcher {
        intr_event_rdy_op,
    }
    CBS {
        udi_intr_event_cb_t,
    }
}
