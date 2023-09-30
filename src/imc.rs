pub trait ChannelHandler: 'static {
}

pub const fn task_size<T: ChannelHandler>() -> usize {
    0
}
pub unsafe extern "C" fn channel_event_ind_op<T: ChannelHandler>(cb: *mut crate::ffi::imc::udi_channel_event_cb_t) {
}