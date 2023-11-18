/* TODO: This doesn't work properly, seems to not be dynamic linking, at least in `create_so_rust.sh` */

udi_strcpy = 0;
udi_strncpy_rtrim = 0;

udi_debug_printf = 0;
udi_assert = 0;

udi_devmgmt_ack = 0;
udi_final_cleanup_ack = 0;

udi_channel_spawn = 0;
udi_channel_event_complete = 0;

udi_buf_write = 0;
udi_buf_copy = 0;
udi_buf_read = 0;
udi_buf_free = 0;

udi_usage_res = 0;
udi_enumerate_ack = 0;

udi_cb_alloc = 0;
udi_cb_alloc_batch = 0;

udi_pio_map = 0;
udi_pio_trans = 0;
udi_bus_bind_req = 0;
udi_intr_attach_req = 0;
udi_intr_event_rdy = 0;

udi_gio_bind_ack = 0;
udi_gio_unbind_ack = 0;
udi_gio_xfer_ack = 0;
udi_gio_xfer_nak = 0;
udi_gio_event_ind = 0;

udi_dma_constraints_attr_set = 0;
udi_dma_constraints_attr_reset = 0;
udi_dma_constraints_free = 0;
udi_dma_limits = 0;
udi_dma_prepare = 0;
udi_dma_buf_map = 0;
udi_dma_buf_unmap = 0;
udi_dma_mem_alloc = 0;
udi_dma_sync = 0;
udi_dma_scgth_sync = 0;
udi_dma_mem_barrier = 0;
udi_dma_free = 0;
udi_dma_mem_to_buf = 0;

udi_nsr_bind_ack = 0;
udi_nsr_enable_ack = 0;
udi_nsr_rx_ind = 0;
udi_nsr_tx_rdy = 0;

memcpy = 0;
memmove = 0;
memset = 0;
strlen = 0;
bcmp = 0;
_Unwind_Resume = 0;