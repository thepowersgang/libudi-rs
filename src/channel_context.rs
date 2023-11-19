

/// Channel context for child bind channels
#[repr(C)]
#[fundamental]
pub struct ChildBind<Driver,ChildData>
{
	pd: ::core::marker::PhantomData<Driver>,
	/// Required inner for child channel context
	inner: ::udi_sys::init::udi_child_chan_context_t,
	channel_cb: *mut crate::ffi::imc::udi_channel_event_cb_t,
	child_data: ChildData,
}
impl<Driver,ChildData> ChildBind<Driver,ChildData>
{
	pub fn child_id(&self) -> ::udi_sys::udi_ubit32_t {
		self.inner.child_id
	}
	pub fn dev(&self) -> &Driver {
		unsafe { & (*(self.inner.rdata as *const crate::init::RData<Driver>)).inner }
	}
	pub fn dev_mut(&mut self) -> &mut Driver {
		unsafe { &mut (*(self.inner.rdata as *mut crate::init::RData<Driver>)).inner }
	}
}
impl<Driver,ChildData> AsRef<crate::init::RData<Driver>> for ChildBind<Driver,ChildData> {
	fn as_ref(&self) -> &crate::init::RData<Driver> {
		unsafe { & (*(self.inner.rdata as *const crate::init::RData<Driver>)) }
	}
}
impl<Driver,ChildData> ::core::ops::Deref for ChildBind<Driver,ChildData>
{
	type Target = ChildData;
	fn deref(&self) -> &Self::Target {
		&self.child_data
	}
}
impl<Driver,ChildData> ::core::ops::DerefMut for ChildBind<Driver,ChildData>
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.child_data
	}
}
impl<Driver,ChildData> crate::async_trickery::CbContext for ChildBind<Driver,ChildData>
{
    fn channel_cb_slot(&mut self) -> &mut *mut ::udi_sys::imc::udi_channel_event_cb_t {
        &mut self.channel_cb
    }
}
impl<Driver,ChildData> crate::imc::ChannelInit for ChildBind<Driver,ChildData>
where
	ChildData: Default,
{
    unsafe fn init(&mut self) {
		::core::ptr::write(&mut self.child_data, ChildData::default())
	}
}