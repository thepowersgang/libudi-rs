
use ::udi_sys::physio::udi_dma_constraints_attr_spec_t;
use ::udi_sys::physio::udi_dma_constraints_t;

#[derive(Debug)]
pub struct DmaConstraints(udi_dma_constraints_t);
impl Drop for DmaConstraints
{
    fn drop(&mut self) {
        unsafe {
            ::udi_sys::physio::udi_dma_constraints_free(self.0)
        }
    }
}
impl Default for DmaConstraints {
    fn default() -> Self {
        Self::null()
    }
}
impl DmaConstraints
{
    pub fn null() -> DmaConstraints {
        DmaConstraints(::udi_sys::physio::UDI_NULL_DMA_CONSTRAINTS)
    }
    pub unsafe fn from_raw(v: udi_dma_constraints_t) -> Self {
        DmaConstraints(v)
    }

    /// Reset the specifided attribute to its default (usually the least restrictive)
    pub fn reset(&mut self, attr_type: ::udi_sys::physio::udi_dma_constraints_attr_t)
    {
        unsafe {
            ::udi_sys::physio::udi_dma_constraints_attr_reset(self.0, attr_type)
        }
    }

    /// Set a collection of attributes
    pub fn set<'a>(
        &'a mut self,
        gcb: crate::cb::CbRef<::udi_sys::udi_cb_t>,
        attrs: &'a [udi_dma_constraints_attr_spec_t]
    ) -> impl ::core::future::Future<Output=crate::Result<()>> + 'a
    {
        unsafe extern "C" fn callback(gcb: *mut ::udi_sys::udi_cb_t, new_ptr: udi_dma_constraints_t, status: ::udi_sys::udi_status_t) {
            let res = crate::Error::from_status(status).map(|()| new_ptr as _);
            crate::async_trickery::signal_waiter(&mut *gcb, crate::async_trickery::WaitRes::PointerResult(res))
        }
        let src_constraints = self.0;
        crate::async_trickery::wait_task(
            gcb,
            move |gcb| unsafe {
                ::udi_sys::physio::udi_dma_constraints_attr_set(
                    callback, gcb, src_constraints, attrs.as_ptr(), attrs.len() as _, 0
                )
            },
            move |res|
                match res {
                crate::async_trickery::WaitRes::PointerResult(v) => match v
                    {
                    Ok(p) => { self.0 = p as _; Ok(())},
                    Err(e) => Err(e),
                    },
                _ => panic!(),
                }
            )
    }
}