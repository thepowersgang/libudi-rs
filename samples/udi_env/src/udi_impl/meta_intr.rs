
use ::udi::ffi::meta_intr::{udi_intr_handler_ops_t,udi_intr_dispatcher_ops_t};
use ::udi::ffi::meta_intr::udi_intr_event_cb_t;

impl_metalang_ops_for!{udi_intr_handler_ops_t,udi_intr_dispatcher_ops_t}

dispatch_call!{
    fn udi_intr_event_rdy(cb: *mut udi_intr_event_cb_t)
        => udi_intr_dispatcher_ops_t:intr_event_rdy_op;
    fn udi_intr_event_ind(cb: *mut udi_intr_event_cb_t, flags: ::udi::ffi::udi_ubit8_t)
        => udi_intr_handler_ops_t:intr_event_ind_op;
}