//! Inter-Module Communication
//! 
//! 
use ::udi_sys::imc::udi_channel_event_cb_t;

unsafe impl crate::async_trickery::GetCb for udi_channel_event_cb_t {
    fn get_gcb(&self) -> &::udi_sys::udi_cb_t {
        &self.gcb
    }
}

/// Spawn a new channel
pub fn channel_spawn(
	cb: crate::CbRef<::udi_sys::udi_cb_t>,
    spawn_idx: ::udi_sys::udi_index_t,
    ops_idx: ::udi_sys::udi_index_t,
) -> impl ::core::future::Future<Output=::udi_sys::udi_channel_t> {
	extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t, handle: ::udi_sys::udi_channel_t) {
		unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
	}
	crate::async_trickery::wait_task::<::udi_sys::udi_cb_t, _,_,_>(
        cb,
		move |cb| unsafe {
            ::udi_sys::imc::udi_channel_spawn(callback, cb as *const _ as *mut _, (*cb).channel, spawn_idx, ops_idx, (*cb).context)
			},
		|res| {
			let crate::WaitRes::Pointer(p) = res else { panic!(""); };
			p as *mut _
			}
		)
}

/// Spawn a new channel (with custom channel and context values)
pub fn channel_spawn_ex(
	cb: crate::CbRef<::udi_sys::udi_cb_t>,
    channel: ::udi_sys::udi_channel_t,
    spawn_idx: ::udi_sys::udi_index_t,
    ops_idx: ::udi_sys::udi_index_t,
    channel_context: *mut ::core::ffi::c_void
) -> impl ::core::future::Future<Output=::udi_sys::udi_channel_t> {
	extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t, handle: ::udi_sys::udi_channel_t) {
		unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
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

pub trait ChannelInit {
    /// SAFETY: Caller must ensure that this is only called once (on channel bind)
    unsafe fn init(&mut self) {}
    /// SAFETY: Caller must ensure that this is only called once (on channel close)
    unsafe fn deinit(&mut self) {}
}

pub trait ChannelHandler<Marker>: 'static + crate::async_trickery::CbContext + crate::imc::ChannelInit {
    fn channel_closed(&mut self) {

    }
    fn channel_bound(&mut self, params: &::udi_sys::imc::udi_channel_event_cb_t_params) {
        let _ = params;
        let slot = self.channel_cb_slot();
        let channel_cb = ::core::mem::replace(slot, ::core::ptr::null_mut());
        unsafe { crate::ffi::imc::udi_channel_event_complete(channel_cb, ::udi_sys::UDI_OK as _) }
    }
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
    ::udi_sys::imc::UDI_CHANNEL_CLOSED => state.channel_closed(),
    ::udi_sys::imc::UDI_CHANNEL_BOUND => {
        crate::async_trickery::set_channel_cb::<T>(cb);
        crate::imc::ChannelInit::init(state);
        state.channel_bound( &(*cb).params );
        },
    ::udi_sys::imc::UDI_CHANNEL_OP_ABORTED => {
        let aborted_cb = (*cb).params.orig_cb;
        crate::async_trickery::abort_task(aborted_cb);
        },
    _ => {},
    }
}