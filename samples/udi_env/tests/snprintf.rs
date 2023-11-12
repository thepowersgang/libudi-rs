#[test]
fn snprintf_bitfields() {
    // HACK: Reference using the implementation's path, so it's available
    let _ = ::udi_environment::udi_impl::libc::udi_snprintf;

    let mut buf = [0u8; 80];
    // The below is an example from the UDI spec
    let len = ::udi_macros::snprintf!(&mut buf,
        "%<15=Active,14=DMA Ready,13=XMIT,~13=RCV,0-2=Mode:0=HDX:1=FDX:2=Sync HDX:3=Sync FDX:4=Coded,3-6=TX Threshold,7-10=RX Threshold>",
        0xc093
    );
    let out = ::std::str::from_utf8(&buf[..len]).unwrap();
    assert_eq!(out, "<Active, DMA Ready, RCV, Mode=Sync FDX, TX Threshold=2, RX Threshold=1>");
}