
mod channels;
mod udi_impl;

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
            //channel: channels::allocate_empty_channel(&driver_module),
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
            let Some((ri,rgn)) = driver_module.sec_init.iter().enumerate().find(|(_,v)| v.region_idx == region_idx) else {
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
            fn get_meta_ops<'a>(module: &'a DriverModule, ops: &::udi::ffi::init::udi_ops_init_t) -> &'static dyn channels::MetalangOps {
                todo!();
            }
            //let ch = channels::allocate_channel(instance.regions[0].context, get_meta_ops(&driver_module, ops_pri), cb.scratch_requirement);
            //channels::bind_channel_other(ch, driver_module, instance.regions[ri].context, get_meta_ops(&driver_module, ops_sec), cb.scratch_requirement);
        }
    }
    #[cfg(false_)]
    for r in driver_module.sec_init {
        let scratch_requirement = match driver_module.get_cb_init(cb_idx)
            {
            None => panic!(""),
            Some(v) => v.scratch_requirement,
            };
        let ops_pri = match driver_module.get_ops_init(pri_ops_idx)
            {
            None => panic!(""),
            Some(v) => v.ops_vector,
            };
        channels::allocate_channel(instance.regions[0].context, ops_pri, scratch_requirement);
    }
    // - Bind to the parent driver
}

struct DriverModule<'a> {
    pri_init: &'a ::udi::ffi::init::udi_primary_init_t,
    sec_init: &'a [::udi::ffi::init::udi_secondary_init_t],
    ops: &'a [::udi::ffi::init::udi_ops_init_t],
    cbs: &'a [::udi::ffi::init::udi_cb_init_t],
    udiprops: ::udiprops_parse::EncodedIter<'a>,
}
impl<'a> DriverModule<'a> {
    unsafe fn new(init: &'a ::udi::ffi::init::udi_init_t, udiprops: ::udiprops_parse::EncodedIter<'a>) -> Self {
        Self {
            pri_init: init.primary_init_info.expect("No primary_init_info for primary module"),
            sec_init: terminated_list(init.secondary_init_list, |si| si.region_idx == 0),
            ops: terminated_list(init.ops_init_list, |v| v.ops_idx == 0),
            cbs: terminated_list(init.cb_init_list, |cbi: &udi::ffi::init::udi_cb_init_t| cbi.cb_idx == 0),
            udiprops,
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