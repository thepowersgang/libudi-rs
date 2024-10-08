use ::udi::ffi::cb::udi_cb_t;
use ::udi::ffi::imc::udi_channel_anchor_call_t;
use ::udi::ffi::imc::udi_channel_event_cb_t;
use ::udi::ffi::imc::udi_channel_spawn_call_t;
use ::udi::ffi::udi_channel_t;
use ::udi::ffi::udi_index_t;
use ::udi::ffi::c_void;
use ::udi::ffi::udi_status_t;

/// Anchor a loose channel end (binding it permanently to a region)
#[no_mangle]
unsafe extern "C" fn udi_channel_anchor(
    callback: udi_channel_anchor_call_t,
    gcb: *mut udi_cb_t,
    channel: udi_channel_t,
    ops_idx: udi_index_t,
    channel_context: *mut c_void
)
{
    // Get the driver instance from the gcb
    let driver_instance = crate::channels::get_driver_instance(&(*gcb).channel);
    let ops_init = driver_instance.module.get_ops_init(ops_idx).unwrap();
    let ops = driver_instance.module.get_meta_ops(ops_init);
    crate::channels::anchor(channel, driver_instance, ops, channel_context);
    crate::async_call(gcb, move |gcb| callback(gcb, channel))
}

/// Spawn a new channel
/// 
/// `channel` and `spawn_idx` are used to match two different calls to join ends
#[no_mangle]
unsafe extern "C" fn udi_channel_spawn(
    callback: udi_channel_spawn_call_t,
    gcb: *mut udi_cb_t,
    // Original channel used as the basis for the new channel
    channel: udi_channel_t,
    // Small integer used to join two spawn requests together
    spawn_idx: udi_index_t,
    // Ops index to use (can be zero, indicating a loose channel end)
    ops_idx: udi_index_t,
    // Context to use if anchoring
    channel_context: *mut c_void,
)
{
    let driver_instance = crate::channels::get_driver_instance(&(*gcb).channel);

    // Create an unanchored channel either fresh or in `channel` with `spawn_idx`
    let new_channel = crate::channels::spawn(channel, spawn_idx);

    if ops_idx == 0.into() {
        // Loose end requested
    }
    else {
        let ops_init = driver_instance.module.get_ops_init(ops_idx).unwrap();
        let ops = driver_instance.module.get_meta_ops(ops_init);
        crate::channels::anchor(new_channel, driver_instance, ops, channel_context);
    }

    crate::async_call(gcb, move |gcb| callback(gcb, new_channel))
}

#[no_mangle]
unsafe extern "C" fn udi_channel_close(channel: udi_channel_t) {
    crate::channels::close(channel);
}

#[no_mangle]
unsafe extern "C" fn udi_channel_event_complete(cb: *mut udi_channel_event_cb_t, status: udi_status_t)
{
    match (*cb).event
    {
    ::udi::ffi::imc::UDI_CHANNEL_BOUND => {
        let instance = crate::channels::get_driver_instance(&(*cb).gcb.channel);
        instance.management_state.bind_complete(&instance, cb, ::udi::Error::from_status(status));
        ::udi::ffi::cb::udi_cb_free(::core::ptr::addr_of_mut!( (*cb).gcb ));
        },
    _ => todo!("udi_channel_event_complete({})", (*cb).event),
    }
}