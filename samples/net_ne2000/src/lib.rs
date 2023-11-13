#![feature(impl_trait_in_assoc_type)]

mod pio_ops;

struct Driver
{
	pio_handles: PioHandles,
	intr_channel: ::udi::imc::ChannelHandle,
	channel_tx: ::udi::imc::ChannelHandle,
	channel_rx: ::udi::imc::ChannelHandle,
	rx_cb_queue: ::udi::meta_nic::ReadCbQueue,
	mac_addr: [u8; 6],
	rx_next_page: u8,
	tx_next_page: u8,
}
impl Default for Driver {
    fn default() -> Self {    
		Driver {
			pio_handles: Default::default(),
			intr_channel: Default::default(),
			channel_tx: Default::default(),
			channel_rx: Default::default(),
			rx_cb_queue: Default::default(),
			mac_addr: [0; 6],
			rx_next_page: mem::RX_FIRST_PG,
			tx_next_page: mem::TX_FIRST,
		}
    }
}
#[derive(Default)]
struct PioHandles {
	reset: ::udi::pio::Handle,
	enable: ::udi::pio::Handle,
	disable: ::udi::pio::Handle,
	rx: ::udi::pio::Handle,
	tx: ::udi::pio::Handle,
	irq_ack: ::udi::pio::Handle,
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
		_dma_constraints: ::udi::ffi::physio::udi_dma_constraints_t,
		_preferred_endianness: ::udi::meta_bridge::PreferredEndianness,
		_status: ::udi::ffi::udi_status_t
	) -> Self::Future_bind_ack<'a> {
		async move {
			::udi::debug_printf!("NIC bus_bind_ack: self(%p)", &*self);
			let pio_map = |trans_list| ::udi::pio::map(cb.gcb(), 0/*UDI_PCI_BAR_0*/, 0x00,0x20, trans_list, 0/*UDI_PIO_LITTLE_ENDIAN*/, 0, 0.into());
			self.pio_handles.reset   = pio_map(&pio_ops::RESET).await;
			self.pio_handles.enable  = pio_map(&pio_ops::ENABLE).await;
			self.pio_handles.disable = pio_map(&pio_ops::DISBALE).await;
			self.pio_handles.rx      = pio_map(&pio_ops::RX).await;
			self.pio_handles.tx      = pio_map(&pio_ops::TX).await;
			self.pio_handles.irq_ack = pio_map(&pio_ops::IRQACK).await;

			// Spawn channel
			self.intr_channel = ::udi::imc::channel_spawn(cb.gcb(), /*interrupt number*/0.into(), OpsList::Irq as _).await;
			let mut intr_cb = ::udi::cb::alloc::<CbList::Intr>(cb.gcb(), ::udi::get_gcb_channel().await).await;
			intr_cb.interrupt_index = 0.into();
			intr_cb.min_event_pend = 2;
			intr_cb.preprocessing_handle = self.pio_handles.irq_ack.as_raw();	// NOTE: This transfers ownership
			::udi::meta_bridge::attach_req(intr_cb);
			// TODO: Does this need to wait until the attach ACKs?
			// - Probably should, just in case the operation fails
			//::udi::Error::from_status(self.intr_attach_res.wait().await)?;

			for _ in 0 .. 4/*NE2K_NUM_INTR_EVENT_CBS*/ {
				let intr_event_cb = ::udi::cb::alloc::<CbList::IntrEvent>(cb.gcb(), self.intr_channel.raw()).await;
				::udi::meta_bridge::event_rdy(intr_event_cb);
			}

			// Reset the hardware, and get the MAC address
			let d = &mut **self;
			::udi::pio::trans(cb.gcb(), &d.pio_handles.reset, 0.into(), None, Some(unsafe { ::udi::pio::MemPtr::new(&mut d.mac_addr) })).await?;

			::udi::debug_printf!("NIC bus_bind_ack (RET): %p", &*self);
			// Binding is complete!
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
			if cb.intr_result & 0x01 != 0 {
				// RX complete
				// - Pop a RX CB off the list
				if let Some(mut rx_cb) = self.rx_cb_queue.pop() {
					let mut buf = unsafe { ::udi::buf::Handle::from_raw(rx_cb.rx_buf) };
					// Ensure that it's big enough for an entire packet
					buf.ensure_size(cb.gcb(), 1520).await;
					// Pull the packet off the device
					match ::udi::pio::trans(
						cb.gcb(), &self.inner.pio_handles.rx, 0.into(),
						Some(&mut buf), Some(unsafe { ::udi::pio::MemPtr::new(::core::slice::from_mut(&mut self.inner.rx_next_page)) })
					).await
					{
					Ok(res) => {
						// If that succeeded, then set the size and hand to the NSR
						buf.truncate(res as usize);
						rx_cb.rx_buf = buf.into_raw();
						::udi::meta_nic::nsr_rx_ind(rx_cb);
						},
					Err(_e) => {
						// Otherwise, return the cb to the queue
						self.rx_cb_queue.push(rx_cb);
						}
					}
				}
				else {
					// RX undeflow :(
					// - TODO: Flush from the device
				}
			}
			if cb.intr_result & 0x02 != 0 {
				// TX complete
			}
			if cb.intr_result & 0x04 != 0 {
				// RX complete, but had errors
			}
			if cb.intr_result & 0x08 != 0 {
				// Transmission halted due to excessive collisions
			}
			if cb.intr_result & 0x10 != 0 {
				// RX buffer exhausted
			}
			if cb.intr_result & 0x40 != 0 {
				// Remote DMA is complete
			}
			if cb.intr_result & 0x80 != 0 {
				// Card reset complet
				// - ignore
			}
		}
    }
}

impl ::udi::meta_nic::Control for ::udi::ChildBind<Driver,()>
{
	type Future_bind_req<'s> = impl ::core::future::Future<Output=::udi::Result<::udi::meta_nic::NicInfo>> + 's;
    fn bind_req<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNicBind<'a>, tx_chan_index: udi::ffi::udi_index_t, rx_chan_index: udi::ffi::udi_index_t) -> Self::Future_bind_req<'a> {
        async move {
			::udi::debug_printf!("NIC bind_req: %p %p", &*self, self.dev());
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
			// Close the channels?
			//::udi::imc::
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
			// Linked list of cbs for multiple packets
			loop {
				let mut buf = unsafe { ::udi::buf::Handle::take_raw(&mut cb.tx_buf) };

				let len = (buf.len() as u16).to_ne_bytes();
				let mut mem_buf = [len[0], len[1], self.tx_next_page];
				let mem_ptr = unsafe { ::udi::pio::MemPtr::new(&mut mem_buf) };
				self.tx_next_page += ((buf.len() + 256-1) / 256) as u8;
				if self.tx_next_page > mem::TX_LAST {
					self.tx_next_page -= mem::TX_BUF_SIZE;
				}
				match ::udi::pio::trans(cb.gcb(), &self.pio_handles.tx, ::udi::ffi::udi_index_t(0), Some(&mut buf), Some(mem_ptr)).await
				{
				Ok(_) => {},
				Err(_) => {},
				}
				buf.free();

				let next = if cb.chain.is_null() {
					None
				}
				else {
					let n = cb.chain;
					cb.chain = ::core::ptr::null_mut();
					Some( unsafe { ::udi::meta_nic::CbHandleNicTx::from_raw(n) } )
				};
				::udi::meta_nic::nsr_tx_rdy(cb);
				if let Some(next) = next {
					cb = next;
				}
				else {
					break;
				}
			}
			todo!()
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

mod regs {
	// -- Registers present in all pages
	pub const APG_CMD  : u8 = 0x00;
	pub const APG_MEM  : u8 = 0x10;
	pub const APG_RESET: u8 = 0x1F;

	// -- Page 0
	pub const PG0_CLDA0: u8 = 0x01;	// When read, PSTART when written
	pub const PG0_CLDA1: u8 = 0x02;	// When read, PSTOP when written
	/// Boundary Pointer (for ringbuffer)
	pub const PG0_BNRY : u8 = 0x03;
	/// - READ: Transmit Status Register
	pub const PG0R_TSR   : u8 = 0x04;	// When read, TPSR when written
	/// - WRITE: Transmit Page Start address Register
	pub const PG0W_TPSR  : u8 = PG0R_TSR;
	pub const PG0R_NCR  : u8 = 0x05;	// TBCR0 when wrtiten
	pub const PG0W_TBCR0: u8 = PG0R_NCR;
	pub const PG0R_FIFO : u8 = 0x06;	// TBCR1 when wrtiten
	pub const PG0W_TBCR1: u8 = PG0R_FIFO;
	pub const PG0_ISR  : u8 = 0x07;
	/// Remote Start AddRess (Lo)
	pub const PG0_RSAR0: u8 = 0x08;
	/// Remote Start AddRess (Hi)
	pub const PG0_RSAR1: u8 = 0x09;
	/// Remote Byte Count (Lo)
	pub const PG0_RBCR0: u8 = 0x0A;
	/// Remote Byte Count (Hi)
	pub const PG0_RBCR1: u8 = 0x0B;
	/// Receive Config Register
	pub const PG0_RCR  : u8 = 0x0C;
	/// Transmit Config Register
	pub const PG0_TCR  : u8 = 0x0D;
	/// Data Config Register
	pub const PG0_DCR  : u8 = 0x0E;
	/// Interrupt Mask Register
	pub const PG0_IMR  : u8 = 0x0F;

	// -- Page 1
	pub const PG1_CURR: u8 = 7;
}

mod mem {
	// Hardware limit?
	pub const MEM_START: u8 = 0x40;
	pub const MEM_END  : u8 = 0xC0;

	// Internal values
	pub const RX_BUF_SIZE: u8 = 0x40;
	pub const TX_BUF_SIZE: u8 = 0x40;

	pub const RX_FIRST_PG: u8 = MEM_START;
	pub const RX_LAST_PG : u8 = MEM_START + RX_BUF_SIZE - 1;

	pub const TX_FIRST: u8 = MEM_START+RX_BUF_SIZE;
	pub const TX_LAST: u8 = MEM_END;
}


mod udiprops {
	include!{ concat!(env!("OUT_DIR"), "/udiprops.rs") }
}

::udi::define_driver!{Driver;
	ops: {
		// TODO: How to enforce the right mapping to metalangs?
		Dev : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_bridge::udi_bus_device_ops_t,
		Ctrl: Meta=udiprops::meta::udi_nic   , ::udi::ffi::meta_nic::udi_nd_ctrl_ops_t : ChildBind<_,()>,
		Tx  : Meta=udiprops::meta::udi_nic   , ::udi::ffi::meta_nic::udi_nd_tx_ops_t,
		Rx  : Meta=udiprops::meta::udi_nic   , ::udi::ffi::meta_nic::udi_nd_rx_ops_t,
		Irq : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_bridge::udi_intr_handler_ops_t,
		},
	cbs: {
		BusBind  : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_bridge::udi_bus_bind_cb_t,
		Intr     : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_bridge::udi_intr_attach_cb_t,
		IntrEvent: Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_bridge::udi_intr_event_cb_t,

		_IntrDetach: Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_bridge::udi_intr_detach_cb_t,

		Nic    : Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_cb_t,
		NicBind: Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_bind_cb_t,
		NicCtrl: Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_ctrl_cb_t,
		NicInfo: Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_info_cb_t,
		NicTx  : Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_tx_cb_t,
		NicRx  : Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_rx_cb_t,
		}
}
