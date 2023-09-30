#![feature(type_alias_impl_trait)]

struct Driver
{
	pio_handles: [::udi::pio::Handle; 1],
}
impl ::udi::init::Driver for Driver
{
    type Future_init = impl ::core::future::Future<Output=Driver>;

    fn init(_resouce_level: u8) -> Self::Future_init {
        async move {
			Driver { pio_handles: Default::default() }
		}
    }
}
impl ::udi::imc::ChannelHandler for Driver
{
}
impl ::udi::meta_bus::BusDevice for Driver
{
    type Future_bind_ack<'s> = impl ::core::future::Future<Output=()> + 's;

    fn bind_ack(&mut self, _dma_constraints: udi::ffi::physio::udi_dma_constraints_t, _preferred_endianness: bool, _status: udi::ffi::udi_status_t) -> Self::Future_bind_ack<'_> {
		async move {
			self.pio_handles[0] = ::udi::pio::map(0/*UDI_PCI_BAR_0*/, 0x00,0x20, &PIO_OPS_RESET, 0/*UDI_PIO_LITTLE_ENDIAN*/, 0, 0).await;

			// Spawn channel
			//self.intr_channel = ::udi::channel::spawn(::udi::get_gcb_channel().await, /*interrupt number*/0, OpsList::Irq, ::udi::get_gcb_context().await).await;
			//let intr_cb = ::udi::cb::alloc::<::udi::ffi::meta_intr::udi_intr_attach_cb_t>(Cb::Intr, ::udi::get_gcb_channel().await).await;
			//intr_cb.interrupt_index = 0;
			//intr_cb.min_event_pend = 2;
			//intr_cb.preprocessing_handle = self.pio_handles.irq_ack;
			//::udi::meta_intr::attach_req(intr_cb);
		}
    }
}
impl ::udi::meta_intr::IntrHandler for Driver
{
    type Future_intr_event_ind<'s> = impl ::core::future::Future<Output=()>+'s;

    fn intr_event_ind(&mut self, flags: u8) -> Self::Future_intr_event_ind<'_> {
		async move {
			todo!()
		}
    }
}

mod regs {
	// -- Registers present in all pages
	pub const APG_CMD  : u8 = 0x00;
	pub const APG_MEM  : u8 = 0x10;
	pub const APG_RESET: u8 = 0x1F;

	// -- Page 0
	pub const PG0_CLDA0: u8 = 0x01;
	pub const PG0_CLDA1: u8 = 0x02;
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

::udi::define_pio_ops!{PIO_OPS_RESET =
	// Send reset
	IN.B R0, regs::APG_RESET as _;
	OUT.B regs::APG_RESET as _, R0;
	// Wait for ISR[7] to set
	LABEL 1;
	 IN.B R0, regs::PG0_ISR as _;
	 AND_IMM.B R0, 0x80;
	 CSKIP.B R0 NZ;
	 BRANCH 1;
	// Write back the 0x80 currently in `R0`
	OUT.B regs::PG0_ISR as _, R0;

	// - Init pass 1
	// CMD = 0x40|0x21 [Page1, NoDMA, Stop]
	LOAD_IMM.B R0, 0x40|0x21;
	OUT.B regs::APG_CMD as _, R0;
	// CURR = First RX page
	LOAD_IMM.B R0, mem::RX_FIRST_PG;
	OUT.B regs::PG1_CURR as _, R0;
	// CMD = 0x21 [Page0, NoDMA, Stop]
	LOAD_IMM.B R0, 0x21; OUT.B regs::APG_CMD as _, R0;
	// DCR = ? [WORD, ...]
	LOAD_IMM.B R0, 0x49; OUT.B regs::PG0_DCR as _, R0;
	// IMR = 0 [Disable all]
	LOAD_IMM.B R0, 0x00; OUT.B regs::PG0_IMR as _, R0;
	// ISR = 0xFF [ACK all]
	LOAD_IMM.B R0, 0xFF; OUT.B regs::PG0_ISR as _, R0;
	// RCR = 0x20 [Monitor]
	LOAD_IMM.B R0, 0x20; OUT.B regs::PG0_RCR as _, R0;
	// TCR = 0x02 [TX Off, Loopback]
	LOAD_IMM.B R0, 0x02; OUT.B regs::PG0_TCR as _, R0;
	// - Read MAC address from EEPROM (24 bytes from 0)
	LOAD_IMM.B R0, 0;
	LOAD_IMM.B R1, 0;
	OUT.B regs::PG0_RSAR0 as _, R0;
	OUT.B regs::PG0_RSAR1 as _, R1;
	LOAD_IMM.B R0, 6*4;
	LOAD_IMM.B R1, 0;
	OUT.B regs::PG0_RBCR0 as _, R0;
	OUT.B regs::PG0_RBCR1 as _, R1;
	// CMD = 0x0A [Start remote DMA]
	LOAD_IMM.B R0, 0x0A; OUT.B regs::APG_CMD as _, R0;
	// Read MAC address
	LOAD_IMM.B R0, 0;	// R0: buffer offset
	LOAD_IMM.B R1, regs::APG_MEM as _;	// R1: Register offset (this is not incremented)
	LOAD_IMM.B R2, 6;	// R2: Iteration count (6)
	REP_IN_IND.B mem R0 STEP1, R1, R2;
	END_IMM 0;
}

mod udiprops {
	include!{ concat!(env!("OUT_DIR"), "/udiprops.rs") }
}

::udi::define_driver!{Driver;
	ops: {
		Dev : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_bus::udi_bus_device_ops_t,
		//Ctrl: Meta=udiprops::meta::udi_nic   , Op=::udi::meta_nic::OpsNum::NdCtrl,
		Irq : Meta=udiprops::meta::udi_bridge, ::udi::ffi::meta_intr::udi_intr_handler_ops_t,
		},
	cbs: {
		BusBind  : Meta=udiprops::meta::udi_bridge Num=::udi::meta_bus::CbNum::Bind,
		Intr     : Meta=udiprops::meta::udi_bridge Num=::udi::meta_bus::CbNum::IntrAttach,
		IntrEvent: Meta=udiprops::meta::udi_bridge Num=::udi::meta_bus::CbNum::IntrEvent,
		}
}
