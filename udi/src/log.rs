//! UDI logging functions
use ::udi_sys::init::udi_init_context_t;
use ::udi_sys::log::udi_trevent_t;
use ::udi_sys::udi_index_t;
use ::udi_sys::udi_index_t as MetaIdx;

/// A pre-defined message in udiprops
pub trait Message
{
    /// Message number (link between udiprops and the code)
    const NUM: u32;
    /// Argument types for the message
    type Args: MessageDispatch;
}
/// A trait used to provide type checking on [Message::Args]
pub trait MessageDispatch {
    /// Call `udi_trace_write` with the arguments in `self`
    unsafe fn trace_write(self, init_context: *const udi_init_context_t, trace_event: udi_trevent_t, meta_idx: udi_index_t, msgnum: u32);
}
macro_rules! impl_dispatch {
    () => {
        impl MessageDispatch for () {
            unsafe fn trace_write(self, init_context: *const udi_init_context_t, trace_event: udi_trevent_t, meta_idx: udi_index_t, msgnum: u32) {
                ::udi_sys::log::udi_trace_write(init_context, trace_event, meta_idx, msgnum )
            }
        }
    };
    ( $t0:ident $(, $t:ident)* $(,)? ) => {
        impl<$t0, $( $t,)* > MessageDispatch for ( $t0, $( $t,)* )
        where
            $t0: super::libc::SnprintfArg,
            $( $t : super::libc::SnprintfArg, )*
        {
            unsafe fn trace_write(self, init_context: *const udi_init_context_t, trace_event: udi_trevent_t, meta_idx: udi_index_t, msgnum: u32) {
                #[allow(non_snake_case)]
                let ( $t0, $( $t,)* ) = self;
                ::udi_sys::log::udi_trace_write(init_context, trace_event, meta_idx, msgnum, $t0.into_arg() $(, $t.into_arg() )* )
            }
        }
        impl_dispatch!{ $( $t,)* }
    };
}
impl_dispatch!{A, B, C, D, E, F, G, H, I, J, }

/// Rust version of the `udi_trevent_t` type
pub enum TraceEvent {
    /// `UDI_TREVENT_LOCAL_PROC_ENTRY` - Trace entry to all procedures that are local to the driver. Include argument values in the trace output.
    LocalProcEntry,
}
impl TraceEvent {
    
}

/// Write a trace message
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