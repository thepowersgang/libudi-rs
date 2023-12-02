use ::udi::ffi::init::udi_init_context_t;
use ::udi::ffi::log::udi_log_write_call_t;
use ::udi::ffi::log::udi_trevent_t;
use ::udi::ffi::udi_cb_t;
use ::udi::ffi::udi_index_t;
use ::udi::ffi::udi_status_t;

use ::std::io::Write;

#[no_mangle]
pub unsafe extern "C" fn udi_debug_printf(fmt: *const ::core::ffi::c_char, mut args: ...) {
    let format = ::core::ffi::CStr::from_ptr(fmt);
    let mut sink = Sink([0; 64], 0);
    print!("udi_debug_printf: ");
    super::libc::snprintf_inner(&mut sink, format.to_bytes(), args.as_va_list());
    ::std::io::stdout().write_all(&sink.0[..sink.1]).unwrap();
    println!("");
}

struct Sink([u8; 64], usize);
impl super::libc::SnprintfSink for Sink {
    fn push(&mut self, byte: u8) {
        if self.1 == self.0.len() {
            ::std::io::stdout().write_all(&self.0).unwrap();
            self.1 = 0;
        }
        self.0[self.1] = byte;
        self.1 += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn udi_assert(expr: ::udi::ffi::udi_boolean_t) {
    if !expr.to_bool() {
        panic!("`udi_assert` failure")
    }
}

#[no_mangle]
pub unsafe extern "C" fn udi_trace_write(
    init_context: *const udi_init_context_t,
    trace_event: udi_trevent_t, meta_idx: udi_index_t,
    msgnum: u32, mut args: ...
)
{
    let module = crate::DriverRegion::driver_module_from_context(&*init_context);
    let Some(format) = module.get_message(::udiprops_parse::parsed::MsgNum(msgnum as u16)) else { todo!() };

    let mut sink = Sink([0; 64], 0);
    print!("udi_trace_write[{} T]: ", trace_event);
    super::libc::snprintf_inner(&mut sink, format.as_bytes(), args.as_va_list());
    ::std::io::stdout().write_all(&sink.0[..sink.1]).unwrap();
    println!("");
}
#[no_mangle]
pub unsafe extern "C" fn udi_log_write(
    callback: udi_log_write_call_t, cb: *mut udi_cb_t,
    trace_event: udi_trevent_t, severity: u8, meta_idx: udi_index_t, original_status: udi_status_t,
    msgnum: u32, mut args: ...
)
{
    let i = crate::channels::get_driver_instance(&(*cb).channel);
    let Some(format) = i.module.get_message(::udiprops_parse::parsed::MsgNum(msgnum as u16)) else { todo!() };

    let mut sink = Sink([0; 64], 0);
    print!("udi_log_write[{} {}]: ", trace_event, severity);
    super::libc::snprintf_inner(&mut sink, format.as_bytes(), args.as_va_list());
    ::std::io::stdout().write_all(&sink.0[..sink.1]).unwrap();
    println!("");

    callback(cb, ::udi::ffi::UDI_OK as _);
}