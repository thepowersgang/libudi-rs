#![feature(impl_trait_in_assoc_type)]

#[macro_use]
mod channels;
mod udi_impl;

mod bridge_pci;

extern crate udi_net_ne2000;

mod driver {
    extern "C" {
        pub static udi_init_info: ::udi::ffi::init::udi_init_t;
        // TODO: udiprops (formally they're in a section as nul terminated strings)
        pub static libudi_rs_udiprops: [u8; 0];
        pub static libudi_rs_udiprops_len: usize;
    }
}


fn main() {
    let udiprops = ::udiprops_parse::load_from_raw_section(unsafe {
        let ptr = driver::libudi_rs_udiprops.as_ptr();
        let len = driver::libudi_rs_udiprops_len;
        ::core::slice::from_raw_parts(ptr, len)
        });
    let driver_module = unsafe { DriverModule::new(&driver::udi_init_info, udiprops) };


    if false {
        create_driver_instance(&driver_module, None);
    }
    else {
        let (channel_par, channel_child) = channels::spawn_raw();
        create_driver_instance(&driver_module, Some(channel_child));
    }
}

fn create_driver_instance(driver_module: &DriverModule<'static>, channel_to_parent: Option<::udi::ffi::udi_channel_t>)
{
    // See UDI Spec 10.1.2

    // - Create primary region
    let instance = Box::new(DriverInstance {
        //module: &driver_module,
        regions: {
            let mut v = vec![DriverRegion::new(0, driver_module.pri_init.rdata_size)];
            for secondary_region in driver_module.sec_init {
                v.push(DriverRegion::new(secondary_region.region_idx, secondary_region.rdata_size));
            }
            v
            },
    });
    // - call `udi_usage_ind`
    let mut state = InstanceInitState {
        instance: &instance,
        state: DriverState::UsageInd
        };
    let mut cb = ::udi::ffi::meta_mgmt::udi_usage_cb_t {
        gcb: ::udi::ffi::udi_cb_t {
            channel: ::core::ptr::null_mut(),
            context: instance.regions[0].context,
            scratch: ::core::ptr::null_mut(),
            initiator_context: &mut state as *mut _ as *mut ::udi::ffi::c_void,
            origin: ::core::ptr::null_mut(),
        },
        trace_mask: 0,
        meta_idx: 0,
    };
    unsafe {
        cb.gcb.scratch = ::libc::malloc(driver_module.pri_init.mgmt_scratch_requirement);
        (driver_module.pri_init.mgmt_ops.usage_ind_op)(&mut cb, 3 /*UDI_RESOURCES_NORMAL*/);
        ::libc::free(cb.gcb.scratch);
    }
    // - Initialise secondary regions (bind them to the primary region)
    for bind in driver_module.udiprops.clone() {
        if let ::udiprops_parse::Entry::InternalBindOps { meta_idx, region_idx, primary_ops_idx, secondary_ops_idx, bind_cb_idx } = bind {
            let Some(rgn) = driver_module.get_region(&instance, region_idx) else {
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
            let (channel_1, channel_2) = channels::spawn_raw();
            let bind_cb = udi_impl::cb::alloc_internal(driver_module, bind_cb_idx, rgn.context, channel_1);
            unsafe {
                channels::anchor(channel_1, driver_module, driver_module.get_meta_ops(ops_pri), instance.regions[0].context);
                channels::anchor(channel_2, driver_module, driver_module.get_meta_ops(ops_sec), rgn.context);
                channels::event_ind_bound_internal(channel_1, bind_cb);
            }
        }
    }
    // - Bind to the parent driver
    if let Some(channel_to_parent) = channel_to_parent {
        for ent in driver_module.udiprops.clone() {
            if let ::udiprops_parse::Entry::ParentBindOps { meta_idx, region_idx, ops_idx, bind_cb_idx } = ent {
                let Some(rgn) = driver_module.get_region(&instance, region_idx) else {
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

                let bind_cb = udi_impl::cb::alloc_internal(driver_module, bind_cb_idx, rgn.context, channel_to_parent);
                unsafe {
                    channels::anchor(channel_to_parent, driver_module, driver_module.get_meta_ops(ops_init), rgn.context);
                    channels::event_ind_bound_internal(channel_to_parent, bind_cb);
                }
            }
        }
    }
}

struct DriverModule<'a> {
    pri_init: &'a ::udi::ffi::init::udi_primary_init_t,
    sec_init: &'a [::udi::ffi::init::udi_secondary_init_t],
    ops: &'a [::udi::ffi::init::udi_ops_init_t],
    cbs: &'a [::udi::ffi::init::udi_cb_init_t],
    udiprops: ::udiprops_parse::EncodedIter<'a>,

    // Parsed info from `udiprops`
}
impl<'a> DriverModule<'a> {
    unsafe fn new(init: &'a ::udi::ffi::init::udi_init_t, udiprops: ::udiprops_parse::EncodedIter<'a>) -> Self {
        let rv = Self {
            pri_init: init.primary_init_info.expect("No primary_init_info for primary module"),
            sec_init: terminated_list(init.secondary_init_list, |si| si.region_idx == 0),
            ops: terminated_list(init.ops_init_list, |v| v.ops_idx == 0),
            cbs: terminated_list(init.cb_init_list, |cbi: &udi::ffi::init::udi_cb_init_t| cbi.cb_idx == 0),
            udiprops: udiprops.clone(),
        };
        #[cfg(false_)]
        for ent in udiprops.clone()
        {
            match ent
            {
            ::udiprops_parse::Entry::Requires(interface_name, version) => {},
            ::udiprops_parse::Entry::Metalang { meta_idx, interface_name } => {
                // TODO: Make sure that we know about this interface
                },
            ::udiprops_parse::Entry::InternalBindOps { meta_idx, region_idx, primary_ops_idx, secondary_ops_idx, bind_cb_idx } => {
                // TODO: Sanity check the values
                },
            _ => {},
            }
        }
        rv
    }

    fn get_region<'o>(&self, instance: &'o DriverInstance, region_idx: u8) -> Option<&'o DriverRegion> {
        if region_idx == 0 {
            return Some(&instance.regions[0]);
        }
        else {
            Iterator::zip(
                self.sec_init.iter(),
                instance.regions[1..].iter(),
                )
                .find(|(v,_)| v.region_idx == region_idx)
                .map(|(_,v)| v)
        }
    }

    fn get_ops_init(&self, ops_idx: ::udi::ffi::udi_index_t) -> Option<&::udi::ffi::init::udi_ops_init_t> {
        self.ops.iter()
            .find(|v| v.ops_idx == ops_idx)
    }
    fn get_cb_init(&self, cb_idx: ::udi::ffi::udi_index_t) -> Option<&::udi::ffi::init::udi_cb_init_t> {
        self.cbs.iter()
            .find(|v| v.cb_idx == cb_idx)
    }

    fn get_metalang_name(&self, des_meta_idx: ::udi::ffi::udi_index_t) -> Option<&str> {
        for entry in self.udiprops.clone()
        {
            if let ::udiprops_parse::Entry::Metalang { meta_idx, interface_name } = entry {
                if meta_idx == des_meta_idx {
                    return Some(interface_name);
                }
            }
        }
        None
    }
    fn get_metalang(&self, des_meta_idx: ::udi::ffi::udi_index_t) -> Option<&dyn udi::metalang_trait::Metalanguage> {
        Some(match self.get_metalang_name(des_meta_idx)?
        {
        "udi_bridge" => /*udi_impl::meta_bus::METALANG_SPEC*/todo!(),
        "udi_nic" => todo!(),
        l => todo!("Unknown metalang {:?}", l),
        })
    }
    unsafe fn get_meta_ops(&self, ops: &::udi::ffi::init::udi_ops_init_t) -> &'static dyn udi::metalang_trait::MetalangOpsHandler {
        match self.get_metalang(ops.meta_idx)
        {
        None => panic!("Unknown meta_idx {}", ops.meta_idx),
        Some(l) => l.get_ops(ops.meta_ops_num, ops.ops_vector).unwrap(),
        }
    }
    fn get_cb_spec(&self, cb_init: &::udi::ffi::init::udi_cb_init_t) -> &dyn udi::metalang_trait::MetalangCbHandler {
        match self.get_metalang(cb_init.meta_idx)
        {
        None => panic!("Unknown meta_idx {}", cb_init.meta_idx),
        Some(l) => l.get_cb(cb_init.meta_cb_num).unwrap(),
        }
    }
}

/// Get a slice from a NUL-terminated list
/// 
/// SAFETY: The caller attests that the input pointer is either NULL, or points to a list that ends with an
/// item that causes `cb` to return `true`
unsafe fn terminated_list<'a, T: 'a>(input: *const T, cb: impl Fn(&T)->bool) -> &'a [T] {
    if input.is_null() {
        return &[];
    }
    let mut p = input;
    let mut count = 0;
    while ! cb(&*p) {
        p = p.add(1);
        count += 1;
    }
    ::std::slice::from_raw_parts(input, count)
}

struct DriverInstance
{
    regions: Vec<DriverRegion>,
    //management_channel: ::udi::ffi::udi_channel_t,
    //cur_state: DriverState,
}
struct InstanceInitState<'a> {
    instance: &'a DriverInstance,
    state: DriverState,
}
impl InstanceInitState<'_> {
    fn usage_ind(&mut self) {
        match self.state
        {
        DriverState::UsageInd => {
            self.state = DriverState::SecondaryBind;
            },
        _ => {},
        }
    }
}
enum DriverState {
    UsageInd,
    SecondaryBind,
    ParentBind,
    EnumChildren,
    Active,
}
struct DriverRegion
{
    context: *mut ::udi::ffi::c_void,
}
impl DriverRegion
{
    fn new(region_index: ::udi::ffi::udi_index_t, rdata_size: usize) -> DriverRegion {
        DriverRegion {
            context: unsafe {
                let v: *mut udi::ffi::init::udi_init_context_t = ::libc::malloc(rdata_size) as *mut ::udi::ffi::init::udi_init_context_t;
                (*v).region_index = region_index;
                (*v).limits.max_safe_alloc = 0x1000;
                (*v).limits.max_legal_alloc = 1 << 20;
                v as *mut ::udi::ffi::c_void
                },
        }
    }
}