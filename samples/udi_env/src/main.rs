#![feature(impl_trait_in_assoc_type)]
use ::std::sync::Arc;

#[macro_use]
mod channels;
mod udi_impl;

mod bridge_pci;
mod sink_nsr;

mod management_agent;

mod emulated_devices;

extern crate udi_net_ne2000;

mod driver {
    extern "C" {
        pub static udi_init_info: ::udi::ffi::init::udi_init_t;
        // TODO: udiprops (formally they're in a section as nul terminated strings)
        pub static libudi_rs_udiprops: [u8; 0];
        pub static libudi_rs_udiprops_len: usize;
    }
}

struct GlobalState {
    modules: Vec< Arc<DriverModule<'static>> >,
    instances: Vec< Arc<DriverInstance> >,
}

fn main() {

    let mut state = GlobalState{
        modules: vec![],
        instances: vec![],
    };

    // ----
    register_driver_module(&mut state, unsafe {
        let udiprops = ::udiprops_parse::load_from_raw_section(bridge_pci::UDIPROPS.as_bytes());
        ::std::sync::Arc::new( DriverModule::new(&bridge_pci::INIT_INFO_PCI, udiprops) )
    });
    register_driver_module(&mut state, unsafe {
        let udiprops = ::udiprops_parse::load_from_raw_section(sink_nsr::UDIPROPS.as_bytes());
        ::std::sync::Arc::new( DriverModule::new(&sink_nsr::INIT_INFO_NSR, udiprops) )
    });

    // ----
    let driver_module_ne2000 = unsafe {
        let udiprops = ::udiprops_parse::load_from_raw_section({
            let ptr = driver::libudi_rs_udiprops.as_ptr();
            let len = driver::libudi_rs_udiprops_len;
            ::core::slice::from_raw_parts(ptr, len)
            });
        ::std::sync::Arc::new(DriverModule::new(&driver::udi_init_info, udiprops))
    };
    let _ = driver_module_ne2000;
    register_driver_module(&mut state, driver_module_ne2000);

    for a in ::std::env::args_os().skip(1)
    {
        let path = ::std::ffi::CString::new(a.as_encoded_bytes()).unwrap();
        println!("LOADING {:?}", path);
        let driver_module_uart = unsafe {
            let h = ::libc::dlopen(path.as_ptr() as _, ::libc::RTLD_NOW);
            if h.is_null() {
                panic!("Load failed: {:?}", ::std::ffi::CStr::from_ptr(::libc::dlerror()));
            }
            let udi_init_info = ::libc::dlsym(h, "udi_init_info\0".as_ptr() as _);
            let udiprops_start = ::libc::dlsym(h, "UDIPROPS_start\0".as_ptr() as _);
            let udiprops_end = ::libc::dlsym(h, "UDIPROPS_end\0".as_ptr() as _);
            assert!(!udi_init_info.is_null());
            assert!(!udiprops_start.is_null());
            assert!(!udiprops_end.is_null());

            let udi_init_info = &*(udi_init_info as *const ::udi::ffi::init::udi_init_t);
            let udiprops = ::core::slice::from_raw_parts(udiprops_start as _, udiprops_end as usize - udiprops_start as usize);

            let udiprops = ::udiprops_parse::load_from_raw_section(udiprops);
            ::std::sync::Arc::new(DriverModule::new(udi_init_info, udiprops))
        };

        register_driver_module(&mut state, driver_module_uart);
    }

    // TODO: Run device emulation?
    println!("--- DONE ---")
}

/// Create an instance for all matching parent instances
fn register_driver_module(state: &mut GlobalState, driver_module: Arc<DriverModule<'static>>)
{
    let mut is_orphan = true;

    let mut new_instances = vec![];
    for entry in driver_module.udiprops.clone() {
        let ::udiprops_parse::Entry::Device { device_name, meta_idx, attributes } = entry else {
            continue
            };
        is_orphan = false;
        let meta = driver_module.get_metalang_name(meta_idx);
        println!("meta {:?} Device {:?}", meta, driver_module.get_message(device_name));
        // Search all known devices (all children of all loaded instances) for one that matches this attribute list and metalang
        for parent in state.instances.iter_mut()
        {
            for child in parent.children.lock().unwrap().iter()
            {
                if child.is_bound.get() {
                    continue ;
                }
                let meta2 = parent.module.get_metalang_name(child.meta_idx);

                if let Some(i) = maybe_child_bind(&driver_module, meta, attributes.clone(), parent, meta2, child)
                {
                    new_instances.push(i);
                }
            }
        }
    }

    if is_orphan {
        new_instances.push( create_driver_instance(driver_module.clone(), None) );
    }

    // Find matching drivers for children of the new instances
    let i = state.instances.len();
    state.instances.extend(new_instances.drain(..));

    // For all new instances
    for parent in &mut state.instances[i..]
    {
        // Look for child drivers
        for child in parent.children.lock().unwrap().iter()
        {
            if child.is_bound.get() {
                continue ;
            }
            let meta2 = parent.module.get_metalang_name(child.meta_idx);

            // Find a matching device
            for driver_module in state.modules.iter()
            {
                for entry in driver_module.udiprops.clone() {
                    let ::udiprops_parse::Entry::Device { device_name: _, meta_idx, attributes } = entry else {
                        continue
                        };
                    let meta = driver_module.get_metalang_name(meta_idx);

                    if let Some(i) = maybe_child_bind(&driver_module, meta, attributes, parent, meta2, child)
                    {
                        new_instances.push(i);
                    }
                }
            }
        }
    }

    state.instances.extend(new_instances.drain(..));
    state.modules.push(driver_module);
}

fn check_attributes(filter_attributes: ::udiprops_parse::parsed::AttributeList, child_attrs: &[::udi::ffi::attr::udi_instance_attr_list_t]) -> bool
{
    fn find_attr<'a>(des_attr_name: &str, attrs: &'a [::udi::ffi::attr::udi_instance_attr_list_t]) -> Option<&'a ::udi::ffi::attr::udi_instance_attr_list_t> {
        for a in attrs.iter()
        {
            let a_name = {
                let name_len = a.attr_name.iter().position(|v| *v == 0).unwrap_or(a.attr_name.len());
                let name = &a.attr_name[..name_len];
                ::std::str::from_utf8(name).unwrap_or("")
                };
            if a_name == des_attr_name {
                return Some(a);
            }
        }
        None
    }
    for (attr_name,attr_value) in filter_attributes
    {
        if let Some(a) = find_attr(attr_name, child_attrs) {
            let is_match = match a.attr_type
                {
                ::udi::ffi::attr::UDI_ATTR_STRING => {
                    let v = &a.attr_value[..a.attr_length as usize];
                    let v = ::std::str::from_utf8(v).unwrap_or("");
                    println!("check_attributes: {} {:?} == {:?}", attr_name, attr_value, v);

                    match attr_value {
                    ::udiprops_parse::parsed::Attribute::String(val) => val == v,
                    _ => false,
                    }
                    }
                ::udi::ffi::attr::UDI_ATTR_ARRAY8 => {
                    let v = &a.attr_value[..a.attr_length as usize];
                    println!("check_attributes: {} {:?} == {:?}", attr_name, attr_value, v);
                    match attr_value {
                    ::udiprops_parse::parsed::Attribute::Array8(val) => val == v,
                    _ => false,
                    }
                    }
                ::udi::ffi::attr::UDI_ATTR_UBIT32 => {
                    let v = u32::from_le_bytes(a.attr_value[..4].try_into().unwrap());
                    println!("check_attributes: {} {:?} == {:?}", attr_name, attr_value, v);
                    match attr_value {
                    ::udiprops_parse::parsed::Attribute::Ubit32(val) => val == v,
                    _ => false,
                    }
                    }
                ::udi::ffi::attr::UDI_ATTR_BOOLEAN => {
                    let v = a.attr_value[0] != 0;
                    println!("{} {:?} == {:?}", attr_name, attr_value, v);
                    match attr_value {
                    ::udiprops_parse::parsed::Attribute::Boolean(val) => val == v,
                    _ => false,
                    }
                    }
                _ => todo!("Handle attribute type {}", a.attr_type),
                };
            if !is_match {
                return false;
            }
        }
        else {
            // Fail if the attribute is missing?
            // - The meta matching implies that it should work though
            return false;
        }
    }

    true
}


fn maybe_child_bind(
    driver_module: &Arc<DriverModule<'static>>,
    meta: Option<&str>,
    attributes: ::udiprops_parse::parsed::AttributeList,
    parent: &Arc<DriverInstance>,
    meta2: Option<&str>,
    child: &DriverChild
) -> Option<Arc<DriverInstance>>
{
    if meta != meta2 {
        return None;
    }

    if !check_attributes(attributes.clone(), &child.attrs) {
        return None;
    }

    child.is_bound.set(true);

    let (channel_parent, channel_child) = channels::spawn_raw();
    unsafe {
        let ops_init = parent.module.get_ops_init(child.ops_idx).unwrap();
        let ops = parent.module.get_meta_ops(ops_init);
        let rdata = parent.regions[child.region_idx_real].context;
        let context = if ops_init.chan_context_size > 0 {
                assert!(ops_init.chan_context_size >= ::core::mem::size_of::<::udi::ffi::init::udi_child_chan_context_t>());
                let ccx = ::libc::malloc(ops_init.chan_context_size) as *mut ::udi::ffi::init::udi_child_chan_context_t;
                (*ccx).rdata = rdata;
                (*ccx).child_id = child.child_id;
                println!("udi_child_chan_context_t(s={}): {:p} #{}", ops_init.chan_context_size, rdata, child.child_id);
                ccx as *mut ::udi::ffi::c_void
            }
            else {
                rdata
            };
        channels::anchor(channel_parent, parent.clone(), ops, context);
    }

    println!("maybe_child_bind: Creating instance of `{}` bound to {} #{}",
        driver_module.name(),
        parent.module.name(), child.child_id
        );
    Some( create_driver_instance(driver_module.clone(), Some(channel_child)) )
}

fn create_driver_instance<'a>(driver_module: Arc<DriverModule<'static>>, channel_to_parent: Option<::udi::ffi::udi_channel_t>) -> Arc<DriverInstance>
{
    // See UDI Spec 10.1.2

    // - Create primary region
    let instance = Arc::new(DriverInstance {
        regions: {
            let mut v = vec![DriverRegion::new(0.into(), driver_module.pri_init.rdata_size)];
            for secondary_region in driver_module.sec_init {
                v.push(DriverRegion::new(secondary_region.region_idx, secondary_region.rdata_size));
            }
            v
            },
        module: driver_module,
        children: Default::default(),
        device: Default::default(),
    });
    let mut state = management_agent::InstanceInitState::new(instance, channel_to_parent);
    
    while let Some((cb, fcn)) = state.next_op().take()
    {
        unsafe {
            (*cb).initiator_context = &mut state as *mut _ as *mut ::udi::ffi::c_void;
            fcn(cb);

            let returned_cb = state.returned_cb().expect("No returned CB?");
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
            sec_init: terminated_list(init.secondary_init_list, |si| si.region_idx.0 == 0),
            ops: terminated_list(init.ops_init_list, |v| v.ops_idx.0 == 0),
            cbs: terminated_list(init.cb_init_list, |cbi: &udi::ffi::init::udi_cb_init_t| cbi.cb_idx.0 == 0),
            udiprops: udiprops.clone(),
        };
        if true {
            if let Some(pi) = init.primary_init_info {
                dbg!(pi.mgmt_ops.devmgmt_req_op);
            }
            dbg!(rv.ops.len());
            for o in rv.ops {
                dbg!(*o.ops_vector);
            }
        }

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

    fn name(&self) -> &str {
        for p in self.udiprops.clone() {
            if let ::udiprops_parse::Entry::Name(n) = p {
                return self.get_message(n).unwrap_or("BADMSG");
            }
        }
        return "?";
    }

    fn get_region_index(&self, region_idx: ::udi::ffi::udi_index_t) -> Option<usize> {
        if region_idx.0 == 0 {
            return Some(0);
        }
        else {
            self.sec_init.iter()
                .enumerate()
                .find(|(_, v)| v.region_idx == region_idx)
                .map(|(i,_)| i)
        }
    }
    fn get_region<'o>(&self, instance: &'o DriverInstance, region_idx: ::udi::ffi::udi_index_t) -> Option<&'o DriverRegion> {
        if let Some(i) = self.get_region_index(region_idx) {
            Some(&instance.regions[i])
        }
        else {
            None
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

    fn get_message(&self, message_idx: ::udiprops_parse::parsed::MsgNum) -> Option<&str> {
        for entry in self.udiprops.clone()
        {
            if let ::udiprops_parse::Entry::Message(idx, value) = entry {
                if message_idx == idx {
                    return Some(value);
                }
            }
        }
        None
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
        "udi_bridge" => &::udi::meta_bus::METALANG_SPEC,
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
    module: Arc<DriverModule<'static>>,
    regions: Vec<DriverRegion>,
    children: ::std::sync::Mutex< Vec<DriverChild> >,
    //management_channel: ::udi::ffi::udi_channel_t,
    //cur_state: DriverState,

    device: ::std::sync::OnceLock<Box<dyn crate::emulated_devices::PioDevice>>,
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
                if rdata_size == 0 {
                }
                else if rdata_size < ::core::mem::size_of::<::udi::ffi::init::udi_init_context_t>() {
                    eprintln!("WARNING: rdata size is too small! ({} < {})", 
                        rdata_size, ::core::mem::size_of::<::udi::ffi::init::udi_init_context_t>()
                        )
                }
                else {
                    (*v).region_index = region_index;
                    (*v).limits.max_safe_alloc = 0x1000;
                    (*v).limits.max_legal_alloc = 1 << 20;
                }
                v as *mut ::udi::ffi::c_void
                },
        }
    }
}
struct DriverChild {
    is_bound: ::std::cell::Cell<bool>,
    child_id: ::udi::ffi::udi_ubit32_t,
    meta_idx: ::udi::ffi::udi_index_t,
    ops_idx: ::udi::ffi::udi_index_t,
    region_idx_real: usize,
    attrs: Vec<::udi::ffi::attr::udi_instance_attr_list_t>,
}