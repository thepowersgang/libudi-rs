//! A common set of traits for metalanguage definitions

/// A trait used to get metalanguage ops and CB structures from numbers
pub trait Metalanguage
{
    fn name() -> &'static str where Self: Sized;
    /// Cast `ops_vector` into a ops structure
    unsafe fn get_ops(&self, ops_idx: ::udi_sys::udi_index_t, ops_vector: crate::ffi::udi_ops_vector_t) -> Option<&'static dyn MetalangOpsHandler>;
    /// Obtain a metalanguage-specific definition of a CB
    fn get_cb(&self, cb_idx: ::udi_sys::udi_index_t) -> Option<&dyn MetalangCbHandler>;
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
    const META_OPS_NUM: crate::ffi::udi_index_t;
}

/// Trait used for dynamic dispatch on a CB definition
pub trait MetalangCbHandler
{
    fn size(&self) -> usize;

    unsafe fn get_buffer<'a>(&self, cb: &'a mut crate::ffi::udi_cb_t) -> Option<&'a mut *mut crate::ffi::udi_buf_t> { let _ = cb; None }
    unsafe fn get_inline_data<'a>(&self, cb: &'a mut crate::ffi::udi_cb_t) -> Option<&'a mut *mut crate::ffi::c_void> { let _ = cb; None }
    unsafe fn get_chain<'a>(&self, cb: &'a mut crate::ffi::udi_cb_t) -> Option<&'a mut *mut crate::ffi::udi_cb_t> { let _ = cb; None }
}
/// Trait for to hold a CB's metalanguage number
/// 
/// SAFETY: The underlying type must start with a `udi_cb_t` structure
pub unsafe trait MetalangCb
{
    type MetalangSpec: Metalanguage;
    const META_CB_NUM: crate::ffi::udi_index_t;

    //fn get_buffer<'a>(&self, cb: &'a mut crate::ffi::udi_cb_t) -> Option<&'a mut *mut crate::ffi::udi_buf_t> { let _ = cb; None }
    //fn get_inline_data<'a>(&self, cb: &'a mut crate::ffi::udi_cb_t) -> Option<&'a mut *mut crate::ffi::c_void> { let _ = cb; None }
    fn get_chain(&mut self) -> Option<&mut *mut crate::ffi::udi_cb_t> { None }
}

macro_rules! impl_metalanguage
{
    (
        static $spec_name:ident;
        NAME $name:ident ;
        OPS $( $ops_idx:literal => $ops_ty:ty),* $(,)? ;
        CBS $( $cb_idx:literal => $cb_ty:ty $(: BUF $buf_fld:ident)? $(: INLINE_DATA $inline_data_fld:ident)? $(: CHAIN $chain_fld:ident)? ),* $(,)? ;
    ) => {
        pub struct Metalang;
        pub static $spec_name: Metalang = Metalang;
        impl $crate::metalang_trait::Metalanguage for Metalang {
            fn name() -> &'static str {
                stringify!($name)
            }
            unsafe fn get_ops(&self, ops_idx: $crate::ffi::udi_index_t, ops_vector: $crate::ffi::udi_ops_vector_t) -> Option<&'static dyn $crate::metalang_trait::MetalangOpsHandler> {
                match ops_idx.0
                {
                $( $ops_idx => Some(&*(ops_vector as *const $ops_ty)), )*
                _ => None,
                }
            }
        
            fn get_cb(&self, cb_idx: $crate::ffi::udi_index_t) -> Option<&dyn $crate::metalang_trait::MetalangCbHandler> {
                match cb_idx.0 {
                $( $cb_idx => {
                    struct H;
                    impl $crate::metalang_trait::MetalangCbHandler for H {
                        fn size(&self) -> usize {
                            ::core::mem::size_of::<$cb_ty>()
                        }
                        $(
                        unsafe fn get_buffer<'a>(&self, cb: &'a mut $crate::ffi::udi_cb_t) -> Option<&'a mut *mut $crate::ffi::udi_buf_t> {
                            let cb = &mut *(cb as *mut $crate::ffi::udi_cb_t as *mut $cb_ty);
                            Some(&mut cb.$buf_fld)
                        }
                        )?
                        $(
                        unsafe fn get_inline_data<'a>(&self, cb: &'a mut $crate::ffi::udi_cb_t) -> Option<&'a mut *mut $crate::ffi::c_void> {
                            let cb = &mut *(cb as *mut $crate::ffi::udi_cb_t as *mut $cb_ty);
                            Some(&mut cb.$inline_data_fld)
                        }
                        )?
                        $(
                        unsafe fn get_chain<'a>(&self, cb: &'a mut $crate::ffi::udi_cb_t) -> Option<&'a mut *mut $crate::ffi::udi_cb_t> {
                            let cb = &mut *(cb as *mut $crate::ffi::udi_cb_t as *mut $cb_ty);
                            Some(&mut *( &mut cb.$chain_fld as *mut *mut $cb_ty as *mut *mut $crate::ffi::udi_cb_t))
                        }
                        )?
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
            const META_OPS_NUM: $crate::ffi::udi_index_t = $crate::ffi::udi_index_t($ops_idx);
        }
        )*
        $(
        unsafe impl $crate::metalang_trait::MetalangCb for $cb_ty {
            type MetalangSpec = Metalang;
            const META_CB_NUM: $crate::ffi::udi_index_t = {
                #[allow(dead_code)]
                fn assert_cb_field(input: &$cb_ty) {
                    let _ = input.gcb;
                    //let _ = [(); {
                    //    let p = ::core::ptr::null::<$cb_ty>();
                    //    if ::core::ptr::addr_of!((*p).gcb) == p as *mut crate::ffi::udi_cb_t { 0 } else { panic!() }
                    //    }];
                }
                $crate::ffi::udi_index_t($cb_idx)
                };
            $(
            fn get_chain(&mut self) -> Option<&mut *mut $crate::ffi::udi_cb_t> {
                Some( unsafe { &mut *( &mut self.$chain_fld as *mut *mut Self as *mut *mut $crate::ffi::udi_cb_t) } )
            }
            )?
        }
        )*
    };
}

/// Helper to generate code to create a ops structure based on a trait
/// 
/// See [future_wrapper]
macro_rules! map_ops_structure {
    (
        $struct:path => $trait:path,$marker:ty {
            $($name:ident,)*
        }
        CBS {
            $($cb:ty,)*
        }
        $( EXTRA_OP $extra_op:ident );*
    ) => {
        impl<T,CbList> crate::OpsStructure<$struct, T,CbList>
        where
            T: $trait,
            $( CbList: crate::HasCb<$cb>, )*
        {
            pub const fn scratch_requirement() -> usize {
                let v = crate::imc::task_size::<T, $marker>();
                $(let v = crate::const_max(v, $name::task_size::<T>());)*
                $(let v = crate::const_max(v, $extra_op::task_size::<T>());)*
                v
            }
            /// SAFETY: Caller must ensure that the ops are only used with matching `T` region
            /// SAFETY: The scratch size must be >= value returned by [Self::scratch_requirement]
            pub const unsafe fn for_driver() -> $struct {
                $struct {
                    channel_event_ind_op: crate::imc::channel_event_ind_op::<T, $marker>,
                    $( $name: $name::<T>, )*
                }
            }
        }
        
    };
}