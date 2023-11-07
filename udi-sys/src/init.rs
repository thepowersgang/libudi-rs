
use super::*;

#[repr(C)]
pub struct udi_init_context_t
{
	pub region_index: udi_index_t,
	pub limits: udi_limits_t,
}
#[repr(C)]
pub struct udi_limits_t
{
	/// Maximum legal ammount of memory that can be allocated
	pub max_legal_alloc: udi_size_t,
	pub max_safe_alloc: udi_size_t,
	pub max_trace_log_formatted_len: udi_size_t,
	pub max_instance_attr_len: udi_size_t,
	/// Minumum time difference (in nanoseconds between unique values returned by `udi_time_current``
	pub min_curtime_res: u32,
	pub min_timer_res: u32,
}

#[repr(C)]
pub struct udi_child_chan_context_t
{
	pub rdata: *mut c_void,
	pub child_id: udi_ubit32_t,
}

#[repr(C)]
pub struct udi_primary_init_t
{
	pub mgmt_ops: &'static super::meta_mgmt::udi_mgmt_ops_t,
	pub mgmt_op_flags:	*const u8,
	pub mgmt_scratch_requirement: super::udi_size_t,
	pub enumeration_attr_list_length: u8,
	pub rdata_size: super::udi_size_t,
	/// Number of bytes for each call to `udi_enumerate_req`
	pub child_data_size: super::udi_size_t,
	/// Number of path handles from each parent
	pub per_parent_paths: u8,
}

#[repr(C)]
pub struct udi_secondary_init_t
{
	pub region_idx: udi_index_t,
	pub rdata_size: udi_size_t,
}
#[repr(C)]
pub struct udi_ops_init_t
{
	pub ops_idx: udi_index_t,
	pub meta_idx: udi_index_t,
	pub meta_ops_num: udi_index_t,
	pub chan_context_size: udi_size_t,
	pub ops_vector: udi_ops_vector_t,
	pub op_flags: *const u8,
}
impl udi_ops_init_t {
	pub const fn end_of_list() -> Self {
		Self {
			ops_idx: udi_index_t(0),	// All that matters.
			meta_idx: udi_index_t(0),
			meta_ops_num: udi_index_t(0),
			chan_context_size: 0,
			ops_vector: ::core::ptr::null(),
			op_flags: ::core::ptr::null(),
		}
	}
}
#[repr(C)]
pub struct udi_cb_init_t
{
	pub cb_idx: udi_index_t,
	pub meta_idx: udi_index_t,
	pub meta_cb_num: udi_index_t,
	pub scratch_requirement: udi_size_t,
	pub inline_size: udi_size_t,
	pub inline_layout: *const udi_layout_t,
}
impl udi_cb_init_t {
	pub const fn end_of_list() -> Self {
		Self {
			cb_idx: udi_index_t(0),	// All that matters.
			meta_idx: udi_index_t(0),
			meta_cb_num: udi_index_t(0),
			scratch_requirement: 0,
			inline_size: 0,
			inline_layout: ::core::ptr::null(),
		}
	}
}

#[repr(C)]
pub struct udi_cb_select_t
{
	pub ops_idx: udi_index_t,
	pub cb_idx: udi_index_t,
}

#[repr(C)]
pub struct udi_gcb_init_t
{
	pub cb_idx: udi_index_t,
	pub scratch_requirement: udi_size_t,
}

#[repr(C)]
pub struct udi_init_t
{
	// Can be NULL for secondary modules
	pub primary_init_info: Option<&'static udi_primary_init_t>,
	// Sequence terminated by `region_idx=0`, can be null
	pub secondary_init_list: *const udi_secondary_init_t,
	pub ops_init_list: *const udi_ops_init_t,
	pub cb_init_list: *const udi_cb_init_t,
	pub gcb_init_list: *const udi_gcb_init_t,
	pub cb_select_list: *const udi_cb_select_t,
}
unsafe impl Sync for udi_init_t {}