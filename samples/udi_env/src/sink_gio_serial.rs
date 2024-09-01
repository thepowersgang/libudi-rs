
#[derive(Default)]
struct Driver {
    parent_channel: ::core::cell::OnceCell< ::udi::ffi::udi_channel_t >,
    cb_pool: ::udi::cb::SharedQueue<::udi::ffi::meta_gio::udi_gio_xfer_cb_t>,
}
impl ::udi::init::Driver for ::udi::init::RData<Driver>
{
    const MAX_ATTRS: u8 = 0;
    type Future_init<'s> = impl ::core::future::Future<Output=()>;
    fn usage_ind<'s>(&'s self, _cb: udi::init::CbRefUsage<'s>, _resouce_level: u8) -> Self::Future_init<'s> {
        async move { }
    }

    type Future_enumerate<'s> = impl ::core::future::Future<Output=(udi::init::EnumerateResult,udi::init::AttrSink<'s>)> + 's;
    fn enumerate_req<'s>(
        &'s self,
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
    fn devmgmt_req<'s>(&'s self, _cb: udi::init::CbRefMgmt<'s>, _mgmt_op: udi::init::MgmtOp, _parent_id: udi::ffi::udi_ubit8_t) -> Self::Future_devmgmt<'s> {
        async move {
            todo!("devmgmt_req");
        }
    }
}

impl ::udi::meta_gio::Client for ::udi::init::RData<Driver>
{
    type Future_bind_ack<'s> = impl ::core::future::Future<Output=()>;
    fn bind_ack<'s>(&'s self, cb: ::udi::cb::CbRef<'s,::udi::ffi::meta_gio::udi_gio_bind_cb_t>, size: ::udi::Result<u64>) -> Self::Future_bind_ack<'s> {
        async move {
            match size {
            Ok(0) => {
                if let Err(_) = self.parent_channel.set( cb.gcb.channel ) {
                    panic!("Bound twice?")
                }
                
                // Allocate a pool of CBs with 1KiB buffers
                let mut cbs = ::udi::cb::alloc_batch::<CbList::Xfer>(cb.gcb(), 3, Some((1024, ::udi::ffi::buf::UDI_NULL_PATH_BUF))).await;
                while let Some(mut xfer_cb) = cbs.pop_front() {
                    // Channel should already be the same one?
                    // - Except that it isn't.
                    unsafe {
                        xfer_cb.get_mut().gcb.channel = cb.gcb.channel;
                    }
                    self.cb_pool.push_back(xfer_cb);
                }

                // TEST: Send some data
                let mut tx_cb = self.cb_pool.pop_front().unwrap();
                {
                    tx_cb.set_op(::udi::ffi::meta_gio::UDI_GIO_DIR_WRITE);
                    let buf = tx_cb.data_buf_mut();
                    buf.write(cb.gcb(), 0..buf.len(), b"hello").await;
                }
                ::udi::meta_gio::xfer_req(tx_cb);
                },
            Ok(_) => {
                println!("Unexpected non-zero size for a UART");
                },
            Err(e) => println!("Bind failure: {:?}", e),
            }
        }
    }

    type Future_unbind_ack<'s> = impl ::core::future::Future<Output=()>;
    fn unbind_ack<'s>(&'s self, _cb: ::udi::cb::CbRef<'s,::udi::ffi::meta_gio::udi_gio_bind_cb_t>) -> Self::Future_unbind_ack<'s> {
        async move { todo!("unbind_ack") }
    }

    type Future_xfer_ack<'s> = impl ::core::future::Future<Output=()>;
    fn xfer_ack<'s>(&'s self, cb: ::udi::cb::CbRef<'s, ::udi::ffi::meta_gio::udi_gio_xfer_cb_t>) -> Self::Future_xfer_ack<'s> {
        async move {
            match cb.op
            {
            ::udi::ffi::meta_gio::UDI_GIO_OP_READ => {
                self.handle_read(cb.data_buf(), false).await;
                },
            ::udi::ffi::meta_gio::UDI_GIO_OP_WRITE => {},
            _ => todo!("xfer_ack - Unknown operation: {:#x}", cb.op),
            }
        }
    }

    type Future_xfer_nak<'s> = impl ::core::future::Future<Output=()>;
    fn xfer_nak<'s>(&'s self, cb: ::udi::cb::CbRef<'s, ::udi::ffi::meta_gio::udi_gio_xfer_cb_t>, res: ::udi::Result<()>) -> Self::Future_xfer_nak<'s> {
        async move {
            match res {
            Ok(_) => {},
            Err(e) if e.into_inner() == ::udi::ffi::UDI_STAT_DATA_UNDERRUN as _ && cb.op == ::udi::ffi::meta_gio::UDI_GIO_OP_READ => {
                // Signal the drivers
                self.handle_read(cb.data_buf(), true).await;
                },
            Err(e) => {
                println!("xfer_nak - Error {:?}", e)
                },
            }
        }
    }

    fn xfer_ret(&self, cb: ::udi::cb::CbHandle<udi::ffi::meta_gio::udi_gio_xfer_cb_t>) {
        self.cb_pool.push_back(cb);
    }

    type Future_event_ind<'s> = impl ::core::future::Future<Output=()>;
    fn event_ind<'s>(&'s self, _cb: ::udi::cb::CbRef<'s,::udi::ffi::meta_gio::udi_gio_event_cb_t>) -> Self::Future_event_ind<'s> {
        async move {
            // Grab a CB and populate it for read
            if let Some(mut xfer_cb) = self.cb_pool.pop_front() {
                xfer_cb.set_op(::udi::ffi::meta_gio::UDI_GIO_OP_READ);
                ::udi::meta_gio::xfer_req(xfer_cb);
            }
            else {

            }
        }
    }
}

impl Driver {
    async fn handle_read(&self, buf: &::udi::buf::Handle, is_underrun: bool) {
        let mut data = vec![0; buf.len()];
        buf.read(0, &mut data);
        println!("RX{} - {:02x?}", if is_underrun { " (underrun)" } else { "" }, data);
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
        Xfer : Meta=META_GIO, ::udi::ffi::meta_gio::udi_gio_xfer_cb_t,
        _Event: Meta=META_GIO, ::udi::ffi::meta_gio::udi_gio_event_cb_t,
    }
}