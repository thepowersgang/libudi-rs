use crate::ffi::imc::udi_channel_event_cb_t;

pub fn channel_spawn(
    channel: crate::ffi::udi_channel_t,
    spawn_idx: crate::ffi::udi_index_t,
    ops_idx: crate::ffi::udi_index_t,
    channel_context: *mut ::core::ffi::c_void
) -> impl ::core::future::Future<Output=crate::ffi::udi_channel_t> {
	extern "C" fn callback(gcb: *mut crate::ffi::udi_cb_t, handle: crate::ffi::udi_channel_t) {
		unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
	}
	crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
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
    type Future<'s>: ::core::future::Future<Output=crate::Result<()>>;
    fn event_ind(&mut self) -> Self::Future<'_>;
}
struct Wrapper<'a, T: ChannelHandler<Marker>, Marker> {
    inner: T::Future<'a>,
}
impl<'a, T: ChannelHandler<Marker>, Marker> ::core::future::Future for Wrapper<'a, T, Marker>
{
    type Output = ();
    fn poll(mut self: ::core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> ::core::task::Poll<Self::Output> {
        match pin_project!(self, inner).poll(cx)
        {
        ::core::task::Poll::Pending => ::core::task::Poll::Pending,
        ::core::task::Poll::Ready(r) => {
            let cb: &udi_channel_event_cb_t = crate::async_trickery::cb_from_waker(cx.waker());
            // SAFE: Correct FFI
            unsafe {
                crate::ffi::imc::udi_channel_event_complete(cb as *const _ as *mut _, match r
                    {
                    Ok(()) => 0,
                    Err(e) => e.into_inner(),
                    });
            }
            ::core::task::Poll::Ready(())
            }
        }
    }
}

pub const fn task_size<T: ChannelHandler<Marker>,Marker: 'static>() -> usize {
    crate::async_trickery::task_size::<Wrapper<'static, T, Marker>>()
}
pub unsafe extern "C" fn channel_event_ind_op<T: ChannelHandler<Marker>, Marker: 'static>(cb: *mut udi_channel_event_cb_t) {
    // SAFE: Caller has ensured that the context is valid for this type
    let state: &mut T = unsafe { &mut *((*cb).gcb.context as *mut T) };
    // SAFE: Valid raw pointer deref, caller ensured cb scratch validity
    unsafe { crate::async_trickery::init_task(&*cb, Wrapper::<T,Marker> { inner: state.event_ind() }); }
}