
use super::*;

pub type udi_trevent_t = u32;

pub const UDI_LOG_DISASTER: u8 = 1;

pub type udi_log_write_call_t = extern "C" fn(*mut udi_cb_t, udi_status_t);

extern "C" {
	pub fn udi_log_write(callback: udi_log_write_call_t, cb: *mut udi_cb_t, trace_event: udi_trevent_t, severity: u8, meta_idx: udi_index_t, original_status: udi_status_t, msgnum: u32, ...);
}

