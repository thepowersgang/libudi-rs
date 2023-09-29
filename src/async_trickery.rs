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
}
/// A trait for top-level future types (stored in `scratch`)
pub(crate) trait AsyncState {
	fn get_future(self: Pin<&mut Self>) -> Pin<&mut dyn Future<Output=()>>;
}

pub(crate) unsafe fn init_task<T: 'static+AsyncState>(gcb: &mut udi_cb_t, inner: T)
{
	::core::ptr::write(gcb.scratch as *mut _, Task::new(inner));
}
pub(crate) const fn task_size<T: 'static+AsyncState>() -> usize {
	::core::mem::size_of::<Task<T>>()
}

pub(crate) unsafe fn run<Cb: GetCb>(cb: &mut Cb) {
	let gcb = cb.get_gcb();
	let scratch = Pin::new(&mut *( (*gcb).scratch as *mut Task<()>));
	let f = scratch.get_future();
	
	let waker = cb.to_waker();
	let mut ctxt = ::core::task::Context::from_waker(&waker);
	match f.poll(&mut ctxt)
	{
	Poll::Ready( () ) => {},
	Poll::Pending => {},
	}
}
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

// An async task state
#[repr(C)]
struct Task<T> {
	// TODO: Add typeid of the initial CB used
	waiting: ::core::cell::Cell<bool>,
	res: ::core::cell::Cell< Option<WaitRes> >,
	get_async: unsafe fn(&mut ())->&mut dyn AsyncState,
	inner: T,
}

impl<T: 'static + AsyncState> Task<T>
{
	fn new(inner: T) -> Self {
		Task {
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
		let cb = Cb::from_waker(cx.waker());
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


macro_rules! make_null_waker_vt {
	( $addr:expr ) => {
	::core::task::RawWakerVTable::new(
		|v| ::core::task::RawWaker::new(v, $addr),
		|_| (),
		|_| (),
		|_| (),
		)
	}
}
fn checked_waker(waker: &::core::task::Waker, vt: &'static ::core::task::RawWakerVTable) -> *const () {
	let raw_waker = waker.as_raw();
	let have_vt = raw_waker.vtable();
	if have_vt as *const _ != vt as *const _ {
		panic!("Unexpected context used!");
	}
	raw_waker.data()
}
unsafe fn make_waker(ptr: *const (), vt: &'static ::core::task::RawWakerVTable) -> ::core::task::Waker {
	::core::task::Waker::from_raw( ::core::task::RawWaker::new(ptr, vt) )
}

pub(crate) trait GetCb
{
	fn from_waker(w: &::core::task::Waker) -> &Self;
	fn to_waker(&self) -> ::core::task::Waker;
	fn get_gcb(&self) -> &udi_cb_t;
}

static VTABLE_CB_T: ::core::task::RawWakerVTable = make_null_waker_vt!(&VTABLE_CB_T);
impl GetCb for udi_cb_t {
	fn from_waker(w: &::core::task::Waker) -> &Self {
		// SAFE: Checked that the vtable matches below
		unsafe { &*(checked_waker(w, &VTABLE_CB_T) as *const _) }
	}
	fn to_waker(&self) -> ::core::task::Waker {
		unsafe { make_waker(self as *const _ as *const _, &VTABLE_CB_T) }
	}
	fn get_gcb(&self) -> &udi_cb_t {
		self
	}
}
static VTABLE_USAGE_CB_T: ::core::task::RawWakerVTable = make_null_waker_vt!(&VTABLE_USAGE_CB_T);
impl GetCb for crate::ffi::meta_mgmt::udi_usage_cb_t {
	fn from_waker(w: &::core::task::Waker) -> &Self {
		// SAFE: Checked that the vtable matches below
		unsafe { &*(checked_waker(w, &VTABLE_USAGE_CB_T) as *const _) }
	}
	fn to_waker(&self) -> ::core::task::Waker {
		unsafe { make_waker(self as *const _ as *const _, &VTABLE_USAGE_CB_T) }
	}
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

