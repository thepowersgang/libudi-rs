pub struct Rtl8029 {

}
impl Rtl8029 {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {

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
        dst[0] = match reg {
            0x1F => 0,
            0x07 => 0x80,
            0x00 => 0x99,
            _ => todo!("Handle reg {:#X}", reg),
            };
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        assert!(regset_idx == 0);
        assert!(src.len() == 1);
        match reg
        {
        _ => {},
        }
    }
    
}