#![feature(impl_trait_in_assoc_type)]

mod pio_ops;

#[derive(Default)]
struct Driver
{
	pio_handles: PioHandles,
	mac_addr: [u8; 6],
	rx_cb_queue: ::udi::meta_nic::ReadCbQueue,
	dma_constraints: ::udi::physio::dma::DmaConstraints,
	dma_handles: Option<DmaStructures>,

	intr_channel: ::udi::imc::ChannelHandle,
	channel_tx: ::udi::imc::ChannelHandle,
	channel_rx: ::udi::imc::ChannelHandle,

	/// Next TX slot (out of four) that will be used by the hardware
	next_tx_slot: u8,
	/// Currently active TX slot, if equal to `next_tx_slot` then all entries are unused
	cur_tx_slot: u8,
	/// Information about each TX slot
	tx_cbs: [Option<TxSlot>; 4],
}
// TODO: Move this to within `pio_ops` and use privacy to handle safety
#[derive(Default)]
struct PioHandles {
	reset: ::udi::pio::Handle,
	irq_ack: ::udi::pio::Handle,
	enable: ::udi::pio::Handle,
	disable: ::udi::pio::Handle,

	tx: ::udi::pio::Handle,
	get_tsd: ::udi::pio::Handle,
}
struct DmaStructures {
	rx_buf: ::udi::physio::dma::DmaAlloc,
	tx_slots: [::udi::physio::dma::DmaBuf; 4],
	tx_bounce: [::udi::physio::dma::DmaAlloc; 4],
}
struct TxSlot {
	cb: ::udi::meta_nic::CbHandleNicTx,
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
			let pio_map = |trans_list| ::udi::pio::map(cb.gcb(), 0/*UDI_PCI_BAR_0*/, 0x00,0x80, trans_list, 0/*UDI_PIO_LITTLE_ENDIAN*/, 0, 0.into());
			self.pio_handles.reset   = pio_map(&pio_ops::RESET).await;
			self.pio_handles.irq_ack = pio_map(&pio_ops::IRQACK).await;
			self.pio_handles.enable  = pio_map(&pio_ops::ENABLE).await;
			self.pio_handles.disable = pio_map(&pio_ops::DISBALE).await;
			self.pio_handles.tx      = pio_map(&pio_ops::TX).await;
			self.pio_handles.get_tsd = pio_map(&pio_ops::GET_TSD).await;

			self.intr_channel = ::udi::imc::channel_spawn(cb.gcb(), /*interrupt number*/0.into(), OpsList::Irq as _).await;
			let mut intr_cb = ::udi::cb::alloc::<CbList::Intr>(cb.gcb(), ::udi::get_gcb_channel().await).await;
			intr_cb.interrupt_index = 0.into();
			intr_cb.min_event_pend = 2;
			intr_cb.preprocessing_handle = self.pio_handles.irq_ack.as_raw();	// NOTE: This transfers ownership
			::udi::meta_bridge::attach_req(intr_cb);

			self.dma_constraints = unsafe {
				use ::udi::ffi::physio::udi_dma_constraints_attr_spec_t as Spec;
				use ::udi::ffi::physio;
				let mut dc = ::udi::physio::dma::DmaConstraints::from_raw(dma_constraints);
				dc.set(cb.gcb(), &[
					// 32-bit device
					Spec { attr_type: physio::UDI_DMA_ADDRESSABLE_BITS, attr_value: 32 },
					// Maximum scatter-gather elements = 1 (only one TX slot)
					Spec { attr_type: physio::UDI_DMA_SCGTH_MAX_ELEMENTS, attr_value: 1 },
					Spec { attr_type: physio::UDI_DMA_NO_PARTIAL, attr_value: 1 },
					]).await?;
				dc
				};
			self.inner.dma_handles = Some({
				use ::udi::physio::dma::{DmaBuf,Direction,Endianness};
				let alloc_single = |size: usize| ::udi::physio::dma::DmaAlloc::alloc_single(
					cb.gcb(), &self.inner.dma_constraints, Direction::In, Endianness::NeverSwap,
					true, size
				);
				DmaStructures {
				// Allocate the RX buffer (12KiB - 3 standard pages)
				rx_buf: alloc_single(3*4096).await,
				// DMA information for direct TX of the four TX slots
				tx_slots: [
					DmaBuf::prepare(cb.gcb(), &self.inner.dma_constraints, Some(Direction::Out)).await,
					DmaBuf::prepare(cb.gcb(), &self.inner.dma_constraints, Some(Direction::Out)).await,
					DmaBuf::prepare(cb.gcb(), &self.inner.dma_constraints, Some(Direction::Out)).await,
					DmaBuf::prepare(cb.gcb(), &self.inner.dma_constraints, Some(Direction::Out)).await,
				],
				// Bounce buffers for the TX slots
				tx_bounce: [
					alloc_single(1520+20).await,
					alloc_single(1520+20).await,
					alloc_single(1520+20).await,
					alloc_single(1520+20).await,
				],
				}
				});
			let rbstart: u32 = self.inner.dma_handles.as_ref().unwrap()
				.rx_buf.scgth().single_entry_32().expect("Environment broke the RX buffer into chunks, not allowed")
				.block_busaddr;
			// Reset the card and get the MAC address
			{
				// SAFE: Correct DMA
				let mut reset_data = unsafe { pio_ops::MemReset::new(rbstart) };
				::udi::pio::trans(cb.gcb(), &self.inner.pio_handles.reset, Default::default(),
					None, Some(reset_data.get_ptr())).await?;
				self.mac_addr.copy_from_slice(&reset_data.mac);
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
			// TX OK or TX Error
			if isr & pio_ops::FLAG_ISR_TOK != 0 || isr & pio_ops::FLAG_ISR_TER != 0 {
				// - Release the buffers and CBs involved
				while self.inner.cur_tx_slot != self.inner.next_tx_slot {
					let slot = self.inner.cur_tx_slot as usize;
					
					let tsd = match ::udi::pio::trans(cb.gcb(), &self.pio_handles.get_tsd, (slot as u8).into(), None, None).await
						{
						Ok(tsd) => tsd as u32,
						Err(_e) => break,
						};
					if tsd & 0x8000 == 0 {
						break;
					}
					mod_inc(&mut self.inner.cur_tx_slot, 4);

					let dma = self.inner.dma_handles.as_mut().unwrap();
					match self.inner.tx_cbs[slot].take()
					{
					Some(mut s) => {
						if let Some(buf) = unsafe { dma.tx_slots[slot].buf_unmap(0) }
						{
							s.cb.tx_buf = buf.into_raw();
						}
						::udi::meta_nic::nsr_tx_rdy(s.cb);
					},
					None => {
						// Huh, that shouldn't happen
						break;
					},
					}
				}
			}
		}
    }
}

impl ::udi::meta_nic::Control for ::udi::ChildBind<Driver,()>
{
	type Future_bind_req<'s> = impl ::core::future::Future<Output=::udi::Result<::udi::meta_nic::NicInfo>> + 's;
    fn bind_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNicBind<'a>, tx_chan_index: udi::ffi::udi_index_t, rx_chan_index: udi::ffi::udi_index_t) -> Self::Future_bind_req<'a> {
        async move {
			self.dev_mut().channel_tx = ::udi::imc::channel_spawn(cb.gcb(), tx_chan_index, OpsList::Tx as _).await;
			self.dev_mut().channel_rx = ::udi::imc::channel_spawn(cb.gcb(), rx_chan_index, OpsList::Rx as _).await;
			::udi::debug_printf!("NIC mac_addr = %02x:%02x:%02x:%02x:%02x:%02x",
				self.dev().mac_addr[0] as _, self.dev().mac_addr[1] as _, self.dev().mac_addr[2] as _,
				self.dev().mac_addr[3] as _, self.dev().mac_addr[4] as _, self.dev().mac_addr[5] as _,
				);
			{
				let s = ::core::ffi::CStr::from_bytes_with_nul(b"eth\0").unwrap();
				::udi::debug_printf!("NIC ether_type = %s", s);
			}
			Ok(::udi::meta_nic::NicInfo {
				media_type: ::udi::ffi::meta_nic::MediaType::UDI_NIC_ETHER,
				min_pdu_size: 0,
				max_pdu_size: 0,
				rx_hw_threshold: 2,
				capabilities: 0,
				max_perfect_multicast: 0,
				max_total_multicast: 0,
				mac_addr_len: 6,
				mac_addr: [
					self.dev().mac_addr[0], self.dev().mac_addr[1], self.dev().mac_addr[2],
					self.dev().mac_addr[3], self.dev().mac_addr[4], self.dev().mac_addr[5],
					0,0,0,0,
					0,0,0,0,0,0,0,0,0,0,
				],
			})
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
impl Driver
{
	fn tx_inner<'s>(&'s mut self, mut cb: ::udi::meta_nic::CbHandleNicTx) -> impl ::core::future::Future<Output=::udi::Result<()>> + 's {
		async move {
			// SAFE: Input contract
			let buf = unsafe { ::udi::buf::Handle::from_raw(cb.tx_buf) };
			let len = buf.len();
			let slot = self.next_tx_slot as usize;
			let dma = self.dma_handles.as_mut().unwrap();
			let h = dma.tx_slots[slot]
				.buf_map(cb.gcb(), buf, .., udi::physio::dma::Direction::Out)
				.await;
			let ent = match h {
				Ok((scgth, complete)) => {
					assert!(complete, "Environment bug: `complete` was false");
					*scgth.single_entry_32().expect("Environment bug: TX buffer in multiple chunks")
					},
				Err(_e) => { // Cannot map - Most likely due to DMA constraints, so use the bounce buffer
					let buf = unsafe { dma.tx_slots[slot].buf_unmap(len).unwrap() };
					let dst = unsafe { ::core::slice::from_raw_parts_mut(dma.tx_bounce[slot].mem_ptr as *mut u8, len) };
					buf.read(0, dst);
					cb.tx_buf = buf.into_raw();
					*dma.tx_bounce[slot].scgth().single_entry_32().expect("Environment bug: TX bounce buffer in multiple chunks")
				},
				};
			let addr = ent.block_busaddr;
			let len = ent.block_length;
			assert!(self.tx_cbs[slot].is_none(), "TX slot already active, too many TX CBs around?");
			let mut mem = pio_ops::MemTx {
				addr,
				len: len as u16,
				index: slot as u8,
			};
			::udi::pio::trans(cb.gcb(), &self.pio_handles.tx, Default::default(), None, Some(unsafe { mem.get_ptr() })).await?;
			self.tx_cbs[slot] = Some(TxSlot { cb });
			Ok( () )
		}
	}
}
impl ::udi::meta_nic::NdTx for ::udi::init::RData<Driver>
{
	type Future_tx_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn tx_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbHandleNicTx) -> Self::Future_tx_req<'a> {
        async move {
			match self.inner.tx_inner(cb).await
			{
			Ok(()) => {},
			Err(_) => {
				// Would be nice to return the CB, but unlikely to happen so meh.
				},
			}
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

fn mod_inc(v: &mut u8, max: u8) {
	*v += 1;
	while *v >= max {
		*v -= max;
	}
}