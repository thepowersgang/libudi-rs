
use super::*;
use crate::init::udi_init_context_t;

pub type udi_trevent_t = u32;

pub const UDI_LOG_DISASTER: u8 = 1;

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

