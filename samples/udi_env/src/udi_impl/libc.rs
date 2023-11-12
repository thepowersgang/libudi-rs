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


#[no_mangle]
pub unsafe extern "C" fn udi_snprintf(s: *mut c_char, max_bytes: udi_size_t, format: *const c_char, mut args: ...) -> udi_size_t {
    udi_vsnprintf(s, max_bytes, format, args.as_va_list())
}
#[no_mangle]
unsafe extern "C" fn udi_vsnprintf(s: *mut c_char, max_bytes: udi_size_t, format: *const c_char, ap: ::core::ffi::VaList) -> udi_size_t {
    let dst = ::core::slice::from_raw_parts_mut(s, max_bytes);
    let mut rv = Output(dst, 0);
    
    snprintf_inner(&mut rv, format, ap);

    // Ensure NUL terminated
    let nul_pos = usize::min(rv.1, rv.0.len());
    if nul_pos < rv.0.len() {
        rv.0[nul_pos] = 0;
    }
    return rv.1;

    struct Output<'a>(&'a mut [i8], usize);
    impl<'a> SnprintfSink for Output<'a> {
        fn push(&mut self, byte: u8) {
            if self.1 < self.0.len() - 1 {
                self.0[self.1] = byte as i8;
            }
            self.1 += 1;
        }
    }
}

/// Effectively fmt::Write, but simpler
pub trait SnprintfSink {
    fn push(&mut self, byte: u8);
}
/// The innards of [udi_snprintf]/[udi_vsnprintf]/[super::log::udi_debug_printf]
pub unsafe fn snprintf_inner(rv: &mut dyn SnprintfSink, format: *const c_char, mut ap: ::core::ffi::VaList)
{
    let format = ::core::ffi::CStr::from_ptr(format);
    let mut it = format.to_bytes().iter().copied();
    'outer: while let Some(mut c) = it.next()
    {
        macro_rules! nextc {
            () => {
                match it.next() { Some(c) => c, None => break 'outer, }
            };
        }
        if c != b'%' {
            rv.push(c);
        }
        else {
            c = nextc!();
            let pad = if c == b'0' {
                    // Zero pad
                    c = nextc!();
                    Pad::ZeroRight
                }
                else if c == b'-' {
                    // Left-justify
                    c = nextc!();
                    Pad::SpaceLeft
                }
                else {
                    Pad::SpaceRight
                };
            let mut width = 0;
            while c.is_ascii_digit() {
                width *= 10;
                width += (c as char).to_digit(10).unwrap() as usize;
                c = nextc!();
            }

            let mut tmpbuf = [0; 64/4];
            fn to_str_radix(dst: &mut [u8], radix: u32, upper: bool, value: u64) -> &[u8] {
                assert!(2 <= radix && radix <= 36);

                let mut v = value;
                let mut len = 0;
                loop {
                    let c = char::from_digit((v % radix as u64) as u32, radix).unwrap() as u8;
                    dst[len] = if upper {
                        c.to_ascii_uppercase()
                    }
                    else {
                        c.to_ascii_lowercase()
                    };
                    len += 1;
                    v /= radix as u64;
                    if v == 0 {
                        break &dst[..len];
                    }
                }
            }
            match c {
            b'x' => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 16, false, ap.arg::<udi_ubit32_t>() as _), None),
            b'X' => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 16, true , ap.arg::<udi_ubit32_t>() as _), None),
            b'u' => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 10, false, ap.arg::<udi_ubit32_t>() as _), None),
            b'd' => {
                let v = ap.arg::<udi_sbit32_t>();
                let prefix = if v.is_negative() { Some(b'-') } else { None };
                fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 10, false, v.unsigned_abs() as _), prefix)
                },
            b'h' => {
                c = match it.next() { Some(c) => c, None => break, };
                match c {
                b'x' => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 16, false, ap.arg::<udi_ubit16_t>() as _), None),
                b'X' => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 16, true , ap.arg::<udi_ubit16_t>() as _), None),
                b'u' => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 10, false, ap.arg::<udi_ubit16_t>() as _), None),
                b'd' => {
                    let v = ap.arg::<udi_sbit16_t>();
                    let prefix = if v.is_negative() { Some(b'-') } else { None };
                    fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 10, false, v.unsigned_abs() as _), prefix)
                    },
                _ => {
                    rv.push(b'%'); rv.push(b'h'); rv.push(c);
                },
                }
                },
            b'b' => {
                c = match it.next() { Some(c) => c, None => break, };
                match c {
                b'x'|b'X' => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 16, c == b'X', ap.arg::<udi_ubit8_t>() as _), None),
                b'u' => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 10, false, ap.arg::<udi_ubit8_t>() as _), None),
                b'd' => {
                    let v = ap.arg::<udi_sbit8_t>();
                    let prefix = if v.is_negative() { Some(b'-') } else { None };
                    fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 10, false, v.unsigned_abs() as _), prefix)
                    },
                _ => {
                    rv.push(b'%'); rv.push(b'b'); rv.push(c);
                },
                }
                },
            b'p'|b'P' => {
                rv.push(b'0'); rv.push(b'x');
                fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 16, c == b'P', ap.arg::<*const ()>() as usize as u64), None)
                }
            //b'a'|b'A' => rv.fmt_pad_rev(pad, width, to_str_radix(&mut tmpbuf, 16, c == b'A', ap.arg::<udi_busaddr64_t>()), None),
            b'c' => {
                fmt_pad_rev(rv, pad, width, &[ap.arg::<udi_ubit8_t>()], None)
                },
            b's' => {
                let s = ::core::ffi::CStr::from_ptr(ap.arg::<*const ::core::ffi::c_char>());
                fmt_pad(rv, pad, width, s.to_bytes().iter().copied(), None)
                },
            b'<' => {
                rv.push(b'<');
                let val = ap.arg::<udi_ubit32_t>();
                c = nextc!();
                if c == b',' {
                    c = nextc!();
                }
                let mut need_comma = false;
                loop {
                    if c == b'>' {
                        break;
                    }
                    let is_not = if c == b'~' { c = nextc!(); true } else { false };
                    let bitnum = {
                        let mut v = 0;
                        while let Some(d) = (c as char).to_digit(10) {
                            v *= 10;
                            v += d;
                            c = nextc!();
                        }
                        v
                        };
                    if c == b'-' {
                        c = nextc!();
                        let end_bitnum = {
                            let mut v = 0;
                            while let Some(d) = (c as char).to_digit(10) {
                                v *= 10;
                                v += d;
                                c = nextc!();
                            }
                            v
                        };
                        if c == b'=' {
                            c = nextc!();
                        }
                        let nbit = end_bitnum.saturating_sub(bitnum);
                        let val = (val >> bitnum) & !(!0 << nbit);
                        if need_comma {
                            rv.push(b',');
                            rv.push(b' ');
                        }
                        loop {
                            if c == b'>' || c == b',' || c == b':' {
                                break;
                            }
                            rv.push(c);
                            c = nextc!();
                        }
                        rv.push(b'=');
                        let mut found = false;
                        while c == b':' {
                            c = nextc!();
                            let target_val = {
                                let mut v = 0;
                                while let Some(d) = (c as char).to_digit(10) {
                                    v *= 10;
                                    v += d;
                                    c = nextc!();
                                }
                                v
                            };
                            if c == b'=' {
                                c = nextc!();
                            }
                            else {
                                // Error.
                            }
                            let is_match = val == target_val;
                            if is_match {
                                found = true;
                            }
                            loop {
                                if c == b'>' || c == b',' || c == b':' {
                                    break;
                                }
                                if is_match {
                                    rv.push(c);
                                }
                                c = nextc!();
                            }
                        }
                        if !found {
                            fmt_pad_rev(rv, Pad::SpaceRight, 0, to_str_radix(&mut tmpbuf, 16, false, val as _), None)
                        }
                        need_comma = true;
                    }
                    else {
                        if c == b'=' {
                            c = nextc!();
                        }
                        else {
                            // Error.
                        }
                        let is_match = ((val >> bitnum) & 1 != 0) != is_not;

                        if is_match && need_comma {
                            rv.push(b',');
                            rv.push(b' ');
                        }
                        
                        loop {
                            if c == b'>' || c == b',' {
                                break;
                            }
                            if is_match {
                                rv.push(c);
                            }
                            c = nextc!();
                        }
                        if is_match {
                            need_comma = true;
                        }
                    }
                    if c != b',' {
                        break;
                    }
                    c = nextc!();   // Consume the comma
                }
                rv.push(b'>');
                },
            _ => {
                rv.push(b'%'); rv.push(c);
                },
            }
        }
    }

    enum Pad {
        SpaceRight,
        ZeroRight,
        SpaceLeft,
    }

    fn fmt_pad_rev(v: &mut dyn SnprintfSink, pad: Pad, width: usize, src: &[u8], prefix: Option<u8>) {
        fmt_pad(v, pad, width, src.iter().rev().copied(), prefix)
    }
    fn fmt_pad(v: &mut dyn SnprintfSink, pad: Pad, width: usize, src: impl ExactSizeIterator<Item=u8>, prefix: Option<u8>) {
        let len = src.len() + prefix.is_some() as usize;
        match pad {
        Pad::SpaceLeft => {
            if let Some(prefix) = prefix {
                v.push(prefix);
            }
        },
        Pad::SpaceRight => {
            if len < width { for _ in 0..(width-len) { v.push(b' '); } }
            if let Some(prefix) = prefix {
                v.push(prefix);
            }
        },
        Pad::ZeroRight => {
            if let Some(prefix) = prefix {
                v.push(prefix);
            }
            if len < width { for _ in 0..(width-len) { v.push(b'0'); } }
        },
        }
        for b in src {
            v.push(b);
        }
        if len < width {
            match pad {
            Pad::SpaceLeft => for _ in 0..(width-len) { v.push(b' '); },
            Pad::SpaceRight => {},
            Pad::ZeroRight => {},
            }
        }
    }
}
