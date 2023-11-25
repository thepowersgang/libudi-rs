use ::core::future::Future;
use ::udi::FutureExt;

type Gcb<'a> = ::udi::CbRef<'a, ::udi::ffi::udi_cb_t>;

// TODO: Move this to within `pio_ops` and use privacy to handle safety
#[derive(Default)]
pub struct PioHandles {
	reset: ::udi::pio::Handle,
	//irq_ack: ::udi::pio::Handle,
	enable: ::udi::pio::Handle,
	disable: ::udi::pio::Handle,

	tx: ::udi::pio::Handle,
	get_tsd: ::udi::pio::Handle,

    rx_check: ::udi::pio::Handle,
    rx_update: ::udi::pio::Handle,
}
impl PioHandles {
    pub fn new(gcb: ::udi::CbRef<::udi::ffi::udi_cb_t>) -> impl Future<Output=(Self,::udi::pio::Handle)> + '_ {
        async move {
            let pio_map = |trans_list|
                ::udi::pio::map(gcb, 0/*UDI_PCI_BAR_0*/, 0x00,0xFF, trans_list,
                    ::udi::ffi::pio::UDI_PIO_LITTLE_ENDIAN, 0, 0.into());
            let irq_ack = pio_map(&IRQACK).await;
            let handles = PioHandles {
                reset   : pio_map(&RESET).await,
                enable  : pio_map(&ENABLE).await,
                disable : pio_map(&DISBALE).await,
                tx      : pio_map(&TX).await,
                get_tsd : pio_map(&GET_TSD).await,

                rx_check : pio_map(&RX_CHECK).await,
                rx_update: pio_map(&RX_UPDATE).await,
            };
            (handles,irq_ack)
        }
    }
    /// Reset the card and set RBStart
    /// 
    /// SAFETY: rbstart is a DMA address, caller must ensure validity
    pub unsafe fn reset<'a>(&'a self, gcb: Gcb<'a>, rbstart: u32) -> impl Future<Output=::udi::Result<[u8; 6]>> + 'a {
        async move {
            let mut reset_data = MemReset::new(rbstart);
            ::udi::pio::trans(gcb, &self.reset, Default::default(), None, Some(reset_data.get_ptr())).await?;
            Ok(reset_data.mac)
        }
    }

    pub fn enable<'a>(&'a self, gcb: Gcb<'a>) -> impl Future<Output=::udi::Result<()>> + 'a {
        ::udi::pio::trans(gcb, &self.enable, ::udi::ffi::udi_index_t(0), None, None)
            .map(|res| res.map(|_| ()))
    }
    pub fn disable<'a>(&'a self, gcb: Gcb<'a>) -> impl Future<Output=()> + 'a {
        ::udi::pio::trans(gcb, &self.disable, ::udi::ffi::udi_index_t(0), None, None)
            .map(|res| match res {
                Ok(_) => (),
                Err(e) => {
                    ::udi::debug_printf!("ERROR: Error disabling card - %x", e.into_inner());
                },
            })
    }
    /// Pass a buffer to the card for TX in the given slot
    /// 
    /// SAFETY: Takes DMA addresses, caller must ensure that addresses are valid until TX completes
    pub unsafe fn tx_packet<'a>(&'a self, gcb: Gcb<'a>, slot: usize, addr: u32, len: u16) -> impl Future<Output=::udi::Result<()>> + 'a {
        assert!(slot < 4);
        async move {
			let mut mem = MemTx {
				addr,
				len,
				index: slot as u8,
			};
			::udi::pio::trans(gcb, &self.tx, Default::default(), None, Some(unsafe { mem.get_ptr() })).await?;
            Ok(())
        }
    }
    /// Get the TSD value for a given TX slot
    pub fn get_tsd<'a>(&'a self, gcb: Gcb<'a>, slot: usize) -> impl Future<Output=::udi::Result<u32>> + 'a {
        assert!(slot < 4);
        ::udi::pio::trans(gcb, &self.get_tsd, (slot as u8).into(), None, None)
            .map(|res| match res
                {
                Ok(tsd) => Ok(tsd as u32),
                Err(e) => Err(e),
                })
    }

    pub fn rx_check(&'_ self, gcb: Gcb<'_>) -> impl Future<Output=::udi::Result<Option<u16>>> + '_ {
        ::udi::pio::trans(gcb, &self.rx_check, Default::default(), None, None)
            .map(|res| match res
                {
                Ok(0xFFFF) => Ok(None),
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(e),
                })
    }
    pub fn rx_update<'a>(&'a self, gcb: Gcb<'a>, delta: u16) -> impl Future<Output=::udi::Result<()>> + 'a {
        assert!(delta < 0x2000);
        async move {
            let mut mem = MemRxUpdate { delta };
            ::udi::pio::trans(gcb, &self.rx_update, Default::default(), None, Some(mem.get_ptr()))
                .map(|res| match res
                    {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                    }).await
        }
    }
}


#[repr(u8)]
enum Regs {
    /// Transmit Status of Descriptors 0 - 3
    Tsd0 = 0x10,
    /// Transmit Start Address(es)
    Tsad0 = 0x20,
    /// Recieve Buffer Start (DWord)
    RBStart = 0x30,
    Cmd = 0x37,
    /// Current address of packet read
    Capr = 0x38,
    /// Current Buffer Address - Total byte count in RX buffer
    Cba = 0x3A,
    /// Interrupt Mask Register
    Imr = 0x3C,
    /// Interrupt Status Register
    Isr = 0x3E,
    /// Recieve Configuration Register
    Rcr = 0x44,
    //Config0 = 0x51,
    Config1 = 0x52,
}

/// System error
pub const FLAG_ISR_SERR  : u16 = 0x8000;
/// Timer timeout (See TIMERINT)
pub const FLAG_ISR_TIMEO : u16 = 0x4000;
/// Cable length changed
pub const FLAG_ISR_LENCHG: u16 = 0x2000;
/// Rx FIFO Underflow
pub const FLAG_ISR_FOVW  : u16 = 0x0040;
/// Packet Underrun
pub const FLAG_ISR_PUN   : u16 = 0x0020;
/// Rx Buffer Overflow
pub const FLAG_ISR_RXOVW : u16 = 0x0010;
/// Tx Error
pub const FLAG_ISR_TER   : u16 = 0x0008;
/// Tx OK
pub const FLAG_ISR_TOK   : u16 = 0x0004;
/// Rx Error
pub const FLAG_ISR_RER   : u16 = 0x0002;
/// Rx OK
pub const FLAG_ISR_ROK   : u16 = 0x0001;

#[repr(C)]
struct MemReset {
    rbstart: u32,
    mac: [u8; 6],
    _pad: [u8; 2],
}
impl MemReset {
    /// SAFETY: DMA addresses are included, caller must ensure safe DMA
    pub unsafe fn new(rbstart: u32) -> Self {
        Self {
            rbstart,
            mac: [0; 6],
            _pad: [0; 2],
        }
    }
    pub fn get_ptr(&mut self) -> ::udi::pio::MemPtr {
        // SAFE: Correct size for the operation, and structure has no padding fields
        unsafe {
            ::udi::pio::MemPtr::new(
                ::core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, ::core::mem::size_of::<Self>())
            )
        }
    }
}
::udi::define_pio_ops!{pub RESET =
    // - Get the MAC address from the first six bytes of register space
	LOAD_IMM.B R0, 4;	// R0: buffer offset
	LOAD_IMM.B R1, 0;	// R1: Register offset
	LOAD_IMM.B R2, 6;	// R2: Iteration count (6)
	REP_IN_IND.B [mem R0 STEP1], R1 STEP1, R2;

    // - Reset Sequence
    // > Power on
    LOAD_IMM.B R0, 0x00;
    OUT.B Regs::Config1 as _, R0;
    // > Reset and wait for the reset bit to clear
    LOAD_IMM.B R0, 0x10;
    OUT.B Regs::Cmd as _ ,R0;
    LABEL 1;
    IN.B R0, Regs::Cmd as _;
    AND_IMM.B R0, 0x10;
    CSKIP.B R0 Z;
    BRANCH 1;
    // > Reset complete
    // > Disable interrupts
    LOAD_IMM.S R0, 0x00;
    OUT.S Regs::Imr as _, R0;
    
    // - RX buffer
    LOAD_IMM.B R0, 0;
    LOAD.L R0, [mem R0];
    OUT.L Regs::RBStart as _, R0;
    LOAD_IMM.B R0, 0;
    OUT.S Regs::Capr as _ , R0;
    //OUT.S Regs::Cba as _ , R0;    // Technically read-only

    // - RCR - hw::RCR_DMA_BURST_1024|hw::RCR_BUFSZ_8K16|hw::RCR_FIFO_1024|hw::RCR_OVERFLOW|0x1F
    LOAD_IMM.S R0, ((6<<13)|(0<<11)|(6<<8)|0x80|0x1F);
    OUT.L Regs::Rcr as _, R0;
    // - Enable the RX and TX engines
    LOAD_IMM.B R0, 0x0C;
    OUT.B Regs::Cmd as _, R0;

	END_IMM 0;
}

::udi::define_pio_ops!{pub ENABLE =
    END_IMM 0;
}
::udi::define_pio_ops!{pub DISBALE =
    END_IMM 0;
}

#[repr(C)]
struct MemTx {
    addr: u32,
    len: u16,
    index: u8,
}
impl MemTx {
    /// SAFETY: DMA addresses are included, caller must ensure safe DMA
    pub unsafe fn get_ptr(&mut self) -> ::udi::pio::MemPtr {
        ::udi::pio::MemPtr::new(
            ::core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, ::core::mem::size_of::<Self>())
        )
    }
}
::udi::define_pio_ops!{pub TX =
    // - Read the address/length/index from the input
    LOAD_IMM.B R0, 0; LOAD.L R5, [mem R0];
    LOAD_IMM.B R0, 4; LOAD.S R6, [mem R0];
    LOAD_IMM.B R0, 6; LOAD.B R7, [mem R0];
    // Set TSAD[R7] to the address
    LOAD_IMM.B R0, Regs::Tsad0 as _;
    ADD.B R0, R7;
    OUT_IND.L R0, R5;
    // Set TSD to the length
    LOAD_IMM.B R0, Regs::Tsd0 as _;
    ADD.B R0, R7;
    OUT_IND.L R0, R6;
    END_IMM 0;
}
::udi::define_pio_ops!{pub GET_TSD =
    LOAD_IMM.B R7, 0; BRANCH 4;
    LABEL 1;
    LOAD_IMM.B R7, 1; BRANCH 4;
    LABEL 2;
    LOAD_IMM.B R7, 2; BRANCH 4;
    LABEL 3;
    LOAD_IMM.B R7, 3;
    LABEL 4;
    LOAD_IMM.B R0, Regs::Tsd0 as _;
    ADD.B R0, R7;
    IN_IND.L R0, R0;
    END.S R0;
}

::udi::define_pio_ops!{pub RX_CHECK =
    // - Read `CAPR` - if it's equal to `CBA` then the buffer is empty
    IN.S R0, Regs::Capr as _;
    ADD_IMM.S R0, 0x10; // Account for a hardware ?bug
    IN.L R1, Regs::Cba as _;
    LOAD.L R2, R0;  // Save CAPR for later
    // CBA - CAPR. If negative then the buffer has wrapped
    // - Need to handle the final packet, which can extend past the wrap point
    SUB.L R0, R1;
    // - If equal, return 0xFFFF immediately (indicating no packet)
    CSKIP.L R0 NZ;
    END_IMM 0xFFFF;
    // Otherwise, return the CAPR value
    END.S R2;
}
#[repr(C)]
struct MemRxUpdate {
    delta: u16,
}
impl MemRxUpdate {
    fn get_ptr(&mut self) -> ::udi::pio::MemPtr {
        // SAFE: Valid
        unsafe {
            ::udi::pio::MemPtr::new(
                ::core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, ::core::mem::size_of::<Self>())
            )
        }
    }
}
::udi::define_pio_ops!{pub RX_UPDATE =
    // Get the byte count
    LOAD_IMM.S R0, 0;
    LOAD.S R0, [mem R0];
    // Get CAPR, and add the offset
    IN.S R1, Regs::Capr as _;
    ADD_IMM.S R1, 0x10; // Account for a hardware ?bug
    ADD.L R1, R0;
    // Check if it's above the wraparound point
    LOAD.L R0, R1;
    ADD_IMM.S R0, -(super::RX_BUF_LENGTH as i16) as u16;
    // if `R1 - RX_BUF_LENGTH >= 0` then `R1 = 0`
    CSKIP.S R0 Neg;
    LOAD_IMM.S R1, 0;

    ADD_IMM.S R1, -0x10i16 as u16; // Account for a hardware ?bug
    // CAPR = R1
    OUT.S Regs::Capr as _, R1;
}

::udi::define_pio_ops!{pub IRQACK =
    // Entrypoint 0: Enable interrupts
    END_IMM 0;
    // 1: Normal
    LABEL 1;
    // - Read ISR and ack all set bits
    IN.S R0, Regs::Isr as _;
    OUT.S Regs::Isr as _, R0;
    END.B R0;
    // 2: Overrun
    LABEL 2;
    // 3: Overrun irqs
    LABEL 3;
    END_IMM 0;
}