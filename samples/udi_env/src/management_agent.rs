use ::std::sync::Arc;

pub struct InstanceInitState
{
    instance: Arc<crate::DriverInstance>,
    channel_to_parent: Option<::udi::ffi::udi_channel_t>,
    returned_cb: *mut ::udi::ffi::udi_cb_t,
    is_active: bool,
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

type Op = (
    *mut ::udi::ffi::udi_cb_t,
    Box<dyn FnOnce(*mut ::udi::ffi::udi_cb_t)>,
    );
impl InstanceInitState
{
    pub fn new(instance: Arc<crate::DriverInstance>, channel_to_parent: Option<::udi::ffi::udi_channel_t>) -> Self {
        Self {
            instance,
            channel_to_parent,
            returned_cb: ::core::ptr::null_mut(),
            is_active: false,
            state: DriverState::UsageInd,
        }
    }
    /// Assert that initialisation is complete, and return the fully populated instance
    pub fn assert_complete(self) -> Arc<crate::DriverInstance>
    {
        let DriverState::Active = self.state else {
            panic!("assert_complete but not yet completed init");
            };
        self.instance
    }

    unsafe fn alloc_cb_raw<T>(&self) -> *mut T {
        let rv = ::libc::malloc( ::core::mem::size_of::<T>() ) as *mut ::udi::ffi::udi_cb_t;
        ::core::ptr::write(rv, ::udi::ffi::udi_cb_t {
            channel: ::core::ptr::null_mut(),
            context: self.instance.regions[0].context,
            scratch: ::core::ptr::null_mut(),
            initiator_context: ::core::ptr::null_mut(),
            origin: ::core::ptr::null_mut(),
        });
        rv as *mut T
    }

    /// Advance the state machine
    pub fn next_op(&mut self) -> Option<Op>
    {
        assert!( !self.is_active );
        println!("next_op: {:?}", self.state);
        match self.state
        {
        DriverState::UsageInd => {
            self.is_active = true;
            Some(self.next_op_usageind())
            },
        DriverState::SecondaryBind { cur_skip } => {
            let driver_module = &*self.instance.module;
            for (i, bind) in driver_module.udiprops.clone().skip(cur_skip).enumerate() {
                if let ::udiprops_parse::Entry::InternalBindOps { .. } = bind {
                    self.state = DriverState::SecondaryBind { cur_skip: cur_skip + i + 1 };
                    self.is_active = true;
                    return Some(self.next_op_childbind(bind));
                }
            }
            self.state = DriverState::ParentBind;
            self.next_op()
            },
        DriverState::ParentBind =>
            if let Some(channel_to_parent) = self.channel_to_parent {
                self.is_active = true;
                Some(self.next_op_parentbind(channel_to_parent))
            }
            else {
                self.state = DriverState::EnumChildrenStart;
                self.next_op()
            },
        DriverState::EnumChildrenStart => {
            self.state = DriverState::EnumChildren { flagged_complete: false };
            self.is_active = true;
            Some(self.next_op_enumerate(true))
            },
        DriverState::EnumChildren { flagged_complete } => if flagged_complete {
                self.state = DriverState::Active;
                self.next_op()
            }
            else {
                self.is_active = true;
                Some(self.next_op_enumerate(false))
            },
        DriverState::Active => None,
        }
    }

    fn next_op_usageind(&mut self) -> Op {
        println!("next_op_usageind");
        let pri_init = self.instance.module.pri_init;
        let usage_ind_op = pri_init.mgmt_ops.usage_ind_op;
        unsafe {
            let cb: *mut ::udi::ffi::meta_mgmt::udi_usage_cb_t = self.alloc_cb_raw();
            (*cb).gcb.scratch = ::libc::malloc(pri_init.mgmt_scratch_requirement);
            (*cb).trace_mask = 0;
            (*cb).meta_idx = Default::default();
            (
                cb as *mut ::udi::ffi::udi_cb_t,
                Box::new(move |cb| { (usage_ind_op)(cb as *mut _, 3 /*UDI_RESOURCES_NORMAL*/) })
                )
        }
    }
    fn next_op_childbind(&mut self, entry: ::udiprops_parse::Entry) -> Op
    {
        println!("next_op_childbind({:?})", entry);
        let ::udiprops_parse::Entry::InternalBindOps { meta_idx, region_idx, primary_ops_idx, secondary_ops_idx, bind_cb_idx } = entry else {
            panic!();
        };

        let driver_module = &*self.instance.module;
        let Some(rgn) = driver_module.get_region(&self.instance, region_idx) else {
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
            crate::channels::anchor(channel_1, self.instance.clone(), driver_module.get_meta_ops(ops_pri), self.instance.regions[0].context);
            crate::channels::anchor(channel_2, self.instance.clone(), driver_module.get_meta_ops(ops_sec), rgn.context);

            let (op, cb) = crate::channels::event_ind_bound_internal(channel_1, bind_cb as *mut _);
            (cb as *mut _, Box::new(move |cb| op(cb as *mut _)))
        }
    }
    fn next_op_parentbind(&mut self, channel_to_parent: ::udi::ffi::udi_channel_t) -> Op
    {
        println!("next_op_parentbind");
        let driver_module = &*self.instance.module;
        for ent in driver_module.udiprops.clone() {
            if let ::udiprops_parse::Entry::ParentBindOps { meta_idx, region_idx, ops_idx, bind_cb_idx } = ent {
                let Some(rgn) = driver_module.get_region(&self.instance, region_idx) else {
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
                    crate::channels::anchor(channel_to_parent, self.instance.clone(), driver_module.get_meta_ops(ops_init), rgn.context);
                    let (op, cb)
                        = crate::channels::event_ind_bound_parent(channel_to_parent, bind_cb as *mut _, 0, ::core::ptr::null());
                    return (cb as *mut _, Box::new(move |cb| op(cb as *mut _)));
                }
            }
        }
        panic!("No ParentBindOps?");
    }

    fn next_op_enumerate(&mut self, is_first: bool) -> Op
    {
        //println!("next_op_enumerate");
        let pri_init = self.instance.module.pri_init;
        let level = if is_first {
            ::udi::ffi::meta_mgmt::UDI_ENUMERATE_START
        }
        else {
            ::udi::ffi::meta_mgmt::UDI_ENUMERATE_NEXT
        };
        unsafe {
            let cb: *mut ::udi::ffi::meta_mgmt::udi_enumerate_cb_t = self.alloc_cb_raw();
            (*cb).gcb.scratch = ::libc::calloc(1, pri_init.mgmt_scratch_requirement);
            (*cb).attr_list = ::libc::calloc(pri_init.enumeration_attr_list_length as _, ::core::mem::size_of::<udi::ffi::attr::udi_instance_attr_list_t>()) as _;
            (*cb).child_data = ::libc::malloc(pri_init.child_data_size);
            //(*cb).trace_mask = 0;
            //(*cb).meta_idx = 0;
            (
                cb as *mut ::udi::ffi::udi_cb_t,
                Box::new(move |cb| { (pri_init.mgmt_ops.enumerate_req_op)(cb as *mut _, level) })
                )
        }
    }

    /// Take the CB returned from the driver (via a `*_res` or `*_ack`) call
    pub fn returned_cb(&mut self) -> Option<*mut ::udi::ffi::udi_cb_t> {
        if self.returned_cb.is_null() {
            None
        }
        else {
            assert!(self.is_active);
            self.is_active = false;
            Some(::core::mem::replace(&mut self.returned_cb, ::core::ptr::null_mut()))
        }
    }

    /// Called by [crate::udi_impl::meta_mgmt::udi_usage_res]
    pub(crate) fn usage_res(&mut self, cb: *mut ::udi::ffi::meta_mgmt::udi_usage_cb_t)
    {
        match self.state
        {
        DriverState::UsageInd => {
            self.returned_cb = cb as *mut _;
            self.state = DriverState::SecondaryBind { cur_skip: 0 };
            },
        _ => panic!("usage_ind called when not expected"),
        }
    }
    /// Called by [crate::udi_impl::meta_mgmt::udi_enumerate_ack]
    pub(crate) fn enumerate_ack(&mut self, cb: *mut ::udi::ffi::meta_mgmt::udi_enumerate_cb_t, enumeration_result: ::udi::init::EnumerateResult)
    {
        let DriverState::EnumChildren { ref mut flagged_complete } = self.state else {
            panic!("`enumerate_ack` called when not expected");
        };
        self.returned_cb = cb as *mut _;
        match enumeration_result
        {
        udi::init::EnumerateResult::Ok(child_info) => {
            // The driver now owns this pointer
            unsafe { (*cb).child_data = ::core::ptr::null_mut(); }
            let attrs = unsafe { ::core::slice::from_raw_parts((*cb).attr_list, (*cb).attr_valid_length as usize) };
            for a in attrs {
                let a_name = {
                    let name_len = a.attr_name.iter().position(|v| *v == 0).unwrap_or(a.attr_name.len());
                    let name = &a.attr_name[..name_len];
                    ::std::str::from_utf8(name).unwrap_or("")
                    };
                println!("attr = {:?} {} {:?}", a_name, a.attr_type, &a.attr_value[..a.attr_length as usize]);
            }

            let mut child_bind_ops = None;
            for entry in self.instance.module.udiprops.clone() {
                if let ::udiprops_parse::Entry::ChildBindOps { meta_idx, region_idx, ops_idx } = entry {
                    if ops_idx == child_info.ops_idx() {
                        child_bind_ops = Some((meta_idx, region_idx));
                    }
                }
            }
            if let Some((meta_idx, region_idx)) = child_bind_ops {
                let region_idx_real = self.instance.module.get_region_index(region_idx).unwrap();
                self.instance.children.lock().unwrap().push(crate::DriverChild {
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
            if ! (*cb).child_data.is_null() {
                ::libc::free((*cb).child_data as _);
            }
            ::libc::free((*cb).attr_list as _);
        }
    }

    pub fn bind_complete(&mut self, cb: *mut ::udi::ffi::imc::udi_channel_event_cb_t, result: ::udi::Result<()>) {
        self.returned_cb = cb as *mut _;
        unsafe {
            ::libc::free((*cb).params.parent_bound.bind_cb as _);
        }
        match self.state {
        DriverState::ParentBind => {
            self.state = DriverState::EnumChildrenStart;
            }
        _ => todo!(),
        }
        if let Err(e) = result {
            todo!("bind_complete error {:?}", e);
        }
    }
}