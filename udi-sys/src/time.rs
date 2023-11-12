// 14 - Time Management
use super::*;

/// The udi_time_t structure is used to specify a timeout interval for use with
/// the UDI Timer Services or an elapsed time interval returned by UDI
/// Timestamp Services. The fields in this structure allow very precise
/// specification of time values relative to the current time; the [super::init::udi_limits_t]
/// values should be consulted to determine the actual granularity of the
/// environment’s timers, as all specified [udi_time_t] values will be rounded
/// up to integral multiples of the minimum system timer resolution.
/// 
/// This structure is not used to represent absolute (“wall-clock”) times. UDI
/// provides no facility to determine absolute time.
#[repr(C)]
pub struct udi_time_t
{
    pub seconds: udi_ubit32_t,
    pub microseconds: udi_ubit32_t,
}

pub type udi_timer_expired_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t);
pub type udi_timer_tick_call_t = unsafe extern "C" fn(context: *mut c_void, nmissed: udi_ubit32_t);

extern "C" {
    pub fn udi_timer_start(callback: udi_timer_expired_call_t, gcb: *mut udi_cb_t, interval: udi_time_t);
    pub fn udi_timer_start_repeating(callback: udi_timer_tick_call_t, gcb: *mut udi_cb_t, interval: udi_time_t);
    pub fn udi_timer_cancel(gcb: *mut udi_cb_t);
}

extern "C" {
    pub fn udi_time_current() -> udi_timestamp_t;
    pub fn udi_time_between(start_time: udi_timestamp_t, end_time: udi_timestamp_t) -> udi_time_t;
    pub fn udi_time_since(start_time: udi_timestamp_t) -> udi_time_t;
}