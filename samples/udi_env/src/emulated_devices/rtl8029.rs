pub struct Rtl8029 {
    regs: ::std::sync::Mutex<Regs>,
}
impl Rtl8029 {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {
            regs: Default::default(),
        })
    }
}
impl super::PioDevice for Rtl8029 {
    fn set_interrupt_channel(&self, index: ::udi::ffi::udi_index_t, channel: udi::ffi::udi_channel_t) {
        if index.0 != 0 {
            panic!("Bad IRQ index");
        }
        //todo!("set_interrupt_channel")
    }
    fn push_intr_cb(&self, index: ::udi::ffi::udi_index_t, cb: ::udi::meta_intr::CbHandleEvent) {
    }

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]) {
        assert!(regset_idx == 0);
        assert!(dst.len() == 1);
        // TODO: "Proper" emulation
        let regs = self.regs.lock().unwrap();
        dst[0] = match (regs.cmd_pg(), reg) {
            (_, 0) => regs.cmd,
            (0, 1) => regs.clda[0],
            (0, 2) => regs.clda[1],
            (0, 3) => regs.bnry,
            (0, 4) => regs.tsr,
            (0, 5) => regs.nsr,
            (0, 6) => todo!("fifo?"),
            (0, 7) => regs.isr,
            (0, 8) => regs.cadr[0],
            (0, 9) => regs.cadr[1],
            (0, 10) => regs.rtl_8019id[0],
            (0, 11) => regs.rtl_8019id[1],
            (0, 12) => regs.rsr,
            (0, 13) => regs.cntr[0],
            (0, 14) => regs.cntr[1],
            (0, 15) => regs.cntr[2],
            (_, 0x10 ..= 0x17) => 0x99,   // Remote DMA
            (_, 0x18 ..= 0x1F) => 0,   // Reset
            (_, 0x20..) => panic!("Invalid reg"),
            _ => todo!("Handle reg {}:{:#X}", regs.cmd_pg(), reg),
            };
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        assert!(regset_idx == 0);
        assert!(src.len() == 1);
        let mut regs = self.regs.lock().unwrap();
        let v = src[0];
        match (regs.cmd_pg(), reg)
        {
        (_, 0) => regs.cmd = v,
        (0, 1) => regs.pstart = v,
        (0, 2) => regs.pstop = v,
        (0, 3) => regs.bnry = v,
        (0, 4) => regs.tpsr = v,
        (0, 5) => regs.tbcr[0] = v,
        (0, 6) => regs.tbcr[1] = v,
        (0, 7) => regs.isr &= !v,
        (0, 8) => regs.rsar[0] = v,
        (0, 9) => regs.rsar[1] = v,
        (0, 10) => regs.rbcr[0] = v,
        (0, 11) => regs.rbcr[1] = v,
        (0, 12) => regs.rcr = v,
        (0, 13) => regs.tcr = v,
        (0, 14) => regs.dcr = v,
        (0, 15) => regs.imr = v,
        (1, 1..=6)
            => regs.par[reg as usize - 1] = v,
        (1, 7) => regs.curr = v,
        (1, 8..=15)
            => regs.mar[reg as usize - 8] = v,
        (_, 0x18 ..= 0x1F) => regs.reset(),   // Reset
        (_, 0x20..) => panic!("Invalid reg"),
        _ => todo!("Write reg {}:{:#X}", regs.cmd_pg(), reg),
        }
    }    
}

#[derive(Default)]
struct Regs
{
    // 0
    cmd: u8,
    // 1&2
    clda: [u8; 2],
    pstart: u8,
    pstop: u8,
    // 3
    bnry: u8,
    // 4
    tsr: u8,
    tpsr: u8,
    // 5&6
    nsr: u8,
    fifo: u8,   // Is this a real reg, or just a port
    tbcr: [u8; 2],
    // 7
    isr: u8,
    // 8&9
    cadr: [u8; 2],
    rsar: [u8; 2],
    // 10&11
    rtl_8019id: [u8; 2],
    rbcr: [u8; 2],
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
}
impl Regs
{
    fn reset(&mut self) {
        self.cmd = 0x00;
        self.isr = 0x80;
    }
    fn cmd_pg(&self) -> u8 {
        self.cmd >> 6
    }
}