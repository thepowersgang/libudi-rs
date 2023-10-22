use super::udi_ubit8_t;

pub const UDI_MAX_ATTR_NAMELEN: usize = 32;
pub const UDI_MAX_ATTR_SIZE: usize = 64;

pub type udi_instance_attr_type_t = udi_ubit8_t;

#[repr(C)]
pub struct udi_instance_attr_list_t
{
    attr_name: [::core::ffi::c_char; UDI_MAX_ATTR_NAMELEN],
    attr_value: [udi_ubit8_t; UDI_MAX_ATTR_SIZE],
    attr_length: udi_ubit8_t,
    attr_type: udi_instance_attr_type_t,
}

#[repr(C)]
pub enum _udi_instance_attr_type_t
{
    UDI_ATTR_NONE,
    UDI_ATTR_STRING,
    UDI_ATTR_ARRAY8,
    UDI_ATTR_UBIT32,
    UDI_ATTR_BOOLEAN,
    UDI_ATTR_FILE
}
pub use _udi_instance_attr_type_t::*;