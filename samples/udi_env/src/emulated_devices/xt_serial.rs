pub struct XTSerial {

}
impl XTSerial {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {

        })
    }
}
impl super::PioDevice for XTSerial {
    fn set_interrupt_channel(&self, index: ::udi::ffi::udi_index_t, channel: udi::ffi::udi_channel_t) {
        if index.0 != 0 {
            panic!("Bad IRQ index");
        }
        //todo!("set_interrupt_channel")
    }
    fn push_intr_cb(&self, index: ::udi::ffi::udi_index_t, cb: ::udi::meta_intr::CbHandleEvent) {
    }

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]) {
        todo!("pio_read")
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        todo!("pio_write")
    }
    
}