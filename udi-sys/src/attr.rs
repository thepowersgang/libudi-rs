use super::*;

pub const UDI_MAX_ATTR_NAMELEN: usize = 32;
pub const UDI_MAX_ATTR_SIZE: usize = 64;

pub type udi_instance_attr_type_t = udi_ubit8_t;

#[repr(C)]
#[derive(Copy,Clone)]
pub struct udi_instance_attr_list_t
{
    pub attr_name: [u8; UDI_MAX_ATTR_NAMELEN],
    pub attr_value: [udi_ubit8_t; UDI_MAX_ATTR_SIZE],
    pub attr_length: udi_ubit8_t,
    pub attr_type: udi_instance_attr_type_t,
}

pub const UDI_ATTR_NONE   : udi_instance_attr_type_t = 0;
pub const UDI_ATTR_STRING : udi_instance_attr_type_t = 1;
pub const UDI_ATTR_ARRAY8 : udi_instance_attr_type_t = 2;
pub const UDI_ATTR_UBIT32 : udi_instance_attr_type_t = 3;
pub const UDI_ATTR_BOOLEAN: udi_instance_attr_type_t = 4;
pub const UDI_ATTR_FILE   : udi_instance_attr_type_t = 5;

pub type udi_instance_attr_get_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, attr_type: udi_instance_attr_type_t, actual_length: udi_size_t);
pub type udi_instance_attr_set_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, status: udi_status_t);
extern "C" {
    fn udi_instance_attr_get(
        callback: udi_instance_attr_get_call_t,
        gcb: *mut udi_cb_t,
        attr_name: *const ::core::ffi::c_char,
        child_ID: udi_ubit32_t,
        attr_value: *mut c_void,
        attr_length: udi_size_t
    );
    pub fn udi_instance_attr_set(
        callback: udi_instance_attr_set_call_t,
        gcb: *mut udi_cb_t,
        attr_name: *const ::core::ffi::c_char,
        child_ID: udi_ubit32_t,
        attr_value: *const c_void,
        attr_length: udi_size_t,
        attr_type: udi_ubit8_t
    );
}

#[allow(non_snake_case)]
pub unsafe fn UDI_INSTANCE_ATTR_DELETE(callback: udi_instance_attr_set_call_t, gcb: *mut udi_cb_t, attr_name: *const ::core::ffi::c_char) {
    udi_instance_attr_set(callback, gcb, attr_name, 0, ::core::ptr::null(), 0, UDI_ATTR_NONE);
}