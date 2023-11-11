//! Generic I/O Metalanguage
//! 
//! This is a very generic metalanguage for drivers for whom a specialised
//! metalanguage isn't (yet) possible.
use ::udi_sys::meta_gio::*;
use ::udi_sys::meta_gio as ffi;

impl_metalanguage!{
    static METALANG_SPEC;
    NAME udi_gio;
    OPS
        1 => udi_gio_provider_ops_t,
        2 => udi_gio_client_ops_t,
        ;
    CBS
        1 => udi_gio_bind_cb_t,
        2 => udi_gio_xfer_cb_t,
        3 => udi_gio_event_cb_t,
        ;
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
    async_method!(fn bind_ack  (&'s mut self, cb: crate::cb::CbRef<'s, ffi::udi_gio_bind_cb_t>, size: crate::Result<u64>)->() as Future_bind_ack);
    async_method!(fn unbind_ack(&'s mut self, cb: crate::cb::CbRef<'s, ffi::udi_gio_bind_cb_t>)->() as Future_unbind_ack);
    async_method!(fn xfer_ack  (&'s mut self, cb: crate::cb::CbHandle<ffi::udi_gio_xfer_cb_t>)->() as Future_xfer_ack);
    async_method!(fn xfer_nak  (&'s mut self, cb: crate::cb::CbHandle<ffi::udi_gio_xfer_cb_t>, res: crate::Result<()>)->() as Future_xfer_nak);
    async_method!(fn event_ind (&'s mut self, cb: crate::cb::CbRef<'s, ffi::udi_gio_event_cb_t>)->() as Future_event_ind);
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
    crate::async_trickery::with_ack(
        val.bind_ack(cb, size),
        |cb,()| unsafe { crate::async_trickery::channel_event_complete::<T,ffi::udi_gio_bind_cb_t>(cb, ::udi_sys::UDI_OK as _) }
        )
});
future_wrapper!(gio_unbind_ack_op => <T as Client>(cb: *mut ffi::udi_gio_bind_cb_t) val @ {
    val.unbind_ack(cb)
});
future_wrapper!(gio_xfer_ack_op => <T as Client>(cb: *mut ffi::udi_gio_xfer_cb_t) val @ {
    val.xfer_ack(unsafe { cb.into_owned() })
});
future_wrapper!(gio_xfer_nak_op => <T as Client>(cb: *mut ffi::udi_gio_xfer_cb_t, status: ::udi_sys::udi_status_t) val @ {
    val.xfer_nak(unsafe { cb.into_owned() }, crate::Error::from_status(status))
});
future_wrapper!(gio_event_ind_op => <T as Client>(cb: *mut ffi::udi_gio_event_cb_t) val @ {
    crate::async_trickery::with_ack(
        val.event_ind(cb),
        |cb,()| unsafe { ::udi_sys::meta_gio::udi_gio_event_res(cb) }
        )
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
    async_method!(fn bind_req(&'s mut self, cb: crate::cb::CbRef<'s, ffi::udi_gio_bind_cb_t>)->() as Future_bind_req);
    async_method!(fn unbind_req(&'s mut self, cb: crate::cb::CbRef<'s, ffi::udi_gio_bind_cb_t>)->() as Future_unbind_req);
    async_method!(fn xfer_req(&'s mut self, cb: crate::cb::CbRef<'s, ffi::udi_gio_xfer_cb_t>)->() as Future_xfer_req);
    async_method!(fn event_res(&'s mut self, cb: crate::cb::CbRef<'s, ffi::udi_gio_event_cb_t>)->() as Future_event_res);
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
