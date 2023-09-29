#![feature(type_alias_impl_trait)]
mod driver;

extern "C" {
	fn malloc(size: usize) -> *mut ::std::ffi::c_void;
}

#[no_mangle]
extern "C" fn udi_usage_res(cb: *mut ::udi::ffi::meta_mgmt::udi_usage_cb_t) {
	println!("udi_usage_res()");
	InnerCtxt::from_cb(unsafe { &(&mut (*cb).gcb as *mut _) }).set_complete();
}
#[no_mangle]
extern "C" fn udi_pio_map(
	callback: extern "C" fn (*mut ::udi::ffi::udi_cb_t, *mut ()),
	gcb: *mut ::udi::ffi::udi_cb_t,
	_regset_idx: u32, base_offset: u32, _length: u32,
	_trans_list: *const ::udi::ffi::pio::udi_pio_trans_t, _list_length: u16,
	_pio_attributes: u16, _pace: u32, _serialization_domain: ::udi::ffi::udi_index_t
	)
{
	println!("udi_pio_map(..., {:#x}, ...)", base_offset);
	InnerCtxt::from_cb(&gcb).push_op(move || {
		println!(">> udi_pio_map(..., {:#x}, ...)", base_offset);
		(callback)(gcb, base_offset as usize as *mut _)
		})
}

#[derive(Default)]
struct InnerCtxt {
	complete: ::std::cell::Cell<bool>,
	ops: ::std::cell::RefCell<Option<Box<dyn FnOnce()>>>,
}
impl InnerCtxt {
	fn from_cb(gcb: &*mut ::udi::ffi::udi_cb_t) -> &Self {
		unsafe { &mut *((**gcb).initiator_context as *mut InnerCtxt) }
	}
	fn set_complete(&self) {
		self.complete.set(true);
	}
	fn push_op(&self, op: impl FnOnce() + 'static) {
		if true {
			op();
		}
		else {
			let v = self.ops.borrow_mut().replace(Box::new(op));
			assert!(v.is_none());
		}
	}
}

fn main()
{
	let pi = ::udi::make_pri_init::<driver::Driver>();
	println!("malloc {},{}", pi.rdata_size, pi.mgmt_scratch_requirement);
	let (context,mgmt_scratch) = unsafe {
		(malloc( pi.rdata_size ), malloc( pi.mgmt_scratch_requirement ),)
		};
	let mut inner_ctxt = InnerCtxt::default();
	let mut cb = ::udi::ffi::meta_mgmt::udi_usage_cb_t {
		gcb: ::udi::ffi::udi_cb_t {
			context,
			scratch: mgmt_scratch,
			channel: ::core::ptr::null_mut(),
			initiator_context: &mut inner_ctxt as *mut _ as *mut _,
			origin: ::core::ptr::null_mut(),
			},
		meta_idx: 0,
		trace_mask: 0,
		};
	unsafe {
		(pi.mgmt_ops.usage_ind_op)(&mut cb, 1);
		loop {
			let op = inner_ctxt.ops.borrow_mut().take();
			if let Some(op) = op {
				op();
				continue
			}
			break
		}
		assert!(inner_ctxt.complete.get());
	}
}


