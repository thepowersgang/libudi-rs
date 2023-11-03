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
    let driver_module = crate::channels::get_driver_module(&(*gcb).channel);
    let ops_init = driver_module.get_ops_init(ops_idx).unwrap();
    let ops = driver_module.get_meta_ops(ops_init);
    crate::channels::anchor(channel, driver_module, ops, channel_context);
    (callback)(gcb, channel);
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
    let driver_module = crate::channels::get_driver_module(&(*gcb).channel);

    // Create an unanchored channel either fresh or in `channel` with `spawn_idx`
    let new_channel = crate::channels::spawn(channel, spawn_idx);

    if ops_idx == 0 {
        // Loose end requested
    }
    else {
        let ops_init = driver_module.get_ops_init(ops_idx).unwrap();
        let ops = driver_module.get_meta_ops(ops_init);
        crate::channels::anchor(new_channel, driver_module, ops, channel_context);
    }

    callback(gcb, new_channel);
}

#[no_mangle]
unsafe extern "C" fn udi_channel_event_complete(cb: *mut udi_channel_event_cb_t, status: udi_status_t)
{
    match (*cb).event
    {
    ::udi::ffi::imc::UDI_CHANNEL_BOUND => {
        // TODO: Check the initiator context to know who to signal
        if !(*cb).gcb.initiator_context.is_null() {
            let is = &mut *( (*cb).gcb.initiator_context as *mut crate::management_agent::InstanceInitState);
            is.bind_complete(cb, ::udi::Error::from_status(status));
        }
        else {
            todo!("udi_channel_event_complete({}) null", (*cb).event);
        }
        },
    _ => todo!("udi_channel_event_complete({})", (*cb).event),
    }
}