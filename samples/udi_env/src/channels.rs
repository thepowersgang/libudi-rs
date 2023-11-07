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
    /// Target module
    driver_module: ::std::sync::Arc< crate::DriverModule<'static> >,
    // TODO: Include instance instead?
    /// Metalanguage Operations
    ops: &'static dyn udi::metalang_trait::MetalangOpsHandler,
    /// Context pointer to use
    context: *mut ::udi::ffi::c_void,
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
pub unsafe fn get_driver_module(ch: &::udi::ffi::udi_channel_t) -> ::std::sync::Arc<crate::DriverModule> {
    let cr = ChannelRef::from_handle(*ch);
    cr.get_side().unwrap().driver_module.clone()
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
/// Anchor a channel end
pub unsafe fn anchor(
    channel: ::udi::ffi::udi_channel_t,
    driver_module: ::std::sync::Arc<crate::DriverModule<'static>>,
    ops: &'static dyn udi::metalang_trait::MetalangOpsHandler,
    context: *mut ::udi::ffi::c_void,
)
{
    let cr = ChannelRef::from_handle(channel);
    cr.0.sides[cr.1 as usize].set(ChannelInnerSide { driver_module, context, ops }).ok().expect("Anchoring an anchored end");
}

/// Call through a channel
pub unsafe fn remote_call<O: udi::metalang_trait::MetalangOpsHandler, Cb: udi::metalang_trait::MetalangCb>(cb: *mut Cb, call: impl FnOnce(&O, *mut Cb))
{
    // Get the channel currently in the cb, and reverse it
    let gcb = cb as *mut ::udi::ffi::udi_cb_t;
    let ch = ChannelRef::from_handle((*gcb).channel);
    let ch_side = ch.0.sides[!ch.1 as usize].get().unwrap();


    // Get the scratch as the max of all CB instances for this type
    let driver_module = &*ch_side.driver_module;
    let Some(meta_idx) = driver_module.get_metalang_by_name(<Cb::MetalangSpec as ::udi::metalang_trait::Metalanguage>::name()) else {
        panic!("No metalang `{}` in driver?!", <Cb::MetalangSpec as ::udi::metalang_trait::Metalanguage>::name());
    };
    let scratch_requirement = driver_module.cbs.iter()
        .filter(|cb| cb.meta_idx == meta_idx)
        .filter(|cb| cb.meta_cb_num == Cb::META_CB_NUM)
        .map(|cb| cb.scratch_requirement)
        .max();
    println!("remote_call({},{}): Context = {:p}, scratch_requirement = {:?}",
        ::core::any::type_name::<O>(),
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
    call(ops, cb);
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
            initiator_context: ::core::ptr::null_mut(),
            origin: ::core::ptr::null_mut(),
        },
        event,
        params,
    });
    (event_ind_op, cb)
}
