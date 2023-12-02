//! Inner glue for UDI channels
//! 

/// The common innards of a channel
/// 
/// `udi_channel_t` is a tagged pointer to this (with the tag indicating which side)
struct ChannelInner {
    /// Child channels matched by the `spawn_index`
    spawns: ::std::sync::Mutex< ::std::collections::HashMap<::udi::ffi::udi_index_t,::udi::ffi::udi_channel_t> >,
    /// Channel side information
    sides: [::std::cell::OnceCell<ChannelInnerSide>; 2],
    // TODO: Track the currently "active" side?
}
struct ChannelInnerSide {
    /// Target driver instance
    driver_instance: ::std::sync::Arc< crate::DriverInstance >,
    /// Metalanguage Operations
    ops: &'static dyn udi::metalang_trait::MetalangOpsHandler,
    /// Context pointer to use
    context: *mut ::udi::ffi::c_void,
    /// Is the `context` owned by this channel
    context_allocated: bool,
    /// Has this side of the channel been closed
    is_closed: ::std::sync::atomic::AtomicBool,
}
impl Drop for ChannelInnerSide {
    fn drop(&mut self) {
        if self.context_allocated {
            unsafe { ::libc::free(self.context as *mut _); }
        }
    }
}

/// Inner helper: A reference to a channel w/ side
struct ChannelRef<'a>(&'a ChannelInner, bool);
impl<'a> ChannelRef<'a> {
    unsafe fn from_handle(h: ::udi::ffi::udi_channel_t) -> Self {
        assert!(!h.is_null());
        let (ptr,is_b) = ((h as usize & !1) as *const ChannelInner, h as usize & 1 != 0);
        ChannelRef(&*ptr, is_b)
    }
    fn get_handle_reversed(&self) -> ::udi::ffi::udi_channel_t {
        ((self.0 as *const _) as usize | (!self.1 as usize)) as *mut _
    }
    fn get_side(&self) -> Option<&ChannelInnerSide> {
        self.0.sides[self.1 as usize].get()
    }
}

/// Internal helper: Query which module owns this channel handle
pub unsafe fn get_driver_instance(ch: &::udi::ffi::udi_channel_t) -> ::std::sync::Arc<crate::DriverInstance> {
    let cr = ChannelRef::from_handle(*ch);
    cr.get_side().unwrap().driver_instance.clone()
}
/// Internal helper: Query which instance is on the other end of this channel handle
pub unsafe fn get_other_instance(ch: &::udi::ffi::udi_channel_t) -> ::std::sync::Arc<crate::DriverInstance> {
    let cr = ChannelRef::from_handle(*ch);
    ChannelRef::from_handle(cr.get_handle_reversed()).get_side().unwrap().driver_instance.clone()
}
pub unsafe fn get_region(ch: &::udi::ffi::udi_channel_t) -> &crate::DriverRegion {
    let cr = ChannelRef::from_handle(*ch);
    let ch_side = cr.get_side().unwrap();
    // Launder the pointer - so we can return the borrow
    let ch_side = &*(ch_side as *const ChannelInnerSide);
    for r in &ch_side.driver_instance.regions {
        if r.context() == ch_side.context {
            return r;
        }
        if r.context() == *(ch_side.context as *mut *mut ::udi::ffi::c_void) {
            return r;
        }
    }
    todo!();
}

/// Spawn a channel without needing a parent/source channel
/// 
/// Returns the two channel handles
pub fn spawn_raw() -> (::udi::ffi::udi_channel_t,::udi::ffi::udi_channel_t)
{
    let h = Box::into_raw(Box::new(ChannelInner {
        spawns: Default::default(),
        sides: Default::default(),
        })) as ::udi::ffi::udi_channel_t;
    (h, (h as usize | 1) as ::udi::ffi::udi_channel_t,)
}
/// Spawn a new channel end (matching to an existing call from the same base channel)
pub unsafe fn spawn(
    base_channel: ::udi::ffi::udi_channel_t,
    spawn_idx: ::udi::ffi::udi_index_t
) -> ::udi::ffi::udi_channel_t
{
    let cr = ChannelRef::from_handle(base_channel);
    let mut spawns = cr.0.spawns.lock().unwrap();
    if let Some(handle) = spawns.remove(&spawn_idx) {
        handle
    }
    else {
        let (rv, other_end) = spawn_raw();
        spawns.insert(spawn_idx, other_end);
        rv
    }
}
pub unsafe fn close(channel: ::udi::ffi::udi_channel_t)
{
    let cr = ChannelRef::from_handle(channel);
    // Consider the other end as free if hasn't been anchored yet.
    let is_other_free = cr.0.sides[!cr.1 as usize].get()
        .map(|s| s.is_closed.load(::std::sync::atomic::Ordering::SeqCst))
        .unwrap_or(true);
    if is_other_free {
        // Both sides are closed, to free all of the handle
        assert!(! cr.0.sides[cr.1 as usize].get().unwrap()
            .is_closed.load(::std::sync::atomic::Ordering::SeqCst) );
        drop(Box::from_raw(cr.0 as *const ChannelInner as *mut ChannelInner));
    }
    else {
        // Mark this side as closed
        cr.0.sides[cr.1 as usize].get().unwrap()
            .is_closed.store(true, ::std::sync::atomic::Ordering::SeqCst);
    }
}
/// Anchor a channel end
pub unsafe fn anchor(
    channel: ::udi::ffi::udi_channel_t,
    driver_instance: ::std::sync::Arc<crate::DriverInstance>,
    ops: &'static dyn udi::metalang_trait::MetalangOpsHandler,
    context: *mut ::udi::ffi::c_void,
)
{
    let cr = ChannelRef::from_handle(channel);
    cr.0.sides[cr.1 as usize]
        .set(ChannelInnerSide { driver_instance, context, context_allocated: false, ops, is_closed: Default::default(), })
        .ok().expect("Anchoring an anchored end");
}
/// Anchor a channel end, allocating a channel context instead of using a pre-allocated context
pub unsafe fn anchor_with_context<T: 'static>(
    channel: ::udi::ffi::udi_channel_t,
    driver_instance: ::std::sync::Arc<crate::DriverInstance>,
    ops: &'static dyn udi::metalang_trait::MetalangOpsHandler,
    size: usize,
    inner: T,
)
{
    assert!(size >= ::core::mem::size_of::<T>());
    let context = ::libc::calloc(1, size) as *mut T;
    ::core::ptr::write(context, inner);

    let cr = ChannelRef::from_handle(channel);
    cr.0.sides[cr.1 as usize]
        .set(ChannelInnerSide { driver_instance, context: context as *mut _, context_allocated: true, ops, is_closed: Default::default(), })
        .ok().expect("Anchoring an anchored end");
}

/// Call through a channel
/// 
/// - `name` is the name of the function being called (for debugging)
/// - `cb` is the control block through which the call is happening
/// - `call` invokes the callback in the metalanguage ops structure
pub unsafe fn remote_call<O: udi::metalang_trait::MetalangOpsHandler, Cb: udi::metalang_trait::MetalangCb>(
    name: &'static str, cb: *mut Cb, call: impl FnOnce(&O, *mut Cb) + 'static)
{
    // Get the channel currently in the cb, and reverse it
    let gcb = cb as *mut ::udi::ffi::udi_cb_t;
    let ch = ChannelRef::from_handle((*gcb).channel);
    let ch_side = ch.0.sides[!ch.1 as usize].get().expect("Calling with no remote handle");
    
    // Get the scratch as the max of all CB instances for this type
    let driver_module = &*ch_side.driver_instance.module;
    let meta_name = <Cb::MetalangSpec as ::udi::metalang_trait::Metalanguage>::name();
    let Some(meta_idx) = driver_module.get_metalang_by_name(meta_name) else {
        panic!("No metalang `{}` in driver ({driver_module:p}) '{}'?!", meta_name, driver_module.name());
    };
    let scratch_requirement = driver_module.cbs.iter()
        .filter(|cb| cb.meta_idx == meta_idx)
        .filter(|cb| cb.meta_cb_num == Cb::META_CB_NUM)
        .map(|cb| cb.scratch_requirement)
        .max();
    println!("remote_call({}[{}]cb={}): Context = {:p}, scratch_requirement = {:?}",
        ::core::any::type_name::<O>(),
        name,
        ::core::any::type_name::<Cb>(),
        ch_side.context, scratch_requirement);

    (*gcb).channel = ch.get_handle_reversed();
    // Update context and scratch
    (*gcb).context = ch_side.context;
    if let Some(scratch_requirement) = scratch_requirement {
        (*gcb).scratch = ::libc::realloc((*gcb).scratch, scratch_requirement);
    }

    // Then check that the metalanguage ops in that side matches the expectation
    if ch_side.ops.type_id() != ::std::any::TypeId::of::<O>() {
        panic!("Metalang mismatch: Expected {:?}, got {:?}", ch_side.ops.type_name(), ::std::any::type_name::<O>());
    }
    let ops = &*(ch_side.ops as *const _ as *const O);

    crate::async_call(cb as *mut _, move |cb| call(ops, cb as *mut Cb));
}

/// Call `udi_event_ind` over the channel, for an internal bind event
pub unsafe fn event_ind_bound_internal(channel: ::udi::ffi::udi_channel_t, bind_cb: *mut ::udi::ffi::udi_cb_t)
    -> (::udi::ffi::imc::udi_channel_event_ind_op_t, *mut ::udi::ffi::imc::udi_channel_event_cb_t) {
    event_ind(
        channel,
        ::udi::ffi::imc::UDI_CHANNEL_BOUND,
        ::udi::ffi::imc::udi_channel_event_cb_t_params {
            internal_bound: ::udi::ffi::imc::udi_channel_event_cb_t_params_internal_bound {
                bind_cb,
            },
        },
    )
}
/// Call `udi_event_ind` over the channel, for a parent bind event
pub unsafe fn event_ind_bound_parent(
    channel: ::udi::ffi::udi_channel_t,
    bind_cb: *mut ::udi::ffi::udi_cb_t,
    parent_id: u8,
    path_handles: *const ::udi::ffi::buf::udi_buf_path_t
) -> (::udi::ffi::imc::udi_channel_event_ind_op_t, *mut ::udi::ffi::imc::udi_channel_event_cb_t) {
    event_ind(
        channel,
        ::udi::ffi::imc::UDI_CHANNEL_BOUND,
        ::udi::ffi::imc::udi_channel_event_cb_t_params {
            parent_bound: ::udi::ffi::imc::udi_channel_event_cb_t_params_parent_bound {
                bind_cb,
                parent_id,
                path_handles,
            },
        },
    )
}
/// Innards to generate a call to `event_ind`
unsafe fn event_ind(channel: ::udi::ffi::udi_channel_t, event: u8, params: ::udi::ffi::imc::udi_channel_event_cb_t_params)
    -> (::udi::ffi::imc::udi_channel_event_ind_op_t, *mut ::udi::ffi::imc::udi_channel_event_cb_t)
{
    let ch = ChannelRef::from_handle(channel);
    let side = ch.get_side().unwrap();
    let event_ind_op = side.ops.channel_event_ind_op();
    let cb = ::libc::malloc( ::core::mem::size_of::<::udi::ffi::imc::udi_channel_event_cb_t>() ) as *mut ::udi::ffi::imc::udi_channel_event_cb_t;
    ::core::ptr::write(cb, ::udi::ffi::imc::udi_channel_event_cb_t {
        gcb: ::udi::ffi::udi_cb_t {
            channel,
            context: side.context,
            scratch: ::core::ptr::null_mut(),   // TODO: How?
            // TODO: Set `initiator_context` to something that allows `udi_channel_event_complete` to communicated back
            // - For init, it's set in `create_driver_instance`
            initiator_context: ::core::ptr::null_mut(),
            origin: ::core::ptr::null_mut(),
        },
        event,
        params,
    });
    (event_ind_op, cb)
}
