#![feature(impl_trait_in_assoc_type)]
#![feature(c_variadic)]

use ::std::sync::Arc;

#[macro_use]
pub mod channels;
pub mod udi_impl;

pub mod bridge_pci;
pub mod sink_nsr;
pub mod sink_gio_serial;

pub mod management_agent;

pub mod emulated_devices;

pub struct DriverModule<'a> {
    pri_init: &'a ::udi::ffi::init::udi_primary_init_t,
    sec_init: &'a [::udi::ffi::init::udi_secondary_init_t],
    ops: &'a [::udi::ffi::init::udi_ops_init_t],
    cbs: &'a [::udi::ffi::init::udi_cb_init_t],
    udiprops: ::udiprops_parse::EncodedIter<'a>,

    // Parsed info from `udiprops`
}
impl<'a> DriverModule<'a> {
    pub unsafe fn new(init: &'a ::udi::ffi::init::udi_init_t, udiprops: ::udiprops_parse::EncodedIter<'a>) -> Self {

        let rv = Self {
            pri_init: init.primary_init_info.expect("No primary_init_info for primary module"),
            sec_init: terminated_list(init.secondary_init_list, |si| si.region_idx.0 == 0),
            ops: terminated_list(init.ops_init_list, |v| v.ops_idx.0 == 0),
            cbs: terminated_list(init.cb_init_list, |cbi: &udi::ffi::init::udi_cb_init_t| cbi.cb_idx.0 == 0),
            udiprops: udiprops.clone(),
        };

        for ent in udiprops.clone()
        {
            println!("UDIPROPS: {:?}", ent);
        }

        // TODO: Pre-cache/check some entries
        #[cfg(any())]
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

    pub fn udiprops(&self) -> ::udiprops_parse::EncodedIter<'a> {
        self.udiprops.clone()
    }

    pub fn name(&self) -> &str {
        for p in self.udiprops.clone() {
            if let ::udiprops_parse::Entry::Name(n) = p {
                return self.get_message(n).unwrap_or("BADMSG");
            }
        }
        return "-noname-";
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

    pub fn get_ops_init(&self, ops_idx: ::udi::ffi::udi_index_t) -> Option<&::udi::ffi::init::udi_ops_init_t> {
        self.ops.iter()
            .find(|v| v.ops_idx == ops_idx)
    }
    pub fn get_cb_init(&self, cb_idx: ::udi::ffi::udi_index_t) -> Option<&::udi::ffi::init::udi_cb_init_t> {
        self.cbs.iter()
            .find(|v| v.cb_idx == cb_idx)
    }

    pub fn get_message(&self, message_idx: ::udiprops_parse::parsed::MsgNum) -> Option<&str> {
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
    pub fn get_metalang_name(&self, des_meta_idx: ::udi::ffi::udi_index_t) -> Option<&str> {
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
    pub fn get_metalang_by_name(&self, des_name: &str) -> Option<::udi::ffi::udi_index_t> {
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
        "udi_gio" => &::udi::meta_gio::METALANG_SPEC,
        "udi_bridge" => &::udi::meta_bridge::METALANG_SPEC,
        "udi_nic" => &::udi::meta_nic::METALANG_SPEC,
        name => todo!("Unknown metalang {:?}", name),
        })
    }
    pub unsafe fn get_meta_ops(&self, ops: &::udi::ffi::init::udi_ops_init_t) -> &'static dyn udi::metalang_trait::MetalangOpsHandler {
        match self.get_metalang(ops.meta_idx)
        {
        None => panic!("Unknown meta_idx {}", ops.meta_idx),
        Some(l) => l.get_ops(ops.meta_ops_num, ops.ops_vector).unwrap(),
        }
    }
    pub fn get_cb_spec(&self, cb_init: &::udi::ffi::init::udi_cb_init_t) -> &dyn udi::metalang_trait::MetalangCbHandler {
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

pub struct DriverInstance
{
    pub module: Arc<DriverModule<'static>>,
    pub regions: Vec<DriverRegion>,
    pub children: ::std::sync::Mutex< Vec<DriverChild> >,
    //management_channel: ::udi::ffi::udi_channel_t,
    //cur_state: DriverState,

    pub device: ::std::sync::OnceLock<Box<dyn crate::emulated_devices::PioDevice>>,
    pub pio_abort_sequence: ::std::sync::Mutex<Option<(udi_impl::pio::Handle, usize)>>,

    pub management_state: management_agent::ManagementAgent,
}
impl DriverInstance {
    pub fn new(driver_module: Arc<DriverModule<'static>>) -> Self
    {
        DriverInstance {
            regions: {
                let mut v = vec![DriverRegion::new(&driver_module, 0.into(), driver_module.pri_init.rdata_size)];
                for secondary_region in driver_module.sec_init {
                    v.push(DriverRegion::new(&driver_module, secondary_region.region_idx, secondary_region.rdata_size));
                }
                v
                },
            module: driver_module,
            children: Default::default(),
            device: Default::default(),
            pio_abort_sequence: Default::default(),
            management_state: Default::default(),
        }
    }
}
pub struct DriverRegion
{
    context_raw: *mut RegionContextRaw,
    pub task_queue: ::std::sync::Mutex< ::std::collections::VecDeque<crate::Operation> >,
}
#[repr(C)]
struct RegionContextRaw {
    module: Arc<DriverModule<'static>>,
    _align: [u64; 0],
    context: ::udi::ffi::init::udi_init_context_t,
}
impl DriverRegion
{
    pub fn new(module: &Arc<DriverModule<'static>>, region_index: ::udi::ffi::udi_index_t, rdata_size: usize) -> DriverRegion {
        DriverRegion {
            context_raw: unsafe {
                let alloc_size = if rdata_size == 0 {
                        0
                    }
                    else if rdata_size < ::core::mem::size_of::<::udi::ffi::init::udi_init_context_t>() {
                        eprintln!("WARNING: rdata size is too small! ({} < {})", 
                            rdata_size, ::core::mem::size_of::<::udi::ffi::init::udi_init_context_t>()
                            );
                        ::core::mem::size_of::<RegionContextRaw>()
                    }
                    else {
                        ::core::mem::size_of::<RegionContextRaw>()
                            - ::core::mem::size_of::<::udi::ffi::init::udi_init_context_t>()
                            + rdata_size
                    };
                let v: *mut RegionContextRaw = ::libc::calloc(1, alloc_size) as *mut _;
                if rdata_size == 0 {
                }
                else {
                    ::core::ptr::write(v, RegionContextRaw {
                        module: module.clone(),
                        _align: [],
                        context: udi::ffi::init::udi_init_context_t {
                            region_index,
                            limits: udi::ffi::init::udi_limits_t {
                                max_legal_alloc: 1 << 20,
                                max_safe_alloc: 0x1000,
                                max_trace_log_formatted_len: 512,
                                max_instance_attr_len: 512,
                                min_curtime_res: 1,
                                min_timer_res: 1
                            },
                        }
                    });
                }
                v
                },
            task_queue: Default::default(),
        }
    }
    pub fn context(&self) -> *mut ::udi::ffi::c_void {
        unsafe {
            if self.context_raw.is_null() {
                ::core::ptr::null_mut()
            }
            else {
                ::core::ptr::addr_of_mut!( (*self.context_raw).context ) as *mut _
            }
        }
    }
    unsafe fn driver_module_from_context(r: &udi::ffi::init::udi_init_context_t) -> &DriverModule {
        let ofs = ::core::mem::size_of::<RegionContextRaw>() - ::core::mem::size_of::<udi::ffi::init::udi_init_context_t>();
        let p = (r as *const _ as *const u8).offset(-(ofs as isize));
        let p = p as *const RegionContextRaw;
        assert_eq!(::core::ptr::addr_of!((*p).context), r as *const _);
        &*(*p).module
    }
}
pub struct DriverChild {
    pub is_bound: ::std::cell::Cell<bool>,
    pub child_id: ::udi::ffi::udi_ubit32_t,
    pub meta_idx: ::udi::ffi::udi_index_t,
    pub ops_idx: ::udi::ffi::udi_index_t,
    pub region_idx_real: usize,
    pub attrs: Vec<::udi::ffi::attr::udi_instance_attr_list_t>,
}

pub struct Operation {
    cb: *mut ::udi::ffi::udi_cb_t,
    op: Box<dyn FnOnce(*mut ::udi::ffi::udi_cb_t)>,
}
impl Operation {
    pub fn new<Cb>(cb: *mut Cb, op: impl FnOnce(*mut Cb)+'static) -> Self {
        Operation {
            cb: cb as *mut _,
            op: Box::new(move |cb| op(cb as *mut _)),
        }
    }
    pub fn invoke(self) {
        (self.op)(self.cb);
    }
}

unsafe fn async_call(gcb: *mut ::udi::ffi::udi_cb_t, op: impl FnOnce(*mut ::udi::ffi::udi_cb_t)+'static) {
    // TODO: Check the stack depth, and if it's too deep push onto the region's queue
    if true {
        let region = channels::get_region( &(*gcb).channel );
        region.task_queue.lock().unwrap()
            .push_back(crate::Operation::new(gcb, op))
    }
    else {
        op(gcb);
    }
}