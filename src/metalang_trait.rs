//! A common set of traits for metalanguage definitions

/// A trait used to get metalanguage ops and CB structures from numbers
pub trait Metalanguage
{
    /// Cast `ops_vector` into a ops structure
    unsafe fn get_ops(&self, ops_idx: u8, ops_vector: crate::ffi::udi_ops_vector_t) -> Option<&'static dyn MetalangOpsHandler>;
    /// Obtain a metalanguage-specific definition of a CB
    fn get_cb(&self, cb_idx: u8) -> Option<&dyn MetalangCbHandler>;
}
/// Trait used for dynamic dispatch of an `ops` structure
/// 
/// To be used by environment implementors to dispatch
pub trait MetalangOpsHandler: 'static
{
    /// Human-readable (debug) type name of the ops type
    fn type_name(&self) -> &'static str;
    /// Type ID (debug checking)
    fn type_id(&self) -> ::core::any::TypeId;
    /// Obtain the `channel_event_ind_op` field of the vector
    fn channel_event_ind_op(&self) -> crate::ffi::imc::udi_channel_event_ind_op_t;
}
/// A trait to hold the `ops_num for an ops structure
/// 
/// SAFETY: The pointed data must be valid as [crate::ffi::init::udi_ops_init_t]
pub unsafe trait MetalangOps: MetalangOpsHandler
{
    const META_OPS_NUM: u8;
}

/// Trait used for dynamic dispatch on a CB definition
pub trait MetalangCbHandler
{
    fn size(&self) -> usize;
}
/// Trait for to hold a CB's metalanguage number
/// 
/// SAFETY: The underlying type must start with a `udi_cb_t` structure
pub unsafe trait MetalangCb
{
    const META_CB_NUM: u8;
}

macro_rules! impl_metalanguage
{
    (static $name:ident; OPS $( $ops_idx:literal => $ops_ty:ty),* $(,)? ; CBS $( $cb_idx:literal => $cb_ty:ty),* $(,)? ; ) => {
        pub struct Metalang;
        pub static $name: Metalang = Metalang;
        impl $crate::metalang_trait::Metalanguage for Metalang {
            unsafe fn get_ops(&self, ops_idx: u8, ops_vector: $crate::ffi::udi_ops_vector_t) -> Option<&'static dyn $crate::metalang_trait::MetalangOpsHandler> {
                match ops_idx
                {
                $( $ops_idx => Some(&*(ops_vector as *const $ops_ty)), )*
                _ => None,
                }
            }
        
            fn get_cb(&self, cb_idx: u8) -> Option<&dyn $crate::metalang_trait::MetalangCbHandler> {
                match cb_idx {
                $( $cb_idx => {
                    struct H;
                    impl $crate::metalang_trait::MetalangCbHandler for H {
                        fn size(&self) -> usize {
                            ::core::mem::size_of::<$cb_ty>()
                        }
                    }
                    Some(&H)
                    }, )*
                _ => None,
                }
            }
        }
        $(
        impl $crate::metalang_trait::MetalangOpsHandler for $ops_ty {
            fn type_name(&self) -> &'static str {
                ::core::any::type_name::<Self>()
            }
            fn type_id(&self) -> ::core::any::TypeId {
                ::core::any::TypeId::of::<Self>()
            }
            fn channel_event_ind_op(&self) -> $crate::ffi::imc::udi_channel_event_ind_op_t {
                self.channel_event_ind_op
            }
        }
        unsafe impl $crate::metalang_trait::MetalangOps for $ops_ty {
            const META_OPS_NUM: u8 = $ops_idx;
        }
        )*
        $(
        unsafe impl $crate::metalang_trait::MetalangCb for $cb_ty {
            const META_CB_NUM: u8 = {
                #[allow(dead_code)]
                fn assert_cb_field(input: &$cb_ty) {
                    let _ = input.gcb;
                    //let _ = [(); {
                    //    let p = ::core::ptr::null::<$cb_ty>();
                    //    if ::core::ptr::addr_of!((*p).gcb) == p as *mut crate::ffi::udi_cb_t { 0 } else { panic!() }
                    //    }];
                }
                $cb_idx
                };
        }
        )*
    };
}