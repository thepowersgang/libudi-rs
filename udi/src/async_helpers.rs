
/// Synchronous point, useful to wait for metalang operation to complete
/// 
/// E.g. Use to check the result of [crate::meta_bridge::attach_req] within the `bind_req` handler
pub struct Wait<R>
{
    result: Option<R>,
    cb: *mut crate::ffi::udi_cb_t,
}
impl<R> Default for Wait<R> {
    fn default() -> Self {
        Self { result: Default::default(), cb: ::core::ptr::null_mut() }
    }
}
impl<R> Wait<R>
where
    R: ::core::marker::Unpin
{
    // TODO: This has a slight violation of the unique borrow rules.
    // - We're expecting `signal` to be called from a different async, which will mutate `self.res` while
    //   `Self::wait` is holding a borrow to it
    // Larger issue with use of `&mut` for region contexts, when there can be multiple requests in flight.
    // - While regions are serialised (which I assume means only one thread), async calls are a boundary on that.
    //   - It is valid (and `udi-environment` does) for an environment to run one CB until an async call, then run
    //     another CB for the same region, before returning to the original CB.
    // - Could require interior mutability for all regions, although that'd be "fun"

    /// Wait until the result is set
    pub fn wait<'s>(&'s mut self, cb: crate::CbRef<'s, ::udi_sys::udi_cb_t>) -> impl ::core::future::Future<Output=R> + 's {
        let wake_instant = self.result.is_some();
        let cb_slot = &mut self.cb;
        let res_slot = &mut self.result;
        crate::async_trickery::wait_task(cb, move |gcb| {
            if wake_instant {
                Self::signal_inner(gcb);
            }
            else {
                *cb_slot = gcb;
            }
        }, move |_| res_slot.take().unwrap())
    }
    /// Set the result, and wake the waiter
    pub fn signal(&mut self, r: R) {
        self.result = Some(r);
        if ! self.cb.is_null() {
            Self::signal_inner(self.cb);
        }
    }

    fn signal_inner(gcb: *mut crate::ffi::udi_cb_t) {
        // SAFE: The GCB is valid (this function is private)
        unsafe {
            crate::async_trickery::signal_waiter(&mut *gcb, crate::async_trickery::WaitRes::Data([0; 4]));
        }
    }
}