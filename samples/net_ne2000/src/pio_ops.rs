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
	REP_IN_IND.B [mem R0 STEP1], R1, R2;
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
    OUT.B regs::PG0W_TPSR as _, R0;
    END_IMM 0;
}
::udi::define_pio_ops!{pub DISBALE =
    // CMD = 0x22 [NoDMA, !Start, Stop]
    LOAD_IMM.B R0, 0x21;
    OUT.B regs::APG_CMD as _, R0;
    END_IMM 0;
}

// NOTE: This expects `rx_next_page` as the input memory buffer
::udi::define_pio_ops!{pub RX =
    // Get current page into R7
    LOAD_IMM.B R0, 0; LOAD.B R7, [mem R0];
    // Read header into regs, then data into buffer
    // - CMD = 0x22
    LOAD_IMM.B R0, 0x22; OUT.B regs::APG_CMD as _, R0;
    // - Clear RDMA flag in ISR
    LOAD_IMM.B R0, 0x40; OUT.B regs::PG0_ISR as _, R0;
    // - Set up transaction for 1 page from CurRX
    LOAD_IMM.B R0, 0;   // Zero used for page offset and subpage count
    LOAD_IMM.B R1, 1;   // used for page count
    OUT.B regs::PG0_RSAR0 as _, R0;
    OUT.B regs::PG0_RSAR1 as _, R7; // `rx_next_page`
    OUT.B regs::PG0_RBCR0 as _, R0;
    OUT.B regs::PG0_RBCR1 as _, R1;
    // - Start read (CMD=0x0A)
    LOAD_IMM.B R0, 0x0A; OUT.B regs::APG_CMD as _, R0;
    DELAY 0;
    //  > Header to registers
    IN.S R6, regs::APG_MEM as _;    // Status,NextPacketPage
    IN.S R5, regs::APG_MEM as _;    // R5 = Length (bytes)
    STORE.S R4, R5; // save length until return
    //  > Data to buffer (126 words)
    LOAD_IMM.B R0, 0;   // - Buffer offset (incremented by 1 each iteration)
    LOAD_IMM.B R1, regs::APG_MEM;  // - Reg offset (no increment)
    LOAD_IMM.B R2, (256u16/2 - 2) as u8;   // Count
    REP_IN_IND.B [buf R0 STEP1], R1, R2;
    // - Subtract 256-4 from R5(length), if <=0 we've grabbed the entire packet
    LOAD_IMM.B R3, (256-4) as u8;   // R3 is the offset in the output buffer, we've loaded 126 words currently
    ADD_IMM.S R5, -(256i16-4) as u16;
    LABEL 2;
    CSKIP.S R5 NZ;
    BRANCH 1;   // 1: End if R5(length)==0
    CSKIP.S R5 NNeg;
    BRANCH 1;   // 1: End if R5(length)<0
    // - Read pages until all of packet RXd
    ADD_IMM.B R7, 1;
    STORE.B R0, R7;
    ADD_IMM.B R0, -(mem::RX_LAST_PG as i16+1) as u8;
    CSKIP.B R0 NZ;  // if R7-RX_LAST == 0
    LOAD_IMM.B R7, mem::RX_FIRST_PG;    // R7 = RX_FIRST
    //  > Transaction start
    LOAD_IMM.B R0, 0;   // Zero used for page offset and subpage count
    LOAD_IMM.B R1, 1;   // used for page count
    OUT.B regs::PG0_RSAR0 as _, R0;
    OUT.B regs::PG0_RSAR1 as _, R7; // Current RX page
    OUT.B regs::PG0_RBCR0 as _, R0;
    OUT.B regs::PG0_RBCR1 as _, R1;
    // - Start read (CMD=0x0A)
    LOAD_IMM.B R0, 0x0A; OUT.B regs::APG_CMD as _, R0;
    DELAY 0;
    //  > Data to buffer (128 words)
    // buffer offset maintained in R3
    LOAD_IMM.B R1, regs::APG_MEM;
    LOAD_IMM.B R2, (256u16/2) as u8;   // Count (full 128 words)
    REP_IN_IND.B [buf R3 STEP1], R1, R2;
    ADD_IMM.S R3, 256;
    ADD_IMM.S R3, -256i16 as u16;
    // - Jump to length check
    BRANCH 2;   // 2: Check against length

    // Cleanup
    LABEL 1;
    // - Update next RX page, return status
    OUT.B regs::PG0_BNRY as _, R7;
    STORE.S R7, R6;  // Saved tuple of Status,Next
    SHIFT_RIGHT.S R7, 8;    // Get `Next` from that tuple
    LOAD_IMM.B R0, 0;
    STORE.S [mem R0], R7; // Store in `rx_next_page`

    END.S R4;   // End with packet length
}

// Expects `mem` to be `length: u16, page: u8`
::udi::define_pio_ops!{pub TX =
    // Read header into regs, then data into buffer
    // - CMD = 0x22 (Page0, Start, NoDMA)
    LOAD_IMM.B R0, 0x22; OUT.B regs::APG_CMD as _, R0;
    // - Clear RDMA flag in ISR
    LOAD_IMM.B R0, 0x40; OUT.B regs::PG0_ISR as _, R0;

    LOAD_IMM.S R7, 0;
    LOAD.S R6, [mem R7];
    LOAD.S R0, R6;
    // "Transmite Byte Count" and "Remote Byte Count"
    OUT.B regs::PG0W_TBCR0 as _, R0;
    OUT.B regs::PG0_RBCR0 as _, R0;
    SHIFT_RIGHT.S R0, 8;
    OUT.B regs::PG0W_TBCR1 as _, R0;
    OUT.B regs::PG0_RBCR1 as _, R0;
    // Remote Start Address
    LOAD_IMM.S R0, 0;
    OUT.B regs::PG0_RSAR0 as _, R0;
    LOAD_IMM.S R7, 2;
    LOAD.B R0, [mem R7];
    OUT.B regs::PG0_RSAR1 as _, R0;
    // Start a remote write
    // - CMD = 0x12 (Page0, ?)
    LOAD_IMM.B R0, 0x12; OUT.B regs::APG_CMD as _, R0;
    // - Write `length` bytes
    LOAD_IMM.B R0, 0;
    LOAD_IMM.B R1, regs::APG_MEM;
    LOAD.S R2, R6;
    ADD_IMM.S R2, 1;    SHIFT_RIGHT.S R2, 1;    // (len+1)/2 to write u16s
    REP_OUT_IND.S [buf R0 STEP1], R1, R2;

    // Wait for 0x40 (RDMA Complete) in ISR
    LABEL 1;
    IN.B R0, regs::PG0_ISR as _;
    AND_IMM.B R0, 0x40;
    CSKIP.B R0 NZ;
    BRANCH 1;
    // ACK/clear the Interrupt
    OUT.B regs::PG0_ISR as _, R0;

    // TPSR = page
    LOAD_IMM.S R7, 2;
    LOAD.B R0, [mem R7];
    OUT.B regs::PG0W_TPSR as _, R0;
    // Trigger TX
    // - CMD = 0x16
    LOAD_IMM.B R0, 0x16; OUT.B regs::APG_CMD as _, R0;

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