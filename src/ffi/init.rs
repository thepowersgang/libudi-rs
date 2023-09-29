
pub struct udi_primary_init_t
{
	pub mgmt_ops:	&'static super::meta_mgmt::udi_mgmt_ops_t,
	pub mgmt_op_flags:	*const u8,
	pub mgmt_scratch_requirement: super::udi_size_t,
	pub enumeration_attr_list_length: u8,
	pub rdata_size: super::udi_size_t,
	pub child_data_size: super::udi_size_t,
	pub per_parent_paths: u8,
}
