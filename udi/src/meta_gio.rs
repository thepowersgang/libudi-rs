//! Generic I/O Metalanguage
//! 
//! This is a very generic metalanguage for drivers for whom a specialised
//! metalanguage isn't (yet) possible.
use ::udi_sys::meta_gio::*;
use ::udi_sys::meta_gio as ffi;

/// Dispatch a transfer request CB to the other end of the associated channel
pub fn xfer_req(cb: crate::cb::CbHandle<ffi::udi_gio_xfer_cb_t>) {
    unsafe { ffi::udi_gio_xfer_req(cb.into_raw()) }
}

impl_metalanguage!{
    static METALANG_SPEC;
    NAME udi_gio;
    OPS
        1 => udi_gio_provider_ops_t,
        2 => udi_gio_client_ops_t,
        ;
    CBS
        1 => udi_gio_bind_cb_t,
        2 => udi_gio_xfer_cb_t : BUF data_buf,
        3 => udi_gio_event_cb_t,
        ;
}

impl crate::cb::CbRef<'_, ffi::udi_gio_xfer_cb_t>
{
    /// Read from the data buffer in the CB
    pub fn data_buf(&self) -> &crate::buf::Handle {
        // SAFE: Valid pointers
        unsafe {
            crate::buf::Handle::from_ref(&self.data_buf)
        }
    }
}
impl crate::cb::CbHandle<ffi::udi_gio_xfer_cb_t>
{
    /// Get a mutable handle to the data buffer
    pub fn data_buf_mut(&mut self) -> &mut crate::buf::Handle {
        // SAFE: Valid pointers, validity will be maintained (`get_mut`)
        unsafe {
            crate::buf::Handle::from_mut(&mut self.get_mut().data_buf)
        }
    }

    /// Set the operation code
    pub fn set_op(&mut self, op: u8) {
        unsafe {
            self.get_mut().op = op;
        }
    }
}

impl crate::ops_markers::ParentBind<::udi_sys::meta_gio::udi_gio_bind_cb_t> for ::udi_sys::meta_gio::udi_gio_client_ops_t {
    const ASSERT: () = ();
}
impl crate::ops_markers::ChildBind for ::udi_sys::meta_gio::udi_gio_provider_ops_t {
    const ASSERT: () = ();
}

/// GIO Client (e.g. the user of a serial port)
pub trait Client: 'static + crate::imc::ChannelInit + crate::async_trickery::CbContext
{
    async_method!(
        /// Acknowledge a successful binding with a provider
        fn bind_ack  (&'s self, cb: crate::cb::CbRef<'s, ffi::udi_gio_bind_cb_t>, size: crate::Result<u64>)->()
        as Future_bind_ack
    );
    async_method!(
        /// Acknowledge a successful un-binding from the provider
        fn unbind_ack(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_gio_bind_cb_t>)->()
        as Future_unbind_ack
    );
    async_method!(
        /// A transfer has completed successfully
        fn xfer_ack  (&'s self, cb: crate::cb::CbRef<'s, ffi::udi_gio_xfer_cb_t>)->()
        as Future_xfer_ack
    );
    async_method!(
        /// A transfer failed somehow
        fn xfer_nak  (&'s self, cb: crate::cb::CbRef<'s, ffi::udi_gio_xfer_cb_t>, res: crate::Result<()>)->()
        as Future_xfer_nak
    );
    async_method!(
        /// Handle an event from the provider
        fn event_ind (&'s self, cb: crate::cb::CbRef<'s, ffi::udi_gio_event_cb_t>)->()
        as Future_event_ind
    );

    /// Release/return ownership of a now-unused transfer CB
    fn xfer_ret(&mut self, cb: crate::cb::CbHandle<ffi::udi_gio_xfer_cb_t>);
}
struct MarkerClient;
impl<T> crate::imc::ChannelHandler<MarkerClient> for T
where
    T: Client
{
    fn channel_bound(&mut self, params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
        unsafe {
            let cb = params.parent_bound.bind_cb as *mut ffi::udi_gio_bind_cb_t;
            ffi::udi_gio_bind_req(cb)
        }
    }
}
future_wrapper!(gio_bind_ack_op => <T as Client>(
    cb: *mut ffi::udi_gio_bind_cb_t,
    device_size_lo: ::udi_sys::udi_ubit32_t,
    device_size_hi: ::udi_sys::udi_ubit32_t,
    status: ::udi_sys::udi_status_t
) val @ {
    let size = crate::Error::from_status(status)
        .map(|()| device_size_lo as u64 | (device_size_hi as u64) << 32)
        ;
    val.bind_ack(cb, size)
} finally( () ) {
    unsafe { crate::async_trickery::channel_event_complete::<T,ffi::udi_gio_bind_cb_t>(cb, ::udi_sys::UDI_OK as _) }
});
future_wrapper!(gio_unbind_ack_op => <T as Client>(cb: *mut ffi::udi_gio_bind_cb_t) val @ {
    val.unbind_ack(cb)
});
future_wrapper!(gio_xfer_ack_op => <T as Client>(cb: *mut ffi::udi_gio_xfer_cb_t) val @ {
    val.xfer_ack(cb)
} finally( () ) {
    val.xfer_ret(unsafe { crate::cb::CbHandle::from_raw(cb) })
});
future_wrapper!(gio_xfer_nak_op => <T as Client>(cb: *mut ffi::udi_gio_xfer_cb_t, status: ::udi_sys::udi_status_t) val @ {
    val.xfer_nak(cb, crate::Error::from_status(status))
} finally( () ) {
    val.xfer_ret(unsafe { crate::cb::CbHandle::from_raw(cb) })
});
future_wrapper!(gio_event_ind_op => <T as Client>(cb: *mut ffi::udi_gio_event_cb_t) val @ {
    val.event_ind(cb)
} finally( () ) {
    unsafe { ::udi_sys::meta_gio::udi_gio_event_res(cb) }
});
map_ops_structure!{
    ffi::udi_gio_client_ops_t => Client,MarkerClient {
        gio_bind_ack_op,
        gio_unbind_ack_op,
        gio_xfer_ack_op,
        gio_xfer_nak_op,
        gio_event_ind_op,
    }
    CBS {
        ffi::udi_gio_bind_cb_t,
        ffi::udi_gio_xfer_cb_t,
        ffi::udi_gio_event_cb_t,
    }
}

/// GIO Provider (e.g. a serial port)
pub trait Provider: 'static + crate::imc::ChannelInit + crate::async_trickery::CbContext
{
    async_method!(
        /// A binding has been requested
        fn bind_req(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_gio_bind_cb_t>)->()
        as Future_bind_req
    );
    async_method!(
        /// Unbinding as been requested by the bounc client
        fn unbind_req(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_gio_bind_cb_t>)->()
        as Future_unbind_req
    );
    async_method!(
        /// A transfer has been requested
        fn xfer_req(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_gio_xfer_cb_t>)->()
        as Future_xfer_req
    );
    async_method!(
        /// The event has been handled
        fn event_res(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_gio_event_cb_t>)->()
        as Future_event_res
    );
    /// Return/relase an event CB
    fn event_ret(&mut self, cb: crate::cb::CbHandle<ffi::udi_gio_event_cb_t>);
}
struct MarkerProvider;
impl<T> crate::imc::ChannelHandler<MarkerProvider> for T
where
    T: Provider
{
}

future_wrapper!(gio_bind_req_op => <T as Provider>(cb: *mut ffi::udi_gio_bind_cb_t) val @ {
    val.bind_req(cb)
});
future_wrapper!(gio_unbind_req_op => <T as Provider>(cb: *mut ffi::udi_gio_bind_cb_t) val @ {
    val.unbind_req(cb)
});
future_wrapper!(gio_xfer_req_op => <T as Provider>(cb: *mut ffi::udi_gio_xfer_cb_t) val @ {
    val.xfer_req(cb)
});
future_wrapper!(gio_event_res_op => <T as Provider>(cb: *mut ffi::udi_gio_event_cb_t) val @ {
    val.event_res(cb)
} finally( () ) {
    val.event_ret(unsafe { crate::cb::CbHandle::from_raw(cb) })
});
map_ops_structure!{
    ffi::udi_gio_provider_ops_t => Provider,MarkerProvider {
        gio_bind_req_op,
        gio_unbind_req_op,
        gio_xfer_req_op,
        gio_event_res_op,
    }
    CBS {
        ffi::udi_gio_bind_cb_t,
        ffi::udi_gio_xfer_cb_t,
        ffi::udi_gio_event_cb_t,
    }
}
