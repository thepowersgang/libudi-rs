
#[repr(u8)]
enum Regs {
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
/// Packet Underrung
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
    OUT.L Regs::Capr as _ , R0;
    OUT.L Regs::Cba as _ , R0;
    // - TX buffers?
    // - RCR - hw::RCR_DMA_BURST_1024|hw::RCR_BUFSZ_8K16|hw::RCR_FIFO_1024|hw::RCR_OVERFLOW|0x1F
    LOAD_IMM.S R0, ((6<<13)|(0<<11)|(6<<8)|0x80|0x1F);
    OUT.S Regs::Rcr as _, R0;
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

::udi::define_pio_ops!{pub IRQACK =
    // Entrypoint 0: Enable interrupts
    END_IMM 0;
    // 1: Normal
    LABEL 1;
    // - Read ISR and ack all set bits
    IN.B R0, Regs::Isr as _;
    OUT.B Regs::Isr as _, R0;
    END.B R0;
    // 2: Overrun
    LABEL 2;
    // 3: Overrun irqs
    LABEL 3;
    END_IMM 0;
}