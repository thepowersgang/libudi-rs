
use super::*;

pub type udi_pio_handle_t = *mut udi_pio_handle_s;
pub type udi_pio_map_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_pio_handle: udi_pio_handle_t);
pub type udi_pio_trans_call_t = unsafe extern "C" fn(gcb: *mut udi_cb_t, new_buf: *mut udi_buf_t, status: udi_status_t, result: u16);


extern "C" {
	pub fn udi_pio_map(
		callback: udi_pio_map_call_t,
		gcb: *mut udi_cb_t,
        regset_idx: u32, base_offset: u32, length: u32,
		trans_list: *const udi_pio_trans_t, list_length: u16,
        pio_attributes: u16, pace: u32, serialization_domain: udi_index_t
		);
	pub fn udi_pio_unmap(pio_handle: udi_pio_handle_t);
	pub fn udi_pio_atmic_sizes(pio_handle: udi_pio_handle_t) -> u32;
	pub fn udi_pio_abort_sequence(pio_handle: udi_pio_handle_t, scratch_requirement: udi_size_t);
	pub fn udi_pio_trans(callback: udi_pio_trans_call_t, gcb: *mut udi_cb_t,
		pio_handle: udi_pio_handle_t, start_label: udi_index_t, buf: *mut udi_buf_t, mem_ptr: *mut c_void
		);
}

#[repr(C)]
pub struct udi_pio_handle_s([u8;0]);

#[repr(C)]
#[derive(Copy,Clone)]
pub struct udi_pio_trans_t
{
	pub pio_op: u8,
	pub tran_size: u8,
	pub operand: u16,
}

pub const UDI_PIO_DIRECT : u8 = 0x00;
pub const UDI_PIO_SCRATCH: u8 = 0x08;
pub const UDI_PIO_BUF    : u8 = 0x10;
pub const UDI_PIO_MEM    : u8 = 0x18;
// Values for `tran_size`
pub const UDI_PIO_1BYTE: u8 = 0;
pub const UDI_PIO_2BYTE: u8 = 1;

// Values for `pio_attributes`
pub const UDI_PIO_STRICTORDER    : u16 = 1<<0;
pub const UDI_PIO_UNORDERED_OK   : u16 = 1<<1;
pub const UDI_PIO_MERGING_OK     : u16 = 1<<2;
pub const UDI_PIO_LOADCACHING_OK : u16 = 1<<3;
pub const UDI_PIO_STORECACHING_OK: u16 = 1<<4;
pub const UDI_PIO_BIG_ENDIAN     : u16 = 1<<5;
pub const UDI_PIO_LITTLE_ENDIAN  : u16 = 1<<6;
pub const UDI_PIO_NEVERSWAP      : u16 = 1<<7;
pub const UDI_PIO_UNALIGNED      : u16 = 1<<8;
