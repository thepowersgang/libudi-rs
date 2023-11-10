#[derive(Default)]
pub struct XTSerial {
    interrupt_channel: ::std::sync::Mutex<::udi::imc::ChannelHandle>,
    irq_cbs: ::std::sync::Mutex< ::std::collections::VecDeque<::udi::meta_intr::CbHandleEvent> >,
}
impl XTSerial {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Default::default())
    }
}
impl super::PioDevice for XTSerial {
    fn set_interrupt_channel(&self, index: ::udi::ffi::udi_index_t, channel: ::udi::imc::ChannelHandle) {
        if index.0 != 0 {
            panic!("Bad IRQ index");
        }
        *self.interrupt_channel.lock().unwrap() = channel;
    }
    fn push_intr_cb(&self, index: ::udi::ffi::udi_index_t, cb: ::udi::meta_intr::CbHandleEvent) {
        assert!(index.0 == 0, "Bad IRQ index");
        self.irq_cbs.lock().unwrap()
            .push_back(cb);
    }

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]) {
        todo!("pio_read")
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        todo!("pio_write")
    }
    
}