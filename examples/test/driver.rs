
pub struct Driver
{
}
#[allow(non_camel_case_types)]
type Future_init = impl ::core::future::Future<Output=Driver> + 'static;
impl ::udi::Driver for Driver
{
	type Future_init = Future_init;
	fn init(mut cb: ::udi::Gcb, _resouce_level: u8) -> Future_init {
		async move {
			println!("Entry");
			let h1 = cb.pio_map(0,0x1000,4, &[], 0, 0/*, 0*/).await;
			println!("h1 = {:?}", h1);
			let h2 = cb.pio_map(0,0x1004,4, &[], 0, 0/*, 0*/).await;
			println!("h2 = {:?}", h2);
			Driver {}
		}
	}
}
