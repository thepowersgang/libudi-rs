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
unsafe extern "C" fn udi_strncpy_rtrim(mut s1: *mut c_char, mut s2: *const c_char, mut n: udi_size_t) -> *mut c_char {
    let init_s1 = s1;
    while n > 0 && *s2 != 0 {
        *s1 = *s2;
        s1 = s1.offset(1);
        s2 = s2.offset(1);
        n -= 1;
    }
    *s1 = 0;
    while s1 != init_s1 {
        s1 = s1.offset(-1);
        if (*s1 as u8 as char).is_ascii_whitespace() {
            *s1 = 0;
        }
    }
    init_s1
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
unsafe extern "C" fn udi_strtou32(mut s: *const c_char, endptr: *mut *mut c_char, base: ::core::ffi::c_int) -> udi_ubit32_t {
    let mut rv = 0;
    // Strip leading whitespace
    while (*s) != 0 && (*s as u8 as char).is_ascii_whitespace() {
        s = s.offset(1);
    }
    let is_neg = match *s as u8
        {
        b'+' => false,
        b'-' => true,
        _ => false,
        };
    // Handle prefix to determine base
    let base = match base
        {
        0 => if *s as u8 == b'0' {
            s = s.offset(1);
            if *s as u8 == b'x' || *s as u8 == b'X' {
                s = s.offset(1);
                16
            }
            else {
                8
            }
        }
        else {
            10
        }
        16 => {
            // Consume the `0x` if it is there
            if *s as u8 == b'0' {
                s = s.offset(1);
                if *s as u8 == b'x' || *s as u8 == b'X' {
                    s = s.offset(1);
                }
            }
            16
        }
        base => base,
        };
    while (*s) != 0 {
        match (*s as u8 as char).to_digit(base as u32)
        {
        Some(v) => {
            rv *= base as u32;
            rv += v;
            },
        None => break,
        }
        s = s.offset(1);
    }
    if !endptr.is_null() {
        *endptr = s as *mut c_char;
    }

    if is_neg {
        !rv + 1
    }
    else {
        rv
    }
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
