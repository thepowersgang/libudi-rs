//! Inter-Module Communication
//! 
//! Defines channels and methods on them
use ::udi_sys::imc::udi_channel_event_cb_t;

/// Channel handle
#[repr(transparent)]
pub struct ChannelHandle(::udi_sys::udi_channel_t);
impl Drop for ChannelHandle {
    fn drop(&mut self) {
        if ! self.0.is_null() {
            todo!("drop ChannelHandle")
        }
    }
}
impl Default for ChannelHandle {
    fn default() -> Self {
        Self::null()
    }
}
impl ChannelHandle {
    /// Create a new invalid channel handle
    pub const fn null() -> Self {
        ChannelHandle(::core::ptr::null_mut())
    }
    /// Create a safe channel handle from a raw handle
    pub const unsafe fn from_raw(h: ::udi_sys::udi_channel_t) -> Self {
        ChannelHandle(h)
    }
    /// Get the raw UDI channel handle
    pub fn raw(&self) -> ::udi_sys::udi_channel_t{
        self.0
    }
}

unsafe impl crate::async_trickery::GetCb for udi_channel_event_cb_t {
    fn get_gcb(&self) -> &::udi_sys::udi_cb_t {
        &self.gcb
    }
}

/// Spawn a new channel
/// 
/// - `cb` is the current task control block
/// - `context` is the current context structure, used to determine the correct context for the ops on the channel
///   - It is a bug (checked with an assertion) for this pointer to be different to `cb.context`
/// - `spawn_idx` is an index used to match channel pairs together
// TODO: Is there a safety hazard here if `spawn_idx` is for the wrong ops type
// - For my impl, there's checks that the ops vector matches.
pub fn channel_spawn<Ops>(
	cb: crate::CbRef<::udi_sys::udi_cb_t>,
    context: &impl AsRef<Ops::Context>,
    spawn_idx: ::udi_sys::udi_index_t,
) -> impl ::core::future::Future<Output=ChannelHandle>
where
    Ops: crate::ops_markers::Ops,
{
    let expected_context = context as *const _;
    let channel_context = context.as_ref() as *const _;
	extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t, handle: ::udi_sys::udi_channel_t) {
		unsafe { crate::async_trickery::signal_waiter(gcb, crate::WaitRes::Pointer(handle as *mut ())); }
	}
	crate::async_trickery::wait_task::<::udi_sys::udi_cb_t, _,_,_>(
        cb,
		move |cb| unsafe {
            assert!( (*cb).context == expected_context as *mut _, "BUG: `channel_spawn` was passed a context that doesn't match input CB" );
            ::udi_sys::imc::udi_channel_spawn(callback, cb as *const _ as *mut _, (*cb).channel, spawn_idx, Ops::INDEX, channel_context as *mut _)
			},
		|res| {
			let crate::WaitRes::Pointer(p) = res else { panic!(""); };
			ChannelHandle(p as *mut _)
			}
		)
}

/// Spawn a new channel (with custom channel and context values)
pub unsafe fn channel_spawn_ex(
	cb: crate::CbRef<::udi_sys::udi_cb_t>,
    channel: ::udi_sys::udi_channel_t,
    spawn_idx: ::udi_sys::udi_index_t,
    ops_idx: ::udi_sys::udi_index_t,
    channel_context: *mut ::core::ffi::c_void
) -> impl ::core::future::Future<Output=::udi_sys::udi_channel_t> {
	unsafe extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t, handle: ::udi_sys::udi_channel_t) {
		unsafe { crate::async_trickery::signal_waiter(gcb, crate::WaitRes::Pointer(handle as *mut ())); }
	}
	crate::async_trickery::wait_task::<::udi_sys::udi_cb_t, _,_,_>(
        cb,
		move |cb| unsafe {
            ::udi_sys::imc::udi_channel_spawn(callback, cb as *const _ as *mut _, channel, spawn_idx, ops_idx, channel_context)
			},
		|res| {
			let crate::WaitRes::Pointer(p) = res else { panic!(""); };
			p as *mut _
			}
		)
}

/// Trait handling initialisation/de-initialisation of channel context blobs
pub trait ChannelInit {
    /// SAFETY: Caller must ensure that this is only called once (on channel bind)
    unsafe fn init(&mut self) {}
    /// SAFETY: Caller must ensure that this is only called once (on channel close)
    unsafe fn deinit(&mut self) {}
}

/// Handler trait for generic operations on a channel
///
/// The marker here is to allow multiple implementations depending on the metalanguage
pub trait ChannelHandler<Marker>: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    /// Channel has been closed
    fn channel_closed(&mut self) {
    }
    /// The channel has been bound to an endpoint, this must always call `udi_channel_event_complete` on `*self.channel_cb_slot()`
    /// eventually.
    fn channel_bound(&self, params: &::udi_sys::imc::udi_channel_event_cb_t_params) {
        let _ = params;
        let slot = self.channel_cb_slot();
        let channel_cb = slot.replace(::core::ptr::null_mut());
        unsafe { crate::ffi::imc::udi_channel_event_complete(channel_cb, ::udi_sys::UDI_OK as _) }
    }
}

/// Task size helper function for a default channel handler
pub const fn task_size<T: ChannelHandler<Marker>,Marker: 'static>() -> usize {
    0
}
/// Generic handler for `channel_event_ind` to be stored in metalanguage ops structures
pub unsafe extern "C" fn channel_event_ind_op<T: ChannelHandler<Marker>, Marker: 'static>(cb: *mut udi_channel_event_cb_t) {
    // NOTE: There's no scratch availble to this function, so cannot use async

    match (*cb).event
    {
    // Called when the remote end of the channel is closed, this function is expected to close the channel afer `udi_channel_event_complete`
    ::udi_sys::imc::UDI_CHANNEL_CLOSED => {
        // SAFE: Caller has ensured that the context is valid for this type
        let state: &mut T = crate::async_trickery::get_rdata_t_mut(&*cb);
        state.channel_closed();
        let channel = (*cb).gcb.channel;
        crate::ffi::imc::udi_channel_event_complete(cb, ::udi_sys::UDI_OK as _);
        crate::ffi::imc::udi_channel_close(channel);
        },
    // Another region has been bound to this via parent or internal bind
    // Note: Only called for the non-initiating end (for child for a parent-child, and for primary for sec-primary)
    ::udi_sys::imc::UDI_CHANNEL_BOUND => {
        crate::async_trickery::set_channel_cb::<T>(cb);
        // SAFE: Caller has ensured that the context is valid for this type
        let state: &mut T = crate::async_trickery::get_rdata_t_mut(&*cb);
        crate::imc::ChannelInit::init(state);
        state.channel_bound( &(*cb).params );
        // no `udi_channel_event_complete` call, it's done by `channel_bound` (maybe indirectly)
        },
    // Called when an async operation is to be aborted
    ::udi_sys::imc::UDI_CHANNEL_OP_ABORTED => {
        let aborted_cb = (*cb).params.orig_cb;
        crate::async_trickery::abort_task(aborted_cb);
        crate::ffi::imc::udi_channel_event_complete(cb, ::udi_sys::UDI_OK as _);
        },
    _ => {},
    }
}