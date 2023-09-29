
#[derive(Debug)]
pub struct Handle(crate::ffi::pio::udi_pio_handle_t);
impl super::Gcb
{
	pub fn pio_map<'a>(&mut self, regset: u32, offset: u32, length: u32, trans_list: &'a [crate::ffi::pio::udi_pio_trans_t], pio_attributes: u16, pace: u32) -> impl ::core::future::Future<Output=Handle>+'a
	{
		extern "C" fn cb_pio_map(gcb: *mut crate::ffi::udi_cb_t, handle: crate::ffi::pio::udi_pio_handle_t) {
			unsafe { crate::async_trickery::signal_waiter(&mut *gcb, crate::WaitRes::Pointer(handle as *mut ())); }
		}
		crate::async_trickery::wait_task::<crate::ffi::udi_cb_t, _,_,_>(
			move |cb| unsafe { crate::ffi::pio::udi_pio_map(cb_pio_map, cb as *const _ as *mut _, regset, offset, length, trans_list.as_ptr(), trans_list.len() as u16, pio_attributes, pace, 0) },
			|res| {
				let crate::WaitRes::Pointer(p) = res else { panic!(""); };
				Handle(p as *mut _)
				}
			)
	}
}
