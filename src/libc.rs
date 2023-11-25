pub trait SnprintfArg {
    type Output;
    fn into_arg(self) -> Self::Output;
}

impl SnprintfArg for crate::ffi::udi_ubit32_t {
    type Output = crate::ffi::udi_ubit32_t;
    fn into_arg(self) -> Self::Output {
        self
    }
}
impl SnprintfArg for crate::ffi::udi_sbit32_t {
    type Output = crate::ffi::udi_sbit32_t;
    fn into_arg(self) -> Self::Output {
        self
    }
}
// NOTE: Cannot pass values smaller than u32
impl SnprintfArg for crate::ffi::udi_ubit16_t {
    type Output = crate::ffi::udi_ubit32_t;
    fn into_arg(self) -> Self::Output {
        self as _
    }
}
impl SnprintfArg for crate::ffi::udi_sbit16_t {
    type Output = crate::ffi::udi_sbit16_t;
    fn into_arg(self) -> Self::Output {
        self as _
    }
}
impl SnprintfArg for crate::ffi::udi_ubit8_t {
    type Output = crate::ffi::udi_ubit32_t;
    fn into_arg(self) -> Self::Output {
        self as _
    }
}
impl SnprintfArg for crate::ffi::udi_sbit8_t {
    type Output = crate::ffi::udi_sbit16_t;
    fn into_arg(self) -> Self::Output {
        self as _
    }
}

// CString for `%s`
impl SnprintfArg for &::core::ffi::CStr {
    type Output = *const ::core::ffi::c_char;
    fn into_arg(self) -> Self::Output {
        self.as_ptr()
    }
}

// Pointer types for `%p`
impl<T> SnprintfArg for *const T {
    type Output = *const ();
    fn into_arg(self) -> Self::Output {
        self as *const ()
    }
}