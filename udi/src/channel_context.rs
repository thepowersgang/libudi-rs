//! Wrapper types for channel `context` structures (i.e. the structures used as the `context` field of a CB)

// TODO: Figure out a way to get `&mut` to region data (or channel context?) without technically violating aliasing
// - Could just leave interior mutability to the driver
// - Otherwise, can get `&mut` if no async can happen, which means no calls that take a CB
//   - That CB could be any CB - so short of somehow banning any calls to the UDI environment (how?) it has to be a lock

/// Channel context for child bind channels
#[repr(C)]
#[fundamental]
pub struct ChildBind<Driver,ChildData>
{
	pd: ::core::marker::PhantomData<Driver>,
	/// Required inner for child channel context
	inner: ::udi_sys::init::udi_child_chan_context_t,
	channel_cb: *mut crate::ffi::imc::udi_channel_event_cb_t,
	// - Internal
	is_init: bool,
	child_data: ChildData,
}
impl<Driver,ChildData> ChildBind<Driver,ChildData>
{
	/// `udi_child_chan_context_t` `child_id` field
	pub fn child_id(&self) -> ::udi_sys::udi_ubit32_t {
		self.inner.child_id
	}
	/// Reference to the driver (region data)
	pub fn dev(&self) -> &Driver {
		unsafe { & (*(self.inner.rdata as *const crate::init::RData<Driver>)).inner }
	}
}
impl<Driver,ChildData> AsRef<Self> for ChildBind<Driver,ChildData> {
	fn as_ref(&self) -> &Self {
		self
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
where
	ChildData: Default,
{
    fn maybe_init(&mut self) {
        if !self.is_init {
			unsafe {
				::core::ptr::write(&mut self.child_data, Default::default())
			}
			self.is_init = true;
		}
    }
    fn channel_cb_slot(&mut self) -> &mut *mut ::udi_sys::imc::udi_channel_event_cb_t {
        &mut self.channel_cb
    }
    unsafe fn drop_in_place(&mut self) {
		// A context can be bound to multiple channels
        todo!()
    }
	
}
impl<Driver,ChildData> crate::imc::ChannelInit for ChildBind<Driver,ChildData>
where
	ChildData: Default,
{
    unsafe fn init(&mut self) {
		assert!(self.is_init == false);
		self.is_init = true;
		::core::ptr::write(&mut self.child_data, ChildData::default())
	}
}