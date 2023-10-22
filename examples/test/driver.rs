
pub struct Driver
{
}
impl ::udi::init::Driver for Driver
{
	const MAX_ATTRS: u8 = 4;

	type Future_init<'s> = impl ::core::future::Future<Output=Driver> + 's;
	fn usage_ind(cb: ::udi::CbRef<::udi::ffi::meta_mgmt::udi_usage_cb_t>, _resouce_level: u8) -> Self::Future_init<'_> {
		async move {
			println!("Entry");
			let h1 = ::udi::pio::map(cb.gcb(), 0,0x1000,4, &[], 0, 0, 0).await;
			println!("h1 = {:?}", h1);
			let h2 = ::udi::pio::map(cb.gcb(), 0,0x1004,4, &[], 0, 0, 0).await;
			println!("h2 = {:?}", h2);
			Driver {}
		}
	}

    type Future_enumerate<'s> = impl ::core::future::Future<Output=(::udi::init::EnumerateResult,::udi::init::AttrSink<'s>)> + 's;
    fn enumerate_req<'s>(
		&'s mut self,
		cb: ::udi::init::CbRefEnumerate<'s>,
		level: ::udi::init::EnumerateLevel,
		attrs_out: ::udi::init::AttrSink<'s>
	) -> Self::Future_enumerate<'s> {
        async move {
			todo!()
		}
    }

    type Future_devmgmt<'s> = impl ::core::future::Future<Output=::udi::Result<u8>> + 's;
    fn devmgmt_req<'s>(&'s mut self, cb: ::udi::init::CbRefMgmt<'s>, mgmt_op: udi::init::MgmtOp, parent_id: ::udi::ffi::udi_index_t) -> Self::Future_devmgmt<'s> {
        async move {
			todo!()
		}
    }
}

::udi::define_driver!{Driver;
	ops: {},
	cbs: {}
}