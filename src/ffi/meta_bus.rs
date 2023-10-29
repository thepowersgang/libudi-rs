use crate::ffi::*;

extern "C" {
    pub fn udi_bus_bind_req(cb: *mut udi_bus_bind_cb_t);
    /**
     * The [udi_bus_bind_ack] operation is used by a bridge driver to
     * acknowledge binding with a child device driver (or failure to do so, as
     * indicated by status), as requested by a [udi_bus_bind_req] operation.
     * When a bind is acknowledged with this operation, the bridge driver must be
     * prepared for DMA, PIO, or interrupt registration operations to be performed
     * to the associated device and for the device to begin generating interrupts.
     * 
     * Some devices are bi-endian; that is, they can be placed in either a little-endian
     * mode or a big-endian mode. `preferred_endianness` provides a hint to
     * drivers for such devices, as to which endianness is likely to be most efficient.
     * If this is set to [UDI_DMA_ANY_ENDIAN], at least one interposed bridge is
     * bi-endian, so either endianness can be supported without significant additional
     * cost (i.e. without software byte swapping).
     * 
     * Drivers for fixed-endianness devices can ignore `preferred_endianness`.
     * 
     * - `dma_constraints` specifies the DMA constraints requirements of the bus
     *   bridge. The child driver must apply its own specific constraints
     *   attributes to this constraints object (using
     *   [physio::udi_dma_constraints_attr_set]) before using it for its
     *   own DMA mappings.
     * - `preferred_endianness` indicates the device endianness which works
     *   most effectively with the bridges in this path. It may be set to one
     *   of the following values:
     *   - [UDI_DMA_LITTLE_ENDIAN]
     *   - [UDI_DMA_BIG_ENDIAN]
     *   - [UDI_DMA_ANY_ENDIAN]
     */
    pub fn udi_bus_bind_ack(
        cb: *mut udi_bus_bind_cb_t,
        dma_constraints: physio::udi_dma_constraints_t,
        preferred_endianness: udi_ubit8_t,
        status: udi_status_t
    );
    pub fn udi_bus_unbind_req(cb: *mut udi_bus_bind_cb_t);
    pub fn udi_bus_unbind_ack(cb: *mut udi_bus_bind_cb_t);
}

impl_metalanguage!{
    static METALANG_SPEC;
    NAME udi_bridge;
    OPS
        1 => udi_bus_device_ops_t,
        2 => udi_bus_bridge_ops_t,
        3 => super::meta_intr::udi_intr_handler_ops_t,
        4 => super::meta_intr::udi_intr_dispatcher_ops_t,
        ;
    CBS
        1 => udi_bus_bind_cb_t,
        2 => super::meta_intr::udi_intr_attach_cb_t,
        3 => super::meta_intr::udi_intr_detach_cb_t,
        4 => super::meta_intr::udi_intr_event_cb_t,
        ;
}

pub struct udi_bus_device_ops_t
{
    pub channel_event_ind_op: imc::udi_channel_event_ind_op_t,
    pub bus_bind_ack_op: unsafe extern "C" fn(*mut udi_bus_bind_cb_t, physio::udi_dma_constraints_t, u8, udi_status_t),
    pub bus_unbind_ack_op: unsafe extern "C" fn(*mut udi_bus_bind_cb_t),
    pub intr_attach_ack_op: meta_intr::udi_intr_attach_ack_op_t,
    pub intr_detach_ack_op: meta_intr::udi_intr_detach_ack_op_t,
}

pub struct udi_bus_bridge_ops_t
{
    pub channel_event_ind_op: imc::udi_channel_event_ind_op_t,
    pub bus_bind_req_op: unsafe extern "C" fn(*mut udi_bus_bind_cb_t),
    pub bus_unbind_req_op: unsafe extern "C" fn(*mut udi_bus_bind_cb_t),
    pub intr_attach_req_op: unsafe extern "C" fn(*mut meta_intr::udi_intr_attach_cb_t),
    pub intr_detach_req_op: unsafe extern "C" fn(*mut meta_intr::udi_intr_detach_cb_t),
}

#[repr(C)]
pub struct udi_bus_bind_cb_t
{
    pub gcb: udi_cb_t,
}

pub const UDI_DMA_BIG_ENDIAN: u8 = 1<<5;
pub const UDI_DMA_LITTLE_ENDIAN: u8 = 1<<6;
pub const UDI_DMA_ANY_ENDIAN: u8 = 1<<0;