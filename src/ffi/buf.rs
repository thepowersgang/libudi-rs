use super::{udi_size_t, udi_buf_t, udi_cb_t};


pub type udi_tagtype_t = super::udi_ubit32_t;
pub type udi_buf_path_t = super::_udi_handle_t;
pub const UDI_NULL_PATH_BUF: udi_buf_path_t = ::core::ptr::null_mut();

pub type udi_buf_copy_call_t = extern "C" fn(gcb: *mut udi_cb_t, new_dst_buf: *mut udi_buf_t);
pub type udi_buf_write_call_t = extern "C" fn(gcb: *mut udi_cb_t, new_dst_buf: *mut udi_buf_t);

#[repr(C)]
pub struct udi_buf_tag_t
{
    pub tag_type: udi_tagtype_t,
    pub tag_value: super::udi_ubit32_t,
    pub tag_off: super::udi_size_t,
    pub tag_len: super::udi_size_t,
}

#[repr(C)]
pub struct udi_xfer_constraints_t
{
    pub udi_xfer_max: super::udi_ubit32_t,
    pub udi_xfer_typical: super::udi_ubit32_t,
    pub udi_xfer_granularity: super::udi_ubit32_t, 
    pub udi_xfer_one_piece: super::udi_boolean_t,
    pub udi_xfer_exact_size: super::udi_boolean_t,
    pub udi_xfer_no_reorder: super::udi_boolean_t,
}

#[allow(non_snake_case)]
pub unsafe fn UDI_BUF_ALLOC(
    callback: udi_buf_copy_call_t,
    gcb: *mut super::udi_cb_t,
    src_buf: *const super::c_void,
    src_len: udi_size_t,
    path_handle: udi_buf_path_t
) {
    udi_buf_write(callback, gcb, src_buf, src_len, ::core::ptr::null_mut(), 0, 0, path_handle)
}

extern "C" {
    pub fn udi_buf_copy(
        callback: udi_buf_copy_call_t,
        gcb: *mut super::udi_cb_t,
        src_buf: *mut super::udi_buf_t,
        src_off: udi_size_t,
        src_len: udi_size_t,
        dst_buf: *mut udi_buf_t,
        dst_off: udi_size_t,
        dst_len: udi_size_t,
        path_handle: udi_buf_path_t
    );
    pub fn udi_buf_write(
        callback: udi_buf_copy_call_t,
        gcb: *mut super::udi_cb_t,
        src_buf: *const super::c_void,
        src_len: udi_size_t,
        dst_buf: *mut udi_buf_t,
        dst_off: udi_size_t,
        dst_len: udi_size_t,
        path_handle: udi_buf_path_t
    );
}