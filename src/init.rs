//! Driver initialisation (related to [crate::meta_mgmt])
use ::core::future::Future;

use crate::async_trickery;
use crate::ffi;

use crate::ffi::meta_mgmt::udi_usage_cb_t;
use crate::ffi::meta_mgmt::udi_enumerate_cb_t;
use crate::ffi::meta_mgmt::udi_mgmt_cb_t;

pub type CbRefUsage<'a> = crate::CbRef<'a, udi_usage_cb_t>;
pub type CbRefEnumerate<'a> = crate::CbRef<'a, udi_enumerate_cb_t>;
pub type CbRefMgmt<'a> = crate::CbRef<'a, udi_mgmt_cb_t>;

unsafe impl crate::async_trickery::GetCb for udi_enumerate_cb_t {
    fn get_gcb(&self) -> &ffi::udi_cb_t {
        &self.gcb
    }
}
unsafe impl crate::async_trickery::GetCb for udi_mgmt_cb_t {
    fn get_gcb(&self) -> &ffi::udi_cb_t {
        &self.gcb
    }
}

#[allow(non_camel_case_types)]
/// Trait for all drivers
pub trait Driver: 'static + crate::async_trickery::CbContext {
	const MAX_ATTRS: u8;

	type Future_init<'s>: Future<Output=()> + 's;
	fn usage_ind<'s>(&'s mut self, cb: CbRefUsage<'s>, resouce_level: u8) -> Self::Future_init<'s>;

	type Future_enumerate<'s>: Future<Output=(EnumerateResult,AttrSink<'s>)> + 's;
	fn enumerate_req<'s>(&'s mut self, cb: CbRefEnumerate<'s>, level: EnumerateLevel, attrs_out: AttrSink<'s>) -> Self::Future_enumerate<'s>;

	type Future_devmgmt<'s>: Future<Output=crate::Result<u8>> + 's;
	fn devmgmt_req<'s>(&'s mut self, cb: CbRefMgmt<'s>, mgmt_op: MgmtOp, parent_id: crate::ffi::udi_ubit8_t) -> Self::Future_devmgmt<'s>;
}
pub enum EnumerateLevel
{
	Start,
	StartRescan,
	Next,
	New,
	Directed,
	Release,
}
pub struct EnumerateResultOk {
	ops_idx: crate::ffi::udi_index_t,
	child_id: crate::ffi::udi_ubit32_t,
}
impl EnumerateResultOk {
	pub fn new<Ops: crate::ops_wrapper_markers::ChildBind>(child_id: crate::ffi::udi_ubit32_t) -> Self {
		Self {
			ops_idx: Ops::IDX,
			child_id,
		}
	}
	pub unsafe fn from_raw(ops_idx: crate::ffi::udi_index_t, child_id: crate::ffi::udi_ubit32_t) -> Self {
		Self {
			ops_idx,
			child_id,
		}
	}
	pub fn ops_idx(&self) -> crate::ffi::udi_index_t {
		self.ops_idx
	}
	pub fn child_id(&self) -> crate::ffi::udi_ubit32_t {
		self.child_id
	}
}
pub enum EnumerateResult
{
	Ok(EnumerateResultOk),
	Leaf,
	Done,
    Rescan,
    Removed,
    RemovedSelf,
    Released,
	Failed,
}
impl EnumerateResult {
	pub fn ok<Ops: crate::ops_wrapper_markers::ChildBind>(child_id: crate::ffi::udi_ubit32_t) -> Self {
		EnumerateResult::Ok(EnumerateResultOk::new::<Ops>(child_id))
	}
}
impl From<EnumerateResultOk> for EnumerateResult {
	fn from(value: EnumerateResultOk) -> Self {
		EnumerateResult::Ok(value)
	}
}
/// A place to store attributes, limited to [Driver::MAX_ATTRS]
pub struct AttrSink<'a>
{
	dst: *mut crate::ffi::attr::udi_instance_attr_list_t,
	remaining_space: usize,
	pd: ::core::marker::PhantomData<&'a mut crate::ffi::attr::udi_instance_attr_list_t>,
}
impl<'a> AttrSink<'a>
{
	fn get_entry(&mut self) -> Option<&mut crate::ffi::attr::udi_instance_attr_list_t> {
		if self.remaining_space == 0 {
			None
		}
		else {
			self.remaining_space -= 1;
			// SAFE: This type controls the `*mut` as a unique borrow, pointer is in-range
			unsafe {
				let rv = self.dst;
				self.dst = self.dst.offset(1);
				Some(&mut *rv)
			}
		}
	}
	fn set_name_and_type(e: &mut crate::ffi::attr::udi_instance_attr_list_t, name: &str, ty: crate::ffi::attr::udi_instance_attr_type_t) {
		let len = usize::min(name.len(), e.attr_name.len());
		e.attr_name[..len].copy_from_slice(&name.as_bytes()[..len]);
		if len < e.attr_name.len() {
			e.attr_name[len] = 0;
		}
		e.attr_type = ty as _;
	}
	pub fn push_u32(&mut self, name: &str, val: u32) {
		if let Some(e) = self.get_entry() {
			Self::set_name_and_type(e, name, crate::ffi::attr::UDI_ATTR_UBIT32);
			e.attr_length = 4;
			e.attr_value[..4].copy_from_slice(&val.to_ne_bytes());
		}
	}
	pub fn push_string(&mut self, name: &str, val: &str) {
		if let Some(e) = self.get_entry() {
			Self::set_name_and_type(e, name, crate::ffi::attr::UDI_ATTR_STRING);
			e.attr_length = val.len() as _;
			e.attr_value[..val.len()].copy_from_slice(val.as_bytes());
		}
	}
	pub fn push_string_fmt(&mut self, name: &str, val: ::core::fmt::Arguments) {
		if let Some(e) = self.get_entry() {
			Self::set_name_and_type(e, name, crate::ffi::attr::UDI_ATTR_STRING);
			// Create a quick helper that implements `fmt::Write` backed by a fixed-size buffer
			struct Buf<'a>(&'a mut [u8]);
			impl<'a> ::core::fmt::Write for Buf<'a> {
				fn write_str(&mut self, s: &str) -> core::fmt::Result {
					let len = usize::min(s.len(), self.0.len());
					let (d, t) = ::core::mem::replace(&mut self.0, &mut []).split_at_mut(len);
					d.copy_from_slice(&s.as_bytes()[..len]);
					self.0 = t;
					Ok( () )
				}
			}
			let mut buf = Buf(&mut e.attr_value[..]);
			let _ = ::core::fmt::write(&mut buf, val);
			// Calculate the length using pointer differences
			// SAFE: These two pointers are from the same source
			let len = unsafe { buf.0.as_ptr().offset_from( e.attr_value.as_ptr() ) };
			e.attr_length = len as u8;
		}
	}
}

pub enum MgmtOp
{
	/// Indicates that a Suspend operation is about to take place relative to the indicated parent.
	PrepareToSuspend,
	/// Requests the instance to suspend all operation
	/// relative to the indicated parent, and queue or fail new requests that
	/// are received. The instance must not acknowledge the request until all
	/// outstanding requests to the indicated parent are complete. The device
	/// must be put in a state that is prepared for the possibility of having
	/// power removed (for example, disk caches must be flushed), but
	/// device state and communications connections should not be
	/// completely shut down.
	Suspend,
	/// Treated as `UDI_DMGMT_SUSPEND, with the
	/// addition that the device must be completely shut down (in particular,
	/// all communications connections should be terminated).
	Shutdown,
	/// Indicates that outbound traffic via the
	/// indicated parent has been suspended.
	ParentSuspend,
	/// Indicates that the instance is to cancel any suspended
	/// or throttled state and is to resume full operation. I/O shall resume
	/// onto the then-active set of parents; if a multi-parent driver has parent-
	/// specific routing requirements, it must compare parent_ID against
	/// the set of currently-bound parents and fail if that parent is no longer
	/// (re-)bound
	Resume,
	/// Indicates that the driver must unbind from the
	/// indicated parent. The driver must first complete a metalanguage-
	/// specific unbind sequence with its parent and free resources related to
	/// that parent (it may choose to defer freeing some resources until it
	/// receives a udi_final_cleanup_req). As much as possible, the
	/// device should be shut down, as if it might be removed or powered off
	/// after this operation completes if this is the last parent.
	/// Communications connections should be terminated. Storage device
	/// write-back caches should be flushed to permanent storage, for
	/// example. When the unbinding is complete (and not before), the driver
	/// must respond to the `UDI_DMGMT_UNBIND`` request with a
	/// corresponding `udi_devmgmt_ack``.
	Unbind,
}

/// Region context
#[repr(C)]
#[fundamental]
pub struct RData<T> {
    init_context: ::udi_sys::init::udi_init_context_t,
	channel_cb: *mut ::udi_sys::imc::udi_channel_event_cb_t,
	// NOTE: According to the docs on `udi_primary_init_info_t`, this structure is null-initialised.
	// So, this field will be `false` on first use
	// Needed because `usage_ind` can be called multiple times
	is_init: bool,
    pub inner: T,
}
impl<Driver> AsRef<RData<Driver>> for RData<Driver> {
    fn as_ref(&self) -> &RData<Driver> {
        self
    }
}
impl<Driver> crate::imc::ChannelInit for RData<Driver> {
}
impl<Driver> crate::async_trickery::CbContext for RData<Driver>
where
	Driver: Default,
{
	fn maybe_init(&mut self) {
		if !self.is_init {
			unsafe { ::core::ptr::write(&mut self.inner, Default::default()); }
		}
	}
    fn channel_cb_slot(&mut self) -> &mut *mut ::udi_sys::imc::udi_channel_event_cb_t {
        &mut self.channel_cb
    }
    unsafe fn drop_in_place(&mut self) {
		// Do nothing, this is not a channel context
	}
}
impl<Driver> ::core::ops::Deref for RData<Driver>
{
	type Target = Driver;
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
impl<Driver> ::core::ops::DerefMut for RData<Driver>
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

// TODO: Figure out where we can store state properly
// - Probably in `context`, as `scratch` is limited and not always available :(
// - But, what are the rules for `context` being updated?

future_wrapper!{enumerate_req_op => <T as Driver>(cb: *mut udi_enumerate_cb_t, enumeration_level: u8) val @ {
	let attrs = AttrSink {
		dst: cb.attr_list,
		remaining_space: T::MAX_ATTRS as usize,
		pd: ::core::marker::PhantomData,
		};
	let enumeration_level = match enumeration_level
		{
		crate::ffi::meta_mgmt::UDI_ENUMERATE_START => EnumerateLevel::Start,
		crate::ffi::meta_mgmt::UDI_ENUMERATE_START_RESCAN => EnumerateLevel::StartRescan,
		crate::ffi::meta_mgmt::UDI_ENUMERATE_NEXT => EnumerateLevel::Next,
		crate::ffi::meta_mgmt::UDI_ENUMERATE_NEW => EnumerateLevel::New,
		crate::ffi::meta_mgmt::UDI_ENUMERATE_DIRECTED => EnumerateLevel::Directed,
		crate::ffi::meta_mgmt::UDI_ENUMERATE_RELEASE => EnumerateLevel::Release,
		_ => todo!(),
		};
	val.enumerate_req(cb, enumeration_level, attrs)
} finally( (res,attrs) ) {
	// Return this CB to the pool on completion
	unsafe {
		use ffi::udi_index_t;
		let (res,ops_idx) = match res
			{
			EnumerateResult::Ok(child) => {
				(*cb).child_id = child.child_id;
				(0,child.ops_idx)
				},
			EnumerateResult::Leaf => (1,udi_index_t(0)),
			EnumerateResult::Done => (2,udi_index_t(0)),
			EnumerateResult::Rescan => (3,udi_index_t(0)),
			EnumerateResult::Removed => (4,udi_index_t(0)),
			EnumerateResult::RemovedSelf => (5,udi_index_t(0)),
			EnumerateResult::Released => (6,udi_index_t(0)),
			EnumerateResult::Failed => (255,udi_index_t(0)),
			};
		(*cb).attr_valid_length = attrs.dst.offset_from((*cb).attr_list).try_into().expect("BUG: Attr list too long");
		crate::ffi::meta_mgmt::udi_enumerate_ack(cb, res, ops_idx)
	}
}}
future_wrapper!{devmgmt_req_op => <T as Driver>(cb: *mut udi_mgmt_cb_t, mgmt_op: crate::ffi::udi_ubit8_t, parent_id: crate::ffi::udi_ubit8_t) val @ {
	let mgmt_op = match mgmt_op
		{
		1 => MgmtOp::PrepareToSuspend,
		2 => MgmtOp::Suspend,
		3 => MgmtOp::Shutdown,
		4 => MgmtOp::ParentSuspend,
		5 => MgmtOp::Resume,
		6 => MgmtOp::Unbind,
		_ => panic!("Unexpected value for `mgmt_op`: {}", mgmt_op),
		};
	val.devmgmt_req(cb, mgmt_op, parent_id)
} finally(res) {
	unsafe {
		let (status,flags) = match res
			{
			Ok(f) => (0,f),
			Err(e) => (e.into_inner(),0)
			};
		crate::ffi::meta_mgmt::udi_devmgmt_ack(cb, flags, status)
	}
}}
future_wrapper!{final_cleanup_req_op => <T as Driver>(cb: *mut udi_mgmt_cb_t) val @ {
	async move {
		let _ = cb;
		// SAFE: We're trusting the environment to only call this once per region
		unsafe { ::core::ptr::drop_in_place(val); }
	}
} finally( () ) {
	unsafe { crate::ffi::meta_mgmt::udi_final_cleanup_ack(cb) }
}}

impl<T,CbList> crate::OpsStructure<::udi_sys::meta_mgmt::udi_mgmt_ops_t, RData<T> ,CbList>
where
	RData<T>: Driver,
	T: Default,
{
    pub const fn scratch_requirement() -> usize {
        let rv = 0;
		let rv = crate::const_max(rv, async_trickery::task_size::< <RData<T> as Driver>::Future_init<'static> >());
		let rv = crate::const_max(rv, enumerate_req_op::task_size::<RData<T>>());
		let rv = crate::const_max(rv, devmgmt_req_op::task_size::<RData<T>>());
		let rv = crate::const_max(rv, final_cleanup_req_op::task_size::<RData<T>>());
		rv
    }
    pub const unsafe fn for_driver() -> ffi::meta_mgmt::udi_mgmt_ops_t {
        // ENTRYPOINT: mgmt_ops.usage_ind
        unsafe extern "C" fn usage_ind<T>(cb: *mut udi_usage_cb_t, resource_level: u8)
		where
			RData<T>: Driver,
			T: Default,
        {
			// This can be called at any time, so needs to handle that.
			let rd = (*cb).gcb.context as *mut RData<T>;
			// If `false` (zero) - then we need to initialise the inner before any access
			if !(*rd).is_init {
				(*rd).is_init = true;
				::core::ptr::write(&mut (*rd).inner, Default::default());
			}
			async_trickery::init_task(&*cb,
				(*rd).usage_ind(crate::CbRef::new(cb), resource_level),
				|cb,()| ffi::meta_mgmt::udi_usage_res(cb)
			);
        }
        ffi::meta_mgmt::udi_mgmt_ops_t {
			usage_ind_op: usage_ind::<T>,
            enumerate_req_op: enumerate_req_op::<RData<T>>,
            devmgmt_req_op: devmgmt_req_op::<RData<T>>,
            final_cleanup_req_op: final_cleanup_req_op::<RData<T>>,
			}
    }
}