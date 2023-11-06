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
    let rv = alloc(&module, cb_idx, (*gcb).context, default_channel);
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
    let module = crate::channels::get_driver_module(&(*gcb).channel);
    let rv = alloc_internal(&module, cb_idx, (*gcb).context, default_channel, None, None, Some((inline_size, inline_layout)));
    callback(gcb, rv);
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
    for _i in 0..count.0 {
        prev_cb = alloc_internal(
            &module, cb_idx, (*gcb).context, ::core::ptr::null_mut(), 
            Some(prev_cb), if with_buf.to_bool() { Some((buf_size, path_handle)) } else { None }, None
        );
    }
    callback(gcb, prev_cb);
}

// --------------------------------------------------------------------

pub fn alloc(driver_module: &crate::DriverModule, cb_idx: udi_index_t, context: *mut ::udi::ffi::c_void, default_channel: udi_channel_t) -> *mut udi_cb_t
{
    alloc_internal(driver_module, cb_idx, context, default_channel, None, None, None)
}

fn alloc_internal(
    driver_module: &crate::DriverModule,
    cb_idx: udi_index_t,
    context: *mut ::udi::ffi::c_void,
    default_channel: udi_channel_t,
    chain: Option<*mut udi_cb_t>,
    buf_info: Option<(udi_size_t, udi_buf_path_t)>,
    inline_info: Option<(udi_size_t, *const udi_layout_t )>,
) -> *mut udi_cb_t
{
    let cb_init = driver_module.get_cb_init(cb_idx).unwrap();
    let cb_spec = driver_module.get_cb_spec(cb_init);

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

        // Buffer allocation
        if let Some((buf_size, buf_path)) = buf_info {
            let dst = cb_spec.get_buffer(&mut *rv).expect("No buffer");
            *dst = crate::udi_impl::buf::allocate(buf_size, buf_path);
        }
        
        // Inline data allocation
        if let Some((alloc_size, _alloc_layout)) = inline_info {
            let dst = cb_spec.get_inline_data(&mut *rv).expect("No inline data present");
            *dst = ::libc::calloc(1, alloc_size);
        }

        // Chained CBs, handles a null layout
        if let Some(chain_cb) = chain {
            if let Some(dst) = cb_spec.get_chain(&mut *rv) {
                *dst = chain_cb as _;
            }
            else {
                (*rv).initiator_context = chain_cb as _;
            }
        }

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