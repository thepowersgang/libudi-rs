use super::{udi_size_t, udi_buf_t, udi_cb_t};


pub type udi_tagtype_t = super::udi_ubit32_t;
extern "C" {
    pub type udi_buf_path_s;
}
pub type udi_buf_path_t = *mut udi_buf_path_s;
pub const UDI_NULL_PATH_BUF: udi_buf_path_t = ::core::ptr::null_mut();

pub type udi_buf_copy_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_dst_buf: *mut udi_buf_t);
pub type udi_buf_write_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_dst_buf: *mut udi_buf_t);
pub type udi_buf_path_alloc_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_buf_path: udi_buf_path_t);
pub type udi_buf_tag_set_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_buf: *mut udi_buf_t);
pub type udi_buf_tag_set_apply_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_buf: *mut udi_buf_t);

#[repr(C)]
#[derive(Clone, Copy)]
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
    pub fn udi_buf_read(
        src_buf: *mut udi_buf_t,
        src_off: udi_size_t,
        src_len: udi_size_t,
        dst_mem: *mut crate::c_void,
    );
    pub fn udi_buf_free(buf: *mut udi_buf_t);


    pub fn udi_buf_best_path(
        buf: *mut udi_buf_t,
        path_handles: *mut udi_buf_path_t,
        npaths: crate::udi_ubit8_t,
        last_fit: crate::udi_ubit8_t,
        best_fit_array: *mut crate::udi_ubit8_t
    );
    pub fn udi_buf_path_alloc(
        callback: udi_buf_path_alloc_call_t,
        gcb: *mut udi_cb_t,
    );
    pub fn udi_buf_path_free(buf_path: udi_buf_path_t);


    pub fn udi_buf_tag_set(
        callback: udi_buf_tag_set_call_t,
        gcb: *mut udi_cb_t,
        buf: *mut udi_buf_t,
        tag_array: *mut udi_buf_tag_t,
        tag_array_length: crate::udi_ubit16_t,
    );
    pub fn udi_buf_tag_get(
        buf: *mut udi_buf_t,
        tag_type: udi_tagtype_t,
        tag_array: *mut udi_buf_tag_t,
        tag_array_length: crate::udi_ubit16_t,
        tag_start_idx: crate::udi_ubit16_t,
    ) -> crate::udi_ubit16_t;
    pub fn udi_buf_tag_compute(
        buf: *mut udi_buf_t,
        off: udi_size_t,
        len: udi_size_t,
        tag_type: udi_tagtype_t,
    ) -> crate::udi_ubit32_t;
    pub fn udi_buf_tag_apply(
        callback: udi_buf_tag_set_apply_t,
        gcb: *mut udi_cb_t,
        buf: *mut udi_buf_t,
        tag_type: udi_tagtype_t,
    );
}
pub const UDI_BUF_PATH_END: crate::udi_ubit8_t = 255;

/* Tag Category Masks */
pub const UDI_BUFTAG_ALL    : udi_tagtype_t = 0xffffffff;
pub const UDI_BUFTAG_VALUES : udi_tagtype_t = 0x000000ff;
pub const UDI_BUFTAG_UPDATES: udi_tagtype_t = 0x0000ff00;
pub const UDI_BUFTAG_STATUS : udi_tagtype_t = 0x00ff0000;
pub const UDI_BUFTAG_DRIVERS: udi_tagtype_t = 0xff000000;

/* Value Category Tag Types */
pub const UDI_BUFTAG_BE16_CHECKSUM: udi_tagtype_t = 1<<0;
/* Update Category Tag Types */
#[allow(non_upper_case_globals)]
pub const UDI_BUFTAG_SET_iBE16_CHECKSUM: udi_tagtype_t = 1<<8;
pub const UDI_BUFTAG_SET_TCP_CHECKSUM  : udi_tagtype_t = 1<<9;
pub const UDI_BUFTAG_SET_UDP_CHECKSUM  : udi_tagtype_t = 1<<10;
/* Status Category Tag Types */
pub const UDI_BUFTAG_TCP_CKSUM_GOOD : udi_tagtype_t = 1<<17;
pub const UDI_BUFTAG_UDP_CKSUM_GOOD : udi_tagtype_t = 1<<18;
pub const UDI_BUFTAG_IP_CKSUM_GOOD  : udi_tagtype_t = 1<<19;
pub const UDI_BUFTAG_TCP_CKSUM_BAD  : udi_tagtype_t = 1<<21;
pub const UDI_BUFTAG_UDP_CKSUM_BAD  : udi_tagtype_t = 1<<22;
pub const UDI_BUFTAG_IP_CKSUM_BAD   : udi_tagtype_t = 1<<23;
/* Drivers Category Tag Types */
pub const UDI_BUFTAG_DRIVER1: udi_tagtype_t = 1<<24;
pub const UDI_BUFTAG_DRIVER2: udi_tagtype_t = 1<<25;
pub const UDI_BUFTAG_DRIVER3: udi_tagtype_t = 1<<26;
pub const UDI_BUFTAG_DRIVER4: udi_tagtype_t = 1<<27;
pub const UDI_BUFTAG_DRIVER5: udi_tagtype_t = 1<<28;
pub const UDI_BUFTAG_DRIVER6: udi_tagtype_t = 1<<29;
pub const UDI_BUFTAG_DRIVER7: udi_tagtype_t = 1<<30;
pub const UDI_BUFTAG_DRIVER8: udi_tagtype_t = 1<<31;
