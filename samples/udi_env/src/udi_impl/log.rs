// A helper to ensure that `udi_debug_printf` is availble
#[doc(hidden)]
pub static _REF: unsafe extern "C" fn(*const ::core::ffi::c_char, ...) = ::udi::ffi::log::udi_debug_printf;