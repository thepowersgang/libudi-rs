#[derive(Default)]
struct Driver {
    enum_dev_idx: usize,
}

struct EmulatedDevice {
    factory: fn(&EmulatedDevice) -> Box<dyn crate::emulated_devices::PioDevice>,
    vendor_id: u16,
    device_id: u16,
    class_word: u32,
}
static DEVICES: &[EmulatedDevice] = &[
    EmulatedDevice { factory: |_| crate::emulated_devices::Rtl8029::new_boxed(), vendor_id: 0x10ec, device_id: 0x8029, class_word: 0 },
    // A "Generic XT-Compatible Serial Controller"
    EmulatedDevice { factory: |_| crate::emulated_devices::XTSerial::new_boxed(), vendor_id: 0x8086, device_id: 0xFFFF, class_word: 0x07_00_00 },
];

impl ::udi::init::Driver for ::udi::init::RData<Driver>
{
    const MAX_ATTRS: u8 = 6;
    type Future_init<'s> = impl ::core::future::Future<Output=()>;
    fn usage_ind<'s>(&'s mut self, _cb: udi::init::CbRefUsage<'s>, _resouce_level: u8) -> Self::Future_init<'s> {
        async move { }
    }

    type Future_enumerate<'s> = impl ::core::future::Future<Output=(udi::init::EnumerateResult,udi::init::AttrSink<'s>)> + 's;
    fn enumerate_req<'s>(
        &'s mut self,
        _cb: udi::init::CbRefEnumerate<'s>,
        level: udi::init::EnumerateLevel,
        mut attrs_out: udi::init::AttrSink<'s>
    ) -> Self::Future_enumerate<'s>
    {
        fn enumerate_dev(this: &mut Driver, attrs_out: &mut udi::init::AttrSink<'_>) -> ::udi::init::EnumerateResult {
            let child_idx = this.enum_dev_idx;
            if let Some(c) = DEVICES.get(child_idx) {
                this.enum_dev_idx += 1;
				attrs_out.push_string("bus_type", "pci");
				attrs_out.push_u32("pci_vendor_id", c.vendor_id as _);
				attrs_out.push_u32("pci_device_id", c.device_id as _);
				attrs_out.push_u32("pci_base_class", ((c.class_word >> 16) & 0xFF) as _);
				attrs_out.push_u32("pci_sub_class", ((c.class_word >> 8) & 0xFF) as _);
				attrs_out.push_u32("pci_prog_if", ((c.class_word >> 0) & 0xFF) as _);
                ::udi::init::EnumerateResult::ok::<OpsList::Bridge>(child_idx as _)
            }
            else {
                ::udi::init::EnumerateResult::Done
            }
        }
        async move {
			match level
			{
			::udi::init::EnumerateLevel::Start
			|::udi::init::EnumerateLevel::StartRescan => {
                self.enum_dev_idx = 0;
                let rv = enumerate_dev(self, &mut attrs_out);
                (rv, attrs_out)
				},
			udi::init::EnumerateLevel::Next => {
                let rv = enumerate_dev(self, &mut attrs_out);
                (rv, attrs_out)
                },
			udi::init::EnumerateLevel::New => todo!(),
			udi::init::EnumerateLevel::Directed => todo!(),
			udi::init::EnumerateLevel::Release => todo!(),
			}
        }
    }

    type Future_devmgmt<'s> = impl ::core::future::Future<Output=::udi::Result<u8>> + 's;
    fn devmgmt_req<'s>(&'s mut self, cb: udi::init::CbRefMgmt<'s>, mgmt_op: udi::init::MgmtOp, parent_id: udi::ffi::udi_ubit8_t) -> Self::Future_devmgmt<'s> {
        async move {
            todo!("devmgmt_req");
        }
    }
}

impl ::udi::meta_bus::BusBridge for ::udi::ChildBind<Driver,()>
{
    type Future_bind_req<'s> = impl ::core::future::Future<Output=::udi::Result<(::udi::meta_bus::PreferredEndianness,)>> + 's;
    fn bus_bind_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefBind<'a>) -> Self::Future_bind_req<'a> {
        async move {
            println!("PCI Bind Request: #{:#x}", self.child_id());
            let di = unsafe { crate::channels::get_other_instance(&cb.gcb.channel) };
            let dev_desc = &DEVICES[self.child_id() as usize];
            di
                .device
                .set( (dev_desc.factory)( dev_desc ) )
                .ok()
                .expect("Driver instance bound to multiple devices?");
            Ok((::udi::meta_bus::PreferredEndianness::Little,))
        }
    }

    type Future_unbind_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn bus_unbind_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefBind<'a>) -> Self::Future_unbind_req<'a> {
        async move {
            let di = unsafe { crate::channels::get_other_instance(&cb.gcb.channel) };
            assert!(di.device.get().is_some());
        }
    }

    type Future_intr_attach_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn intr_attach_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefIntrAttach<'a>) -> Self::Future_intr_attach_req<'a> {
        async move {
            let channel = ::udi::imc::channel_spawn(cb.gcb(), cb.interrupt_index, OpsList::Interrupt as _).await;
            let di = unsafe { crate::channels::get_other_instance(&cb.gcb.channel) };
            di.device.get().unwrap().set_interrupt_channel(cb.interrupt_index, channel);
            Ok( () )
        }
    }

    type Future_intr_detach_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_detach_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefIntrDetach<'a>) -> Self::Future_intr_detach_req<'a> {
        async move {
            let di = unsafe { crate::channels::get_other_instance(&cb.gcb.channel) };
            di.device.get().unwrap().set_interrupt_channel(cb.interrupt_idx, ::core::ptr::null_mut());
        }
    }
}
impl ::udi::meta_intr::IntrDispatcher for ::udi::init::RData<Driver>
{
    type Future_intr_event_rdy<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_event_rdy<'a>(&'a mut self, cb: ::udi::meta_intr::CbHandleEvent) -> Self::Future_intr_event_rdy<'a> {
        async move {
            let di = unsafe { crate::channels::get_other_instance(&cb.gcb.channel) };
            di.device.get().unwrap()
                .push_intr_cb(0.into(), cb);
        }
    }
}

pub const UDIPROPS: &'static str = "\
properties_version 0x101\0\
requires udi_bridge 0x101\0\
meta 1 udi_bridge\0\
child_bind_ops 1 0 1\0\
";
const META_BIRDGE: ::udi::ffi::udi_index_t = ::udi::ffi::udi_index_t(1);
::udi::define_driver! {
    Driver as INIT_INFO_PCI;
    ops: {
        Bridge   : Meta=META_BIRDGE, ::udi::ffi::meta_bus::udi_bus_bridge_ops_t : ChildBind<_,()>,
        Interrupt: Meta=META_BIRDGE, ::udi::ffi::meta_intr::udi_intr_dispatcher_ops_t,
    },
    cbs: {
        BusBind  : Meta=META_BIRDGE, ::udi::ffi::meta_bus::udi_bus_bind_cb_t,
		_IntrAttach: Meta=META_BIRDGE, ::udi::ffi::meta_intr::udi_intr_attach_cb_t,
		_IntrDetach: Meta=META_BIRDGE, ::udi::ffi::meta_intr::udi_intr_detach_cb_t,
		_IntrEvent : Meta=META_BIRDGE, ::udi::ffi::meta_intr::udi_intr_event_cb_t,
    }
}