use super::regs;
use super::mem;

::udi::define_pio_ops!{pub RESET =
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

::udi::define_pio_ops!{pub ENABLE =
    // - Setup
    // PSTART = First RX page [Receive area start]
    LOAD_IMM.B R0, mem::RX_FIRST_PG;
    OUT.B regs::PG0_CLDA0 as _, R0;  // Aka `PSTART`?
    // BNRY = Last RX page - 1 [???]
    LOAD_IMM.B R0, mem::RX_LAST_PG-1;
    OUT.B regs::PG0_BNRY as _, R0;
    // PSTOP = Last RX page [???]
    LOAD_IMM.B R0, mem::RX_LAST_PG;
    OUT.B regs::PG0_CLDA1 as _, R0; // Aka `PSTOP`?
    // CMD = 0x22 [NoDMA, Start]
    LOAD_IMM.B R0, 0x22;
    OUT.B regs::APG_CMD as _, R0;
    // RCR = 0x0F [Wrap, Promisc]
    LOAD_IMM.B R0, 0x0F;
    OUT.B regs::PG0_RCR as _, R0;
    // TCR = 0x00 [Normal]
    LOAD_IMM.B R0, 0x00;
    OUT.B regs::PG0_TCR as _, R0;
    // TPSR = 0x40 [TX Start]
    LOAD_IMM.B R0, 0x40;
    OUT.B regs::PG0_TSR as _, R0;   // Aka `TPSR`
    END_IMM 0;
}

::udi::define_pio_ops!{pub IRQACK =
        // Entrypoint 0: Enable interrupts
        // IMR = 0x3F []
        LOAD_IMM.B R0, 0x3F;
        OUT.B regs::PG0_IMR as _, R0;
        // ISR = 0xFF [ACK all]
        LOAD_IMM.B R0, 0xFF;
        OUT.B regs::PG0_ISR as _, R0;
        END_IMM 0;
        // 1: Normal
        LABEL 1;
        IN.B R0, regs::PG0_ISR as _;
        OUT.B regs::PG0_ISR as _, R0;
        CSKIP.B R0 Z;   // if R0!=0
        END.S R0;
        // - No IRQ, quiet quit
        LOAD_IMM.B R0, ::udi::ffi::meta_intr::UDI_INTR_UNCLAIMED as _;
        LOAD_IMM.B R1, 0;   // scratch offset
        STORE.B [scratch R1], R0;
        END_IMM 0;
        // 2: Overrun
        LABEL 2;
        // 3: Overrun irqs
        LABEL 3;
        END_IMM 0;
}