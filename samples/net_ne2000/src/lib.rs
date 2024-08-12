#![feature(impl_trait_in_assoc_type)]
use ::core::cell::Cell;
use ::core::cell::OnceCell;

mod pio_ops;

#[derive(Default)]
struct Driver
{
	init: OnceCell<Init>,
	intr_bound: ::udi::async_helpers::Wait< ::udi::Result<()> >,
	rx_cb_queue: ::udi::meta_nic::ReadCbQueue,
	channels: OnceCell<Channels>,
	vals: Vals,
}
struct Init {
	pio_handles: pio_ops::PioHandles,
	#[allow(dead_code)]	// Only here to hold the handle
	intr_channel: ::udi::imc::ChannelHandle,
	mac_addr: [u8; 6],
}
struct Channels {
	#[allow(dead_code)]	// Only here to hold the handle
	tx: ::udi::imc::ChannelHandle,
	#[allow(dead_code)]	// Only here to hold the handle
	rx: ::udi::imc::ChannelHandle,
}
struct Vals {
	rx_next_page: Cell<u8>,
	tx_next_page: Cell<u8>,
}
impl Default for Vals {
    fn default() -> Self {    
		Vals {
			rx_next_page: Cell::new(mem::RX_FIRST_PG),
			tx_next_page: Cell::new(mem::TX_FIRST),
		}
    }
}
impl Driver {
	fn mac_addr(&self) -> &[u8; 6] {
		&self.init.get().expect("Not bound to bus").mac_addr
	}
	fn pio_handles(&self) -> &pio_ops::PioHandles {
		&self.init.get().expect("Not bound to bus").pio_handles
	}
}

impl ::udi::init::Driver for ::udi::init::RData<Driver>
{
	const MAX_ATTRS: u8 = 4;

    type Future_init<'s> = impl ::core::future::Future<Output=()> + 's;
    fn usage_ind<'s>(&'s self, _cb: ::udi::meta_mgmt::CbRefUsage<'s>, _resouce_level: u8) -> Self::Future_init<'s> {
        async move {
		}
    }

    type Future_enumerate<'s> = impl ::core::future::Future<Output=(::udi::init::EnumerateResult,::udi::init::AttrSink<'s>)> + 's;
    fn enumerate_req<'s>(
		&'s self,
		_cb: ::udi::init::CbRefEnumerate<'s>,
		level: ::udi::init::EnumerateLevel,
		mut attrs_out: ::udi::init::AttrSink<'s>
	) -> Self::Future_enumerate<'s> {
        async move {
			match level
			{
			::udi::init::EnumerateLevel::Start
			|::udi::init::EnumerateLevel::StartRescan => {
				let mac_addr = self.mac_addr();
				attrs_out.push_u32("if_num", 0);
				attrs_out.push_string("if_media", "eth");
				attrs_out.push_string_fmt("identifier", format_args!("{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
					mac_addr[0], mac_addr[1], mac_addr[2],
					mac_addr[3], mac_addr[4], mac_addr[5],
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
    fn devmgmt_req<'s>(&'s self, _cb: ::udi::init::CbRefMgmt<'s>, mgmt_op: udi::init::MgmtOp, _parent_id: ::udi::ffi::udi_ubit8_t) -> Self::Future_devmgmt<'s> {
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
		&'a self,
		cb: ::udi::meta_bridge::CbRefBind<'a>,
		_dma_constraints: ::udi::physio::dma::DmaConstraints,
		_preferred_endianness: ::udi::meta_bridge::PreferredEndianness,
		_status: ::udi::ffi::udi_status_t
	) -> Self::Future_bind_ack<'a> {
		async move {
			::udi::debug_printf!("NIC bus_bind_ack: self(%p)", &*self);
			let (pio_handles, pio_irq_ack) = pio_ops::PioHandles::new(cb.gcb()).await;

			// Save the channel used to bind to the parent, so we can unbind later on.
			//self.parent_channel = cb.gcb.channel;
			//::udi::debug_printf!("parent_channel=%p", self.parent_channel);

			// Spawn channel
			let intr_channel = ::udi::imc::channel_spawn::<OpsList::Irq>(cb.gcb(), self, /*interrupt number*/0.into()).await;
			::udi::meta_bridge::intr_attach_req({
				let mut intr_cb = ::udi::cb::alloc::<CbList::Intr>(cb.gcb(), ::udi::get_gcb_channel().await).await;
				intr_cb.init(0.into(), 2, pio_irq_ack);	// NOTE: This transfers ownership
				intr_cb});
			self.intr_bound.wait(cb.gcb()).await?;

			for _ in 0 .. 4/*NE2K_NUM_INTR_EVENT_CBS*/ {
				let intr_event_cb = ::udi::cb::alloc::<CbList::IntrEvent>(cb.gcb(), intr_channel.raw()).await;
				::udi::meta_bridge::intr_event_rdy(intr_event_cb);
			}

			// Reset the hardware, and get the MAC addres
			let mut mac_addr = [0; 6];
			pio_handles.reset( cb.gcb(), &mut mac_addr ).await?;

			if let Err(_) = self.init.set(Init {
				pio_handles,
				intr_channel,
				mac_addr,
			}) {
				panic!("Bound twice?")
			}

			::udi::debug_printf!("NIC bus_bind_ack (RET): %p", &*self);
			// Binding is complete!
			Ok( () )
		}
    }

    type Future_unbind_ack<'s> = impl ::core::future::Future<Output=()> + 's;
    fn bus_unbind_ack<'a>(&'a self, _cb: ::udi::meta_bridge::CbRefBind<'a>) -> Self::Future_unbind_ack<'a> {
        async move {
		}
    }

    type Future_intr_attach_ack<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_attach_ack<'a>(&'a self, cb: ::udi::meta_bridge::CbRefIntrAttach<'a>, status: udi::ffi::udi_status_t) -> Self::Future_intr_attach_ack<'a> {
		let _ = cb;
        async move {
			self.intr_bound.signal(::udi::Error::from_status(status))
		}
    }

    type Future_intr_detach_ack<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_detach_ack<'a>(&'a self, cb: ::udi::meta_bridge::CbRefIntrDetach<'a>) -> Self::Future_intr_detach_ack<'a> {
		let _ = cb;
		async move {
		}
    }
}
impl ::udi::meta_bridge::IntrHandler for ::udi::init::RData<Driver>
{
    type Future_intr_event_ind<'s> = impl ::core::future::Future<Output=()>+'s;
    fn intr_event_ind<'a>(&'a self, cb: ::udi::meta_bridge::CbRefEvent<'a>, _flags: u8) -> Self::Future_intr_event_ind<'a> {
		async move {
			if cb.intr_result & 0x01 != 0 {
				// RX complete
				// - Pop a RX CB off the list
				if let Some(mut rx_cb) = self.rx_cb_queue.pop() {
					let mut buf = rx_cb.rx_buf_mut();
					// Ensure that it's big enough for an entire packet
					buf.ensure_size(cb.gcb(), 1520).await;
					// Pull the packet off the device
					let mut page = self.inner.vals.rx_next_page.get();
					match self.inner.pio_handles().rx(cb.gcb(), &mut buf, &mut page).await
					{
					Ok(res) => {
						self.inner.vals.rx_next_page.set(page);
						// If that succeeded, then set the size and hand to the NSR
						buf.truncate(res);
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
    fn bind_req<'a>(&'a self, cb: ::udi::meta_nic::CbRefNicBind<'a>, tx_chan_index: udi::ffi::udi_index_t, rx_chan_index: udi::ffi::udi_index_t) -> Self::Future_bind_req<'a> {
        async move {
			::udi::debug_printf!("NIC bind_req: %p %p", &*self, self.dev());
			if let Err(_) = self.dev().channels.set(Channels { 
				tx: ::udi::imc::channel_spawn::<OpsList::Tx>(cb.gcb(), self, tx_chan_index).await,
				rx: ::udi::imc::channel_spawn::<OpsList::Rx>(cb.gcb(), self, rx_chan_index).await
			}) {
				panic!("Bound twice?")
			}
			let mac_addr = self.dev().mac_addr();
			::udi::debug_printf!("NIC mac_addr = %02x:%02x:%02x:%02x:%02x:%02x",
				mac_addr[0] as _, mac_addr[1] as _, mac_addr[2] as _,
				mac_addr[3] as _, mac_addr[4] as _, mac_addr[5] as _,
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
					mac_addr[0], mac_addr[1], mac_addr[2],
					mac_addr[3], mac_addr[4], mac_addr[5],
					0,0,0,0,
					0,0,0,0,0,0,0,0,0,0,
				],
			})
		}
    }

	type Future_unbind_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn unbind_req<'a>(&'a self, cb: ::udi::meta_nic::CbRefNic<'a>) -> Self::Future_unbind_req<'a> {
        async move {
			let _ = cb;
			// Close the channels?
			//::udi::imc::
			todo!();
		}
    }

	type Future_enable_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn enable_req<'a>(&'a self, cb: ::udi::meta_nic::CbRefNic<'a>) -> Self::Future_enable_req<'a> {
        async move {
			self.dev().pio_handles().enable( cb.gcb() ).await?;
			Ok( () )
		}
    }

	type Future_disable_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn disable_req<'a>(&'a self, cb: ::udi::meta_nic::CbRefNic<'a>) -> Self::Future_disable_req<'a> {
        async move {
			self.dev().pio_handles().disable( cb.gcb() ).await;
		}
    }

	type Future_ctrl_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn ctrl_req<'a>(&'a self, _cb: ::udi::meta_nic::CbRefNicCtrl<'a>) -> Self::Future_ctrl_req<'a> {
        async move { todo!() }
    }

	type Future_info_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn info_req<'a>(&'a self, _cb: ::udi::meta_nic::CbRefNicInfo<'a>, _reset_statistics: bool) -> Self::Future_info_req<'a> {
        async move { todo!() }
    }
}

unsafe impl ::udi::meta_nic::NdTx for ::udi::init::RData<Driver>
{
	type Future_tx_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn tx_req<'a>(&'a self, mut cb: ::udi::meta_nic::CbHandleNicTx) -> Self::Future_tx_req<'a> {
        async move {
			let mut rv: Option<::udi::meta_nic::CbHandleNicTx> = None;
			// Linked list of cbs for multiple packets
			loop {
				let (mut cur_cb, next) = cb.unlink();
				let mut buf = ::core::mem::take(cur_cb.tx_buf_mut());

				let page = {
					let p = self.vals.tx_next_page.get();
					let mut next_p = p.wrapping_add( ((buf.len() + 256-1) / 256) as u8 );
					if next_p > mem::TX_LAST {
						next_p -= mem::TX_BUF_SIZE;
					}
					self.vals.tx_next_page.set(next_p);
					p
				};
				match self.pio_handles().tx(cur_cb.gcb(), &mut buf, page).await
				{
				Ok(_) => {},
				Err(_) => {},
				}
				*cur_cb.tx_buf_mut() = buf;
				//buf.free();

				if let Some(ref mut d) = rv {
					d.link_front(cur_cb);
				}
				else {
					rv = Some(cur_cb);
				}
				if let Some(next) = next {
					cb = next;
				}
				else {
					break;
				}
			}
			if let Some(cb) = rv {
				::udi::meta_nic::nsr_tx_rdy(cb);
			}
		}
    }

	type Future_exp_tx_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn exp_tx_req<'a>(&'a self, cb: ::udi::meta_nic::CbHandleNicTx) -> Self::Future_exp_tx_req<'a> {
        self.tx_req(cb)
    }
}
unsafe impl ::udi::meta_nic::NdRx for ::udi::init::RData<Driver>
{
	type Future_rx_rdy<'s> = impl ::core::future::Future<Output=()> + 's;
    fn rx_rdy<'a>(&'a self, cb: ::udi::meta_nic::CbHandleNicRx) -> Self::Future_rx_rdy<'a> {
		self.rx_cb_queue.push(cb);
        async move {}
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


pub mod udiprops {
	include!{ concat!(env!("OUT_DIR"), "/udiprops.rs") }
}

::udi::define_driver!{Driver;
	ops: {
		Dev : ::udi::ffi::meta_bridge@udi_bus_device_ops_t,
		Ctrl: ::udi::ffi::meta_nic@udi_nd_ctrl_ops_t : ChildBind<_,()>,
		Tx  : ::udi::ffi::meta_nic@udi_nd_tx_ops_t,
		Rx  : ::udi::ffi::meta_nic@udi_nd_rx_ops_t,
		Irq : ::udi::ffi::meta_bridge@udi_intr_handler_ops_t,
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
