#![allow(non_camel_case_types)]
use udi::layout::GetLayout;
use udi::ffi::*;

#[test]
fn test1() {
    #[derive(::udi_macros::GetLayout)]
    #[repr(C)]
    pub struct udi_nic_rx_cb_t
    {
        #[layout(ignore)]
        pub gcb: udi_cb_t,
        #[layout(chain_cb)]
        pub chain: *mut udi_nic_rx_cb_t,
        //#[layout(buf(rx_status & 0x1 == 0x1))]
        pub rx_buf: *mut udi_buf_t,
        pub rx_status: udi_ubit8_t,
        pub addr_match: udi_ubit8_t,
        pub rx_valid: udi_ubit8_t,
    }
    assert_eq!(
        <udi_nic_rx_cb_t as GetLayout>::LAYOUT,
        [
            layout::UDI_DL_CB,
            layout::UDI_DL_BUF, 0, 0,1,
            layout::UDI_DL_UBIT8_T,
            layout::UDI_DL_UBIT8_T,
            layout::UDI_DL_UBIT8_T,
        ]
    );
}
