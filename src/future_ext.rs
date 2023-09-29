use ::core::future::Future;
use ::core::task::Poll;
use ::core::pin::Pin;
use ::core::marker::PhantomData;

macro_rules! pin_project {
	($v:expr, $($fld:ident).+) => {
		unsafe { Pin::new_unchecked( &mut Pin::get_unchecked_mut(Pin::as_mut(&mut $v)) $(.$fld)+ ) }
	}
}

pub trait FutureExt: Future
{
	fn map<F, U>(self, op: F) -> Map<Self,F,U>
	where
		Self: Sized,
		F: FnOnce(Self::Output)->U
	;
}
impl<T: Future> FutureExt for T
{
	fn map<F, U>(self, op: F) -> Map<Self,F,U>
	where
		F: FnOnce(Self::Output)->U
	{
		Map { inner: self, cb: Some(op), _pd: PhantomData, }
	}
}

pub struct Map<I,F,U>
{
	inner: I,
	cb: Option<F>,
	_pd: PhantomData<fn()->U>,
}
impl<I,F,U> Future for Map<I,F,U>
where
	I: Future,
	F: FnOnce(I::Output)->U
{
	type Output = U;
	fn poll(mut self: Pin<&mut Self>, cx: &mut ::core::task::Context<'_>) -> Poll<Self::Output> {
		match pin_project!(self, inner).poll(cx)
		{
		Poll::Ready(v) => Poll::Ready(unsafe { Pin::into_inner_unchecked(self).cb.take().unwrap()(v) }),
		Poll::Pending => Poll::Pending,
		}
	}
}

