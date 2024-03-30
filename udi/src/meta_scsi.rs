//! SCSI metalanguage definition
use ::udi_sys::meta_scsi as ffi;

impl_metalanguage!{
    static METALANG_SPEC;
    NAME udi_scsi;
    OPS
        1 => ffi::udi_scsi_pd_ops_t,
        2 => ffi::udi_scsi_hd_ops_t,
        ;
    CBS
        1 => ffi::udi_scsi_bind_cb_t,
        2 => ffi::udi_scsi_io_cb_t : BUF data_buf,
        3 => ffi::udi_scsi_ctl_cb_t,
        4 => ffi::udi_scsi_event_cb_t : BUF aen_data_buf,
        ;
}

/// Request unbind from the host
pub fn unbind_req(cb: crate::cb::CbHandle<ffi::udi_scsi_bind_cb_t>) {
    unsafe { ffi::udi_scsi_unbind_req(cb.into_raw()) }
}
/// Start a new IO operation
pub fn io_req(cb: crate::cb::CbHandle<ffi::udi_scsi_io_cb_t>) {
    unsafe { ffi::udi_scsi_io_req(cb.into_raw()) }
}
/// Make a control request
pub fn ctl_req(cb: crate::cb::CbHandle<ffi::udi_scsi_ctl_cb_t>) {
    unsafe { ffi::udi_scsi_ctl_req(cb.into_raw()) }
}
/// Indicate to the peripheral that an event has occurred
pub fn event_ind(cb: crate::cb::CbHandle<ffi::udi_scsi_event_cb_t>) {
    unsafe { ffi::udi_scsi_event_ind(cb.into_raw()) }
}

/// Options for binding a peripheral to a host
pub struct BindOpts
{
    bind_flags: ::udi_sys::udi_ubit16_t,
    queue_depth: ::udi_sys::udi_ubit16_t,
    max_sense_len: ::udi_sys::udi_ubit16_t,
    aen_buf_size: ::udi_sys::udi_ubit16_t
}

/// Trait to be implemented by SCSI peripheral drivers
pub trait Peripheral: 'static + crate::imc::ChannelInit + crate::async_trickery::CbContext
{
    /// Obtain binding options to send to the host when binding
    fn bind_opts(&mut self) -> BindOpts;
    async_method!(
        /// Acknowledgement of a successful binding
        fn bind_ack  (&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_bind_cb_t>, hd_timeout_increase: crate::Result<u32>)->()
        as Future_bind_ack
    );
    async_method!(
        /// Acknowledgement of a successful un-binding
        fn unbind_ack(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_bind_cb_t>)->()
        as Future_unbind_ack
    );
    /// Release the CB used for an unbind request
    fn unbind_ret(&mut self, cb: crate::cb::CbHandle<ffi::udi_scsi_bind_cb_t>) { let _ = cb; }
    async_method!(
        /// Acknowledgement of successful IO
        fn io_ack(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_io_cb_t>)->() as Future_io_ack
    );
    async_method!(
        /// IO has failed, includes the status and SCSI sense value
        fn io_nak(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_io_cb_t>, res: ffi::udi_scsi_status_t, sense: crate::buf::Handle)->()
        as Future_io_nak
    );
    /// Release the CB used for an IO request
    fn io_ret(&mut self, cb: crate::cb::CbHandle<ffi::udi_scsi_io_cb_t>) { let _ = cb; }
    async_method!(
        /// Handle completion of a control request
        fn ctl_ack(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_ctl_cb_t>, res: crate::Result<()>)->()
        as Future_ctl_ack
    );
    /// Release the CB used for an IO control
    fn ctl_ret(&mut self, cb: crate::cb::CbHandle<ffi::udi_scsi_ctl_cb_t>) { let _ = cb; }
    async_method!(
        /// Handle an event
        fn event_ind(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_event_cb_t>)->()
        as Future_event_ind
    );
}
/// Trait to be implemented by SCSI host drivers
pub trait Host: 'static + crate::imc::ChannelInit + crate::async_trickery::CbContext
{
    async_method!(
        /// Handle an incoming binding request with a peripheral device driver
        fn bind_req(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_bind_cb_t>, opts: BindOpts)->crate::Result<u32>
        as Future_bind_ack
    );
    async_method!(
        /// Handle an incoming un--binding request from the peripheral device driver
        fn unbind_req(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_bind_cb_t>)->()
        as Future_unbind_req
    );
    async_method!(
        /// Handle an incoming IO request from the peripheral device driver
        ///
        /// NOTE: The buffer should be returned if the result is a NAK
        fn io_req(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_io_cb_t>)->(ffi::udi_scsi_status_t,crate::buf::Handle)
        as Future_io_req
    );
    async_method!(
        /// Handle an incoming control request from the peripheral device driver
        fn ctl_req(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_ctl_cb_t>)->crate::Result<()>
        as Future_ctl_ack
    );
    async_method!(
        /// Called when the peripheral device driver has handled an event
        fn event_res(&'s self, cb: crate::cb::CbRef<'s, ffi::udi_scsi_event_cb_t>)->()
        as Future_event_res
    );
    /// Return/release an event CB
    fn event_ret(&mut self, cb: crate::cb::CbHandle<ffi::udi_scsi_event_cb_t>);
}


struct MarkerPeripheral;
impl<T> crate::imc::ChannelHandler<MarkerPeripheral> for T
where
    T: Peripheral
{
    fn channel_bound(&mut self, params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
        unsafe {
            let cb = params.parent_bound.bind_cb as *mut ffi::udi_scsi_bind_cb_t;
            let opts = self.bind_opts();
            ffi::udi_scsi_bind_req(cb, opts.bind_flags, opts.queue_depth, opts.max_sense_len, opts.aen_buf_size);
        }
    }
}
future_wrapper!(bind_ack_op => <T as Peripheral>(
    cb: *mut ffi::udi_scsi_bind_cb_t,
    status: ::udi_sys::udi_status_t,
    hd_timeout_increase: ::udi_sys::udi_ubit32_t
) val @ {
    let hd_timeout_increase = crate::Error::from_status(status)
        .map(|()| hd_timeout_increase)
        ;
    val.bind_ack(cb, hd_timeout_increase)
} finally( () ) {
    unsafe { crate::async_trickery::channel_event_complete::<T,ffi::udi_scsi_bind_cb_t>(cb, ::udi_sys::UDI_OK as _) }
});
future_wrapper!(unbind_ack_op => <T as Peripheral>(
    cb: *mut ffi::udi_scsi_bind_cb_t
) val @ {
    val.unbind_ack(cb)
} finally( () ) {
    val.unbind_ret(unsafe { crate::cb::CbHandle::from_raw(cb) });
});
future_wrapper!(io_ack_op => <T as Peripheral>(
    cb: *mut ffi::udi_scsi_io_cb_t
) val @ {
    val.io_ack(cb)
} finally( () ) {
    val.io_ret(unsafe { crate::cb::CbHandle::from_raw(cb) });
});
future_wrapper!(io_nak_op => <T as Peripheral>(
    cb: *mut ffi::udi_scsi_io_cb_t,
    status: ffi::udi_scsi_status_t,
    buf: *mut ::udi_sys::udi_buf_t
) val @ {
    val.io_nak(cb, status, unsafe { crate::buf::Handle::from_raw(buf) })
} finally( () ) {
    val.io_ret(unsafe { crate::cb::CbHandle::from_raw(cb) });
});
future_wrapper!(ctl_ack_op => <T as Peripheral>(
    cb: *mut ffi::udi_scsi_ctl_cb_t,
    status: ::udi_sys::udi_status_t
) val @ {
    val.ctl_ack(cb, crate::Error::from_status(status))
} finally( () ) {
    val.ctl_ret(unsafe { crate::cb::CbHandle::from_raw(cb) });
});
future_wrapper!(event_ind_op => <T as Peripheral>(
    cb: *mut ffi::udi_scsi_event_cb_t
) val @ {
    val.event_ind(cb)
} finally( () ) {
    unsafe { ffi::udi_scsi_event_res(cb) }
});
map_ops_structure!{
    ffi::udi_scsi_pd_ops_t => Peripheral,MarkerPeripheral {
        bind_ack_op,
        unbind_ack_op,
        io_ack_op,
        io_nak_op,
        ctl_ack_op,
        event_ind_op,
    }
    CBS {
        ffi::udi_scsi_bind_cb_t,
        ffi::udi_scsi_io_cb_t,
        ffi::udi_scsi_ctl_cb_t,
        ffi::udi_scsi_event_cb_t,
    }
}


struct MarkerHost;
impl<T> crate::imc::ChannelHandler<MarkerHost> for T
where
    T: Host
{
}
future_wrapper!(bind_req_op => <T as Host>(
    cb: *mut ffi::udi_scsi_bind_cb_t,
    bind_flags: ::udi_sys::udi_ubit16_t,
    queue_depth: ::udi_sys::udi_ubit16_t,
    max_sense_len: ::udi_sys::udi_ubit16_t,
    aen_buf_size: ::udi_sys::udi_ubit16_t
) val @ {
    val.bind_req(cb, BindOpts { bind_flags, queue_depth, max_sense_len, aen_buf_size })
} finally( res ) {
    unsafe { ffi::udi_scsi_bind_ack(cb, res.unwrap_or(0), crate::Error::to_status(res.map(|_| ()))) }
});
future_wrapper!(unbind_req_op => <T as Host>(
    cb: *mut ffi::udi_scsi_bind_cb_t
) val @ {
    val.unbind_req(cb)
} finally( () ) {
    unsafe { ffi::udi_scsi_unbind_ack(cb) }
});
future_wrapper!(io_req_op => <T as Host>(
    cb: *mut ffi::udi_scsi_io_cb_t
) val @ {
    val.io_req(cb)
} finally( (status, handle) ) {
    if handle.len() > 0 || status.req_status != ::udi_sys::UDI_OK as _ {
        unsafe { ffi::udi_scsi_io_nak(cb, status, handle.into_raw()) }
    }
    else {
        unsafe { ffi::udi_scsi_io_ack(cb) }
    }
});
future_wrapper!(ctl_req_op => <T as Host>(
    cb: *mut ffi::udi_scsi_ctl_cb_t
) val @ {
    val.ctl_req(cb)
} finally( res ) {
    unsafe { ffi::udi_scsi_ctl_ack(cb, crate::Error::to_status(res)) }
});
future_wrapper!(event_res_op => <T as Host>(
    cb: *mut ffi::udi_scsi_event_cb_t
) val @ {
    val.event_res(cb)
} finally( () ) {
    val.event_ret(unsafe { crate::cb::CbHandle::from_raw(cb) })
});
map_ops_structure!{
    ffi::udi_scsi_hd_ops_t => Host,MarkerHost {
        bind_req_op,
        unbind_req_op,
        io_req_op,
        ctl_req_op,
        event_res_op,
    }
    CBS {
        ffi::udi_scsi_bind_cb_t,
        ffi::udi_scsi_io_cb_t,
        ffi::udi_scsi_ctl_cb_t,
        ffi::udi_scsi_event_cb_t,
    }
}
