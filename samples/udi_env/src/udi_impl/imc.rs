use ::udi::ffi::cb::udi_cb_t;
use ::udi::ffi::imc::udi_channel_anchor_call_t;
use ::udi::ffi::imc::udi_channel_event_cb_t;
use ::udi::ffi::imc::udi_channel_spawn_call_t;
use ::udi::ffi::udi_channel_t;
use ::udi::ffi::udi_index_t;
use ::udi::ffi::c_void;
use ::udi::ffi::udi_status_t;

#[no_mangle]
unsafe extern "C" fn udi_channel_anchor(
    cb: udi_channel_anchor_call_t,
    gcb: *mut udi_cb_t,
    channel: udi_channel_t,
    ops_idx: udi_index_t,
    channel_context: *mut c_void
)
{
    // Get the driver instance from the gcb
    //crate::channels::bind_channel(channel, channel_context, ops_idx);
}
#[no_mangle]
unsafe extern "C" fn udi_channel_spawn(
    cb: udi_channel_spawn_call_t,
    gcb: *mut udi_cb_t,
    channel: udi_channel_t,
    spawn_idx: udi_index_t,
    ops_idx: udi_index_t,
    channel_context: *mut c_void,
)
{
    // TODO: Create the channel, and then call the callback
    //crate::channels::allocate_channel(channel_context, ops, scratch_requirement)
    todo!();
}

#[no_mangle]
unsafe extern "C" fn udi_channel_event_complete(cb: *mut udi_channel_event_cb_t, status: udi_status_t)
{
    todo!();
}