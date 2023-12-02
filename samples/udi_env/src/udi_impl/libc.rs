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
    
    let format = ::core::ffi::CStr::from_ptr(format);
    snprintf_inner(&mut rv, format.to_bytes(), ap);

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
    fn push_str(&mut self, s: &[u8]) {
        for b in s.iter().copied() {
            self.push(b)
        }
    }
}
/// The innards of [udi_snprintf]/[udi_vsnprintf]/[super::log::udi_debug_printf]
pub unsafe fn snprintf_inner(rv: &mut dyn SnprintfSink, format: &[u8], mut ap: ::core::ffi::VaList)
{
    let mut p = ::udi_macro_helpers::printf::Parser::new(format);
    loop {
        let e = match p.next() {
            Ok(Some(v)) => v,
            Ok(None) => break,
            Err(_) => return,
        };

        let mut tmpbuf = [0; 64/4];
        
        match e {
        udi_macro_helpers::printf::FormatArg::StringData(s) => rv.push_str(s),

        udi_macro_helpers::printf::FormatArg::Pointer(is_upper/*, pad, width*/) => {
            rv.push_str(b"0x");
            fmt_pad_rev(rv, Pad::SpaceLeft, 0, to_str_radix(&mut tmpbuf, 16, is_upper, ap.arg::<*const ()>() as usize as u64), None)
        },
        udi_macro_helpers::printf::FormatArg::String(pad, width) => {
            let pad = match pad {
                udi_macro_helpers::printf::PadKind::LeadingZero => Pad::ZeroRight,
                udi_macro_helpers::printf::PadKind::LeftPad => Pad::SpaceLeft,
                udi_macro_helpers::printf::PadKind::Unspec => Pad::SpaceRight,
            };
            let s = ::core::ffi::CStr::from_ptr(ap.arg::<*const ::core::ffi::c_char>());
            fmt_pad(rv, pad, width as _, s.to_bytes().iter().copied(), None)
        },
        udi_macro_helpers::printf::FormatArg::BusAddr(_is_upper) => {
            //rv.fmt_pad_rev(pad, Pad::SpaceLeft, 0, to_str_radix(&mut tmpbuf, 16, is_upper, ap.arg::<udi_busaddr64_t>()), None);
            todo!("bus addr")
        },
        udi_macro_helpers::printf::FormatArg::Char => {
            let pad = Pad::SpaceRight;
            let width = 0;
            fmt_pad_rev(rv, pad, width, &[ap.arg::<udi_ubit8_t>()], None)
        }
        udi_macro_helpers::printf::FormatArg::Integer(pad, width, size, fmt) => {
            let pad = match pad {
                udi_macro_helpers::printf::PadKind::LeadingZero => Pad::ZeroRight,
                udi_macro_helpers::printf::PadKind::LeftPad => Pad::SpaceLeft,
                udi_macro_helpers::printf::PadKind::Unspec => Pad::SpaceRight,
            };
            let width = width as _;
            use udi_macro_helpers::printf::IntFormat;
            match fmt {
            IntFormat::LowerHex => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 16, false, ap.arg::<udi_ubit32_t>() as _), None),
            IntFormat::UpperHex => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 16, true , ap.arg::<udi_ubit32_t>() as _), None),
            IntFormat::Unsigned => fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 10, false, ap.arg::<udi_ubit32_t>() as _), None),
            IntFormat::Decimal => {
                let v = match size {
                    udi_macro_helpers::printf::Size::U32 => ap.arg::<udi_sbit32_t>(),
                    udi_macro_helpers::printf::Size::U16 => ap.arg::<udi_ubit32_t>() as u16 as i16 as i32,
                    udi_macro_helpers::printf::Size::U8 => ap.arg::<udi_ubit32_t>() as u8 as i8 as i32,
                    };
                let prefix = if v.is_negative() { Some(b'-') } else { None };
                fmt_pad_rev(rv, pad, width, to_str_radix(&mut tmpbuf, 10, false, v.unsigned_abs() as _), prefix)
                },
            }
        },
        udi_macro_helpers::printf::FormatArg::BitSet(mut p) => {
            rv.push(b'<');
            let v = ap.arg::<udi_ubit32_t>();
            let mut comma_needed = false;
            loop {
                let e = match p.next() {
                    Ok(Some(v)) => v,
                    Ok(None) => break,
                    Err(_) => return,
                };
                match e {
                udi_macro_helpers::printf::BitsetEnt::Single(bit, is_inv, name) => {
                    if ((v >> bit) & 1 == 0) == is_inv {
                        if comma_needed {
                            rv.push_str(b", ");
                        }
                        rv.push_str(name);
                        comma_needed = true;
                    }
                },
                udi_macro_helpers::printf::BitsetEnt::Range(start, end, name, mut p) => {
                    if comma_needed {
                        rv.push_str(b", ");
                    }
                    rv.push_str(name);
                    rv.push(b'=');
                    let nbits = end - start;
                    let v = (v >> start) & ((1 << nbits) - 1);
                    let mut printed = false;
                    loop {
                        let (chk,name) = match p.next() {
                            Ok(Some(v)) => v,
                            Ok(None) => break,
                            Err(_) => return,
                        };
                        if v == chk {
                            rv.push_str(name);
                            printed = true;
                            break;
                        }
                    }
                    if !printed {
                        fmt_pad_rev(rv, Pad::SpaceRight, 0, to_str_radix(&mut tmpbuf, 16, false, v as _), None);
                    }
                    comma_needed = true;
                },
                }
            }
            rv.push(b'>');
        },
        }
    }

    enum Pad {
        SpaceRight,
        ZeroRight,
        SpaceLeft,
    }

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
