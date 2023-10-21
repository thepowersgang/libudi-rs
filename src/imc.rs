use crate::ffi::imc::udi_channel_event_cb_t;

/// Spawn a new channel
pub fn channel_spawn(
	cb: crate::CbRef<crate::ffi::udi_cb_t>,
    channel: crate::ffi::udi_channel_t,
    spawn_idx: crate::ffi::udi_index_t,
    ops_idx: crate::ffi::udi_index_t,
    channel_context: *mut ::core::ffi::c_void
) -> impl ::core::future::Future<Output=crate::ffi::udi_channel_t> {
	extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, handle: crate::ffi::udi_channel_t) {
		unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
	}
	crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
        cb,
		move |cb| unsafe {
            crate::ffi::imc::udi_channel_spawn(callback, cb as *const _ as *mut _, channel, spawn_idx, ops_idx, channel_context)
			},
		|res| {
			let crate::WaitRes::Pointer(p) = res else { panic!(""); };
			p as *mut _
			}
		)
}

pub trait ChannelHandler<Marker>: 'static {
    fn channel_closed(&mut self);
    fn channel_bound(&mut self, params: &crate::ffi::imc::udi_channel_event_cb_t_params);
}

pub const fn task_size<T: ChannelHandler<Marker>,Marker: 'static>() -> usize {
    0
}
pub unsafe extern "C" fn channel_event_ind_op<T: ChannelHandler<Marker>, Marker: 'static>(cb: *mut udi_channel_event_cb_t) {
    // NOTE: There's no scratch availble to this function, so cannot use async

    // SAFE: Caller has ensured that the context is valid for this type
    let state: &mut T = crate::async_trickery::get_rdata_t(&*cb);
    match (*cb).event
    {
    crate::ffi::imc::UDI_CHANNEL_CLOSED => state.channel_closed(),
    crate::ffi::imc::UDI_CHANNEL_BOUND => {
        crate::async_trickery::set_channel_cb(cb);
        state.channel_bound( &(*cb).params );
        },
    crate::ffi::imc::UDI_CHANNEL_OP_ABORTED => {
        let aborted_cb = (*cb).params.orig_cb;
        crate::async_trickery::abort_task(aborted_cb);
        },
    _ => {},
    }
}