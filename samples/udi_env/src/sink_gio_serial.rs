
#[derive(Default)]
struct Driver {
    //cb_pool: ::udi::cb::Chain<::udi::ffi::meta_gio::udi_gio_xfer_cb_t>,
}
impl ::udi::init::Driver for ::udi::init::RData<Driver>
{
    const MAX_ATTRS: u8 = 0;
    type Future_init<'s> = impl ::core::future::Future<Output=()>;
    fn usage_ind<'s>(&'s mut self, _cb: udi::init::CbRefUsage<'s>, _resouce_level: u8) -> Self::Future_init<'s> {
        async move { }
    }

    type Future_enumerate<'s> = impl ::core::future::Future<Output=(udi::init::EnumerateResult,udi::init::AttrSink<'s>)> + 's;
    fn enumerate_req<'s>(
        &'s mut self,
        _cb: udi::init::CbRefEnumerate<'s>,
        level: udi::init::EnumerateLevel,
        attrs_out: udi::init::AttrSink<'s>
    ) -> Self::Future_enumerate<'s>
    {
        async move {
			match level
			{
			::udi::init::EnumerateLevel::Start
			|::udi::init::EnumerateLevel::StartRescan
			|::udi::init::EnumerateLevel::Next => {
                (::udi::init::EnumerateResult::Done, attrs_out)
                },
			udi::init::EnumerateLevel::New => todo!(),
			udi::init::EnumerateLevel::Directed => todo!(),
			udi::init::EnumerateLevel::Release => todo!(),
			}
        }
    }

    type Future_devmgmt<'s> = impl ::core::future::Future<Output=::udi::Result<u8>> + 's;
    fn devmgmt_req<'s>(&'s mut self, _cb: udi::init::CbRefMgmt<'s>, _mgmt_op: udi::init::MgmtOp, _parent_id: udi::ffi::udi_ubit8_t) -> Self::Future_devmgmt<'s> {
        async move {
            todo!("devmgmt_req");
        }
    }
}

impl ::udi::meta_gio::Client for ::udi::init::RData<Driver>
{
    type Future_bind_ack<'s> = impl ::core::future::Future<Output=()>;
    fn bind_ack<'s>(&'s mut self, cb: ::udi::cb::CbRef<'s,::udi::ffi::meta_gio::udi_gio_bind_cb_t>, size: ::udi::Result<u64>) -> Self::Future_bind_ack<'s> {
        async move {
            match size {
            Ok(0) => {
                // TODO: Save the channel handle?
                // - Allocate a xfer cb to use for TX requests
                let tx_cb = ::udi::cb::alloc::<Cbs::_Xfer>(cb.gcb(), cb.gcb.channel).await;
                },
            Ok(_) => {
                println!("Unexpected non-zero size for a UART");
                },
            Err(e) => println!("Bind failure: {:?}", e),
            }
        }
    }

    type Future_unbind_ack<'s> = impl ::core::future::Future<Output=()>;
    fn unbind_ack<'s>(&'s mut self, _cb: ::udi::cb::CbRef<'s,::udi::ffi::meta_gio::udi_gio_bind_cb_t>) -> Self::Future_unbind_ack<'s> {
        async move { todo!("unbind_ack") }
    }

    type Future_xfer_ack<'s> = impl ::core::future::Future<Output=()>;
    fn xfer_ack<'s>(&'s mut self, cb: ::udi::cb::CbHandle<::udi::ffi::meta_gio::udi_gio_xfer_cb_t>) -> Self::Future_xfer_ack<'s> {
        async move {
            match cb.op
            {
            ::udi::ffi::meta_gio::UDI_GIO_OP_READ => {},
            ::udi::ffi::meta_gio::UDI_GIO_OP_WRITE => {},
            _ => todo!("xfer_ack - Unknown operation: {:#x}", cb.op),
            }
            /*
            self.cb_pool.push_front(cb);
            */
        }
    }

    type Future_xfer_nak<'s> = impl ::core::future::Future<Output=()>;
    fn xfer_nak<'s>(&'s mut self, cb: ::udi::cb::CbHandle<::udi::ffi::meta_gio::udi_gio_xfer_cb_t>, res: ::udi::Result<()>) -> Self::Future_xfer_nak<'s> {
        async move {
            match res {
            Ok(_) => {},
            Err(e) => println!("xfer_nak - Error {:?}", e),
            }
            /*
            self.cb_pool.push_front(cb);
            */
            todo!("xfer_nak")
        }
    }

    type Future_event_ind<'s> = impl ::core::future::Future<Output=()>;
    fn event_ind<'s>(&'s mut self, _cb: ::udi::cb::CbRef<'s,::udi::ffi::meta_gio::udi_gio_event_cb_t>) -> Self::Future_event_ind<'s> {
        async move {
            // Grab a CB and populate it for read
            /*
            if let Some(xfer_cb) = self.cb_pool.pop_front() {
                xfer_cb.op = ::udi::ffi::meta_gio::UDI_GIO_OP_READ;
                xfer_cb.tr_params = ::core::ptr::null_mut();
                ::udi::meta_gio::xfer_req(xfer_cb);
            }
            */
            todo!("event_ind")
        }
    }
}

::udi_macros::udiprops!("
name 100
properties_version 0x101
requires udi_gio 0x101
meta 1 udi_gio
device 101 1 gio_type string uart
parent_bind_ops 1 0 1 1
message 100 Sink GIO serial
message 101 Serial Device

region 0
");
const META_GIO: ::udi::ffi::udi_index_t = udiprops::meta::udi_gio;
::udi::define_driver! {
    Driver as INIT_INFO_GIOSERIAL;
    ops: {
        Client: Meta=META_GIO, ::udi::ffi::meta_gio::udi_gio_client_ops_t,
    },
    cbs: {
        _Bind : Meta=META_GIO, ::udi::ffi::meta_gio::udi_gio_bind_cb_t,
        _Xfer : Meta=META_GIO, ::udi::ffi::meta_gio::udi_gio_xfer_cb_t,
        _Event: Meta=META_GIO, ::udi::ffi::meta_gio::udi_gio_event_cb_t,
    }
}