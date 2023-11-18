use super::*;

// NOTE: IA-ABI doesn't specify this, so just assume u64 is sensible
pub type udi_busaddr64_t = u64;

pub type udi_dma_handle_t = *mut udi_dma_handle_s;
pub type udi_dma_constraints_t = *mut udi_dma_constraints_s;
pub const UDI_NULL_DMA_CONSTRAINTS: udi_dma_constraints_t = ::core::ptr::null_mut();

pub type udi_dma_constraints_attr_t = udi_ubit8_t;

pub type udi_dma_constraints_attr_set_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_constraints: udi_dma_constraints_t, status: udi_status_t);
pub type udi_dma_prepare_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_dma_handle: udi_dma_handle_t);
pub type udi_dma_buf_map_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, scgth: *mut udi_scgth_t, complete: udi_boolean_t, status: udi_status_t);
pub type udi_dma_mem_alloc_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, 
    new_dma_handle: udi_dma_handle_t,
    mem_ptr: *mut c_void,
    actual_gap: udi_size_t,
    single_element: udi_boolean_t,
    scgth: *mut udi_scgth_t,
    must_swap: udi_boolean_t
);
pub type udi_dma_sync_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t);
pub type udi_dma_scgth_sync_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t);
pub type udi_dma_mem_to_buf_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_dst_buf: *mut udi_buf_t);

extern "C" {
    pub type udi_dma_constraints_s;
    pub type udi_dma_handle_s;

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
    pub fn udi_dma_constraints_free(constraints: udi_dma_constraints_t);

    pub fn udi_dma_limits(dma_limits: *mut udi_dma_limits_t);

    pub fn udi_dma_prepare(
        callback: udi_dma_prepare_call_t,
        gcb: *mut udi_cb_t,
        constraints: udi_dma_constraints_t,
        flags: udi_ubit8_t
    );
    pub fn udi_dma_buf_map(
        callback: udi_dma_buf_map_call_t,
        gcb: *mut udi_cb_t,
        dma_handle: udi_dma_handle_t,
        buf: *mut udi_buf_t,
        offset: udi_size_t,
        len: udi_size_t,
        flags: udi_ubit8_t
    );
    pub fn udi_dma_buf_unmap(
        dma_handle: udi_dma_handle_t,
        new_buf_size: udi_size_t
    ) -> *mut udi_buf_t;
    pub fn udi_dma_mem_alloc(
        callback: udi_dma_mem_alloc_call_t,
        gcb: *mut udi_cb_t,
        constraints: udi_dma_constraints_t,
        flags: udi_ubit8_t,
        nelements: udi_ubit16_t,
        element_size: udi_size_t,
        max_gap: udi_size_t
    );
    pub fn udi_dma_sync (
        callback: udi_dma_sync_call_t,
        gcb: *mut udi_cb_t,
        dma_handle: udi_dma_handle_t,
        offset: udi_size_t,
        length: udi_size_t,
        flags: udi_ubit8_t
    );
    pub fn udi_dma_scgth_sync(
        callback: udi_dma_scgth_sync_call_t,
        gcb: *mut udi_cb_t,
        dma_handle: udi_dma_handle_t
    );

    pub fn udi_dma_mem_barrier(dma_handle: udi_dma_handle_t);
    pub fn udi_dma_free(dma_handle: udi_dma_handle_t);
    pub fn udi_dma_mem_to_buf(
        callback: udi_dma_mem_to_buf_call_t,
        gcb: *mut udi_cb_t,
        dma_handle: udi_dma_handle_t,
        src_off: udi_size_t,
        src_len: udi_size_t,
        dst_buf: *mut udi_buf_t
    );
}

#[repr(C)]
#[derive(Copy,Clone)]
pub struct udi_dma_constraints_attr_spec_t
{
    pub attr_type: udi_dma_constraints_attr_t,
    pub attr_value: udi_ubit32_t,
}

/// [udi_dma_constraints_attr_set] Make a copy of src_constraints before applying the new attributes.
pub const UDI_DMA_CONSTRAINTS_COPY : udi_ubit8_t = 1 << 0;
/* DMA Convenience Attribute Codes */
pub const UDI_DMA_ADDRESSABLE_BITS  : udi_dma_constraints_attr_t = 100;
pub const UDI_DMA_ALIGNMENT_BITS    : udi_dma_constraints_attr_t = 101;
/* DMA Constraints on the Entire Transfer */
pub const UDI_DMA_DATA_ADDRESSABLE_BITS : udi_dma_constraints_attr_t = 110;
pub const UDI_DMA_NO_PARTIAL            : udi_dma_constraints_attr_t = 111;
/* DMA Constraints on the Scatter/Gather List */
/// The maximum # of elements that can be handled in one scatter/gather list.
/// For DMA engines without scatter/gather support, this should be set to 1.
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
//pub const UDI_DMA_LITTLE_ENDIAN: udi_ubit32_t = 1<<6;
//pub const UDI_DMA_BIG_ENDIAN: udi_ubit32_t = 1<<5;
/* Values for UDI_DMA_ADDR_FIXED_TYPE */
pub const UDI_DMA_FIXED_ELEMENT: udi_ubit32_t = 1;
pub const UDI_DMA_FIXED_LIST   : udi_ubit32_t = 2;
pub const UDI_DMA_FIXED_VALUE  : udi_ubit32_t = 3;

#[repr(C)]
pub struct udi_dma_limits_t
{
    pub max_legal_contig_alloc: udi_size_t,
    pub max_safe_contig_alloc: udi_size_t,
    pub cache_line_size: udi_size_t,
}
pub const UDI_DMA_MIN_ALLOC_LIMIT: udi_size_t = 4000;

#[repr(C)]
#[derive(Clone, Copy)]  // For union
pub struct udi_scgth_element_32_t
{
    pub block_busaddr: udi_ubit32_t,
    pub block_length: udi_ubit32_t,
}
#[repr(C)]
#[derive(Clone, Copy)]  // For union
pub struct udi_scgth_element_64_t
{
    pub block_busaddr: udi_busaddr64_t,
    pub block_length: udi_ubit32_t,
    pub el_reserved: udi_ubit32_t,
}
    /* Extension Flag */
pub const UDI_SCGTH_EXT: udi_ubit32_t = 0x80000000;
#[repr(C)]
pub struct udi_scgth_t
{
    pub scgth_num_elements: udi_ubit16_t,
    pub scgth_format: udi_ubit8_t,
    pub scgth_must_swap: udi_boolean_t,
    pub scgth_elements: udi_scgth_t_scgth_elements,
    pub scgth_first_segment: udi_scgth_t_scgth_first_segment,
}
#[repr(C)]
pub union udi_scgth_t_scgth_elements {
    pub el32p: *mut udi_scgth_element_32_t,
    pub el64p: *mut udi_scgth_element_64_t,
}
#[repr(C)]
pub union udi_scgth_t_scgth_first_segment {
    pub el32: udi_scgth_element_32_t,
    pub el64: udi_scgth_element_64_t,
}
/* Values for scgth_format */
pub const UDI_SCGTH_32           : udi_ubit8_t = 1<<0;
pub const UDI_SCGTH_64           : udi_ubit8_t = 1<<1;
pub const UDI_SCGTH_DMA_MAPPED   : udi_ubit8_t = 1<<6;
pub const UDI_SCGTH_DRIVER_MAPPED: udi_ubit8_t = 1<<7;

/* Values for [udi_dma_*] flags */
pub const UDI_DMA_OUT: udi_ubit8_t = 1<<2;
pub const UDI_DMA_IN : udi_ubit8_t = 1<<3;
pub const UDI_DMA_REWIND : udi_ubit8_t = 1<<4;
pub const UDI_DMA_BIG_ENDIAN : udi_ubit8_t = 1<<5;
pub const UDI_DMA_LITTLE_ENDIAN : udi_ubit8_t = 1<<6;
pub const UDI_DMA_NEVERSWAP : udi_ubit8_t = 1<<7;

/// PIO Handle Layout Element Type Code
pub const UDI_DL_PIO_HANDLE_T: udi_ubit8_t = 200;
/// DMA Constraints Handle Layout Element Type Code
pub const UDI_DL_DMA_CONSTRAINTS_T: udi_ubit8_t = 201;