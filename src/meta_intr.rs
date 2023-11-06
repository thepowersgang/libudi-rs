//! Interrupt metalanguage (Phsical I/O Specification)
use crate::ffi::meta_intr::{udi_intr_event_cb_t, udi_intr_attach_cb_t};


pub fn attach_req(cb: super::cb::CbHandle<udi_intr_attach_cb_t>) {
    unsafe { crate::ffi::meta_intr::udi_intr_attach_req(cb.into_raw()) }
}
pub fn event_rdy(cb: super::cb::CbHandle<udi_intr_event_cb_t>) {
    unsafe { crate::ffi::meta_intr::udi_intr_event_rdy(cb.into_raw()) }
}

pub type CbRefEvent<'a> = crate::CbRef<'a, udi_intr_event_cb_t>;

pub trait IntrHandler: 'static
{
    async_method!(fn intr_event_ind(&'a mut self, cb: CbRefEvent<'a>, flags: u8)->() as Future_intr_event_ind);
}
struct MarkerIntrHandler;
impl<T> crate::imc::ChannelHandler<MarkerIntrHandler> for T
where
    T: IntrHandler
{
    fn channel_closed(&mut self) {
    }
    fn channel_bound(&mut self, _params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
    }
}


future_wrapper!(intr_event_ind_op => <T as IntrHandler>(cb: *mut udi_intr_event_cb_t, flags: u8) val @ {
    crate::async_trickery::with_ack(
        val.intr_event_ind(cb, flags),
        // Return this CB to the pool on completion
        |cb,_res| unsafe { crate::ffi::meta_intr::udi_intr_event_rdy(cb) }
        )
});

impl<T,CbList> crate::OpsStructure<::udi_sys::meta_intr::udi_intr_handler_ops_t, T,CbList>
where
	T: IntrHandler,
    CbList: crate::HasCb<udi_intr_event_cb_t>,
{
    pub const fn scratch_requirement() -> usize {
        let v = crate::imc::task_size::<T, MarkerIntrHandler>();
        let v = crate::const_max(v, intr_event_ind_op::task_size::<T>());
        v
    }
    /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
    /// SAFETY: The scratch size must be >= value returned by [Self::scratch_requirement]
    pub const unsafe fn for_driver() -> ::udi_sys::meta_intr::udi_intr_handler_ops_t {
        ::udi_sys::meta_intr::udi_intr_handler_ops_t {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T, MarkerIntrHandler>,
            intr_event_ind_op: intr_event_ind_op::<T>,
        }
    }
}


pub trait IntrDispatcher: 'static
{
    async_method!(fn intr_event_rdy(&'a mut self, cb: CbRefEvent<'a>)->() as Future_intr_event_rdy);
}
struct MarkerIntrDispatcher;
impl<T> crate::imc::ChannelHandler<MarkerIntrDispatcher> for T
where
    T: IntrDispatcher
{
    fn channel_closed(&mut self) {
    }
    fn channel_bound(&mut self, _params: &crate::ffi::imc::udi_channel_event_cb_t_params) {
    }
}


future_wrapper!(intr_event_rdy_op => <T as IntrDispatcher>(cb: *mut udi_intr_event_cb_t) val @ {
    val.intr_event_rdy(cb)
});

impl<T,CbList> crate::OpsStructure<::udi_sys::meta_intr::udi_intr_dispatcher_ops_t, T,CbList>
where
	T: IntrDispatcher,
    CbList: crate::HasCb<udi_intr_event_cb_t>,
{
    pub const fn scratch_requirement() -> usize {
        let v = crate::imc::task_size::<T, MarkerIntrDispatcher>();
        let v = crate::const_max(v, intr_event_rdy_op::task_size::<T>());
        v
    }
    /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
    /// SAFETY: The scratch size must be >= value returned by [Self::scratch_requirement]
    pub const unsafe fn for_driver() -> ::udi_sys::meta_intr::udi_intr_dispatcher_ops_t {
        ::udi_sys::meta_intr::udi_intr_dispatcher_ops_t {
            channel_event_ind_op: crate::imc::channel_event_ind_op::<T, MarkerIntrDispatcher>,
            intr_event_rdy_op: intr_event_rdy_op::<T>,
        }
    }
}
