#![feature(impl_trait_in_assoc_type)]

mod pio_ops;

struct Driver
{
	pio_handles: PioHandles,
	intr_channel: ::udi::ffi::udi_channel_t,
	mac_addr: [u8; 6],
}
#[derive(Default)]
struct PioHandles {
	reset: ::udi::pio::Handle,
	enable: ::udi::pio::Handle,
	irq_ack: ::udi::pio::Handle,
}
::udi::define_wrappers! {Driver: DriverIrq DriverNicCtrl}

impl ::udi::init::Driver for Driver
{
    type Future_init = impl ::core::future::Future<Output=Driver>;

    fn init(_resouce_level: u8) -> Self::Future_init {
        async move {
			Driver {
				pio_handles: Default::default(),
				intr_channel: ::core::ptr::null_mut(),
				mac_addr: [0; 6],
			}
		}
    }
}
impl ::udi::meta_bus::BusDevice for Driver
{
    type Future_channel_event_ind<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn channel_event_ind(&mut self, cb: ::udi::imc::ChannelEventCb) -> Self::Future_channel_event_ind<'_> {
		async move {
			//self.active_cb = ::udi::get_cur_cb_raw();
			//let bind_cb = ::udi::imc::get_cb_bind_cb::<::udi::ffi::meta_bus::udi_bus_bind_cb_t>();
			//udi_bus_bind_req(bind_cb);
			Ok( () )
		}
    }

    type Future_bind_ack<'s> = impl ::core::future::Future<Output=()> + 's;
    fn bus_bind_ack(&mut self, _dma_constraints: udi::ffi::physio::udi_dma_constraints_t, _preferred_endianness: bool, _status: udi::ffi::udi_status_t) -> Self::Future_bind_ack<'_> {
		async move {
			let pio_map = |trans_list| ::udi::pio::map(0/*UDI_PCI_BAR_0*/, 0x00,0x20, trans_list, 0/*UDI_PIO_LITTLE_ENDIAN*/, 0, 0);
			self.pio_handles.reset   = pio_map(&pio_ops::RESET).await;
			self.pio_handles.enable  = pio_map(&pio_ops::ENABLE).await;
			self.pio_handles.irq_ack = pio_map(&pio_ops::IRQACK).await;

			// Spawn channel
			self.intr_channel = ::udi::imc::channel_spawn(::udi::get_gcb_channel().await, /*interrupt number*/0, OpsList::Irq as _, ::udi::get_gcb_context().await).await;
			let mut intr_cb = ::udi::cb::alloc::<Cbs::Intr>(::udi::get_gcb_channel().await).await;
			intr_cb.interrupt_index = 0;
			intr_cb.min_event_pend = 2;
			intr_cb.preprocessing_handle = self.pio_handles.irq_ack.as_raw();
			::udi::meta_intr::attach_req(intr_cb);
			// TODO: Does this need to wait until the attach ACKs?
			// - Probably should, just in case the operation fails

			for _ in 0 .. 4/*NE2K_NUM_INTR_EVENT_CBS*/ {
				let intr_event_cb = ::udi::cb::alloc::<Cbs::IntrEvent>(::udi::get_gcb_channel().await).await;
				::udi::meta_intr::event_rdy(intr_event_cb);
			}

			// Reset the hardware, and get the MAC address
			match ::udi::pio::trans(&self.pio_handles.reset, 0, None, Some(unsafe { ::udi::pio::MemPtr::new(&mut self.mac_addr) })).await
			{
			Ok(_) => {},
			Err(_) => {},
			}

			// Binding is complete!
		}
    }

    type Future_unbind_ack<'s> = impl ::core::future::Future<Output=()> + 's;
    fn bus_unbind_ack(&mut self) -> Self::Future_unbind_ack<'_> {
        async move {
			todo!()
		}
    }

    type Future_intr_attach_ack<'s> = impl ::core::future::Future<Output=()> + 's;

    fn intr_attach_ack(&mut self, status: udi::ffi::udi_status_t) -> Self::Future_intr_attach_ack<'_> {
        async move {
			if status != 0 {
				// TODO: Free the CB?
			}
			//self.intr_attach_cb = ::udi::get_gcb();
		}
    }

    type Future_intr_detach_ack<'s> = impl ::core::future::Future<Output=()> + 's;

    fn intr_detach_ack(&mut self) -> Self::Future_intr_detach_ack<'_> {
        async move {
			todo!()
		}
    }
}
impl ::udi::meta_intr::IntrHandler for DriverIrq
{
    type Future_channel_event_ind<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn channel_event_ind(&mut self, cb: ::udi::imc::ChannelEventCb) -> Self::Future_channel_event_ind<'_> {
		async move { Ok(()) }
	}

    type Future_intr_event_ind<'s> = impl ::core::future::Future<Output=()>+'s;
    fn intr_event_ind(&mut self, flags: u8) -> Self::Future_intr_event_ind<'_> {
		async move {
			// TODO: Get the interrupt result from the cb
			todo!()
		}
    }
}

impl ::udi::meta_nic::Control for DriverNicCtrl
{
    type Future_channel_event_ind<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn channel_event_ind(&mut self, cb: ::udi::imc::ChannelEventCb) -> Self::Future_channel_event_ind<'_> {
		async move { Ok(()) }
	}

	type Future_bind_req<'s> = impl ::core::future::Future<Output=::udi::ffi::udi_status_t> + 's;
    fn bind_req(&mut self, tx_chan_index: udi::ffi::udi_index_t, rx_chan_index: udi::ffi::udi_index_t) -> Self::Future_bind_req<'_> {
        async move { todo!() }
    }

	type Future_unbind_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn unbind_req(&mut self) -> Self::Future_unbind_req<'_> {
        async move { todo!() }
    }

	type Future_enable_req<'s> = impl ::core::future::Future<Output=::udi::Result<()>> + 's;
    fn enable_req(&mut self) -> Self::Future_enable_req<'_> {
        async move {
			::udi::pio::trans(&self.0.pio_handles.enable, 0, None, None).await?;
			Ok( () )
		}
    }

	type Future_disable_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn disable_req(&mut self) -> Self::Future_disable_req<'_> {
        async move { todo!() }
    }

	type Future_ctrl_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn ctrl_req(&mut self) -> Self::Future_ctrl_req<'_> {
        async move { todo!() }
    }

	type Future_info_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn info_req(&mut self, reset_statistics: bool) -> Self::Future_info_req<'_> {
        async move { todo!() }
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
	/// - WRITE: Transmit Page Start address Register
	pub const PG0_TSR  : u8 = 0x04;	// When read, TPSR when written
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

	pub const RX_FIRST_PG: u8 = MEM_START;
	pub const RX_LAST_PG : u8 = MEM_START + RX_BUF_SIZE - 1;
}


mod udiprops {
	include!{ concat!(env!("OUT_DIR"), "/udiprops.rs") }
}

::udi::define_driver!{Driver;
	ops: {
		// TODO: How to enforce the right mapping to metalangs?
		Dev : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_bus::udi_bus_device_ops_t,
		Ctrl: Meta=udiprops::meta::udi_nic   , ::udi::meta_nic::ffi::udi_nd_ctrl_ops_t : DriverNicCtrl,
		Irq : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_intr::udi_intr_handler_ops_t : DriverIrq,
		},
	cbs: {
		BusBind  : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_bus::udi_bus_bind_cb_t,
		Intr     : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_intr::udi_intr_attach_cb_t,
		IntrEvent: Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_intr::udi_intr_event_cb_t,
		}
}
