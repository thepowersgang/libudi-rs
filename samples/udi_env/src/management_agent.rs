
// See UDI Spec:
// 10.1.2 - Per-Instance Initialization
// 10.1.3 - Per Region Initialization

use ::std::sync::Arc;

#[derive(Default)]
pub struct ManagementState
{
    inner: ::std::sync::Mutex<ManagementStateInner>,
}
#[derive(Default)]
enum ManagementStateInner {
    #[default]
    PreInit,
    Init(InitState),
    Initialised,
}

pub struct InitState
{
    channel_to_parent: Option<::udi::ffi::udi_channel_t>,
    state: DriverState,
}

#[derive(Debug)]
enum DriverState {
    UsageInd,
    SecondaryBind {
        cur_skip: usize,
    },
    ParentBind,
    EnumChildrenStart,
    EnumChildren {
        flagged_complete: bool,
    },
    Active,
}

impl ManagementState
{
    pub fn start_init(&self, channel_to_parent: Option<::udi::ffi::udi_channel_t>) {
        match *self.inner.lock().unwrap()
        {
        ref mut dst @ ManagementStateInner::PreInit => {
            *dst = ManagementStateInner::Init(InitState { channel_to_parent, state: DriverState::UsageInd  });
        }
        _ => panic!("`start_init` called multiple times?"),
        }
    }
    pub fn next_op(&self, instance: &Arc<crate::DriverInstance>) -> Option<super::Operation>
    {
        let mut state = self.inner.lock().unwrap();
        match *state
        {
        ManagementStateInner::PreInit => panic!("enumerate_ack when in PreInit"),
        ManagementStateInner::Init(ref mut is) => match is.next_op(instance)
            {
            Some(op) => Some(op),
            None => {
                *state = ManagementStateInner::Initialised;
                None
            }
            }
        ManagementStateInner::Initialised => panic!("enumerate_ack when already initialised"),
        }
    }
    pub(crate) fn usage_res(&self, _instance: &crate::DriverInstance, cb: ::udi::cb::CbHandle<::udi::ffi::meta_mgmt::udi_usage_cb_t>) {
        let mut state = self.inner.lock().unwrap();
        let is = match *state
            {
            ManagementStateInner::PreInit => panic!("enumerate_ack when in PreInit"),
            ManagementStateInner::Init(ref mut is) => is,
            ManagementStateInner::Initialised => panic!("enumerate_ack when already initialised"),
            };
        match is.state
        {
        DriverState::UsageInd => {
            //self.returned_cb = cb as *mut _;
            is.state = DriverState::SecondaryBind { cur_skip: 0 };
            },
        _ => panic!("usage_ind called when not expected"),
        }
        drop(cb);
    }
    pub(crate) fn enumerate_ack(
        &self,
        instance: &crate::DriverInstance,
        mut cb: ::udi::cb::CbHandle<::udi::ffi::meta_mgmt::udi_enumerate_cb_t>,
        enumeration_result: ::udi::init::EnumerateResult
    ) {
        let mut state = self.inner.lock().unwrap();
        let is = match *state
            {
            ManagementStateInner::PreInit => panic!("enumerate_ack when in PreInit"),
            ManagementStateInner::Init(ref mut is) => is,
            ManagementStateInner::Initialised => panic!("enumerate_ack when already initialised"),
            };
        let DriverState::EnumChildren { ref mut flagged_complete } = is.state else {
            panic!("`enumerate_ack` called when not expected");
        };

        match enumeration_result
        {
        udi::init::EnumerateResult::Ok(child_info) => {
            // The driver now owns this pointer
            unsafe { cb.get_mut().child_data = ::core::ptr::null_mut(); }
            let attrs = unsafe { ::core::slice::from_raw_parts((*cb).attr_list, (*cb).attr_valid_length as usize) };
            for a in attrs {
                let a_name = {
                    let name_len = a.attr_name.iter().position(|v| *v == 0).unwrap_or(a.attr_name.len());
                    let name = &a.attr_name[..name_len];
                    ::std::str::from_utf8(name).unwrap_or("")
                    };
                println!("enumerate_ack: attr = {:?} {} {:?}", a_name, a.attr_type, &a.attr_value[..a.attr_length as usize]);
            }

            let mut child_bind_ops = None;
            for entry in instance.module.udiprops.clone() {
                if let ::udiprops_parse::Entry::ChildBindOps { meta_idx, region_idx, ops_idx } = entry {
                    if ops_idx == child_info.ops_idx() {
                        child_bind_ops = Some((meta_idx, region_idx));
                    }
                }
            }
            if let Some((meta_idx, region_idx)) = child_bind_ops {
                let region_idx_real = instance.module.get_region_index(region_idx).unwrap();
                instance.children.lock().unwrap().push(crate::DriverChild {
                    is_bound: Default::default(),
                    child_id: child_info.child_id(),
                    meta_idx,
                    ops_idx: child_info.ops_idx(),
                    region_idx_real,
                    attrs: attrs.to_vec()
                });
            }
        },
        udi::init::EnumerateResult::Leaf => { *flagged_complete = true; },
        udi::init::EnumerateResult::Done => { *flagged_complete = true; },
        udi::init::EnumerateResult::Rescan => todo!(),
        udi::init::EnumerateResult::Removed => todo!(),
        udi::init::EnumerateResult::RemovedSelf => todo!(),
        udi::init::EnumerateResult::Released => todo!(),
        udi::init::EnumerateResult::Failed => { *flagged_complete = true; },
        }
        unsafe {
            if ! cb.child_data.is_null() {
                ::libc::free(cb.child_data as _);
            }
            ::libc::free(cb.attr_list as _);
        }
        drop(cb);
    }
    pub fn devmgmt_ack(
        &self,
        instance: &crate::DriverInstance,
        _cb: ::udi::cb::CbHandle<::udi::ffi::meta_mgmt::udi_mgmt_cb_t>,
        flags: u8,
        status: ::udi::Result<()>
    ) {
        let _ = instance;
        match status {
        Ok( () ) => {
            todo!("devmgmt_ack: flags={:#x}", flags);
            }
        Err(e) => {
            todo!("devmgmt_ack: Error {:?}", e);
            }
        }
    }
    pub fn final_cleanup_ack(
        &self,
        instance: &crate::DriverInstance,
        cb: ::udi::cb::CbHandle<::udi::ffi::meta_mgmt::udi_mgmt_cb_t>,
    ) {
        let _ = instance;
        drop(cb);
        todo!();
    }

    pub fn bind_complete(&self, _instance: &crate::DriverInstance, cb: *mut ::udi::ffi::imc::udi_channel_event_cb_t, result: ::udi::Result<()>) {
        let mut state = self.inner.lock().unwrap();
        let is = match *state
            {
            ManagementStateInner::PreInit => panic!("enumerate_ack when in PreInit"),
            ManagementStateInner::Init(ref mut is) => is,
            ManagementStateInner::Initialised => panic!("enumerate_ack when already initialised"),
            };
        unsafe {
            crate::udi_impl::cb::free_internal((*cb).params.parent_bound.bind_cb);
        }
        match is.state {
        DriverState::ParentBind => {
            is.state = DriverState::EnumChildrenStart;
            }
        _ => todo!(),
        }
        if let Err(e) = result {
            todo!("bind_complete error {:?}", e);
        }
    }
}


impl InitState {
    /// Advance the state machine
    pub fn next_op(&mut self, instance: &Arc<crate::DriverInstance>) -> Option<crate::Operation>
    {
        //assert!( !self.is_active );
        println!("next_op: {:?}", self.state);
        match self.state
        {
        DriverState::UsageInd => {
            Some(self.next_op_usageind(instance))
            },
        DriverState::SecondaryBind { cur_skip } => {
            let driver_module = &*instance.module;
            for (i, bind) in driver_module.udiprops.clone().skip(cur_skip).enumerate() {
                if let ::udiprops_parse::Entry::InternalBindOps { .. } = bind {
                    self.state = DriverState::SecondaryBind { cur_skip: cur_skip + i + 1 };
                    return Some(self.next_op_childbind(instance, bind));
                }
            }
            self.state = DriverState::ParentBind;
            self.next_op(instance)
            },
        DriverState::ParentBind =>
            if let Some(channel_to_parent) = self.channel_to_parent {
                Some(self.next_op_parentbind(instance, channel_to_parent))
            }
            else {
                self.state = DriverState::EnumChildrenStart;
                self.next_op(instance)
            },
        DriverState::EnumChildrenStart => {
            self.state = DriverState::EnumChildren { flagged_complete: false };
            Some(self.next_op_enumerate(instance, true))
            },
        DriverState::EnumChildren { flagged_complete } => if flagged_complete {
                self.state = DriverState::Active;
                self.next_op(instance)
            }
            else {
                Some(self.next_op_enumerate(instance, false))
            },
        DriverState::Active => None,
        }
    }

    fn next_op_usageind(&self, instance: &Arc<crate::DriverInstance>) -> super::Operation {
        println!("next_op_usageind");
        let pri_init = instance.module.pri_init;
        let usage_ind_op = pri_init.mgmt_ops.usage_ind_op;
        unsafe {
            let cb: *mut ::udi::ffi::meta_mgmt::udi_usage_cb_t = alloc_cb_raw(instance);
            (*cb).gcb.scratch = ::libc::malloc(pri_init.mgmt_scratch_requirement);
            (*cb).trace_mask = 0;
            (*cb).meta_idx = Default::default();
            super::Operation::new(cb, move |cb| (usage_ind_op)(cb as *mut _, 3 /*UDI_RESOURCES_NORMAL*/))
        }
    }
    fn next_op_childbind(&self, instance: &Arc<crate::DriverInstance>, entry: ::udiprops_parse::Entry) -> crate::Operation
    {
        println!("next_op_childbind({:?})", entry);
        let ::udiprops_parse::Entry::InternalBindOps { meta_idx, region_idx, primary_ops_idx, secondary_ops_idx, bind_cb_idx } = entry else {
            panic!();
        };

        let driver_module = &*instance.module;
        let Some(rgn) = driver_module.get_region(instance, region_idx) else {
            panic!("Unable to find region {} for secondary bind", region_idx);
        };
        let Some(ops_pri) = driver_module.get_ops_init(primary_ops_idx) else {
            panic!("Unable to find primary ops {} for internal bind", primary_ops_idx);
        };
        let Some(ops_sec) = driver_module.get_ops_init(secondary_ops_idx) else {
            panic!("Unable to find secondary ops {} for internal bind", secondary_ops_idx);
        };
        let Some(cb) = driver_module.get_cb_init(bind_cb_idx) else {
            panic!("Unable to find CB {} for internal bind", bind_cb_idx);
        };
        assert_eq!(ops_pri.meta_idx, meta_idx);
        assert_eq!(ops_sec.meta_idx, meta_idx);
        assert_eq!(cb.meta_idx, meta_idx);

        // Spawn the channel
        let (channel_1, channel_2) = crate::channels::spawn_raw();
        let bind_cb = crate::udi_impl::cb::alloc(&driver_module, bind_cb_idx, rgn.context, channel_1);
        unsafe {
            crate::channels::anchor(channel_1, instance.clone(), driver_module.get_meta_ops(ops_pri), instance.regions[0].context);
            crate::channels::anchor(channel_2, instance.clone(), driver_module.get_meta_ops(ops_sec), rgn.context);

            let (op, cb) = crate::channels::event_ind_bound_internal(channel_1, bind_cb as *mut _);
            crate::Operation::new(cb, move |cb| op(cb))
        }
    }
    fn next_op_parentbind(&self, instance: &Arc<crate::DriverInstance>, channel_to_parent: ::udi::ffi::udi_channel_t) -> crate::Operation
    {
        println!("next_op_parentbind");
        let driver_module = &*instance.module;
        for ent in driver_module.udiprops.clone() {
            if let ::udiprops_parse::Entry::ParentBindOps { meta_idx, region_idx, ops_idx, bind_cb_idx } = ent {
                let Some(rgn) = driver_module.get_region(instance, region_idx) else {
                    panic!("Unable to find region {} for parent bind", region_idx);
                };
                let Some(ops_init) = driver_module.get_ops_init(ops_idx) else {
                    panic!("Unable to find ops {} for parent bind", ops_idx);
                };
                let Some(cb) = driver_module.get_cb_init(bind_cb_idx) else {
                    panic!("Unable to find CB {} for parent bind", bind_cb_idx);
                };
                assert_eq!(ops_init.meta_idx, meta_idx);
                assert_eq!(cb.meta_idx, meta_idx);

                let bind_cb = crate::udi_impl::cb::alloc(&driver_module, bind_cb_idx, rgn.context, channel_to_parent);
                unsafe {
                    crate::channels::anchor(channel_to_parent, instance.clone(), driver_module.get_meta_ops(ops_init), rgn.context);
                    let (op, cb)
                        = crate::channels::event_ind_bound_parent(channel_to_parent, bind_cb as *mut _, 0, ::core::ptr::null());
                    return crate::Operation::new(cb, move |cb| op(cb));
                }
            }
        }
        panic!("No ParentBindOps?");
    }

    fn next_op_enumerate(&mut self, instance: &Arc<crate::DriverInstance>, is_first: bool) -> crate::Operation
    {
        //println!("next_op_enumerate");
        let pri_init = instance.module.pri_init;
        let level = if is_first {
            ::udi::ffi::meta_mgmt::UDI_ENUMERATE_START
        }
        else {
            ::udi::ffi::meta_mgmt::UDI_ENUMERATE_NEXT
        };
        unsafe {
            let cb: *mut ::udi::ffi::meta_mgmt::udi_enumerate_cb_t = alloc_cb_raw(&instance);
            (*cb).gcb.scratch = ::libc::calloc(1, pri_init.mgmt_scratch_requirement);
            (*cb).attr_list = ::libc::calloc(pri_init.enumeration_attr_list_length as _, ::core::mem::size_of::<udi::ffi::attr::udi_instance_attr_list_t>()) as _;
            (*cb).child_data = if pri_init.child_data_size > 0 { ::libc::malloc(pri_init.child_data_size) } else { ::core::ptr::null_mut() };
            //(*cb).trace_mask = 0;
            //(*cb).meta_idx = 0;
            crate::Operation::new( cb, move |cb| (pri_init.mgmt_ops.enumerate_req_op)(cb, level) )
        }
    }
}

unsafe fn alloc_cb_raw<T>(instance: &super::DriverInstance) -> *mut T {
    let rv = ::libc::malloc( ::core::mem::size_of::<T>() ) as *mut ::udi::ffi::udi_cb_t;
    ::core::ptr::write(rv, ::udi::ffi::udi_cb_t {
        channel: ::core::ptr::null_mut(),
        context: instance.regions[0].context,
        scratch: ::core::ptr::null_mut(),
        initiator_context: instance as *const _ as *mut _,
        origin: ::core::ptr::null_mut(),
    });
    rv as *mut T
}

