pub struct Rtl8029 {
    regs: ::std::sync::Mutex<Regs>,
    irq: super::Interrupt,
}
impl Rtl8029 {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {
            regs: Default::default(),
            irq: Default::default(),
        })
    }
}
impl super::PioDevice for Rtl8029 {
    fn poll(&self, actions: &mut super::Actions) {
        let is_int = {
            let mut regs = self.regs.lock().unwrap();
            let regs = &mut *regs;
            while let Some(bytes) = actions.pull("nic_rx") {
                if regs.cmd & 0x1 != 0 {
                    // STOP bit is set
                    println!("RTL8019: Drop packet, CMD.STP=1");
                    continue ;
                }
                if regs.pstart == regs.pstop {
                    println!("RTL8019: Drop packet, buffer has zero capacity");
                    continue ;
                }
                assert!(bytes.len() <= 0x4000 - 4);
                let npages = ((4 + bytes.len() + 256 - 1) / 256) as u8;
                if regs.pstart <= regs.bnry && regs.bnry <= regs.pstop {
                    let rel_next = regs.rx_next - regs.pstart;
                    let rel_bnry = regs.bnry - regs.pstart;
                    let space = (regs.pstop - regs.pstart) + rel_bnry - rel_next;
                    if space > 0 && npages >= space {
                        // RX overflow!
                        println!("RTL8019 RX Overflow - {} >= {}", npages, space);

                        regs.isr |= 1 << 4; // OVW
                        continue;
                    }
                }
                let next_page = {
                    let mut next = regs.rx_next + npages;
                    if next >= regs.pstop {
                        next -= regs.pstop - regs.pstart;
                    }
                    next
                    };
                let first_page = regs.rx_next;
                assert!(regs.pstart <= regs.rx_next && regs.rx_next < regs.pstop,
                    "{:#x} <= {:#x} < {:#x}", regs.pstart, regs.rx_next, regs.pstop);
                let split = regs.pstop - regs.rx_next;
                let (a,b) = if 4 + bytes.len() < split as usize * 256 {
                    (bytes.as_slice(),[].as_slice())
                }
                else {
                    bytes.split_at(split as usize * 256 - 4)
                };
                let hdr = [
                    0,  // Status
                    next_page,  // NextPage
                    bytes.len().to_le_bytes()[0],
                    bytes.len().to_le_bytes()[1],
                    ];
                regs.card_ram.write(regs.rx_next as u16 * 256 + 0, &hdr);
                regs.card_ram.write(regs.rx_next as u16 * 256 + 4, a);
                regs.card_ram.write(regs.pstart as u16 * 256, b);
                regs.rx_next = next_page;
                println!("RTL8019 RX {}--{}", first_page, regs.rx_next);
                regs.isr |= 1 << 0; // PRX - Packet RX
            }
            regs.isr & regs.imr != 0
        };
        if is_int {
            self.irq.raise()
        }
    }
    
    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]) {
        assert!(regset_idx == 0);
        assert!(dst.len() == 1 || reg & !0x7 == 0x10, "Unexpected length ({}) reading from {:#x}", dst.len(), reg);
        let mut regs = self.regs.lock().unwrap();
        dst[0] = match (regs.cmd_pg(), reg) {
            (_, 0) => regs.cmd,
            (0, 1) => regs.clda[0],
            (0, 2) => regs.clda[1],
            (0, 3) => regs.bnry,
            (0, 4) => regs.tsr,
            (0, 5) => regs.nsr,
            (0, 6) => todo!("fifo?"),
            (0, 7) => regs.isr,
            (0, 8) => regs.cadr.to_le_bytes()[0],
            (0, 9) => regs.cadr.to_le_bytes()[1],
            (0, 10) => regs.rtl_8019id[0],
            (0, 11) => regs.rtl_8019id[1],
            (0, 12) => regs.rsr,
            (0, 13) => regs.cntr[0],
            (0, 14) => regs.cntr[1],
            (0, 15) => regs.cntr[2],
            (_, 0x10 ..= 0x17) => {
                regs.card_ram.read(regs.cadr, dst);
                regs.cadr += dst.len() as u16;
                regs.current_byte_count -= dst.len() as u16;
                return;
                },   // Remote DMA
            (_, 0x18 ..= 0x1F) => 0,   // Reset
            (_, 0x20..) => panic!("Invalid reg"),
            _ => todo!("Handle reg {}:{:#X}", regs.cmd_pg(), reg),
            };
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        assert!(regset_idx == 0);
        assert!(src.len() == 1);
        let mut regs = self.regs.lock().unwrap();
        let regs = &mut *regs;
        let v = src[0];
        match (regs.cmd_pg(), reg)
        {
        (_, 0) => {
            write_reg(&mut regs.cmd, src, "CMD", 0x00, 0x00);
            match (regs.cmd >> 3) & 7 {
            0 => panic!("RTL8019 RD2-0: 000 Not Allowed"),
            1|2 => {
                regs.cadr = regs.rsar;
                regs.current_byte_count = regs.rbcr;
                println!("Remote DMA: {:#x}+{:#x}", regs.cadr, regs.current_byte_count)
                },
            3 => {
                // TX packet
            },
            _ => {
                // Stop/abort RDMA
            },
            }
        },
        (0, 1) => {
            regs.pstart = v;
            regs.rx_next = regs.pstart;
        },
        (0, 2) => regs.pstop = v,
        (0, 3) => regs.bnry = v,
        (0, 4) => regs.tpsr = v,
        (0, 5) => regs.tbcr[0] = v,
        (0, 6) => regs.tbcr[1] = v,
        (0, 7) => regs.isr &= !v,
        (0, 8) => set_byte(&mut regs.rsar, 0, v),
        (0, 9) => set_byte(&mut regs.rsar, 1, v),
        (0, 10) => set_byte(&mut regs.rbcr, 0, v),
        (0, 11) => set_byte(&mut regs.rbcr, 1, v),
        (0, 12) => regs.rcr = v,
        (0, 13) => regs.tcr = v,
        (0, 14) => regs.dcr = v,
        (0, 15) => regs.imr = v,
        (1, 1..=6)
            => regs.par[reg as usize - 1] = v,
        (1, 7) => regs.curr = v,
        (1, 8..=15)
            => regs.mar[reg as usize - 8] = v,
        (_, 0x10 ..= 0x17) => {   // Remote DMA
            regs.card_ram.write(regs.cadr, src);
            regs.cadr += src.len() as u16;
            regs.current_byte_count -= src.len() as u16;
        },
        (_, 0x18 ..= 0x1F) => regs.reset(),   // Reset
        (_, 0x20..) => panic!("Invalid reg"),
        _ => todo!("Write reg {}:{:#X}", regs.cmd_pg(), reg),
        }
        fn set_byte(dst: &mut u16, idx: usize, v: u8) {
            if idx == 0 {
                *dst = *dst & 0xFF00 | v as u16;
            }
            else {
                *dst = *dst & 0x00FF | ((v as u16) << 8);
            }
        }
    }

    fn irq(&self, index: u8) -> &super::Interrupt {
        assert!(index == 0);
        &self.irq
    }
}

#[derive(Default)]
struct Regs
{
    // 0
    cmd: u8,
    // 1&2
    /// These two registers can be read to get the current local DMA address.
    clda: [u8; 2],
    /// The Page Start register sets the start page address of the receive buffer ring.
    pstart: u8,
    /// The Page Stop register sets the stop page address of the receive buffer ring. In 8 bit
    /// mode the PSTOP register should not exceed to 0x60, in 16 bit mode the PSTOP
    /// register should not exceed to 0x80.
    pstop: u8,
    // 3
    /// This register is used to prevent overwrite of the receive buffer ring. It is typically
    /// used as a pointer indicating the last receive buffer page the host has read.
    bnry: u8,
    // 4
    tsr: u8,
    /// This register sets the start page address of the packet to the transmitted.
    tpsr: u8,
    // 5&6
    nsr: u8,
    //fifo: u8,   // Is this a real reg, or just a port
    tbcr: [u8; 2],
    // 7
    isr: u8,
    // 8&9
    /// These two registers contain the current address of remote DMA.
    cadr: u16,
    /// These two registers set the start address of remote DMA.
    rsar: u16,
    // 10&11
    rtl_8019id: [u8; 2],
    /// These two registers set the data byte counts of remote DMA.
    rbcr: u16,
    // 12
    rsr: u8,
    rcr: u8,
    // 13-15
    cntr: [u8; 3],
    tcr: u8,
    dcr: u8,
    imr: u8,

    // 1.1-6
    par: [u8; 6],
    // 1.7
    curr: u8,
    // 1.8-15
    mar: [u8; 8],

    // -- Hidden registers
    /// Current remote DMA byte count
    current_byte_count: u16,
    /// Next RX page
    rx_next: u8,

    card_ram: CardMem,
}
impl Regs
{
    fn reset(&mut self) {
        self.cmd = 0x01;
        self.isr = 0x80;
    }
    fn cmd_pg(&self) -> u8 {
        self.cmd >> 6
    }
}
struct CardMem(Box<[u8; 0x4000]>); // 16KiB
impl Default for CardMem {
    fn default() -> Self {
        CardMem(Box::new([0; 0x4000]))
    }
}
impl CardMem {
    fn read(&self, addr: u16, dst: &mut [u8]) {
        fn copy_from(src: &[u8], rel_addr: u16, dst: &mut [u8]) {
            dst.copy_from_slice(&src[rel_addr as usize..][..dst.len()]);
        }
        match addr {
        0 ..= 5 => copy_from(&[0x12,0x34,0x56,0x00,0x00,0x01], addr-0, dst),
        0x40_00 ..= 0x7F_FF => copy_from(&*self.0, addr - 0x4000, dst),
        _ => panic!("Out-of-bounds read: {:#x}+{}", addr, dst.len()),
        }
    }
    fn write(&mut self, addr: u16, src: &[u8]) {
        fn copy_to(dst: &mut [u8], rel_addr: u16, src: &[u8]) {
            dst[rel_addr as usize..][..src.len()].copy_from_slice(src);
        }
        match addr {
        0x40_00 ..= 0x80_00 => copy_to(&mut *self.0, addr - 0x40_00, src),
        _ => panic!("Out-of-bounds write: {:#x}+{}", addr, src.len()),
        }
    }
}

fn write_reg(slot: &mut u8, src: &[u8], name: &'static str, mask_rsvd: u8, mask_ro: u8) -> u8
{
    let new = src[0];
    let prev = *slot;
    assert!(new & mask_rsvd == prev & mask_rsvd,
        "Reserved bits changed in {name} {:#x} != {:#x}", new & mask_rsvd, prev & mask_rsvd);
    *slot = (new & !mask_ro) | (prev & mask_ro);
    prev
}