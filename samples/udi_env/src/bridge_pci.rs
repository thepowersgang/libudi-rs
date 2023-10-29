struct Driver {
    
}
impl ::udi::meta_bus::BusBridge for Driver
{
    type Future_bind_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn bus_bind_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefBind<'a>) -> Self::Future_bind_req<'a> {
        async move {
            todo!();
        }
    }

    type Future_unbind_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn bus_unbind_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefBind<'a>) -> Self::Future_unbind_req<'a> {
        async move {
            todo!();
        }
    }

    type Future_intr_attach_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_attach_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefIntrAttach<'a>) -> Self::Future_intr_attach_req<'a> {
        async move {
            todo!();
        }
    }

    type Future_intr_detach_req<'s> = impl ::core::future::Future<Output=()> + 's;
    fn intr_detach_req<'a>(&'a mut self, cb: ::udi::meta_bus::CbRefIntrDetach<'a>) -> Self::Future_intr_detach_req<'a> {
        async move {
            todo!();
        }
    }
}