

struct ChannelInner {
    spawns: ::std::sync::Mutex< ::std::collections::HashMap<u8,::udi::ffi::udi_channel_t> >,
    sides: [::std::cell::OnceCell<ChannelInnerSide>; 2],
}
struct ChannelInnerSide {
    driver_module: *const crate::DriverModule<'static>,
    ops: &'static dyn udi::metalang_trait::MetalangOpsHandler,
    context: *mut ::udi::ffi::c_void,
}

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


pub unsafe fn get_driver_module(ch: &::udi::ffi::udi_channel_t) -> &crate::DriverModule {
    let cr = ChannelRef::from_handle(*ch);
    &*cr.get_side().unwrap().driver_module
}

/// Spawn a channel without needing a source channel
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
    driver_module: *const crate::DriverModule<'static>,
    ops: &'static dyn udi::metalang_trait::MetalangOpsHandler,
    context: *mut ::udi::ffi::c_void,
)
{
    let cr = ChannelRef::from_handle(channel);
    cr.0.sides[cr.1 as usize].set(ChannelInnerSide { driver_module, context, ops }).ok().expect("Anchoring an anchored end");
}


pub unsafe fn remote_call<O: udi::metalang_trait::MetalangOpsHandler, Cb: udi::metalang_trait::MetalangCb>(cb: *mut Cb, call: impl FnOnce(&O, *mut Cb))
{
    // Get the channel currently in the cb, and reverse it
    let gcb = cb as *mut ::udi::ffi::udi_cb_t;
    let ch = ChannelRef::from_handle((*gcb).channel);
    let ch_side = ch.0.sides[ch.1 as usize].get().unwrap();
    //ch_side.dev
    (*gcb).channel = ch.get_handle_reversed();
    // Update context and scratch
    (*gcb).context = ch_side.context;
    //(*cb).scratch = ::libc::realloc((*cb).scratch, ch_side.scratch_requirement);

    // TODO: How is this supposed to get the right CB for the scratch requirement?
    // - Ask the metalang ops for the CB index somehow.

    // Then check that the metalanguage ops in that side matches the expectation
    if ch_side.ops.type_id() != ::std::any::TypeId::of::<O>() {
        panic!("Metalang mismatch: Expected {:?}, got {:?}", ch_side.ops.type_name(), ::std::any::type_name::<O>());
    }
    let ops = &*(ch_side.ops as *const _ as *const O);
    call(ops, cb);
}

pub unsafe fn event_ind_bound_internal(channel: ::udi::ffi::udi_channel_t, bind_cb: *mut ::udi::ffi::udi_cb_t) {
    event_ind(
        channel,
        ::udi::ffi::imc::UDI_CHANNEL_BOUND,
        ::udi::ffi::imc::udi_channel_event_cb_t_params {
            internal_bound: ::udi::ffi::imc::udi_channel_event_cb_t_params_internal_bound {
                bind_cb,
            },
        },
    );
}
unsafe fn event_ind(channel: ::udi::ffi::udi_channel_t, event: u8, params: ::udi::ffi::imc::udi_channel_event_cb_t_params) {
    let ch = ChannelRef::from_handle(channel);
    let side = ch.get_side().unwrap();
    let event_ind_op = side.ops.channel_event_ind_op();
    let mut cb = ::udi::ffi::imc::udi_channel_event_cb_t {
        gcb: ::udi::ffi::udi_cb_t {
            channel,
            context: side.context,
            scratch: ::core::ptr::null_mut(),   // TODO: How?
            initiator_context: ::core::ptr::null_mut(),
            origin: ::core::ptr::null_mut(),
        },
        event,
        params,
    };
    event_ind_op(&mut cb);
}
