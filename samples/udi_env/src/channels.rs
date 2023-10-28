

struct ChannelInner {
    sides: [ChannelInnerSide; 2],
}
struct ChannelInnerSide {
    context: *mut ::udi::ffi::c_void,
    scratch_requirement: usize,
    metalang: ::std::any::TypeId,
    ops: ::udi::ffi::udi_ops_vector_t,
}

pub fn allocate_channel<O: MetalangOps>(context: *mut ::udi::ffi::c_void, ops: &'static O, scratch_requirement: usize) -> ::udi::ffi::udi_channel_t
{
    let handle = Box::new(ChannelInner {
        sides: [
            ChannelInnerSide { context, scratch_requirement, metalang: ::core::any::TypeId::of::<O>(), ops: ops as *const _ as *const _ },
            ChannelInnerSide { context, scratch_requirement: 0, metalang: ::core::any::TypeId::of::<()>(), ops: ::core::ptr::null() },
        ]
    });
    let handle = Box::into_raw(handle);
    handle as *mut _
}
pub unsafe fn bind_channel_other<O: MetalangOps>(
    ch: ::udi::ffi::udi_channel_t,
    driver_module: *const crate::DriverModule,
    context: *mut ::udi::ffi::c_void,
    ops: &'static O,
    scratch_requirement: usize
)
{
    assert!(!ch.is_null(), "Channel is NULL?");
    assert!(ch as usize & 1 == 0);
    let inner = &mut *(ch as *mut ChannelInner);
    let side = &mut inner.sides[1];
    assert!(side.ops.is_null());
    side.context = context;
    side.metalang = ::core::any::TypeId::of::<O>();
    side.ops = ops as *const _ as *const _;
    side.scratch_requirement = scratch_requirement;
}

pub trait MetalangOps: 'static
{
}
pub unsafe fn prepare_cb_for_call<O: MetalangOps>(cb: &mut ::udi::ffi::udi_cb_t) -> &O
{
    unsafe fn reverse_and_get(src: &mut ::udi::ffi::udi_channel_t) -> &ChannelInnerSide {
        let (ptr,is_b) = ((*src as usize & !1) as *const ChannelInner, *src as usize & 1 != 0);
        assert!(!ptr.is_null(), "Channel is NULL?");
        *src = (ptr as usize | !is_b as usize) as ::udi::ffi::udi_channel_t;
        &(*ptr).sides[1 - is_b as usize]
    }

    // Get the channel currently in the cb, and reverse it
    let ch_side = reverse_and_get(&mut (*cb).channel);
    // Update context and scratch
    (*cb).context = ch_side.context;
    (*cb).scratch = ::libc::realloc((*cb).scratch, ch_side.scratch_requirement);
    // Then check that the metalanguage ops in that side matches the expectation
    if ch_side.metalang != ::std::any::TypeId::of::<O>() {
        panic!("Metalang mismatch: Expected {:?}, got {:?}", ch_side.metalang, ::std::any::TypeId::of::<O>());
    }
    else {
        &*(ch_side.ops as *const O)
    }
}

pub unsafe fn get_driver_module(ch: &::udi::ffi::udi_channel_t) -> &crate::DriverModule {
    todo!()
}