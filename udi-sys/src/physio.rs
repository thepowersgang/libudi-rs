use super::*;

pub type udi_dma_constraints_t = *mut udi_dma_constraints_s;
pub const UDI_NULL_DMA_CONSTRAINTS: udi_dma_constraints_t = ::core::ptr::null_mut();

pub type udi_dma_constraints_attr_t = udi_ubit8_t;

pub type udi_dma_constraints_attr_set_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_constraints: udi_dma_constraints_t, status: udi_status_t);

extern "C" {
    pub fn udi_dma_constraints_attr_set(
        callback: udi_dma_constraints_attr_set_call_t,
        gcb: *mut udi_cb_t,
        src_constraints: udi_dma_constraints_t,
        attr_list: *const udi_dma_constraints_attr_spec_t,
        list_length: udi_ubit16_t,
        flags: udi_ubit8_t
        );
    /**
     * [udi_dma_constraints_attr_reset] is used to reset a constraints
     * attribute back to its default value (which is also usually the least restrictive).
     * This is usually needed when a particular module provides special handling
     * relative to the constraints attribute such that any restrictions imposed by parent
     * or child drivers are not transferred through this driver.
     */
    pub fn udi_dma_constraints_attr_reset(
        constraints: udi_dma_constraints_t,
        attr_type: udi_dma_constraints_attr_t
        );

    pub type udi_dma_constraints_s;
}

#[repr(C)]
pub struct udi_dma_constraints_attr_spec_t
{
    attr_type: udi_dma_constraints_attr_t,
    attr_value: udi_ubit32_t,
}

/* DMA Convenience Attribute Codes */
pub const UDI_DMA_ADDRESSABLE_BITS  : udi_dma_constraints_attr_t = 100;
pub const UDI_DMA_ALIGNMENT_BITS    : udi_dma_constraints_attr_t = 101;
/* DMA Constraints on the Entire Transfer */
pub const UDI_DMA_DATA_ADDRESSABLE_BITS : udi_dma_constraints_attr_t = 110;
pub const UDI_DMA_NO_PARTIAL            : udi_dma_constraints_attr_t = 111;
/* DMA Constraints on the Scatter/Gather List */
pub const UDI_DMA_SCGTH_MAX_ELEMENTS    : udi_dma_constraints_attr_t = 120;
pub const UDI_DMA_SCGTH_FORMAT          : udi_dma_constraints_attr_t = 121;
pub const UDI_DMA_SCGTH_ENDIANNESS      : udi_dma_constraints_attr_t = 122;
pub const UDI_DMA_SCGTH_ADDRESSABLE_BITS: udi_dma_constraints_attr_t = 123;
pub const UDI_DMA_SCGTH_MAX_SEGMENTS    : udi_dma_constraints_attr_t = 124;
/* DMA Constraints on Scatter/Gather Segments */
pub const UDI_DMA_SCGTH_ALIGNMENT_BITS  : udi_dma_constraints_attr_t = 130;
pub const UDI_DMA_SCGTH_MAX_EL_PER_SEG  : udi_dma_constraints_attr_t = 131;
pub const UDI_DMA_SCGTH_PREFIX_BYTES    : udi_dma_constraints_attr_t = 132;
/* DMA Constraints on Scatter/Gather Elements */
pub const UDI_DMA_ELEMENT_ALIGNMENT_BITS    : udi_dma_constraints_attr_t =  140;
pub const UDI_DMA_ELEMENT_LENGTH_BITS       : udi_dma_constraints_attr_t =  141;
pub const UDI_DMA_ELEMENT_GRANULARITY_BITS  : udi_dma_constraints_attr_t =  142;
/* DMA Constraints for Special Addressing */
pub const UDI_DMA_ADDR_FIXED_BITS    : udi_dma_constraints_attr_t = 150;
pub const UDI_DMA_ADDR_FIXED_TYPE    : udi_dma_constraints_attr_t = 151;
pub const UDI_DMA_ADDR_FIXED_VALUE_LO: udi_dma_constraints_attr_t = 152;
pub const UDI_DMA_ADDR_FIXED_VALUE_HI: udi_dma_constraints_attr_t = 153;
/* DMA Constraints on DMA Access Behavior */
pub const UDI_DMA_SEQUENTIAL        : udi_dma_constraints_attr_t = 160;
pub const UDI_DMA_SLOP_IN_BITS      : udi_dma_constraints_attr_t = 161;
pub const UDI_DMA_SLOP_OUT_BITS     : udi_dma_constraints_attr_t = 162;
pub const UDI_DMA_SLOP_OUT_EXTRA    : udi_dma_constraints_attr_t = 163;
pub const UDI_DMA_SLOP_BARRIER_BITS : udi_dma_constraints_attr_t = 164;

/* Values for UDI_DMA_SCGTH_ENDIANNESS */
pub const UDI_DMA_LITTLE_ENDIAN: udi_ubit32_t = 1<<6;
pub const UDI_DMA_BIG_ENDIAN: udi_ubit32_t = 1<<5;
/* Values for UDI_DMA_ADDR_FIXED_TYPE */
pub const UDI_DMA_FIXED_ELEMENT: udi_ubit32_t = 1;
pub const UDI_DMA_FIXED_LIST   : udi_ubit32_t = 2;
pub const UDI_DMA_FIXED_VALUE  : udi_ubit32_t = 3;