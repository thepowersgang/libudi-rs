#![no_std]
#![feature(impl_trait_in_assoc_type)]

mod pio_ops;

const MTU: usize = 1520;
const RX_BUF_LENGTH: usize = 0x2000+16;
//const RX_BUF_CAPACITY: usize = RX_BUF_LENGTH+0x000;	// Extra page, to allow one page past the end
const RX_BUF_CAPACITY: usize = RX_BUF_LENGTH+MTU+8;//0x3000;	// Extra page, to allow one page past the end

#[derive(Default)]
struct Driver
{
	pio_handles: pio_ops::PioHandles,
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

	intr_bound: bool,
	bind_complete: bool,
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
			::udi::debug_printf!("bus_bind_ack");
			let (handles,irq_ack) = pio_ops::PioHandles::new(cb.gcb()).await;
			self.pio_handles = handles;

			self.intr_channel = ::udi::imc::channel_spawn::<OpsList::Irq>(cb.gcb(), self, /*interrupt number*/0.into()).await;
			let mut intr_cb = ::udi::cb::alloc::<CbList::Intr>(cb.gcb(), ::udi::get_gcb_channel().await).await;
			intr_cb.init(0.into(), 2, irq_ack);
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
					rx_buf: alloc_single(RX_BUF_CAPACITY).await,
					// DMA information for direct TX of the four TX slots
					tx_slots: [
						DmaBuf::prepare(cb.gcb(), &self.inner.dma_constraints, Some(Direction::Out)).await,
						DmaBuf::prepare(cb.gcb(), &self.inner.dma_constraints, Some(Direction::Out)).await,
						DmaBuf::prepare(cb.gcb(), &self.inner.dma_constraints, Some(Direction::Out)).await,
						DmaBuf::prepare(cb.gcb(), &self.inner.dma_constraints, Some(Direction::Out)).await,
					],
					// Bounce buffers for the TX slots
					tx_bounce: [
						alloc_single(MTU+20).await,
						alloc_single(MTU+20).await,
						alloc_single(MTU+20).await,
						alloc_single(MTU+20).await,
					],
					}
				});
			let rbstart: u32 = self.inner.dma_handles.as_ref().unwrap()
				.rx_buf.scgth().single_entry_32().expect("Environment broke the RX buffer into chunks, not allowed")
				.block_busaddr;
			// Reset the card and get the MAC address
			// SAFE: Correct DMA address
			self.mac_addr = unsafe { self.pio_handles.reset(cb.gcb(), rbstart).await? };
			::udi::debug_printf!("bus_bind_ack: mac_addr = %02X:%02X:%02X:%02X:%02X:%02X",
				self.mac_addr[0] as _,
				self.mac_addr[1] as _,
				self.mac_addr[2] as _,
				self.mac_addr[3] as _,
				self.mac_addr[4] as _,
				self.mac_addr[5] as _,
				);
			if self.intr_bound {
				for _ in 0 .. 4/*NUM_INTR_EVENT_CBS*/ {
					let intr_event_cb = ::udi::cb::alloc::<CbList::IntrEvent>(cb.gcb(), self.intr_channel.raw()).await;
					::udi::meta_bridge::event_rdy(intr_event_cb);
				}
			}
			self.bind_complete = true;
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
			if status != ::udi::ffi::UDI_OK as _ {
				// uh-oh, error
			}
			else {
				self.intr_bound = true;
				if self.bind_complete {
					for _ in 0 .. 4/*NUM_INTR_EVENT_CBS*/ {
						let intr_event_cb = ::udi::cb::alloc::<CbList::IntrEvent>(cb.gcb(), self.intr_channel.raw()).await;
						::udi::meta_bridge::event_rdy(intr_event_cb);
					}
				}
			}
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
			::udi::debug_printf!("intr_event_ind: ISR=0x%04hx", isr);
			if isr & pio_ops::FLAG_ISR_ROK != 0 {
				// RX OK
				let dma_rx_buf = &self.inner.dma_handles.as_ref().unwrap().rx_buf;
				dma_rx_buf.sync_all(cb.gcb(), ::udi::physio::dma::Direction::In).await;
				while let Ok(Some(addr)) = self.inner.pio_handles.rx_check(cb.gcb()).await
				{
					// Get the current packet length and flags
					let (flags, data) = unsafe {
						let ptr = dma_rx_buf.mem_ptr.offset(addr as isize) as *const u8;
						let flags = *ptr.offset(0) as u16 | (*ptr.offset(1) as u16) << 8;
						let raw_len = *ptr.offset(2) as u16 | (*ptr.offset(3) as u16) << 8;
						// NOTE: acess2/rust_os treat this as the packet length, while qemu seems to emit the full buffer length
						assert!(raw_len >= 4, "Raw packet lenght shorter than header");
						(flags, ::core::slice::from_raw_parts(ptr.offset(4), raw_len as usize - 4))
					};
					::udi::debug_printf!("RX packet: @0x%hx %u bytes flags=0x%04hx", addr, data.len() as u32, flags);
					assert!(data.len() > 0);
					// Pull a RX CB off the queue
					if flags & 0x0001 != 0 {
						if let Some(mut rx_cb) = self.inner.rx_cb_queue.pop() {
							let buf = rx_cb.rx_buf_mut();
							buf.write(cb.gcb(), 0..buf.len(), data).await;
							::udi::meta_nic::nsr_rx_ind(rx_cb);
						}
						else {
							// RX underrun, no CBs
						}
					}

					// Advance CAPR (header, data, alignment)
					let delta = (4 + data.len() as u16 + 3) & !3;
					let _ = self.inner.pio_handles.rx_update(cb.gcb(), delta).await;
				}
			}
			if isr & pio_ops::FLAG_ISR_RER != 0 {
				::udi::debug_printf!("TODO: Handle RX Error");
				// Advance the packet?
			}

			// TX OK or TX Error
			if isr & pio_ops::FLAG_ISR_TOK != 0 || isr & pio_ops::FLAG_ISR_TER != 0 {
				// Release TX slots until we find an unused one
				while self.inner.cur_tx_slot != self.inner.next_tx_slot {
					let slot = self.inner.cur_tx_slot as usize;
					
					let tsd = match self.inner.pio_handles.get_tsd(cb.gcb(), slot).await
						{
						Ok(tsd) => tsd,
						Err(_e) => break,
						};
					// Defensive manouver?
					if tsd & 0x8000 == 0 {
						break;
					}
					::udi::debug_printf!("TX%u TSD=%04x", slot as u32, tsd);
					mod_inc(&mut self.inner.cur_tx_slot, 4);

					let dma = self.inner.dma_handles.as_mut().unwrap();
					match self.inner.tx_cbs[slot].take()
					{
					Some(mut s) => {
						// If there was a buffer associated with the TX slot DMA handle, then pull it out and update in the CB
						if let Some(buf) = unsafe { dma.tx_slots[slot].buf_unmap(0) }
						{
							*s.cb.tx_buf_mut() = buf;
						}
						::udi::meta_nic::nsr_tx_rdy(s.cb);
					},
					None => {
						// Huh, that shouldn't happen
						::udi::debug_printf!("TX slot %u complete, but didn't have a populated CB", slot as u32);
						break;
					},
					}
				}
			}

			if isr & pio_ops::FLAG_ISR_RXOVW != 0 {
				::udi::debug_printf!("TODO: Handle RX Overflow");
			}
			if isr & pio_ops::FLAG_ISR_PUN != 0 {
				::udi::debug_printf!("TODO: Handle packet underrun");
			}
			if isr & pio_ops::FLAG_ISR_FOVW != 0 {
				::udi::debug_printf!("TODO: Handle ?RX FIFO underflow");
			}
			if isr & pio_ops::FLAG_ISR_LENCHG != 0 {
				::udi::debug_printf!("TODO: Handle 'Cable Length Changed?'");
			}
			if isr & pio_ops::FLAG_ISR_TIMEO != 0 {
				::udi::debug_printf!("TODO: Handle 'Timer Timeout?'");
			}
			if isr & pio_ops::FLAG_ISR_SERR != 0 {
				::udi::debug_printf!("TODO: Handle 'System Error'");
			}
		}
    }
}

impl ::udi::meta_nic::Control for ::udi::ChildBind<Driver,()>
{
	type Future_bind_req<'s> = impl ::core::future::Future<Output=::udi::Result<::udi::meta_nic::NicInfo>> + 's;
    fn bind_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNicBind<'a>, tx_chan_index: udi::ffi::udi_index_t, rx_chan_index: udi::ffi::udi_index_t) -> Self::Future_bind_req<'a> {
        async move {
			self.dev_mut().channel_tx = ::udi::imc::channel_spawn::<OpsList::Tx>(cb.gcb(), self, tx_chan_index).await;
			self.dev_mut().channel_rx = ::udi::imc::channel_spawn::<OpsList::Rx>(cb.gcb(), self, rx_chan_index).await;

			// Create and send 4 TX CBs
			let mut tx_cbs = ::udi::cb::alloc_batch::<CbList::NicTx>(cb.gcb(), 4, Some((1520, ::udi::ffi::buf::UDI_NULL_PATH_BUF))).await;
			while let Some(mut tx_cb) = tx_cbs.pop_front() {
				// SAFE: Channel is correct for this CB
				/*unsafe*/ {
					tx_cb.set_channel(&self.dev_mut().channel_tx);
				}
				//tx_cb.gcb.channel = self.dev_mut().channel_tx.raw();
				::udi::meta_nic::nsr_tx_rdy(tx_cb );
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

	type Future_unbind_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn unbind_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNic<'a>) -> Self::Future_unbind_req<'a> {
        async move {
			let _ = cb;
			Ok( () )
		}
    }

	type Future_enable_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn enable_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNic<'a>) -> Self::Future_enable_req<'a> {
		self.dev().pio_handles.enable(cb.gcb())
    }

	type Future_disable_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn disable_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNic<'a>) -> Self::Future_disable_req<'a> {
		self.dev().pio_handles.disable(cb.gcb())
    }

	type Future_ctrl_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
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
			use ::udi::physio::dma::Direction;
			// SAFE: Input contract that the buffer is valid
			let buf = ::core::mem::take(cb.tx_buf_mut());
			// Save the buffer length for if the `buf_map` call fails
			let len = buf.len();
			assert!(len <= MTU, "TX buffer exceeds MTU? {} > {}", len, MTU);

			let slot = self.next_tx_slot as usize;
			assert!(self.tx_cbs[slot].is_none(), "TX slot already active, too many TX CBs around?");
			mod_inc(&mut self.next_tx_slot, 4);

			let dma = self.dma_handles.as_mut().unwrap();
			let ent = match dma.tx_slots[slot].buf_map(cb.gcb(), buf, .., Direction::Out).await
				{
				// The buffer is small enough that it could be DMA'd in one chunk - nice
				Ok((scgth, complete)) => {
					assert!(complete, "Environment bug: `complete` was false");
					*scgth.single_entry_32().expect("Environment bug: TX buffer in multiple chunks")
					},
				// Cannot map - Most likely due to DMA constraints, so use the bounce buffer
				Err(_e) => {
					// SAFE: The mapping failed, but there's no path out of the above - get the buffer back
					let buf = unsafe { dma.tx_slots[slot].buf_unmap(len).unwrap() };
					// SAFE: The length is less than the size of this buffer
					let dst = unsafe { ::core::slice::from_raw_parts_mut(dma.tx_bounce[slot].mem_ptr as *mut u8, len) };
					buf.read(0, dst);
					// Return the buffer to the CB (it might have changed)
					*cb.tx_buf_mut() = buf;
					*dma.tx_bounce[slot].scgth().single_entry_32().expect("Environment bug: TX bounce buffer in multiple chunks")
				},
				};
			// SAFE: DMA is correct (assuming environment is behaving)
			unsafe {
				self.pio_handles.tx_packet(cb.gcb(), slot, ent.block_busaddr, ent.block_length as u16).await?;
			}
			self.tx_cbs[slot] = Some(TxSlot { cb });
			Ok( () )
		}
	}
}
// SAFE? Not sure if storing in `self.tx_cbs` is fully sound, as it might get completed/used while the CB is processing.
// - Depends on if the serial-ness of a region is only between async calls
unsafe impl ::udi::meta_nic::NdTx for ::udi::init::RData<Driver>
{
	type Future_tx_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn tx_req<'a>(&'a mut self, mut cb: ::udi::meta_nic::CbHandleNicTx) -> Self::Future_tx_req<'a> {
        async move {
			loop {
				let (cur_cb, next) = cb.unlink();

				match self.inner.tx_inner(cur_cb).await
				{
				Ok(()) => {},
				Err(_udi_err) => {
					// Would be nice to return the CB, but unlikely to happen so meh.
					},
				}

				match next {
				Some(next) => cb = next,
				None => break,
				}
			}
		}
    }

	type Future_exp_tx_req<'s> = Self::Future_tx_req<'s>;
    fn exp_tx_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbHandleNicTx) -> Self::Future_exp_tx_req<'a> {
        self.tx_req(cb)
    }
}
// SAFE: Just pushes to a list
unsafe impl ::udi::meta_nic::NdRx for ::udi::init::RData<Driver>
{
	type Future_rx_rdy<'s> = impl ::core::future::Future<Output=()> + 's;
    fn rx_rdy<'a>(&'a mut self, cb: ::udi::meta_nic::CbHandleNicRx) -> Self::Future_rx_rdy<'a> {
		self.rx_cb_queue.push(cb);
        async move {
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