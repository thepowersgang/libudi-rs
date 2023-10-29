use ::udi::ffi::cb::udi_cb_alloc_call_t;
use ::udi::ffi::udi_index_t;
use ::udi::ffi::{udi_cb_t, udi_channel_t};

pub trait MetalangCb
{
    fn size(&self) -> usize;
    unsafe fn init(&self, cb: *mut udi_cb_t, gcb: udi_cb_t)
    {
        ::core::ptr::write(cb, gcb);
    }
}

#[no_mangle]
unsafe extern "C" fn udi_cb_alloc(callback: udi_cb_alloc_call_t, gcb: *mut udi_cb_t, cb_idx: udi_index_t, default_channel: udi_channel_t)
{
    let module = crate::channels::get_driver_module(&(*gcb).channel);
    let rv = alloc_internal(module, cb_idx, (*gcb).context, default_channel);
    callback(gcb, rv);
}

pub fn alloc_internal(driver_module: &crate::DriverModule, cb_idx: udi_index_t, context: *mut ::udi::ffi::c_void, default_channel: udi_channel_t) -> *mut udi_cb_t
{
    let cb_init = driver_module.get_cb_init(cb_idx).unwrap();
    let cb_spec = driver_module.get_cb_spec(cb_init);

    let size = cb_spec.size();
    assert!(size >= ::core::mem::size_of::<udi_cb_t>());
    unsafe {
        let rv = ::libc::calloc(1, size) as *mut udi_cb_t;
        cb_spec.init(rv, udi_cb_t {
            channel: default_channel,
            context,
            initiator_context: context,
            scratch: ::libc::malloc(cb_init.scratch_requirement),
            origin: ::core::ptr::null_mut(),
            });
        rv
    }
}