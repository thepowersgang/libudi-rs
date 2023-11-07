//! Trickery to convert between completion-based async and polling async
//!
//!
//
//
// According to udi `core_spec_vol1.pdf` 5.2.2.1, `scratch` is preserved over async calls
// so we can (... hopefully) use it as storage for the async task structure
//
// However, it might resize in some circumstances? And the docs aren't super clear as to when
// scratch is invalidated.
use ::core::task::Poll;
use ::core::pin::Pin;
use ::core::marker::{PhantomData,Unpin};
use ::core::future::Future;
use crate::ffi::udi_cb_t;

pub trait CbContext {
	fn channel_cb_slot(&mut self) -> &mut *mut crate::ffi::imc::udi_channel_event_cb_t;
}

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
	::core::ptr::write(cb.get_gcb().scratch as *mut _, Task::<T,Cb>::new(inner));
	run(cb);
}
/// Get the size of the task state (for scratch) for a given async state structure
pub(crate) const fn task_size<T: 'static+AsyncState>() -> usize {
	::core::mem::size_of::<Task<T,udi_cb_t>>()
}
/// Drop a task (due to a channel op_abort event)
pub(crate) unsafe fn abort_task(cb: *mut udi_cb_t)
{
	let task = &mut *((*cb).scratch as *mut Task<(),udi_cb_t>);
	let get_inner = task.get_inner;
	(*(get_inner(task) as *mut dyn TaskTrait)).drop_in_place();
}

/// Obtain a pointer to the driver instance from a cb
pub(crate) unsafe fn get_rdata_t<T: CbContext, Cb: GetCb>(cb: &Cb) -> &mut T {
	&mut *(cb.get_gcb().context as *mut T)
}
/// Set the channel operation cb
pub(crate) unsafe fn set_channel_cb<T: CbContext>(cb: *mut crate::ffi::imc::udi_channel_event_cb_t) {
	let slot = get_rdata_t::<T,_>(&*cb).channel_cb_slot();
	if *slot != ::core::ptr::null_mut() {
		// Uh-oh
	}
	*slot = cb;
}
pub(crate) unsafe fn channel_event_complete<T: CbContext, Cb: GetCb>(cb: *mut Cb, status: crate::ffi::udi_status_t) {
	let slot = get_rdata_t::<T,_>(&*cb).channel_cb_slot();
	let channel_cb = *slot;
	*slot = ::core::ptr::null_mut();
	if channel_cb == ::core::ptr::null_mut() {
		// Uh-oh
	}
	crate::ffi::imc::udi_channel_event_complete(channel_cb, status);
}

/// Run async state stored in `cb`
/// 
/// SAFETY: Caller must ensure that the cb is async
unsafe fn run<Cb: GetCb>(cb: &Cb) {
	let gcb = cb.get_gcb();
	let scratch = Pin::new(&mut *( (*gcb).scratch as *mut Task<(),udi_cb_t>));
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
pub(crate) fn wait_task<Cb,F1,F2,U>(_cb: crate::CbRef<Cb>, start: F1, map_result: F2) -> impl Future<Output=U>
where
	Cb: GetCb,
	F1: FnOnce(*mut udi_cb_t) + Unpin,
	F2: FnOnce(WaitRes) -> U + Unpin,
	U: Unpin,
{
	start(_cb.to_raw() as *mut udi_cb_t);
	WaitTask::<Cb,F1,F2,U> {
		_f1_pd: PhantomData,
		//f1: Some(start),
		f2: Some(map_result),
		_pd: PhantomData,
	}
}

/// Wrap a task to call an ack function once it completes
pub(crate) const fn with_ack<Cb, Fut, Res, Ack>(f: Fut, ack: Ack) -> WithAck<Cb, Fut, Res, Ack>
where
	Fut: Future<Output=Res>,
	Cb: GetCb,
	Ack: FnOnce(*mut Cb,Res)
{
	WithAck { f, ack: Some(ack), _pd: PhantomData }
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
struct Task<T,Cb> {
	pd: PhantomData<Cb>,
	/// Current waiting state
	state: ::core::cell::Cell<TaskState>,
	/// Effectively the vtable for this task
	get_inner: unsafe fn(*const Self)->*const dyn TaskTrait,
	/// Actual task/future data
	inner: T,
}
#[derive(Default,Copy,Clone)]
enum TaskState {
	/// The task is running, or is currently in a call
	#[default]
	Idle,
	/// The task is now waiting on a call
	Waiting,
	/// A callback has been called
	Ready(WaitRes),
}

trait TaskTrait {
	unsafe fn get_async<'a>(&'a mut self) -> &'a mut dyn AsyncState;
	fn get_cb_type(&self) -> ::core::any::TypeId;
	unsafe fn drop_in_place(&mut self);
}
impl<T: 'static + AsyncState, Cb: GetCb> Task<T, Cb>
{
	fn new(inner: T) -> Self {
		Task {
			pd: PhantomData,
			state: Default::default(),
			get_inner: Self::get_inner,
			inner,
		}
	}
	unsafe fn get_inner(this: *const Self) -> *const dyn TaskTrait {
		this
	}
}
impl<T,Cb> TaskTrait for Task<T,Cb>
where
	T: 'static + AsyncState,
	Cb: GetCb
{
    unsafe fn get_async<'a>(&'a mut self) -> &'a mut dyn AsyncState {
        &mut self.inner
    }
    fn get_cb_type(&self) -> ::core::any::TypeId {
        ::core::any::TypeId::of::<Cb>()
    }
	unsafe fn drop_in_place(&mut self) {
		::core::ptr::drop_in_place(self);
	}
}
impl Task<(),udi_cb_t>
{
	pub fn get_future(self: Pin<&mut Self>) -> Pin<&mut dyn Future<Output=()>> {
		let v = self.get_inner;
		let this = unsafe { Pin::get_unchecked_mut(self) };
		// SAFE: Pin projecting
		unsafe { Pin::new_unchecked( (*((v)(this as *mut Self as *const Self) as *mut dyn TaskTrait)).get_async() ).get_future() }
	}
}

struct WaitTask<Cb,F1,F2,U>
{
	_f1_pd: PhantomData<F1>,
	//f1: Option<F1>,
	f2: Option<F2>,
	_pd: PhantomData<(*const Cb, fn(*mut udi_cb_t), fn(WaitRes)->U)>,
}
impl<F1,F2,U,Cb> Future for WaitTask<Cb,F1,F2,U>
where
	Cb: GetCb + Unpin,
	F1: FnOnce(*mut udi_cb_t) + Unpin,
	F2: FnOnce(WaitRes) -> U + Unpin,
	U: Unpin,
{
	type Output = U;
	fn poll(mut self: Pin<&mut Self>, cx: &mut ::core::task::Context<'_>) -> Poll<Self::Output> {
		// get cb out of `cx`
		let cb: &udi_cb_t = cb_from_waker(cx.waker());
		/*if let Some(fcn) = self.f1.take() {
			// Register "wakeup"
			(fcn)(cb as *const _ as *mut _);
		}
		*/
		if let Some(res) = get_result( (*cb).get_gcb()) {
			let fcn = self.f2.take().expect("Completed future polled again");
			Poll::Ready(fcn(res))
		}
		else {
			Poll::Pending
		}
	}
}

pub(crate) struct WithAck<Cb, Fut, Res, Ack>
{
	f: Fut,
	ack: Option<Ack>,
	_pd: PhantomData<(fn(Cb,Res),)>
}
impl<Cb, Fut, Res, Ack> Future for WithAck<Cb, Fut, Res, Ack>
where
	Fut: Future<Output=Res>,
	Cb: GetCb,
	Ack: FnOnce(*mut Cb,Res)
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut ::core::task::Context<'_>) -> Poll<Self::Output> {
		match pin_project!(self, f).poll(cx)
		{
		Poll::Pending => Poll::Pending,
		Poll::Ready(res) => {
			let cb = cb_from_waker(cx.waker());
			unsafe { (self.get_unchecked_mut().ack.take().unwrap())(cb as *const Cb as *mut _, res); }
			Poll::Ready(())
			}
		}
    }
}

/// Obtain the GCB (`udi_cb_t`) from a waker
pub fn gcb_from_waker(waker: &::core::task::Waker) -> &udi_cb_t {
	let raw_waker = waker.as_raw();
	let have_vt = raw_waker.vtable();
	if have_vt as *const _ != &VTABLE_CB_T as *const _ {
		panic!("Unexpected context used!");
	}
	// SAFE: As this waker is for a CB, it has to be pointing at a valid CB
	unsafe { &*(raw_waker.data() as *const udi_cb_t) }
}
/// Obtain any CB (checked) from the waker
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
	let cb_type = unsafe {
		let task = &*(gcb.scratch as *const Task<(),udi_cb_t>);
		(*(task.get_inner)(task)).get_cb_type()
		};
	assert!(cb_type == ::core::any::TypeId::of::<Cb>(),
		"cb_from_waker with mismatched types: {:?} != {:?}", cb_type, ::core::any::TypeId::of::<Cb>());
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
pub unsafe trait GetCb: ::core::any::Any + Unpin
{
	fn get_gcb(&self) -> &udi_cb_t;
}
unsafe impl GetCb for udi_cb_t {
	fn get_gcb(&self) -> &udi_cb_t {
		self
	}
}
unsafe impl<T: crate::metalang_trait::MetalangCb + ::core::any::Any + Unpin> GetCb for T {
	fn get_gcb(&self) -> &udi_cb_t {
		// SAFE: The contract of `MetalangCb` is that the first field is a `udi_cb_t`
		unsafe {
			&*(self as *const T as *const udi_cb_t)
		}
	}
}

fn get_result(gcb: *const udi_cb_t) -> Option<WaitRes>
{
	let state = unsafe { &*((*gcb).scratch as *mut Task<(),udi_cb_t>) };
	match state.state.replace(TaskState::Waiting)
	{
	TaskState::Idle => None,
	TaskState::Waiting => {
		// Should this be possible?
		None
		},
	TaskState::Ready(v) => {
		state.state.set(TaskState::Idle);
		Some(v)
		}
	}
}

/// Flag that an operation is complete. This might be run downstream of the main task.
pub(crate) fn signal_waiter(gcb: &mut udi_cb_t, res: WaitRes) {
	let scratch = unsafe { &mut *(gcb.scratch as *mut Task<(),udi_cb_t>) };
	match scratch.state.replace(TaskState::Ready(res))
	{
	TaskState::Idle => {
		// No run
		},
	TaskState::Waiting => {
		unsafe { run(gcb); }
		},
	TaskState::Ready(_) => {
		// How?
		},
	}
}


/// Define an async trait method
macro_rules! async_method {
    (fn $fcn_name:ident(&mut self$(, $a_n:ident: $a_ty:ty)*) -> $ret_ty:ty as $future_name:ident) => {
        #[allow(non_camel_case_types)]
        type $future_name<'s>: ::core::future::Future<Output=$ret_ty>;
        fn $fcn_name(&mut self$(, $a_n: $a_ty)*) -> Self::$future_name<'_>;
    };
    (fn $fcn_name:ident(&$lft:lifetime mut self$(, $a_n:ident: $a_ty:ty)*) -> $ret_ty:ty as $future_name:ident) => {
        #[allow(non_camel_case_types)]
        type $future_name<'s>: ::core::future::Future<Output=$ret_ty>;
        fn $fcn_name<$lft>(&$lft mut self$(, $a_n: $a_ty)*) -> Self::$future_name<$lft>;
    };
}
/// Define a FFI wrapper 
macro_rules! future_wrapper {
    ($name:ident => <$t:ident as $trait:path>::$method:ident($cb:ident: *mut $cb_ty:ty $(, $a_n:ident: $a_ty:ty)*) ) => {
        future_wrapper!($name => <$t:ty as $trait>::$method($cb: *mut $cb_ty, $(, $a_n: $a_ty)*))
    };
    ($name:ident => <$t:ident as $trait:path>($cb:ident: *mut $cb_ty:ty $(, $a_n:ident: $a_ty:ty)*) $val:ident @ $b:block ) => {
        unsafe extern "C" fn $name<T: $trait + $crate::async_trickery::CbContext>($cb: *mut $cb_ty$(, $a_n: $a_ty)*)
        {
            let job = {
				let $val = crate::async_trickery::get_rdata_t::<T,_>(&*$cb);
				let $cb = unsafe { crate::CbRef::new($cb) };
                $b
                };
            crate::async_trickery::init_task(&*$cb, job);
        }
        mod $name {
            use super::*;
            pub const fn task_size<$t: $trait>() -> usize {
                crate::async_trickery::task_size_from_closure(|$val: &mut $t, ($cb, $($a_n,)*): (crate::CbRef<$cb_ty>, $($a_ty,)*)| $b)
            }
        }
    };
}
/// Get the size of a task using a closure to resolve methods
pub const fn task_size_from_closure<'a, Cls,Cb,Args,Task>(_cb: Cls) -> usize
where
    Cls: FnOnce(&'a mut Cb, Args) -> Task,
    Task: 'a,
    Cb: 'a,
    Task: ::core::future::Future<Output=()> + 'static,
{
    ::core::mem::forget(_cb);
    crate::async_trickery::task_size::<Task>()
}