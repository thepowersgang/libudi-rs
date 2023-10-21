use crate::ffi::imc::udi_channel_event_cb_t;

/// Spawn a new channel
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

pub struct ChannelEventCb<'a>(::core::marker::PhantomData<&'a ()>);
impl<'a> ChannelEventCb<'a> {
    pub fn event(&self) -> impl ::core::future::Future<Output=u8> {
        crate::async_trickery::with_cb::<udi_channel_event_cb_t,_,_>(|cb| cb.event)
    }
    pub fn bind_cb(&self) -> impl ::core::future::Future<Output=*mut crate::ffi::udi_cb_t> {
        crate::async_trickery::with_cb::<udi_channel_event_cb_t,_,_>(|cb| {
            // TODO: It's only valid to access this field if it's populated, which depends on external factors
            // - It's only populated if `bind_cb_idx` is non-zero in `*_bind_ops`
            // SAFE: This union field is a CB pointer in all variants
            unsafe { cb.params.parent_bound.bind_cb }
        })
    }
}


pub trait ChannelHandler<Marker>: 'static {
    type Future<'s>: ::core::future::Future<Output=crate::Result<()>>;
    fn event_ind(&mut self, cb: ChannelEventCb) -> Self::Future<'_>;
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

macro_rules! channel_handler_method {
    () => {
        async_method!(fn channel_event_ind(&mut self, cb: $crate::imc::ChannelEventCb)->crate::Result<()> as Future_channel_event_ind);
    };
}
macro_rules! channel_handler_forward {
    ($marker:ident, $trait:ident) => {
        struct $marker;
        impl<T> $crate::imc::ChannelHandler<$marker> for T
        where
            T: $trait
        {
            type Future<'s> = T::Future_channel_event_ind<'s>;
            fn event_ind(&mut self, cb: $crate::imc::ChannelEventCb) -> Self::Future<'_> {
                self.channel_event_ind(cb)
            }
        }
    };
}

pub const fn task_size<T: ChannelHandler<Marker>,Marker: 'static>() -> usize {
    crate::async_trickery::task_size::<Wrapper<'static, T, Marker>>()
}
pub unsafe extern "C" fn channel_event_ind_op<T: ChannelHandler<Marker>, Marker: 'static>(cb: *mut udi_channel_event_cb_t) {
    // TODO TODO TODO
    // Aparently this cb has no scratch, so the async hackery can't work.
    // - Will need to somehow store the cb and call the various metalang methods?
    // Don't do async, instead pass the CB wrapped in some helper to methods
    // - Metalang bindings can do half of the job

    // SAFE: Caller has ensured that the context is valid for this type
    let state: &mut T = unsafe { &mut *((*cb).gcb.context as *mut T) };
    // SAFE: Valid raw pointer deref, caller ensured cb scratch validity
    unsafe { crate::async_trickery::init_task(&*cb, Wrapper::<T,Marker> {
        inner: state.event_ind(ChannelEventCb(core::marker::PhantomData))
    }); }
}