
pub struct Driver
{
}
impl ::udi::init::Driver for Driver
{
	type Future_init<'s> = impl ::core::future::Future<Output=Driver> + 's;
	fn init(mut cb: ::udi::CbRef<::udi::ffi::meta_mgmt::udi_usage_cb_t>, _resouce_level: u8) -> Self::Future_init<'_> {
		async move {
			println!("Entry");
			let h1 = ::udi::pio::map(cb.gcb(), 0,0x1000,4, &[], 0, 0, 0).await;
			println!("h1 = {:?}", h1);
			let h2 = ::udi::pio::map(cb.gcb(), 0,0x1004,4, &[], 0, 0, 0).await;
			println!("h2 = {:?}", h2);
			Driver {}
		}
	}
}

::udi::define_driver!{Driver;
	ops: {},
	cbs: {}
}