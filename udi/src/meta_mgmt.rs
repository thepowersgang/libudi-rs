//! Management metalanguage

/// Shared handle to a `udi_usage_cb_t`
pub type CbRefUsage<'a> = crate::CbRef<'a, crate::ffi::meta_mgmt::udi_usage_cb_t>;

// NOTE: This doesn't use the `MetalangCb` blanket impl becuase the management CBs don't have numbers
unsafe impl crate::async_trickery::GetCb for crate::ffi::meta_mgmt::udi_usage_cb_t {
    fn get_gcb(&self) -> &crate::ffi::udi_cb_t {
        &self.gcb
    }
}
