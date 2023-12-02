#[test]
fn trace() {
    // HACK: Reference using the implementation's path, so it's available
    let _ = ::udi_environment::udi_impl::log::udi_trace_write;

    ::udi_macros::udiprops!("
message 101 Hello world %d
");
    static MGMT_OPS: ::udi::ffi::meta_mgmt::udi_mgmt_ops_t = ::udi::ffi::meta_mgmt::udi_mgmt_ops_t {
        usage_ind_op: {unsafe extern "C" fn f(_: *mut ::udi::ffi::meta_mgmt::udi_usage_cb_t, _: u8) {} f},
        enumerate_req_op: {unsafe extern "C" fn f(_: *mut ::udi::ffi::meta_mgmt::udi_enumerate_cb_t, _: u8) {} f},
        devmgmt_req_op: {unsafe extern "C" fn f(_: *mut ::udi::ffi::meta_mgmt::udi_mgmt_cb_t, _: u8, _: u8) {} f},
        final_cleanup_req_op: {unsafe extern "C" fn f(_: *mut ::udi::ffi::meta_mgmt::udi_mgmt_cb_t) {} f},
    };
    let pri_init = Box::leak(Box::new(::udi::ffi::init::udi_primary_init_t {
        mgmt_ops: &MGMT_OPS,
        mgmt_op_flags: ::core::ptr::null(),
        mgmt_scratch_requirement: 0,
        enumeration_attr_list_length: 0,
        rdata_size: ::core::mem::size_of::< ::udi::init::RData<()> >(),
        child_data_size: 0,
        per_parent_paths: 0,
    }));
    let init = Box::leak(Box::new(::udi::ffi::init::udi_init_t {
        primary_init_info: Some(pri_init),
        secondary_init_list: ::core::ptr::null(),
        ops_init_list: ::core::ptr::null(),
        cb_init_list: ::core::ptr::null(),
        gcb_init_list: ::core::ptr::null(),
        cb_select_list: ::core::ptr::null(),
    }));
    let m = unsafe { ::udi_environment::DriverModule::new(init, ::udiprops_parse::load_from_raw_section(&udiprops::udiprops)) };
    let m = ::std::sync::Arc::new(m);
    let i = ::udi_environment::DriverInstance::new(m);
    
    let context = i.regions[0].context() as *mut ::udi::init::RData<()>;
    let context = unsafe { &*context };
    ::udi::log::trace_write(&context, ::udi::log::TraceEvent::LocalProcEntry, 0.into(), udiprops::Msg101, (1234,));
}