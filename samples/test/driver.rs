
#[derive(Default)]
pub struct Driver
{
}
impl ::udi::init::Driver for ::udi::init::RData<Driver>
{
	const MAX_ATTRS: u8 = 4;

	type Future_init<'s> = impl ::core::future::Future<Output=()> + 's;
	fn usage_ind<'s>(&'s mut self, cb: ::udi::meta_mgmt::CbRefUsage<'s>, _resouce_level: u8) -> Self::Future_init<'s> {
		async move {
			println!("Entry");
			let h1 = ::udi::pio::map(cb.gcb(), 0,0x1000,4, &[], 0, 0, ::udi::ffi::udi_index_t(0)).await;
			println!("h1 = {:?}", h1);
			let h2 = ::udi::pio::map(cb.gcb(), 0,0x1004,4, &[], 0, 0, ::udi::ffi::udi_index_t(0)).await;
			println!("h2 = {:?}", h2);
		}
	}

    type Future_enumerate<'s> = impl ::core::future::Future<Output=(::udi::init::EnumerateResult,::udi::init::AttrSink<'s>)> + 's;
    fn enumerate_req<'s>(
		&'s mut self,
		_cb: ::udi::init::CbRefEnumerate<'s>,
		level: ::udi::init::EnumerateLevel,
		attrs_out: ::udi::init::AttrSink<'s>
	) -> Self::Future_enumerate<'s> {
        async move {
			match level {
			::udi::init::EnumerateLevel::Start
			|::udi::init::EnumerateLevel::StartRescan
			|::udi::init::EnumerateLevel::Next
				=> (::udi::init::EnumerateResult::Done, attrs_out),
			::udi::init::EnumerateLevel::New => todo!("EnumerateLevel::New"),
			::udi::init::EnumerateLevel::Directed => todo!(),
			::udi::init::EnumerateLevel::Release => todo!(),
			}
		}
    }

    type Future_devmgmt<'s> = impl ::core::future::Future<Output=::udi::Result<u8>> + 's;
    fn devmgmt_req<'s>(&'s mut self, _cb: ::udi::init::CbRefMgmt<'s>, _mgmt_op: udi::init::MgmtOp, _parent_id: ::udi::ffi::udi_ubit8_t) -> Self::Future_devmgmt<'s> {
        async move {
			todo!()
		}
    }
}

::udi::define_driver!{Driver;
	ops: {},
	cbs: {}
}