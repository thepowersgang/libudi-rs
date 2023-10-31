struct Driver {

}
impl ::udi::init::Driver for Driver
{
    const MAX_ATTRS: u8 = 3;
    type Future_init<'s> = impl ::core::future::Future<Output=Self>;
    fn usage_ind(_cb: udi::init::CbRefUsage, _resouce_level: u8) -> Self::Future_init<'_> {
        async move { Driver {} }
    }

    type Future_enumerate<'s> = impl ::core::future::Future<Output=(udi::init::EnumerateResult,udi::init::AttrSink<'s>)> + 's;
    fn enumerate_req<'s>(
        &'s mut self,
        _cb: udi::init::CbRefEnumerate<'s>,
        level: udi::init::EnumerateLevel,
        mut attrs_out: udi::init::AttrSink<'s>
    ) -> Self::Future_enumerate<'s>
    {
        async move {
			match level
			{
			::udi::init::EnumerateLevel::Start
			|::udi::init::EnumerateLevel::StartRescan => {
                // bus_type string pci  pci_vendor_id ubit32 0x10ec  pci_device_id ubit32 0x8029
				attrs_out.push_string("bus_type", "pci");
				attrs_out.push_u32("pci_vendor_id", 0x10ec);
				attrs_out.push_u32("pci_device_id", 0x8029);
				(::udi::init::EnumerateResult::Ok(OpsList::Bridge as _), attrs_out)
				},
			udi::init::EnumerateLevel::Next => (::udi::init::EnumerateResult::Done, attrs_out),
			udi::init::EnumerateLevel::New => todo!(),
			udi::init::EnumerateLevel::Directed => todo!(),
			udi::init::EnumerateLevel::Release => todo!(),
			}
        }
    }

    type Future_devmgmt<'s> = impl ::core::future::Future<Output=::udi::Result<u8>> + 's;
    fn devmgmt_req<'s>(&'s mut self, cb: udi::init::CbRefMgmt<'s>, mgmt_op: udi::init::MgmtOp, parent_id: udi::ffi::udi_index_t) -> Self::Future_devmgmt<'s> {
        async move {
            todo!("devmgmt_req");
        }
    }
}
impl ::udi::meta_bus::BusBridge for Driver
{
    type Future_bind_req<'s> = impl ::core::future::Future<Output=::udi::Result<(::udi::meta_bus::PreferredEndianness,)>> + 's;
    fn bus_bind_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefBind<'a>) -> Self::Future_bind_req<'a> {
        async move {
            Ok((::udi::meta_bus::PreferredEndianness::Little,))
        }
    }

    type Future_unbind_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn bus_unbind_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefBind<'a>) -> Self::Future_unbind_req<'a> {
        async move {
            todo!("bus_unbind_req");
        }
    }

    type Future_intr_attach_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn intr_attach_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefIntrAttach<'a>) -> Self::Future_intr_attach_req<'a> {
        async move {
            let channel = ::udi::imc::channel_spawn(cb.gcb(), cb.interrupt_index, OpsList::Interrupt as _).await;
            //todo!("intr_attach_req");
            Ok( () )
        }
    }

    type Future_intr_detach_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_detach_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefIntrDetach<'a>) -> Self::Future_intr_detach_req<'a> {
        async move {
            todo!();
        }
    }
}
impl ::udi::meta_intr::IntrDispatcher for Driver
{
    type Future_intr_event_rdy<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_event_rdy<'a>(&'a mut self, cb: ::udi::meta_intr::CbRefEvent<'a>) -> Self::Future_intr_event_rdy<'a> {
        async move {
        }
    }
}

pub const UDIPROPS: &'static str = "\
properties_version 0x101\0\
requires udi_bridge 0x101\0\
meta 1 udi_bridge\0\
child_bind_ops 1 0 1\0\
";
::udi::define_driver! {
    Driver as INIT_INFO_PCI;
    ops: {
        Bridge: Meta=1, ::udi::ffi::meta_bus::udi_bus_bridge_ops_t,
        Interrupt: Meta=1, ::udi::ffi::meta_intr::udi_intr_dispatcher_ops_t,
    },
    cbs: {
    }
}