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

/// Trait for the `context` field in a CB
// TODO: Init is covered for now, but what about deallocation
// - On unbind it can be dropped, but it's possible to use the same context for multiple channels
pub trait CbContext {
	fn is_init(&self) -> bool;
	fn maybe_init(&mut self);
	fn channel_cb_slot(&mut self) -> &mut *mut crate::ffi::imc::udi_channel_event_cb_t;
	unsafe fn drop_in_place(&mut self);
}

/// Result of a wait operation, stored in `gcb.scratch`
#[derive(Copy,Clone)]
pub(crate) enum WaitRes {
	//Unit,
	Pointer(*mut ()),
	PointerResult(crate::Result<*mut ()>),
	Data([usize; 4]),
}

/// Initialise a task
/// 
/// SAFETY: Caller must ensure that `cb`'s `scratch` is valid for this task (correct size, not yet initialised)
pub(crate) unsafe fn init_task<Cb, T, R, F>(cb: *mut Cb, inner: T, finally: F)
where
	Cb: GetCb,
	T: 'static + Future<Output=R>,
	R: 'static,
	F: 'static + FnMut(*mut Cb, R),
{
	::core::ptr::write((*cb).get_gcb().scratch as *mut _, Task::<Cb,T,R,F>::new(inner, finally));
	// NOTE: Can't run here, as that makes miri unhappy (if the task doesn't yield, and the drop happens in here)
}
/// Get the size of the task state (for scratch) for a given async state structure
pub(crate) const fn task_size<T: 'static>() -> usize {
	::core::mem::size_of::<Task<udi_cb_t,T,(),()>>()
}
/// Drop a task (due to a channel op_abort event)
/// 
/// SAFETY: Takes a raw pointer, that pointer must be the valid CB for an aborted task
pub(crate) unsafe fn abort_task(cb: *mut udi_cb_t)
{
	let task = &mut *((*cb).scratch as *mut TaskHeader);
	let vt = task.vtable;
	(vt.drop_in_place)( (*cb).scratch as *mut () );
}

/// Obtain a pointer to the driver instance from a cb
/// 
/// SAFETY: Caller must ensure that `T` is valid for the context paraneter of the Cb
pub(crate) unsafe fn get_rdata_t<T: CbContext, Cb: GetCb>(cb: &Cb) -> &T {
	let rv_raw = cb.get_gcb().context as *mut T;
	if !(*rv_raw).is_init() {
		(*rv_raw).maybe_init();
	}
	&*rv_raw
}
/// Obtain a pointer to the driver instance from a cb
/// 
/// SAFETY: Caller must ensure that `T` is valid for the context paraneter of the Cb
pub(crate) unsafe fn get_rdata_t_mut<T: CbContext, Cb: GetCb>(cb: &Cb) -> &mut T {
	let rv = &mut *(cb.get_gcb().context as *mut T);
	rv.maybe_init();
	rv
}
/// Set the channel operation cb
/// 
/// SAFETY: Caller must ensure that `cb` is a valid pointer, and that the context field points to a `T`
pub(crate) unsafe fn set_channel_cb<T: CbContext>(cb: *mut crate::ffi::imc::udi_channel_event_cb_t) {
	let slot = get_rdata_t_mut::<T,_>(&*cb).channel_cb_slot();
	if *slot != ::core::ptr::null_mut() {
		// Uh-oh, 
		panic!("Channel CB was already set");
	}
	*slot = cb;
}
/// Call `udi_channel_event_complete` using the saved event CB (not the passed cb)
/// 
/// SAFETY: Caller must ensure that `cb` is a valid pointer, and that the context field points to a `T`
pub(crate) unsafe fn channel_event_complete<T: CbContext, Cb: GetCb>(cb: *mut Cb, status: crate::ffi::udi_status_t) {
	let slot = get_rdata_t_mut::<T,_>(&*cb).channel_cb_slot();
	let channel_cb = ::core::mem::replace(slot, ::core::ptr::null_mut());
	if channel_cb == ::core::ptr::null_mut() {
		// Uh-oh, no channel CB set
		panic!("no channel CB set")
	}
	crate::ffi::imc::udi_channel_event_complete(channel_cb, status);
}

/// Run async state stored in `cb`
/// 
/// SAFETY: Caller must ensure that the cb has been initialised with an async state
pub unsafe fn run<Cb: GetCb>(cb: *mut Cb) {
	let scratch = (*cb).get_gcb().scratch;
	let waker = make_waker(cb);
	let mut ctxt = ::core::task::Context::from_waker(&waker);
	let vtable = (*(scratch as *mut TaskHeader)).vtable;
	
	match (vtable.poll)( scratch as *mut (), &mut ctxt )
	{
	Poll::Ready( () ) => { },
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

/// Obtain a value by introspecting the cb
pub(crate) fn with_cb<Cb,F,U>(f: F) -> impl Future<Output=U>
where
	Cb: GetCb,
	F: FnOnce(&Cb) -> U + Unpin,
{
	struct WithCbFuture<Cb,F,U> {
		f: Option<F>,
		_pd: PhantomData<(fn(&Cb)->U,)>,
	}
	impl<Cb,F,U> Future for WithCbFuture<Cb,F,U>
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

	WithCbFuture::<Cb,F,U,> {
		f: Some(f),
		_pd: Default::default()
		}
}

/// Top-level async task state (stored in `gcb.scratch`)
#[repr(C)]
struct Task<Cb,T,R,F> {
	pd: PhantomData<(Cb,R,)>,
	/// Common header, must be the first field
	header: TaskHeader,
	/// Actual task/future data
	inner: T,
	finally: ::core::mem::ManuallyDrop<F>,
}
struct TaskHeader {
	/// Current waiting state
	state: ::core::cell::Cell<TaskState>,
	/// Effectively the vtable for this task
	vtable: &'static TaskVtable,
}
struct TaskVtable {
	poll: unsafe fn(*mut (), &mut ::core::task::Context<'_>) -> ::core::task::Poll<()>,
	get_cb_type: fn()->::core::any::TypeId,
	drop_in_place: unsafe fn(*mut ()),
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

#[cfg(any())]
trait TaskTrait {
	/// Poll the inner future
	/// 
	/// SAFETY:
	/// - `self` must be pinned (i.e. once `poll` is called, it should never move)
	/// - Once this returns `Poll::Ready`, `self` must be considered invalid (it's dropped)
	unsafe fn poll<'a>(&mut self, cx: &mut ::core::task::Context<'_>) -> ::core::task::Poll<()>;
	/// Early drop the task (for cancellation)
	unsafe fn drop_in_place(&mut self);
}
impl<Cb, T, R, F> Task<Cb, T, R, F>
where
	Cb: GetCb,
	T: 'static + Future<Output=R>,
	R: 'static,
	F: 'static + FnOnce(*mut Cb, R)
{
	fn new(inner: T, finally: F) -> Self {
		Task {
			pd: PhantomData,
			header: TaskHeader {
				state: Default::default(),
				vtable: &TaskVtable {
					poll: Self::poll_raw,
					get_cb_type: || ::core::any::TypeId::of::<Cb>(),
					drop_in_place: |this| unsafe { ::core::ptr::drop_in_place(this as *mut Self) },
					}
			},
			finally: ::core::mem::ManuallyDrop::new(finally),
			inner,
		}
	}
	/// SAFETY: `this` must point to a valid instance of `Self`
	/// After this returns `Poll::Ready` - `this` is invalid
	unsafe fn poll_raw(this: *mut (), cx: &mut ::core::task::Context<'_>) -> ::core::task::Poll<()> {
		let this = this as *mut Self;
		match Pin::new_unchecked(&mut (*this).inner).poll(cx)
		{
		Poll::Ready(res) => {
			// Request the CB, to assert that the CB type is valid
			{ let _cb = cb_from_waker::<Cb>(cx.waker()); }
			// Read `finally` out of the ManuallDrop before dropping all of `this`
			let finally = ::core::ptr::read(&mut *(*this).finally);
			// Drop the future in `self.inner` (and everything else)
			::core::ptr::drop_in_place(this);
			// Call the cleanup function
			(finally)(gcb_from_waker_raw(cx.waker()) as *mut _, res);
			Poll::Ready(())
		},
		Poll::Pending => Poll::Pending,
		}
	}
}

/// Inner future for [wait_task]
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

/// Obtain the GCB (`udi_cb_t`) from a waker
fn gcb_from_waker_raw(waker: &::core::task::Waker) -> *const udi_cb_t {
	let raw_waker = waker.as_raw();
	let have_vt = raw_waker.vtable();
	if have_vt as *const _ != &VTABLE_CB_T as *const _ {
		panic!("Unexpected context used!");
	}
	raw_waker.data() as *const udi_cb_t
}
/// Obtain any CB (checked) from the waker
pub(crate) fn cb_from_waker<Cb: GetCb>(waker: &::core::task::Waker) -> &Cb {
	let exp_typeid = ::core::any::TypeId::of::<Cb>();
	let gcb = gcb_from_waker_raw(waker);
	// Special case: If we're asking for `udi_cb_t` then allow it
	if exp_typeid == ::core::any::TypeId::of::<udi_cb_t>() {
		// SAFE: Same type!
		return unsafe { &*(gcb as *const Cb) };
	}

	// SAFE: Since the waker is from a cb, that cb has/should have been for an active task. The scratch is non-null
	let cb_type = unsafe {
		// A null scratch indicates that no state was needed
		assert!( !(*gcb).scratch.is_null(), "cb_from_waker with no state?" );
		let vtable = (*((*gcb).scratch as *mut TaskHeader)).vtable;
		(vtable.get_cb_type)()
		};
	assert!(cb_type == ::core::any::TypeId::of::<Cb>(),
		"cb_from_waker with mismatched types: {:?} != {:?}", cb_type, ::core::any::TypeId::of::<Cb>());
	// SAFE: Correct type
	unsafe { &*(gcb as *const Cb) }
}
unsafe fn make_waker<Cb: GetCb>(cb: *mut Cb) -> ::core::task::Waker {
	::core::task::Waker::from_raw( ::core::task::RawWaker::new(cb as *mut Cb as *const (), &VTABLE_CB_T) )
}
static VTABLE_CB_T: ::core::task::RawWakerVTable = ::core::task::RawWakerVTable::new(
	|_| panic!("Cloning would be unsound"),
	|_| panic!("No waking"),
	|_| panic!("No waking"),
	|_| (),
	);

/// Trait for a CB type to get the inner GCB
/// 
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

/// Obtain the TaskState result given a GCB
fn get_result(gcb: *const udi_cb_t) -> Option<WaitRes>
{
	let state = unsafe { &*((*gcb).scratch as *mut TaskHeader) };
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
/// SAFETY: Caller must ensure that `gcb` is a valid async CB
pub(crate) unsafe fn signal_waiter(gcb: *mut udi_cb_t, res: WaitRes) {
	let scratch = unsafe { &mut *( (*gcb).scratch as *mut TaskHeader) };
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


/// Helper: Define an async trait method
/// 
/// Creates a method that returns an associated type (the name of which is after the `as` in the invocation).
/// 
/// ```ignore
/// trait Foo
/// {
///   ::udi::async_method!(fn bar(&self, arg: u8) -> u16 as Future_bar);
/// }
/// ```
#[macro_export]
macro_rules! async_method {
    ($(#[$a:meta])* fn $fcn_name:ident(&self$(, $a_n:ident: $a_ty:ty)*) -> $ret_ty:ty as $future_name:ident) => {
        #[allow(non_camel_case_types)]
		#[doc=concat!("Future type for `", stringify!($fcn_name),"`")]
        type $future_name<'s>: ::core::future::Future<Output=$ret_ty>;
		$( #[$a] )*
        fn $fcn_name<'s>(&'s self$(, $a_n: $a_ty)*) -> Self::$future_name<'s>;
    };
    ($(#[$a:meta])* fn $fcn_name:ident(&$lft:lifetime self$(, $a_n:ident: $a_ty:ty)*) -> $ret_ty:ty as $future_name:ident) => {
        #[allow(non_camel_case_types)]
		#[doc=concat!("Future type for `", stringify!($fcn_name),"`")]
        type $future_name<'s>: ::core::future::Future<Output=$ret_ty>;
		$( #[$a] )*
        fn $fcn_name<$lft>(&$lft self$(, $a_n: $a_ty)*) -> Self::$future_name<$lft>;
    };
}
/// Define a FFI wrapper that invokes a future
/// 
/// ```ignore
/// ::udi::future_wrapper!{udi_foo_bar_req => <T as FooTrait>::bar_req(cb: *mut udi_foo_cb_t, arg1: u8)}
/// ```
#[macro_export]
macro_rules! future_wrapper {
    ($name:ident => <$t:ident as $trait:path>::$method:ident($cb:ident: *mut $cb_ty:ty $(, $a_n:ident: $a_ty:ty)*) ) => {
        $crate::future_wrapper!($name => <$t as $trait>($cb: *mut $cb_ty $(, $a_n: $a_ty)*) val @ {
			val.$method($cb $(, $a_n)*)
		});
    };
    ($name:ident => <$t:ident as $trait:path>($cb:ident: *mut $cb_ty:ty $(, $a_n:ident: $a_ty:ty)*) $val:ident @ $b:block $( finally($res:pat) $f:block )? ) => {
        unsafe extern "C" fn $name<T: $trait + $crate::async_trickery::CbContext>($cb: *mut $cb_ty$(, $a_n: $a_ty)*)
        {
            let job = {
				let $val = unsafe { $crate::async_trickery::get_rdata_t::<T,_>(&*$cb) };
				let $cb = unsafe { $crate::CbRef::new($cb) };
                $b
                };
            $crate::async_trickery::init_task(&mut *$cb, job, |$cb, res| {
				let $val = unsafe { $crate::async_trickery::get_rdata_t::<T,_>(&*$cb) };
				let _ = $val;
				let _ = res;
				$( let $res = res; $f )?
			});
			$crate::async_trickery::run($cb);
        }
        mod $name {
            use super::*;
            pub const fn task_size<$t: $trait>() -> usize {
				#[allow(unused_variables)]
                $crate::async_trickery::task_size_from_closure(
					|$val: &mut $t, ($cb, $($a_n,)*): ($crate::CbRef<$cb_ty>, $($a_ty,)*)| $b,
					|$val: &mut $t, $cb: *mut $cb_ty, res| { $( let $res = res; $f )? }
				)
            }
        }
    };
}
/// Get the size of a task using a closure to resolve methods
pub const fn task_size_from_closure<'a, Closure,ValTy,Cb,Args,Task,Finally>(_cb: Closure, f: Finally) -> usize
where
    Closure: FnOnce(&'a mut ValTy, Args) -> Task,
    Task: 'a,
    ValTy: 'a,
    Task: ::core::future::Future + 'static,
	Finally: FnOnce(&'a mut ValTy, *mut Cb, Task::Output),
{
    ::core::mem::forget(_cb);
    ::core::mem::forget(f);
	::core::mem::size_of::<self::Task<Cb,Task,Task::Output,Finally>>()
}
