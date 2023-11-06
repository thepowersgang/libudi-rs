use super::udi_ubit8_t;

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
