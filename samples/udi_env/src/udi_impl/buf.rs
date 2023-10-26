use ::udi::ffi::buf::udi_buf_copy_call_t;
use ::udi::ffi::buf::udi_buf_path_t;
use ::udi::ffi::udi_buf_t;
use ::udi::ffi::udi_cb_t;
use ::udi::ffi::udi_size_t;
use ::udi::ffi::c_void;


#[no_mangle]
unsafe extern "C" fn udi_buf_copy(
    callback: udi_buf_copy_call_t,
    gcb: *mut udi_cb_t,
    src_buf: *mut udi_buf_t,
    src_off: udi_size_t,
    src_len: udi_size_t,
    dst_buf: *mut udi_buf_t,
    dst_off: udi_size_t,
    dst_len: udi_size_t,
    path_handle: udi_buf_path_t
)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_buf_write(
    callback: udi_buf_copy_call_t,
    gcb: *mut udi_cb_t,
    src_buf: *const c_void,
    src_len: udi_size_t,
    dst_buf: *mut udi_buf_t,
    dst_off: udi_size_t,
    dst_len: udi_size_t,
    path_handle: udi_buf_path_t
)
{
    todo!()
}
#[no_mangle]
unsafe extern "C" fn udi_buf_free(buf: *mut udi_buf_t)
{
    todo!()
}