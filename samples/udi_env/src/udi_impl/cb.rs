use ::udi::ffi::cb::udi_cb_alloc_call_t;
use ::udi::ffi::udi_index_t;
use ::udi::ffi::{udi_cb_t, udi_channel_t};


#[no_mangle]
unsafe extern "C" fn udi_cb_alloc(callback: udi_cb_alloc_call_t, gcb: *mut udi_cb_t, cb_idx: udi_index_t, default_channel: udi_channel_t)
{
}