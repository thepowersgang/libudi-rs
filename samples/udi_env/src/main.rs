#![feature(impl_trait_in_assoc_type)]

#[macro_use]
mod channels;
mod udi_impl;

mod bridge_pci;
mod management_agent;

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
    // ----
    let driver_module_buspci = unsafe {
        let udiprops = ::udiprops_parse::load_from_raw_section(bridge_pci::UDIPROPS.as_bytes());
        ::std::sync::Arc::new( DriverModule::new(&bridge_pci::INIT_INFO_PCI, udiprops) )
    };
    let mut inst_buspci = create_driver_instance(driver_module_buspci, None);
    if inst_buspci.children.is_empty() {
        inst_buspci.children.push(DriverChild { meta_idx: 1, ops_idx: 1, region_idx_real: 0 });
    }

    // ----
    let driver_module = unsafe {
        let udiprops = ::udiprops_parse::load_from_raw_section({
            let ptr = driver::libudi_rs_udiprops.as_ptr();
            let len = driver::libudi_rs_udiprops_len;
            ::core::slice::from_raw_parts(ptr, len)
            });
        ::std::sync::Arc::new(DriverModule::new(&driver::udi_init_info, udiprops))
    };

    let mut is_orphan = true;
    for entry in driver_module.udiprops.clone() {
        if let ::udiprops_parse::Entry::Device { device_name, meta_idx, attributes } = entry {
            is_orphan = false;
            let meta = driver_module.get_metalang_name(meta_idx);
            // Search all known devices (all children of all loaded instances) for one that matches this attribute list and metalang
            for parent in [&inst_buspci] {
                for child in parent.children.iter() {
                    let meta2 = parent.module.get_metalang_name(child.meta_idx);
                    if meta != meta2 {
                        continue ;
                    }
                    // TODO: Check attributes
                    for (attr_name,attr_value) in attributes.clone() {
                    }

                    let (channel_parent, channel_child) = channels::spawn_raw();
                    unsafe {
                        let ops = parent.module.get_meta_ops(parent.module.get_ops_init(child.ops_idx).unwrap());
                        channels::anchor(channel_parent, parent.module.clone(), ops, parent.regions[child.region_idx_real].context);
                    }
                    create_driver_instance(driver_module.clone(), Some(channel_child));
                }
            }
        }
    }

    if is_orphan {
        create_driver_instance(driver_module, None);
    }
}

fn create_driver_instance<'a>(driver_module: ::std::sync::Arc<DriverModule<'static>>, channel_to_parent: Option<::udi::ffi::udi_channel_t>) -> Box<DriverInstance>
{
    // See UDI Spec 10.1.2

    // - Create primary region
    let instance = Box::new(DriverInstance {
        regions: {
            let mut v = vec![DriverRegion::new(0, driver_module.pri_init.rdata_size)];
            for secondary_region in driver_module.sec_init {
                v.push(DriverRegion::new(secondary_region.region_idx, secondary_region.rdata_size));
            }
            v
            },
        module: driver_module,
        children: Vec::new(),
    });
    // - call `udi_usage_ind`
    let mut state = management_agent::InstanceInitState::new(instance, channel_to_parent);
    
    while let Some((cb, fcn)) = state.next_op().take()
    {
        unsafe {
            (*cb).initiator_context = &mut state as *mut _ as *mut ::udi::ffi::c_void;
            fcn(cb);

            let returned_cb = state.returned_cb().unwrap();
            udi_impl::cb::free_internal(returned_cb);
        }
    }
    state.assert_complete()
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
    fn get_metalang_by_name(&self, des_name: &str) -> Option<::udi::ffi::udi_index_t> {
        self.udiprops.clone()
            .filter_map(|v| match v {
                ::udiprops_parse::Entry::Metalang { meta_idx, interface_name } => Some((meta_idx,interface_name)),
                _ => None
            })
            .filter_map(|(idx,name)| if name == des_name { Some(idx) } else { None })
            .next()
    }
    fn get_metalang(&self, des_meta_idx: ::udi::ffi::udi_index_t) -> Option<&dyn udi::metalang_trait::Metalanguage> {
        Some(match self.get_metalang_name(des_meta_idx)?
        {
        "udi_bridge" => &::udi::ffi::meta_bus::METALANG_SPEC,
        "udi_nic" => &::udi::meta_nic::METALANG_SPEC,
        name => todo!("Unknown metalang {:?}", name),
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
    module: ::std::sync::Arc<DriverModule<'static>>,
    regions: Vec<DriverRegion>,
    children: Vec<DriverChild>,
    //management_channel: ::udi::ffi::udi_channel_t,
    //cur_state: DriverState,
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
struct DriverChild {
    meta_idx: u8,
    ops_idx: u8,
    region_idx_real: usize,
}