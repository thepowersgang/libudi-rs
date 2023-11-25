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
    EmulatedDevice { factory: |_| crate::emulated_devices::rtl8139::Device::new_boxed(), vendor_id: 0x10ec, device_id: 0x8139, class_word: 0 },
    // A "Generic XT-Compatible Serial Controller"
    EmulatedDevice { factory: |_| crate::emulated_devices::XTSerial::new_boxed(), vendor_id: 0x8086, device_id: 0xFFFF, class_word: 0x07_00_00 },
];

fn get_driver_instance(gcb: ::udi::CbRef<::udi::ffi::udi_cb_t>) -> ::std::sync::Arc<crate::DriverInstance> {
    unsafe { crate::channels::get_other_instance(&gcb.channel) }
}

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
    fn devmgmt_req<'s>(&'s mut self, _cb: udi::init::CbRefMgmt<'s>, _mgmt_op: udi::init::MgmtOp, _parent_id: udi::ffi::udi_ubit8_t) -> Self::Future_devmgmt<'s> {
        async move {
            todo!("devmgmt_req");
        }
    }
}

#[derive(Default)]
struct ChildState {
    irq_cbs: ::std::sync::Arc<CbQueue>,
}
struct InterruptHandler {
    cbs: ::std::sync::Arc<CbQueue>,
    channel: ::udi::imc::ChannelHandle,
    preproc: ::udi::pio::Handle,
}
#[derive(Default)]
struct CbQueue {
    queue: ::std::sync::Mutex< ::udi::cb::Chain<::udi::ffi::meta_bridge::udi_intr_event_cb_t> >,
}
impl ::udi::meta_bridge::BusBridge for ::udi::ChildBind<Driver,ChildState>
{
    type Future_bind_req<'s> = impl ::core::future::Future<Output=::udi::Result<(::udi::meta_bridge::PreferredEndianness,)>> + 's;
    fn bus_bind_req<'a>(&'a mut self, cb: ::udi::meta_bridge::CbRefBind<'a>) -> Self::Future_bind_req<'a> {
        async move {
            println!("PCI Bind Request: #{:#x}", self.child_id());
            let di = get_driver_instance(cb.gcb());
            let _ = self.irq_cbs.clone();
            let dev_desc = &DEVICES[self.child_id() as usize];
            di
                .device
                .set( (dev_desc.factory)( dev_desc ) )
                .ok()
                .expect("Driver instance bound to multiple devices?");
            Ok((::udi::meta_bridge::PreferredEndianness::Little,))
        }
    }

    type Future_unbind_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn bus_unbind_req<'a>(&'a mut self, cb: ::udi::meta_bridge::CbRefBind<'a>) -> Self::Future_unbind_req<'a> {
        async move {
            let di = get_driver_instance(cb.gcb());
            assert!(di.device.get().is_some());
        }
    }

    type Future_intr_attach_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn intr_attach_req<'a>(&'a mut self, cb: ::udi::meta_bridge::CbRefIntrAttach<'a>) -> Self::Future_intr_attach_req<'a> {
        async move {
            let channel = ::udi::imc::channel_spawn::<OpsList::Interrupt>(cb.gcb(), self, cb.interrupt_index).await;
            // SAFE: We're trusting the client driver to not provide a bad handle
            let preproc_handle = unsafe { ::udi::pio::Handle::from_raw(cb.preprocessing_handle) };

            let di = get_driver_instance(cb.gcb());
            di.device.get().expect("Driver instance not bound to a device")
                .irq(cb.interrupt_index.0)
                .bind(Box::new(InterruptHandler {
                    cbs: self.irq_cbs.clone(),
                    channel,
                    preproc: preproc_handle,
                }));
            Ok( () )
        }
    }

    type Future_intr_detach_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_detach_req<'a>(&'a mut self, cb: ::udi::meta_bridge::CbRefIntrDetach<'a>) -> Self::Future_intr_detach_req<'a> {
        let di = get_driver_instance(cb.gcb());
        di.device.get().expect("Driver instance not bound to a device")
            .irq(cb.interrupt_idx.0)
            .unbind();
        async move { }
    }
}
impl ::udi::meta_bridge::IntrDispatcher for ::udi::ChildBind<Driver,ChildState>
{
    type Future_intr_event_rdy<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_event_rdy<'a>(&'a mut self, _cb: ::udi::meta_bridge::CbRefEvent) -> Self::Future_intr_event_rdy<'a> {
        async move {
        }
    }

    fn intr_event_ret(&mut self, cb: udi::meta_bridge::CbHandleEvent) {
        self.irq_cbs.queue.lock().unwrap().push_front(cb);
    }
}
impl crate::emulated_devices::InterruptHandler for InterruptHandler {
    fn raise(&mut self) {
        let Some(mut cb) = self.cbs.queue.lock().unwrap().pop_front() else {
            println!("No interrupt CBs!");
            return ;
        };

        cb.set_channel(&self.channel);
        if self.preproc.as_raw().is_null() {
            println!("No preprocess");
            let flags = 0;
            unsafe {
                ::udi::ffi::meta_bridge::udi_intr_event_ind(cb.into_raw(), flags);
            }
        }
        else {
            unsafe {
                // NOTE: `scratch` has at least one byte available, as it was used for async above
                ::core::ptr::write(cb.gcb.scratch as *mut u8, 0);
                let buf = cb.event_buf;
                ::udi::ffi::pio::udi_pio_trans(
                    callback, cb.into_raw() as *mut _,
                    self.preproc.as_raw(), 1.into(),    // Normal interrupt
                    buf, ::core::ptr::null_mut()
                );
            }
            extern "C" fn callback(
                gcb: *mut ::udi::ffi::udi_cb_t,
                buf: *mut ::udi::ffi::udi_buf_t,
                _status: ::udi::ffi::udi_status_t,
                res: ::udi::ffi::udi_ubit16_t
            ) {
                assert!(_status == ::udi::ffi::UDI_OK as _);
                unsafe {
                    println!("Preprocess complete");
                    let cb = gcb as *mut ::udi::ffi::meta_bridge::udi_intr_event_cb_t;
                    let flags = ::core::ptr::read( (*cb).gcb.scratch as *const u8 )
                        /*| ::udi::ffi::meta_bridge::UDI_INTR_PREPROCESSED*/
                        ;
                    (*cb).event_buf = buf;
                    (*cb).intr_result = res;
                    ::udi::ffi::meta_bridge::udi_intr_event_ind(cb, flags);
                }
            }
        }
    }
}

::udi_macros::udiprops!("
properties_version 0x101
requires udi_bridge 0x101
meta 1 udi_bridge
child_bind_ops 1 0 1
region 0
");
const META_BRIDGE: ::udi::ffi::udi_index_t = udiprops::meta::udi_bridge;
::udi::define_driver! {
    Driver as INIT_INFO_PCI;
    ops: {
        Bridge   : Meta=META_BRIDGE, ::udi::ffi::meta_bridge::udi_bus_bridge_ops_t : ChildBind<_,ChildState>,
        Interrupt: Meta=META_BRIDGE, ::udi::ffi::meta_bridge::udi_intr_dispatcher_ops_t : ChildBind<_,ChildState>,
    },
    cbs: {
        BusBind  : Meta=META_BRIDGE, ::udi::ffi::meta_bridge::udi_bus_bind_cb_t,
		_IntrAttach: Meta=META_BRIDGE, ::udi::ffi::meta_bridge::udi_intr_attach_cb_t,
		_IntrDetach: Meta=META_BRIDGE, ::udi::ffi::meta_bridge::udi_intr_detach_cb_t,
		_IntrEvent : Meta=META_BRIDGE, ::udi::ffi::meta_bridge::udi_intr_event_cb_t,
    }
}