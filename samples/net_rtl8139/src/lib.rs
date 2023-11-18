#![feature(impl_trait_in_assoc_type)]

mod pio_ops;

#[derive(Default)]
struct Driver
{
	pio_handles: PioHandles,
	mac_addr: [u8; 6],
	rx_cb_queue: ::udi::meta_nic::ReadCbQueue,
	dma_constraints: ::udi::physio::dma::DmaConstraints,
}
#[derive(Default)]
struct PioHandles {
	reset: ::udi::pio::Handle,
	irq_ack: ::udi::pio::Handle,
	enable: ::udi::pio::Handle,
	disable: ::udi::pio::Handle,
}

impl ::udi::init::Driver for ::udi::init::RData<Driver>
{
	const MAX_ATTRS: u8 = 4;

    type Future_init<'s> = impl ::core::future::Future<Output=()> + 's;
    fn usage_ind<'s>(&'s mut self, _cb: ::udi::meta_mgmt::CbRefUsage<'s>, _resouce_level: u8) -> Self::Future_init<'s> {
        async move {
		}
    }

    type Future_enumerate<'s> = impl ::core::future::Future<Output=(::udi::init::EnumerateResult,::udi::init::AttrSink<'s>)> + 's;
    fn enumerate_req<'s>(
		&'s mut self,
		_cb: ::udi::init::CbRefEnumerate<'s>,
		level: ::udi::init::EnumerateLevel,
		mut attrs_out: ::udi::init::AttrSink<'s>
	) -> Self::Future_enumerate<'s> {
        async move {
			match level
			{
			::udi::init::EnumerateLevel::Start
			|::udi::init::EnumerateLevel::StartRescan => {
				attrs_out.push_u32("if_num", 0);
				attrs_out.push_string("if_media", "eth");
				attrs_out.push_string_fmt("identifier", format_args!("{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
					self.mac_addr[0], self.mac_addr[1], self.mac_addr[2],
					self.mac_addr[3], self.mac_addr[4], self.mac_addr[5],
					));
				(::udi::init::EnumerateResultOk::new::<OpsList::Ctrl>(0).into(), attrs_out)
				},
			udi::init::EnumerateLevel::Next => (::udi::init::EnumerateResult::Done, attrs_out),
			udi::init::EnumerateLevel::New => todo!(),
			udi::init::EnumerateLevel::Directed => todo!(),
			udi::init::EnumerateLevel::Release => todo!(),
			}
		}
    }

    type Future_devmgmt<'s> = impl ::core::future::Future<Output=::udi::Result<u8>> + 's;
    fn devmgmt_req<'s>(&'s mut self, _cb: ::udi::init::CbRefMgmt<'s>, mgmt_op: udi::init::MgmtOp, _parent_id: ::udi::ffi::udi_ubit8_t) -> Self::Future_devmgmt<'s> {
        async move {
			use ::udi::init::MgmtOp;
			match mgmt_op
			{
			MgmtOp::PrepareToSuspend => todo!(),
			MgmtOp::Suspend => todo!(),
			MgmtOp::Shutdown => todo!(),
			MgmtOp::ParentSuspend => todo!(),
			MgmtOp::Resume => todo!(),
			MgmtOp::Unbind => todo!(),
			}
		}
    }
}
impl ::udi::meta_bridge::BusDevice for ::udi::init::RData<Driver>
{
    type Future_bind_ack<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn bus_bind_ack<'a>(
		&'a mut self,
		cb: ::udi::meta_bridge::CbRefBind<'a>,
		dma_constraints: ::udi::ffi::physio::udi_dma_constraints_t,
		_preferred_endianness: ::udi::meta_bridge::PreferredEndianness,
		_status: ::udi::ffi::udi_status_t
	) -> Self::Future_bind_ack<'a> {
		async move {
			let pio_map = |trans_list| ::udi::pio::map(cb.gcb(), 0/*UDI_PCI_BAR_0*/, 0x00,0x20, trans_list, 0/*UDI_PIO_LITTLE_ENDIAN*/, 0, 0.into());
			self.pio_handles.reset   = pio_map(&pio_ops::RESET).await;
			self.pio_handles.irq_ack = pio_map(&pio_ops::IRQACK).await;
			self.pio_handles.enable  = pio_map(&pio_ops::ENABLE).await;
			self.pio_handles.disable = pio_map(&pio_ops::DISBALE).await;

			self.dma_constraints = unsafe {
				use ::udi::ffi::physio::udi_dma_constraints_attr_spec_t as Spec;
				use ::udi::ffi::physio;
				let mut dc = ::udi::physio::dma::DmaConstraints::from_raw(dma_constraints);
				dc.set(cb.gcb(), &[
					// 32-bit device
					Spec { attr_type: physio::UDI_DMA_ADDRESSABLE_BITS, attr_value: 32 },
					// Maximum scatter-gather elements = 1 (only one TX slot)
					Spec { attr_type: physio::UDI_DMA_SCGTH_MAX_ELEMENTS, attr_value: 1 },
					]).await?;
				dc
				};
			// Reset the card and get the MAC address
			let rbstart: u32 = todo!("rbstart");
			{
				let mut reset_data = [0; 4+6];
				reset_data[..4].copy_from_slice(&rbstart.to_ne_bytes());
				::udi::pio::trans(cb.gcb(), &self.inner.pio_handles.reset, Default::default(),
					None, Some(unsafe { ::udi::pio::MemPtr::new(&mut reset_data)})).await?;
				self.mac_addr.copy_from_slice(&reset_data[4..]);
			}
			Ok( () )
		}
    }

    type Future_unbind_ack<'s> = impl ::core::future::Future<Output=()> + 's;
    fn bus_unbind_ack<'a>(&'a mut self, _cb: ::udi::meta_bridge::CbRefBind<'a>) -> Self::Future_unbind_ack<'a> {
        async move {
		}
    }

    type Future_intr_attach_ack<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_attach_ack<'a>(&'a mut self, cb: ::udi::meta_bridge::CbRefIntrAttach<'a>, status: udi::ffi::udi_status_t) -> Self::Future_intr_attach_ack<'a> {
        async move {
			let _ = cb;
			if status != 0 {
				// TODO: Free the CB and channel?
			}
			// Flag to the "caller" that this is complete, and what the result was
			//self.intr_attach_res.set(status);
		}
    }

    type Future_intr_detach_ack<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_detach_ack<'a>(&'a mut self, cb: ::udi::meta_bridge::CbRefIntrDetach<'a>) -> Self::Future_intr_detach_ack<'a> {
        async move {
			let _ = cb;
		}
    }
}
impl ::udi::meta_bridge::IntrHandler for ::udi::init::RData<Driver>
{
    type Future_intr_event_ind<'s> = impl ::core::future::Future<Output=()>+'s;
    fn intr_event_ind<'a>(&'a mut self, cb: ::udi::meta_bridge::CbRefEvent<'a>, _flags: u8) -> Self::Future_intr_event_ind<'a> {
		async move {
			let isr = cb.intr_result;
			if isr & pio_ops::FLAG_ISR_ROK != 0 {
				// TODO: RX OK
			}
		}
    }
}

impl ::udi::meta_nic::Control for ::udi::ChildBind<Driver,()>
{
	type Future_bind_req<'s> = impl ::core::future::Future<Output=::udi::Result<::udi::meta_nic::NicInfo>> + 's;
    fn bind_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNicBind<'a>, tx_chan_index: udi::ffi::udi_index_t, rx_chan_index: udi::ffi::udi_index_t) -> Self::Future_bind_req<'a> {
        async move {
			todo!("bind_req")
		}
    }

	type Future_unbind_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn unbind_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNic<'a>) -> Self::Future_unbind_req<'a> {
        async move {
			let _ = cb;
		}
    }

	type Future_enable_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn enable_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNic<'a>) -> Self::Future_enable_req<'a> {
        async move {
			::udi::pio::trans(cb.gcb(), &self.dev().pio_handles.enable, ::udi::ffi::udi_index_t(0), None, None).await?;
			Ok( () )
		}
    }

	type Future_disable_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn disable_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNic<'a>) -> Self::Future_disable_req<'a> {
        async move {
			let _ = ::udi::pio::trans(cb.gcb(), &self.dev().pio_handles.disable, 0.into(), None, None).await;
		}
    }

	type Future_ctrl_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn ctrl_req<'a>(&'a mut self, _cb: ::udi::meta_nic::CbRefNicCtrl<'a>) -> Self::Future_ctrl_req<'a> {
        async move { todo!() }
    }

	type Future_info_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn info_req<'a>(&'a mut self, _cb: ::udi::meta_nic::CbRefNicInfo<'a>, _reset_statistics: bool) -> Self::Future_info_req<'a> {
        async move { todo!() }
    }
}
impl ::udi::meta_nic::NdTx for ::udi::init::RData<Driver>
{
	type Future_tx_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn tx_req<'a>(&'a mut self, mut cb: ::udi::meta_nic::CbHandleNicTx) -> Self::Future_tx_req<'a> {
        async move {
			todo!("TX req")
		}
    }

	type Future_exp_tx_req<'s> = Self::Future_tx_req<'s>;
    fn exp_tx_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbHandleNicTx) -> Self::Future_exp_tx_req<'a> {
        self.tx_req(cb)
    }
}
impl ::udi::meta_nic::NdRx for ::udi::init::RData<Driver>
{
	type Future_rx_rdy<'s> = impl ::core::future::Future<Output=()> + 's;
    fn rx_rdy<'a>(&'a mut self, cb: ::udi::meta_nic::CbHandleNicRx) -> Self::Future_rx_rdy<'a> {
        async move {
			self.rx_cb_queue.push(cb);
		}
    }
}

mod udiprops {
	include!{ concat!(env!("OUT_DIR"), "/udiprops.rs") }
}

::udi::define_driver!{Driver;
	ops: {
		Dev : ::udi::ffi::meta_bridge @ udi_bus_device_ops_t,
		Ctrl: ::udi::ffi::meta_nic @ udi_nd_ctrl_ops_t : ChildBind<_,()>,
		Tx  : ::udi::ffi::meta_nic @ udi_nd_tx_ops_t,
		Rx  : ::udi::ffi::meta_nic @ udi_nd_rx_ops_t,
		Irq : ::udi::ffi::meta_bridge @ udi_intr_handler_ops_t,
		},
	cbs: {
		BusBind  : ::udi::ffi::meta_bridge @ udi_bus_bind_cb_t,
		Intr     : ::udi::ffi::meta_bridge @ udi_intr_attach_cb_t,
		IntrEvent: ::udi::ffi::meta_bridge @ udi_intr_event_cb_t,

		_IntrDetach: ::udi::ffi::meta_bridge @ udi_intr_detach_cb_t,

		Nic    : ::udi::ffi::meta_nic @ udi_nic_cb_t,
		NicBind: ::udi::ffi::meta_nic @ udi_nic_bind_cb_t,
		NicCtrl: ::udi::ffi::meta_nic @ udi_nic_ctrl_cb_t,
		NicInfo: ::udi::ffi::meta_nic @ udi_nic_info_cb_t,
		NicTx  : ::udi::ffi::meta_nic @ udi_nic_tx_cb_t,
		NicRx  : ::udi::ffi::meta_nic @ udi_nic_rx_cb_t,
		}
}
