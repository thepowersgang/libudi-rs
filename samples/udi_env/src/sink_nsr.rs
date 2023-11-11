#[derive(Default)]
struct Driver {
    ch_rx: ::udi::imc::ChannelHandle,
    ch_tx: ::udi::imc::ChannelHandle,
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

impl ::udi::meta_nic::NsrControl for ::udi::init::RData<Driver>
{
    type Future_gbc<'s> = impl ::core::future::Future<Output=::udi::meta_nic::BindChannels>;
    fn get_bind_channels<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNicBind<'a>) -> Self::Future_gbc<'a> {
        async move {
            let rv = ::udi::meta_nic::BindChannels {
                rx: 1.into(),
                tx: 2.into(),
            };
            self.ch_rx = ::udi::imc::channel_spawn(cb.gcb(), rv.rx, OpsList::Rx).await;
            self.ch_tx = ::udi::imc::channel_spawn(cb.gcb(), rv.tx, OpsList::Tx).await;
            rv
        }
    }
    
    type Future_bind_ack<'s> = impl ::core::future::Future<Output=()>;
    fn bind_ack<'a>(&'a mut self, cb: ::udi::meta_nic::CbRefNicBind<'a>, res: ::udi::Result<()>) -> Self::Future_bind_ack<'a> {
        async move {
            match res {
            Ok(()) => println!("--- SINK_NSR: New device, MAC: {:x?}", &cb.mac_addr[..cb.mac_addr_len as usize]),
            Err(e) => println!("Error: {:?}", e),
            }
        }
    }

    type Future_unbind_ack<'s> = impl ::core::future::Future<Output=()>;
    fn unbind_ack<'a>(&'a mut self, _cb: ::udi::meta_nic::CbRefNic<'a>, _res: ::udi::Result<()>) -> Self::Future_unbind_ack<'a> {
        async move { todo!("unbind_ack") }
    }

    type Future_enable_ack<'s> = impl ::core::future::Future<Output=()>;
    fn enable_ack<'a>(&'a mut self, _cb: ::udi::meta_nic::CbRefNic<'a>, _res: ::udi::Result<()>) -> Self::Future_enable_ack<'a> {
        async move { todo!("enable_ack") }
    }

    type Future_ctrl_ack<'s> = impl ::core::future::Future<Output=()>;
    fn ctrl_ack<'a>(&'a mut self, _cb: ::udi::meta_nic::CbRefNicCtrl<'a>, _res: ::udi::Result<()>) -> Self::Future_ctrl_ack<'a> {
        async move { todo!("ctrl_ack") }
    }

    type Future_info_ack<'s> = impl ::core::future::Future<Output=()>;
    fn info_ack<'a>(&'a mut self, _cb: ::udi::meta_nic::CbRefNicInfo<'a>) -> Self::Future_info_ack<'a> {
        async move { todo!("info_ack") }
    }

    type Future_status_ind<'s> = impl ::core::future::Future<Output=()>;
    fn status_ind<'a>(&'a mut self, _cb: ::udi::meta_nic::CbRefNicStatus<'a>) -> Self::Future_status_ind<'a> {
        async move { todo!("status_ind") }
    }
}
impl ::udi::meta_nic::NsrTx for ::udi::init::RData<Driver>
{
    type Future_tx_rdy<'s> = impl ::core::future::Future<Output=()>;
    fn tx_rdy<'a>(&'a mut self, _cb: ::udi::meta_nic::CbHandleNicTx) -> Self::Future_tx_rdy<'a> {
        async move { todo!("tx_rdy") }
    }
}
impl ::udi::meta_nic::NsrRx for ::udi::init::RData<Driver>
{
    type Future_rx_ind<'s> = impl ::core::future::Future<Output=()>;
    fn rx_ind<'a>(&'a mut self, _cb: ::udi::meta_nic::CbHandleNicRx) -> Self::Future_rx_ind<'a> {
        async move { todo!("rx_ind") }
    }

    type Future_exp_rx_ind<'s> = impl ::core::future::Future<Output=()>;
    fn exp_rx_ind<'a>(&'a mut self, _cb: ::udi::meta_nic::CbHandleNicRx) -> Self::Future_exp_rx_ind<'a> {
        async move { todo!("exp_rx_ind") }
    }
}

::udi_macros::udiprops!("
name 100
properties_version 0x101
requires udi_nic 0x101
meta 1 udi_nic
device 101 1
# Meta 1, region 0, Ops 2 (Ctrl), CB 2 (NicBind)
parent_bind_ops 1 0 1 2
message 100 Sink NSR
message 101 Network Device

region 0
");
::udi::define_driver! {
    Driver as INIT_INFO_NSR;
    ops: {
        Ctrl: Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nsr_ctrl_ops_t,
        Tx  : Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nsr_tx_ops_t,
        Rx  : Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nsr_rx_ops_t,
    },
    cbs: {
        _Nic    : Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_cb_t,
        _NicBind: Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_bind_cb_t,
        _NicCtrl: Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_ctrl_cb_t,
        _NicInfo: Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_info_cb_t,
        _NicRx  : Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_rx_cb_t,
        _NicTx  : Meta=udiprops::meta::udi_nic, ::udi::ffi::meta_nic::udi_nic_tx_cb_t,
    }
}