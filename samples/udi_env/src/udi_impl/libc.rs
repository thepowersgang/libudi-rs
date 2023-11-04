use ::core::ffi::c_char;
use ::udi::ffi::*;

#[no_mangle]
unsafe extern "C" fn udi_strlen(s: *const c_char) -> udi_size_t {
    ::libc::strlen(s)
}
    
#[no_mangle]
unsafe extern "C" fn udi_strcat(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    ::libc::strcat(s1, s2)
}
#[no_mangle]
unsafe extern "C" fn udi_strncat(s1: *mut c_char, s2: *const c_char, n: udi_size_t) -> *mut c_char {
    ::libc::strncat(s1, s2, n)
}

#[no_mangle]
unsafe extern "C" fn udi_strcmp(s1: *const c_char, s2: *const c_char) -> udi_sbit8_t {
    ::libc::strcmp(s1, s2) as _
}
#[no_mangle]
unsafe extern "C" fn udi_strncmp(s1: *const c_char, s2: *const c_char, n: udi_size_t) -> udi_sbit8_t {
    ::libc::strncmp(s1, s2, n) as _
}
#[no_mangle]
unsafe extern "C" fn udi_memcmp(s1: *const c_void, s2: *const c_void, n: udi_size_t) -> udi_sbit8_t {
    ::libc::memcmp(s1, s2, n) as _
}

#[no_mangle]
unsafe extern "C" fn udi_strcpy(s1: *mut c_char, s2: *const c_char) -> *mut c_char {
    ::libc::strcpy(s1, s2)
}
#[no_mangle]
unsafe extern "C" fn udi_strncpy(s1: *mut c_char, s2: *const c_char, n: udi_size_t) -> *mut c_char {
    ::libc::strncpy(s1, s2, n)
}
#[no_mangle]
unsafe extern "C" fn udi_memcpy(s1: *mut c_void, s2: *const c_void, n: udi_size_t) -> *mut c_void {
    ::libc::memcpy(s1, s2, n)
}
#[no_mangle]
unsafe extern "C" fn udi_memmeove(s1: *mut c_void, s2: *const c_void, n: udi_size_t) -> *mut c_void {
    ::libc::memmove(s1, s2, n)
}

#[no_mangle]
unsafe extern "C" fn udi_strncpy_rtrim(s1: *mut c_char, s2: *const c_char, n: udi_size_t) -> *mut c_char {
    todo!()
}

#[no_mangle]
unsafe extern "C" fn udi_strchr(s: *const c_char, c: c_char) -> *mut c_char {
    ::libc::strchr(s, c as _)
}
#[no_mangle]
unsafe extern "C" fn udi_strrchr(s: *const c_char, c: c_char) -> *mut c_char {
    ::libc::strrchr(s, c as _)
}
#[no_mangle]
unsafe extern "C" fn udi_memchr(s: *const c_void, c: udi_ubit8_t, n: udi_size_t) -> *const c_void {
    ::libc::memchr(s, c as _, n)
}

#[no_mangle]
unsafe extern "C" fn udi_memset(s: *mut c_void, c: udi_ubit8_t, n: udi_size_t) -> *mut c_void {
    ::libc::memset(s, c as _, n)
}

#[no_mangle]
unsafe extern "C" fn udi_strtou32(s: *const c_char, endptr: *mut *mut c_char, base: ::core::ffi::c_int) -> udi_ubit32_t {
    todo!()
}

/*
#[no_mangle]
unsafe extern "C" fn udi_snprintf(s: *mut c_char, max_bytes: udi_size_t, format: *const c_char, ...) -> udi_size_t {
    ::core::arch::asm!("jmp snprintf", options(noreturn))
}
#[no_mangle]
unsafe extern "C" fn udi_vsnprintf(s: *mut c_char, max_bytes: udi_size_t, format: *const c_char, ap: ::core::ffi::VaList) -> udi_size_t {
    ::core::arch::asm!("jmp vsnprintf", options(noreturn))
}
*/
