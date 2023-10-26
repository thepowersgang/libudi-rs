use ::udi::ffi::meta_intr::{udi_intr_attach_cb_t, udi_intr_event_cb_t};


#[no_mangle]
unsafe extern "C" fn udi_intr_attach_req(cb: *mut udi_intr_attach_cb_t)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_intr_event_rdy(cb: *mut udi_intr_event_cb_t)
{
    todo!()
}