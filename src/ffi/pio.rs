
use super::*;

pub type udi_pio_handle_t = *mut udi_pio_handle_s;
pub type udi_pio_map_call_t = extern "C" fn(gcb: *mut udi_cb_t, new_pio_handle: udi_pio_handle_t);

extern "C" {
	pub fn udi_pio_map(
		callback: udi_pio_map_call_t,
		gcb: *mut udi_cb_t,
        regset_idx: u32, base_offset: u32, length: u32,
		trans_list: *const udi_pio_trans_t, list_length: u16,
        pio_attributes: u16, pace: u32, serialization_domain: udi_index_t
		);
}

#[repr(C)]
pub struct udi_pio_handle_s([u8;0]);

#[repr(C)]
pub struct udi_pio_trans_t
{
	pub pio_op: u8,
	pub tran_size: u8,
	pub operand: u16,
}

