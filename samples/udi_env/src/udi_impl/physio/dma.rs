use ::udi::ffi::physio as ffi;
use ::udi::ffi::{udi_ubit8_t,udi_ubit16_t,udi_size_t};
use ::udi::ffi::{udi_cb_t,udi_buf_t};

#[no_mangle]
unsafe extern "C" fn udi_dma_prepare(
    callback: ffi::udi_dma_prepare_call_t,
    gcb: *mut udi_cb_t,
    constraints: ffi::udi_dma_constraints_t,
    flags: udi_ubit8_t
)
{
    todo!();
}

#[no_mangle]
unsafe extern "C" fn udi_dma_buf_map(
    callback: ffi::udi_dma_buf_map_call_t,
    gcb: *mut udi_cb_t,
    dma_handle: ffi::udi_dma_handle_t,
    buf: *mut udi_buf_t,
    offset: udi_size_t,
    len: udi_size_t,
    flags: udi_ubit8_t
)
{
    todo!();
}

#[no_mangle]
unsafe extern "C" fn udi_dma_buf_unmap(
    dma_handle: ffi::udi_dma_handle_t,
    new_buf_size: udi_size_t
) -> *mut udi_buf_t
{
    todo!();
}

#[no_mangle]
unsafe extern "C" fn udi_dma_mem_alloc(
    callback: ffi::udi_dma_mem_alloc_call_t,
    gcb: *mut udi_cb_t,
    constraints: ffi::udi_dma_constraints_t,
    flags: udi_ubit8_t,
    nelements: udi_ubit16_t,
    element_size: udi_size_t,
    max_gap: udi_size_t
)
{
    todo!();
}

#[no_mangle]
unsafe extern "C" fn udi_dma_sync (
    callback: ffi::udi_dma_sync_call_t,
    gcb: *mut udi_cb_t,
    dma_handle: ffi::udi_dma_handle_t,
    offset: udi_size_t,
    length: udi_size_t,
    flags: udi_ubit8_t
)
{
    todo!();
}

#[no_mangle]
unsafe extern "C" fn udi_dma_scgth_sync(
    callback: ffi::udi_dma_scgth_sync_call_t,
    gcb: *mut udi_cb_t,
    dma_handle: ffi::udi_dma_handle_t
)
{
    todo!();
}


#[no_mangle]
unsafe extern "C" fn udi_dma_mem_barrier(_dma_handle: ffi::udi_dma_handle_t)
{
}

#[no_mangle]
unsafe extern "C" fn udi_dma_free(dma_handle: ffi::udi_dma_handle_t)
{
    todo!();
}

#[no_mangle]
unsafe extern "C" fn udi_dma_mem_to_buf(
    callback: ffi::udi_dma_mem_to_buf_call_t,
    gcb: *mut udi_cb_t,
    dma_handle: ffi::udi_dma_handle_t,
    src_off: udi_size_t,
    src_len: udi_size_t,
    dst_buf: *mut udi_buf_t
)
{
    todo!();
}