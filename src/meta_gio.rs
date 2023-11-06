use ::udi_sys::meta_gio::*;

impl_metalanguage!{
    static METALANG_SPEC;
    NAME udi_gio;
    OPS
        1 => udi_gio_provider_ops_t,
        2 => udi_gio_client_ops_t,
        ;
    CBS
        1 => udi_gio_bind_cb_t,
        2 => udi_gio_xfer_cb_t,
        3 => udi_gio_event_cb_t,
        ;
}