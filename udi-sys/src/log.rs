
use super::*;
use crate::init::udi_init_context_t;

pub type udi_trevent_t = u32;
/* Common Trace Events */
pub const UDI_TREVENT_LOCAL_PROC_ENTRY	: udi_trevent_t = 1<<0;
pub const UDI_TREVENT_LOCAL_PROC_EXIT	: udi_trevent_t = 1<<1;
pub const UDI_TREVENT_EXTERNAL_ERROR	: udi_trevent_t = 1<<2;
/* Common Metalanguage-Selectable Trace Events */
pub const UDI_TREVENT_IO_SCHEDULED	: udi_trevent_t = 1<<6;
pub const UDI_TREVENT_IO_COMPLETED	: udi_trevent_t = 1<<7;
/* Metalanguage-Specific Trace Events */
pub const UDI_TREVENT_META_SPECIFIC_1	: udi_trevent_t = 1<<11;
pub const UDI_TREVENT_META_SPECIFIC_2	: udi_trevent_t = 1<<12;
pub const UDI_TREVENT_META_SPECIFIC_3	: udi_trevent_t = 1<<13;
pub const UDI_TREVENT_META_SPECIFIC_4	: udi_trevent_t = 1<<14;
pub const UDI_TREVENT_META_SPECIFIC_5	: udi_trevent_t = 1<<15;
/* Driver-Specific Trace Events */
pub const UDI_TREVENT_INTERNAL_1	: udi_trevent_t = 1<<16;
pub const UDI_TREVENT_INTERNAL_2	: udi_trevent_t = 1<<17;
pub const UDI_TREVENT_INTERNAL_3	: udi_trevent_t = 1<<18;
pub const UDI_TREVENT_INTERNAL_4	: udi_trevent_t = 1<<19;
pub const UDI_TREVENT_INTERNAL_5	: udi_trevent_t = 1<<20;
pub const UDI_TREVENT_INTERNAL_6	: udi_trevent_t = 1<<21;
pub const UDI_TREVENT_INTERNAL_7	: udi_trevent_t = 1<<22;
pub const UDI_TREVENT_INTERNAL_8	: udi_trevent_t = 1<<23;
pub const UDI_TREVENT_INTERNAL_9	: udi_trevent_t = 1<<24;
pub const UDI_TREVENT_INTERNAL_10	: udi_trevent_t = 1<<25;
pub const UDI_TREVENT_INTERNAL_11	: udi_trevent_t = 1<<26;
pub const UDI_TREVENT_INTERNAL_12	: udi_trevent_t = 1<<27;
pub const UDI_TREVENT_INTERNAL_13	: udi_trevent_t = 1<<28;
pub const UDI_TREVENT_INTERNAL_14	: udi_trevent_t = 1<<29;
pub const UDI_TREVENT_INTERNAL_15	: udi_trevent_t = 1<<30;
/* Logging Event */
pub const UDI_TREVENT_LOG	: udi_trevent_t = 1<<31;

pub const UDI_LOG_DISASTER   : u8 = 1;
pub const UDI_LOG_ERROR      : u8 = 2;
pub const UDI_LOG_WARNING    : u8 = 3;
pub const UDI_LOG_INFORMATION: u8 = 4;

pub type udi_log_write_call_t = extern "C" fn(*mut udi_cb_t, udi_status_t);

extern "C" {
	// --- Tracing
	pub fn udi_trace_write(init_context: *const udi_init_context_t, trace_event: udi_trevent_t, index: udi_index_t, msgnum: u32, ...);
	pub fn udi_log_write(callback: udi_log_write_call_t, cb: *mut udi_cb_t, trace_event: udi_trevent_t, severity: u8, meta_idx: udi_index_t, original_status: udi_status_t, msgnum: u32, ...);

	// --- Debugging
	pub fn udi_assert(expr: udi_boolean_t);
	pub fn udi_debug_break(init_context: *const udi_init_context_t, message: *const ::core::ffi::c_char);
	pub fn udi_debug_printf(format: *const ::core::ffi::c_char, ...);
}

