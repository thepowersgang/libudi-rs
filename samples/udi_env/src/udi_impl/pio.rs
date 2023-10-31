use ::udi::ffi::pio::udi_pio_handle_t;
use ::udi::ffi::pio::udi_pio_map_call_t;
use ::udi::ffi::pio::udi_pio_trans_call_t;
use ::udi::ffi::pio::udi_pio_trans_t;
use ::udi::ffi::udi_buf_t;
use ::udi::ffi::udi_cb_t;
use ::udi::ffi::udi_size_t;
use ::udi::ffi::udi_index_t;
use ::udi::ffi::c_void;



#[no_mangle]
unsafe extern "C" fn udi_pio_map(
    callback: udi_pio_map_call_t,
    gcb: *mut udi_cb_t,
    regset_idx: u32, base_offset: u32, length: u32,
    trans_list: *const udi_pio_trans_t, list_length: u16,
    pio_attributes: u16, pace: u32, serialization_domain: udi_index_t
    )
{
    let trans_list = ::core::slice::from_raw_parts(trans_list, list_length as usize);
    todo!("udi_pio_map");
}
#[no_mangle]
unsafe extern "C" fn udi_pio_unmap(pio_handle: udi_pio_handle_t)
{
    todo!();
}
#[no_mangle]
unsafe extern "C" fn udi_pio_atmic_sizes(pio_handle: udi_pio_handle_t) -> u32
{
    todo!();
}
#[no_mangle]
unsafe extern "C" fn udi_pio_abort_sequence(pio_handle: udi_pio_handle_t, scratch_requirement: udi_size_t)
{
    todo!();
}

#[no_mangle]
unsafe extern "C" fn udi_pio_trans(
    callback: udi_pio_trans_call_t, gcb: *mut udi_cb_t,
    pio_handle: udi_pio_handle_t,
    start_label: udi_index_t,
    buf: *mut udi_buf_t,
    mem_ptr: *mut c_void
    )
{
    todo!("udi_pio_trans");
}