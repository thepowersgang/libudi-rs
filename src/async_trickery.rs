//!
//!
//!
use ::core::task::Poll;
use ::core::pin::Pin;
use ::core::marker::{PhantomData,Unpin};
use ::core::future::Future;
use crate::ffi::udi_cb_t;

#[derive(Copy,Clone)]
pub(crate) enum WaitRes {
	//Unit,
	Pointer(*mut ()),
	Data([usize; 3]),
}
/// A trait for top-level future types (stored in `scratch`)
pub(crate) trait AsyncState {
	fn get_future(self: Pin<&mut Self>) -> Pin<&mut dyn Future<Output=()>>;
}
impl<F> AsyncState for F
where
	F: Future<Output=()>
{
	fn get_future(self: Pin<&mut Self>) -> Pin<&mut dyn Future<Output=()>> {
		self
	}
}

/// Initialise a task
/// 
/// SAFETY: Caller must ensure that `cb`'s `scratch` is valid for this task
pub(crate) unsafe fn init_task<Cb: GetCb, T: 'static+AsyncState>(cb: &Cb, inner: T)
{
	::core::ptr::write(cb.get_gcb().scratch as *mut _, Task::new::<Cb>(inner));
	run(cb);
}
/// Get the size of the task state (for scratch) for a given async state structure
pub(crate) const fn task_size<T: 'static+AsyncState>() -> usize {
	::core::mem::size_of::<Task<T>>()
}

/// Run async state stored in `cb`
/// 
/// SAFETY: Caller must ensure that the cb is async
unsafe fn run<Cb: GetCb>(cb: &Cb) {
	let gcb = cb.get_gcb();
	// TODO: Scratch isn't the right place for this, needs to be in context
	// - Scratch is limited in size, and not all operations fill it?
	let scratch = Pin::new(&mut *( (*gcb).scratch as *mut Task<()>));
	let f = scratch.get_future();
	
	let waker = make_waker(gcb);
	let mut ctxt = ::core::task::Context::from_waker(&waker);
	match f.poll(&mut ctxt)
	{
	Poll::Ready( () ) => {},
	Poll::Pending => {},
	}
}
/// Call an async UDI function
/// 
/// `start` should call the function, passing a closure that runs [signal_waiter]
/// `map_result` converts the wait result into the output type
pub(crate) fn wait_task<Cb,F1,F2,U>(start: F1, map_result: F2) -> impl Future<Output=U>
where
	Cb: GetCb + Unpin,
	F1: FnOnce(&Cb) + Unpin,
	F2: FnOnce(WaitRes) -> U + Unpin,
	U: Unpin,
{
	WaitTask::<Cb,F1,F2,U> {
		f1: Some(start),
		f2: Some(map_result),
		_pd: PhantomData,
	}
}

/// Obtain a value by introspecting the cb
pub(crate) fn with_cb<Cb,F,U>(f: F) -> impl Future<Output=U>
where
	Cb: GetCb,
	F: FnOnce(&Cb) -> U + Unpin,
{
	return W::<Cb,F,U,> {
		f: Some(f),
		_pd: Default::default()
		};
	struct W<Cb,F,U> {
		f: Option<F>,
		_pd: PhantomData<(fn(&Cb)->U,)>,
	}
	impl<Cb,F,U> Future for W<Cb,F,U>
	where
		Cb: GetCb,
		F: FnOnce(&Cb)->U + Unpin,
	{
		type Output = U;
		fn poll(mut self: Pin<&mut Self>, cx: &mut ::core::task::Context<'_>) -> Poll<Self::Output> {
			// get cb out of `cx`
			let cb = cb_from_waker(cx.waker());
			let fcn = self.f.take().expect("Completed future polled again");
			Poll::Ready(fcn(cb))
		}
	}
}

// An async task state
#[repr(C)]
struct Task<T> {
	// NOTE: I would love to be able to remove all of this state if the contained task is empty
	/// TypeId of the control block (allows casting)
	cb_typeid: ::core::any::TypeId,
	/// Flag indicating that this task is currently waiting (so should be resumed)
	waiting: ::core::cell::Cell<bool>,
	/// Result of the most recent 
	res: ::core::cell::Cell< Option<WaitRes> >,
	/// Effectively the vtable for this task
	get_async: unsafe fn(&mut ())->&mut dyn AsyncState,
	/// Actual task/future data
	inner: T,
}

impl<T: 'static + AsyncState> Task<T>
{
	fn new<Cb: GetCb>(inner: T) -> Self {
		Task {
			cb_typeid: ::core::any::TypeId::of::<Cb>(),
			waiting: Default::default(),
			res: Default::default(),
			get_async: Self::get_async,
			inner,
		}
	}
	unsafe fn get_async(v: &mut ()) -> &mut dyn AsyncState {
		&mut *(v as *mut () as *mut T)
	}
}
impl Task<()>
{
	pub fn get_future(self: Pin<&mut Self>) -> Pin<&mut dyn Future<Output=()>> {
		// SAFE: Pin projecting
		unsafe { Pin::new_unchecked( (self.get_async)(&mut Pin::get_unchecked_mut(self).inner) ).get_future() }
	}
}

struct WaitTask<Cb,F1,F2,U>
{
	f1: Option<F1>,
	f2: Option<F2>,
	_pd: PhantomData<(fn(&Cb), fn(WaitRes)->U)>,
}
impl<F1,F2,U,Cb> Future for WaitTask<Cb,F1,F2,U>
where
	Cb: GetCb + Unpin,
	F1: FnOnce(&Cb) + Unpin,
	F2: FnOnce(WaitRes) -> U + Unpin,
	U: Unpin,
{
	type Output = U;
	fn poll(mut self: Pin<&mut Self>, cx: &mut ::core::task::Context<'_>) -> Poll<Self::Output> {
		// get cb out of `cx`
		let cb = cb_from_waker(cx.waker());
		if let Some(fcn) = self.f1.take() {
			// Register "wakeup"
			(fcn)(cb);
		}
		if let Some(res) = get_result( (*cb).get_gcb()) {
			let fcn = self.f2.take().expect("Completed future polled again");
			Poll::Ready(fcn(res))
		}
		else {
			Poll::Pending
		}
	}
}


pub fn gcb_from_waker(waker: &::core::task::Waker) -> &udi_cb_t {
	let raw_waker = waker.as_raw();
	let have_vt = raw_waker.vtable();
	if have_vt as *const _ != &VTABLE_CB_T as *const _ {
		panic!("Unexpected context used!");
	}
	// SAFE: As this waker is for a CB, it has to be pointing at a valid CB
	unsafe { &*(raw_waker.data() as *const udi_cb_t) }
}
pub(crate) fn cb_from_waker<Cb: GetCb>(waker: &::core::task::Waker) -> &Cb {
	let exp_typeid = ::core::any::TypeId::of::<Cb>();
	let gcb = gcb_from_waker(waker);
	// Special case: If we're asking for `udi_cb_t` then allow it
	if exp_typeid == ::core::any::TypeId::of::<udi_cb_t>() {
		// SAFE: Same type!
		return unsafe { &*(gcb as *const udi_cb_t as *const Cb) };
	}

	// A null scratch indicates that no state was needed
	assert!( !gcb.scratch.is_null(), "cb_from_waker with no state?" );
	// SAFE: Since the waker is from a cb, that cb has/should have been for an active task. The scratch is non-null
	let task = unsafe { &*(gcb.scratch as *const Task<()>) };
	assert!(task.cb_typeid == ::core::any::TypeId::of::<Cb>(),
		"cb_from_waker with mismatched types: {:?} != {:?}", task.cb_typeid, ::core::any::TypeId::of::<Cb>());
	// SAFE: Correct type
	unsafe { &*(gcb as *const udi_cb_t as *const Cb) }
}
unsafe fn make_waker(cb: &udi_cb_t) -> ::core::task::Waker {
	::core::task::Waker::from_raw( ::core::task::RawWaker::new(cb as *const _ as *const _, &VTABLE_CB_T) )
}
static VTABLE_CB_T: ::core::task::RawWakerVTable = ::core::task::RawWakerVTable::new(
	|_| panic!("Cloning would be unsound"),
	|_| panic!("No waking"),
	|_| panic!("No waking"),
	|_| (),
	);

/// SAFETY: `get_gcb` must return the first field of the struct
pub(crate) unsafe trait GetCb: ::core::any::Any
{
	fn get_gcb(&self) -> &udi_cb_t;
}

unsafe impl GetCb for udi_cb_t {
	fn get_gcb(&self) -> &udi_cb_t {
		self
	}
}
unsafe impl GetCb for crate::ffi::meta_mgmt::udi_usage_cb_t {
	fn get_gcb(&self) -> &udi_cb_t {
		&self.gcb
	}
}

fn get_result(gcb: *const udi_cb_t) -> Option<WaitRes>
{
	let state = unsafe { &*((*gcb).scratch as *mut Task<()>) };
	if let Some(v) = state.res.take() {
		state.waiting.set(false);
		Some(v)
	}
	else {
		state.waiting.set(true);
		None
	}
}

/// Flag that an operation is complete. This might be run downstream of the main task.
pub(crate) fn signal_waiter(gcb: &mut udi_cb_t, res: WaitRes) {
	let scratch = unsafe { &mut *(gcb.scratch as *mut Task<()>) };
	scratch.res.set(Some(res));
	if scratch.waiting.get() {
		unsafe { run(gcb); }
	}
}

