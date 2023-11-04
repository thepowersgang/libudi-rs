use ::udi::ffi::buf::udi_buf_path_t;
use ::udi::ffi::cb::{udi_cb_alloc_call_t, udi_cb_alloc_batch_call_t};
use ::udi::ffi::{udi_index_t,udi_boolean_t,udi_size_t};
use ::udi::ffi::{udi_cb_t, udi_channel_t, udi_layout_t};

pub trait MetalangCb
{
    fn size(&self) -> usize;
    unsafe fn init(&self, cb: *mut udi_cb_t, gcb: udi_cb_t)
    {
        ::core::ptr::write(cb, gcb);
    }
}
impl<T: ::udi::metalang_trait::MetalangCb> MetalangCb for T {
    fn size(&self) -> usize {
        ::core::mem::size_of::<T>()
    }
}

#[no_mangle]
unsafe extern "C" fn udi_cb_alloc(callback: udi_cb_alloc_call_t, gcb: *mut udi_cb_t, cb_idx: udi_index_t, default_channel: udi_channel_t)
{
    let module = crate::channels::get_driver_module(&(*gcb).channel);
    let rv = alloc_internal(&module, cb_idx, (*gcb).context, default_channel);
    callback(gcb, rv);
}

#[no_mangle]
unsafe extern "C" fn udi_cb_alloc_dynamic(
    callback: udi_cb_alloc_call_t,
    gcb: *mut udi_cb_t,
    cb_idx: udi_index_t,
    default_channel: udi_channel_t,
    inline_size: udi_size_t,
    inline_layout: *const udi_layout_t
)
{
    todo!("udi_cb_alloc_dynamic");
}

#[no_mangle]
unsafe extern "C" fn udi_cb_alloc_batch(
    callback: udi_cb_alloc_batch_call_t,
    gcb: *mut udi_cb_t,
    cb_idx: udi_index_t,
    count: udi_index_t,
    with_buf: udi_boolean_t,
    buf_size: udi_size_t,
    path_handle: udi_buf_path_t
)
{
    let module = crate::channels::get_driver_module(&(*gcb).channel);
    let mut prev_cb = ::core::ptr::null_mut();
    for _i in 0..count {
        let rv = alloc_internal(&module, cb_idx, (*gcb).context, ::core::ptr::null_mut());
        if with_buf != 0 {
            todo!("udi_cb_alloc_batch - with_buf");
        }

        if false {
            // If there's a chaining field, use that
        }
        else {
            (*rv).initiator_context = prev_cb as _;
        }
        prev_cb = rv;
    }
    callback(gcb, prev_cb);
}

// --------------------------------------------------------------------

pub fn alloc_internal(driver_module: &crate::DriverModule, cb_idx: udi_index_t, context: *mut ::udi::ffi::c_void, default_channel: udi_channel_t) -> *mut udi_cb_t
{
    let cb_init = driver_module.get_cb_init(cb_idx).unwrap();
    let cb_spec = driver_module.get_cb_spec(cb_init);

    // TODO: inline allocation
    if !cb_init.inline_layout.is_null() {
        todo!("Handle inline layout");
    }

    let size = cb_spec.size();
    assert!(size >= ::core::mem::size_of::<udi_cb_t>());
    unsafe {
        let rv = ::libc::calloc(1, size) as *mut udi_cb_t;
        ::core::ptr::write(rv, udi_cb_t {
            channel: default_channel,
            context,
            initiator_context: context,
            scratch: ::libc::malloc(cb_init.scratch_requirement),
            origin: ::core::ptr::null_mut(),
            });
        rv
    }
}
pub unsafe fn free_internal(handle: *mut udi_cb_t)
{
    if ! (*handle).scratch.is_null() {
        ::libc::free((*handle).scratch);
    }
    ::libc::free(handle as *mut ::libc::c_void);
}