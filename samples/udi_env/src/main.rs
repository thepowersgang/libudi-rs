
mod channels;
mod udi_impl;

extern crate udi_net_ne2000;

mod driver {
    extern "C" {
        pub static udi_init_info: ::udi::ffi::init::udi_init_t;
        // TODO: udiprops (formally they're in a section as nul terminated strings)
        //static 
    }
}


fn main() {
    let driver_module = unsafe { DriverModule::new(&driver::udi_init_info) };

    // - Create primary region
    let mut instance = Box::new(DriverInstance {
        regions: vec![DriverRegion {
            context: unsafe {
                let v = ::libc::malloc( driver_module.pri_init.rdata_size ) as *mut ::udi::ffi::init::udi_init_context_t;
                //(*v).
                v
                },
            }],
        management_channel: ::core::ptr::null_mut(),
        cur_state: DriverState::UsageInd,
    });
    // - Create the primary mangement channel
    instance.management_channel = unsafe {
        unsafe extern "C" fn usage_res(cb: *mut ::udi::ffi::meta_mgmt::udi_usage_cb_t) {
            // Update state.
            todo!();
        }
        unsafe extern "C" fn devmgmt_ack(cb: *mut ::udi::ffi::meta_mgmt::udi_mgmt_cb_t, _: u8, _: u32) {
            todo!();
        }
        unsafe extern "C" fn enumerate_ack(cb: *mut ::udi::ffi::meta_mgmt::udi_enumerate_cb_t, _: u8, _: u8) {
            todo!();
        }
        unsafe extern "C" fn final_cleanup_ack(cb: *mut ::udi::ffi::meta_mgmt::udi_mgmt_cb_t) {
            todo!();
        }
        static MA_OPS: udi_impl::meta_mgmt::ManagementAgentOps = udi_impl::meta_mgmt::ManagementAgentOps {
            usage_res_op: usage_res,
            devmgmt_ack_op: devmgmt_ack,
            enumerate_ack_op: enumerate_ack,
            final_cleanup_ack_op: final_cleanup_ack,
        };
        let ch = channels::allocate_channel(instance.as_ref() as *const _ as *mut _, &MA_OPS, 0);
        channels::bind_channel_other(ch, instance.regions[0].context as *mut _, driver_module.pri_init.mgmt_ops, driver_module.pri_init.mgmt_scratch_requirement);
        ch
    };
    // - call `udi_usage_ind`
    let mut cb = ::udi::ffi::meta_mgmt::udi_usage_cb_t {
        gcb: ::udi::ffi::udi_cb_t {
            channel: instance.management_channel,
            context: ::core::ptr::null_mut(),
            scratch: ::core::ptr::null_mut(),
            initiator_context: instance.as_ref() as *const _ as *mut _,
            origin: ::core::ptr::null_mut(),
        },
        trace_mask: 0,
        meta_idx: 0,
    };
    unsafe {
        udi_impl::meta_mgmt::udi_usage_ind(&mut cb, 3 /*UDI_RESOURCES_NORMAL*/);
    }
    // - Initialise secondary regions
}

struct DriverModule<'a> {
    pri_init: &'a ::udi::ffi::init::udi_primary_init_t,
    sec_init: &'a [::udi::ffi::init::udi_secondary_init_t],
    // CB list
    cbs: &'a [::udi::ffi::init::udi_cb_init_t],
    // Ops list
    ops: &'a [::udi::ffi::init::udi_ops_init_t],
}
impl<'a> DriverModule<'a> {
    unsafe fn new(init: &'a ::udi::ffi::init::udi_init_t) -> Self {
        let sec_init = if !init.secondary_init_list.is_null() {
            terminated_list(init.secondary_init_list, |si| si.region_idx == 0)
        }
        else {
            &[]
        };
        Self {
            pri_init: init.primary_init_info.expect(""),
            sec_init,
            cbs: terminated_list(init.cb_init_list, |cbi| cbi.cb_idx == 0),
            ops: terminated_list(init.ops_init_list, |v| v.ops_idx == 0),
        }
    }
}
unsafe fn terminated_list<'a, T: 'a>(input: *const T, cb: impl Fn(&T)->bool) -> &'a [T] {
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
    management_channel: ::udi::ffi::udi_channel_t,
    cur_state: DriverState,
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
    context: *mut ::udi::ffi::init::udi_init_context_t,
}