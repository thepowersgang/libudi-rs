use ::udi_sys::init::udi_init_context_t;
use ::udi_sys::log::udi_trevent_t;
use ::udi_sys::udi_index_t;
use ::udi_sys::udi_index_t as MetaIdx;

pub trait Message
{
    const NUM: u32;
    type Args: MessageDispatch;
}
pub trait MessageDispatch {
    unsafe fn trace_write(self, init_context: *const udi_init_context_t, trace_event: udi_trevent_t, meta_idx: udi_index_t, msgnum: u32);
}
impl MessageDispatch for () {
    unsafe fn trace_write(self, init_context: *const udi_init_context_t, trace_event: udi_trevent_t, meta_idx: udi_index_t, msgnum: u32) {
        ::udi_sys::log::udi_trace_write(init_context, trace_event, meta_idx, msgnum)
    }
}
impl<T> MessageDispatch for (T,)
where
    T: super::libc::SnprintfArg,
{
    unsafe fn trace_write(self, init_context: *const udi_init_context_t, trace_event: udi_trevent_t, meta_idx: udi_index_t, msgnum: u32) {
        ::udi_sys::log::udi_trace_write(init_context, trace_event, meta_idx, msgnum, self.0)
    }
}

pub enum TraceEvent {
    LocalProcEntry,
}
impl TraceEvent {
    
}

pub fn trace_write<T, M>(context: &crate::init::RData<T>, trace_event: TraceEvent, meta_idx: MetaIdx, _message: M, args: M::Args)
where
    M: Message,
{
    let trace_event = match trace_event
        {
        TraceEvent::LocalProcEntry => crate::ffi::log::UDI_TREVENT_LOCAL_PROC_ENTRY,
        };
    unsafe {
        args.trace_write(context as *const _ as *const _, trace_event, meta_idx, M::NUM)
    }
}