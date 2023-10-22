
pub type CbRefUsage<'a> = crate::CbRef<'a, crate::ffi::meta_mgmt::udi_usage_cb_t>;

pub fn get_usage_cb_trace_mask() -> impl ::core::future::Future<Output=crate::ffi::log::udi_trevent_t> {
	super::async_trickery::with_cb::<crate::ffi::meta_mgmt::udi_usage_cb_t,_,_>(|cb| cb.trace_mask)
}
pub fn get_usage_cb_meta_idx() -> impl ::core::future::Future<Output=crate::ffi::udi_index_t> {
	super::async_trickery::with_cb::<crate::ffi::meta_mgmt::udi_usage_cb_t,_,_>(|cb| cb.meta_idx)
}
