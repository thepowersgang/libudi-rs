use ::core::cell::Cell;
use super::CbHandle;

/// A FIFO capable queue of chained CBs (shared access supported)
pub struct SharedQueue<T>
{
    head: Cell<*mut T>,
    tail: Cell<*mut T>,
}
impl<T> Default for SharedQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> SharedQueue<T>
{
    pub const fn new() -> Self {
        SharedQueue {
            head: ::core::cell::Cell::new(::core::ptr::null_mut()),
            tail: ::core::cell::Cell::new(::core::ptr::null_mut()),
        }
    }
}
impl<T> SharedQueue<T>
where
    T: crate::metalang_trait::MetalangCb + crate::async_trickery::GetCb
{
    pub fn push_back(&self, cb: CbHandle<T>) {
        let cb = cb.into_raw();
        if self.head.get().is_null() {
            self.head.set(cb);
        }
        else {
            // SAFE: This type logically owns these pointers (so they're non-NULL)
            unsafe {
                assert!( !self.tail.get().is_null() );
                let s = get_chain_slot(&mut *self.tail.get());
                assert!( s.is_null() );
                *s = cb;
            }
        }
        // SAFE: Trusting the `chain` on incoming cbs to be a valid single-linked list
        unsafe {
            let mut tail = cb;
            loop {
                let s = get_chain_slot(&mut *tail);
                if s.is_null() {
                    break;
                }
                tail = *s;
            }
            self.tail.set(tail);
        }
    }
    pub fn pop_front(&self) -> Option< CbHandle<T> > {
        let rv = self.head.get();
        if rv.is_null() {
            None
        }
        else {
            // SAFE: The chain is a valid singularly-linked list of owned pointers
            unsafe {
                self.head.set( ::core::mem::replace(get_chain_slot(&mut *rv), ::core::ptr::null_mut()) );
                if self.head.get().is_null() {
                    // Defensive measure.
                    self.tail.set( ::core::ptr::null_mut() );
                }
                Some( CbHandle::from_raw(rv) )
            }
        }
    }
}

/// A chain of CBs, as returned by [alloc_batch]
/// 
/// This is a last-in-first-out collection (aka a stack)
pub struct Chain<T>( *mut T );
impl<T> Default for Chain<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Chain<T> {
    pub const fn new() -> Self {
        Chain( ::core::ptr::null_mut() )
    }
    pub const unsafe fn from_raw(p: *mut T) -> Self {
        Chain( p )
    }
}
impl<T> Chain<T>
where
    T: crate::metalang_trait::MetalangCb + crate::async_trickery::GetCb
{
    pub fn is_empty(&self) -> bool {
        self.0.is_null()
    }
    pub fn count(&self) -> usize {
        unsafe {
            let mut rv = 0;
            let mut p = self.0;
            while !p.is_null() {
                p = *Self::get_chain_slot(&mut *p);
                rv += 1;
            }
            rv
        }
    }
    pub fn pop_front(&mut self) -> Option<CbHandle<T>> {
        if self.0.is_null() {
            None
        }
        else {
            let rv = self.0;
            // SAFE: For a pointer to be in this structure, it must be chained using `get_chain_slot`
            let new_next = unsafe {
                let slot = Self::get_chain_slot(&mut *rv);
                ::core::mem::replace(slot, ::core::ptr::null_mut())
            };
            self.0 = new_next as *mut T;
            Some(CbHandle(rv))
        }
    }
    pub fn push_front(&mut self, cb: CbHandle<T>) {
        let cb = cb.into_raw();
        // SAFE: `cb` is from a `CbHandle` which is valid
        unsafe {
            let slot = Self::get_chain_slot(&mut *cb);
            *slot = self.0;
        }
        self.0 = cb;
    }

    fn get_chain_slot(cb: &mut T) -> &mut *mut T {
        unsafe fn cast_ptr_mutref<U,T>(p: &mut *mut U) -> &mut *mut T {
            &mut *(p as *mut _ as *mut *mut T)
        }
        // SAFE: Correct pointer manipulations
        unsafe {
            let cb = cb as *mut T;
            match (*cb).get_chain() {
            Some(slot) => cast_ptr_mutref(slot),
            None => cast_ptr_mutref( &mut (*(cb as *mut _ as *mut ::udi_sys::udi_cb_t)).initiator_context ),
            }
        }
    }
}

fn get_chain_slot<T>(cb: &mut T) -> &mut *mut T
where
    T: crate::metalang_trait::MetalangCb + crate::async_trickery::GetCb
{
    unsafe fn cast_ptr_mutref<U,T>(p: &mut *mut U) -> &mut *mut T {
        &mut *(p as *mut _ as *mut *mut T)
    }
    // SAFE: Correct pointer manipulations
    unsafe {
        let cb = cb as *mut T;
        match (*cb).get_chain() {
        Some(slot) => cast_ptr_mutref(slot),
        None => cast_ptr_mutref( &mut (*(cb as *mut _ as *mut ::udi_sys::udi_cb_t)).initiator_context ),
        }
    }
}
