

pub struct Device {
    regs: ::std::sync::Mutex<Regs>,
    interrupt_channel: ::std::sync::Mutex<::udi::imc::ChannelHandle>,
    irq_cbs: ::std::sync::Mutex< ::std::collections::VecDeque<::udi::meta_bridge::CbHandleEvent> >,
}
impl Device {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {
            regs: Default::default(),
            interrupt_channel: Default::default(),
            irq_cbs: Default::default(),
        })
    }
}
impl super::PioDevice for Device {
    fn set_interrupt_channel(&self, index: ::udi::ffi::udi_index_t, channel: ::udi::imc::ChannelHandle) {
        if index.0 != 0 {
            panic!("Bad IRQ index");
        }
        *self.interrupt_channel.lock().unwrap() = channel;
    }
    fn push_intr_cb(&self, index: ::udi::ffi::udi_index_t, cb: ::udi::meta_bridge::CbHandleEvent) {
        assert!(index.0 == 0, "Bad IRQ index");
        self.irq_cbs.lock().unwrap()
            .push_back(cb);
    }

    fn pio_read(&self, regset_idx: u32, reg: u32, dst: &mut [u8]) {
        assert!(regset_idx == 0);
        assert!(dst.len() == 1);
        // TODO: "Proper" emulation
        let regs = self.regs.lock().unwrap();
        dst[0] = match reg {
            _ => todo!("Handle reg {:#X}", reg),
            };
    }

    fn pio_write(&self, regset_idx: u32, reg: u32, src: &[u8]) {
        assert!(regset_idx == 0);
        assert!(src.len() == 1);
        let mut regs = self.regs.lock().unwrap();
        let v = src[0];
        match reg
        {
        _ => todo!("Write reg {:#X}", reg),
        }
    }    
}

#[derive(Default)]
struct Regs
{
}
impl Regs
{
    fn reset(&mut self) {
    }
}