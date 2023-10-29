//! Management metalanguage

pub type CbRefUsage<'a> = crate::CbRef<'a, crate::ffi::meta_mgmt::udi_usage_cb_t>;

// NOTE: This doesn't use the `MetalangCb` blanket impl becuase the management CBs don't have numbers
unsafe impl crate::async_trickery::GetCb for crate::ffi::meta_mgmt::udi_usage_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}

pub fn get_usage_cb_trace_mask() -> impl ::core::future::Future<Output=crate::ffi::log::udi_trevent_t> {
	super::async_trickery::with_cb::<crate::ffi::meta_mgmt::udi_usage_cb_t,_,_>(|cb| cb.trace_mask)
}
pub fn get_usage_cb_meta_idx() -> impl ::core::future::Future<Output=crate::ffi::udi_index_t> {
	super::async_trickery::with_cb::<crate::ffi::meta_mgmt::udi_usage_cb_t,_,_>(|cb| cb.meta_idx)
}
