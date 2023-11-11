pub trait SnprintfArg {
    type Output;
    fn into_arg(self) -> Self::Output;
}

macro_rules! impl_snprintf_arg_identity {
    ( $($t:ty,)* ) => {
        $(
        impl SnprintfArg for $t {
            type Output = Self;
            fn into_arg(self) -> Self::Output {
                self
            }
        }
        )*
    };
}
impl_snprintf_arg_identity!{
    crate::ffi::udi_ubit32_t,
    crate::ffi::udi_sbit32_t,
    crate::ffi::udi_ubit16_t,
    crate::ffi::udi_sbit16_t,
    crate::ffi::udi_ubit8_t,
    crate::ffi::udi_sbit8_t,

    //crate::ffi::udi_busaddr64_t,
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