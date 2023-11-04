
pub type udi_layout_t = u8;

/* Specific-Length Layout Type Codes */
pub const UDI_DL_UBIT8_T    : udi_layout_t = 1;
pub const UDI_DL_SBIT8_T    : udi_layout_t = 2;
pub const UDI_DL_UBIT16_T   : udi_layout_t = 3;
pub const UDI_DL_SBIT16_T   : udi_layout_t = 4;
pub const UDI_DL_UBIT32_T   : udi_layout_t = 5;
pub const UDI_DL_SBIT32_T   : udi_layout_t = 6;
pub const UDI_DL_BOOLEAN_T  : udi_layout_t = 7;
pub const UDI_DL_STATUS_T   : udi_layout_t = 8;
/* Abstract Element Layout Type Codes */
pub const UDI_DL_INDEX_T    : udi_layout_t = 20;
/* Opaque Handle Element Layout Type Codes */
pub const UDI_DL_CHANNEL_T  : udi_layout_t = 30;
pub const UDI_DL_ORIGIN_T   : udi_layout_t = 32;
/* Indirect Element Layout Type Codes */
pub const UDI_DL_BUF                : udi_layout_t = 40;
pub const UDI_DL_CB                 : udi_layout_t = 41;
pub const UDI_DL_INLINE_UNTYPED     : udi_layout_t = 42;
pub const UDI_DL_INLINE_DRIVER_TYPED: udi_layout_t = 43;
pub const UDI_DL_MOVABLE_UNTYPED    : udi_layout_t = 44;
/* Nested Element Layout Type Codes */
pub const UDI_DL_INLINE_TYPED   : udi_layout_t = 50;
pub const UDI_DL_MOVABLE_TYPED  : udi_layout_t = 51;
pub const UDI_DL_ARRAY          : udi_layout_t = 52;

pub const UDI_DL_END            : udi_layout_t = 0;

