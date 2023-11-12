
#[no_mangle]
unsafe extern "C" fn udi_debug_printf(fmt: *const ::core::ffi::c_char, mut args: ...) {
    use ::std::io::Write;
    struct Sink([u8; 64], usize);
    impl super::libc::SnprintfSink for Sink {
        fn push(&mut self, byte: u8) {
            if self.1 == self.0.len() {
                ::std::io::stdout().write_all(&self.0).unwrap();
                self.1 = 0;
            }
            self.0[self.1] = byte;
            self.1 += 1;
        }
    }
    let mut sink = Sink([0; 64], 0);
    print!("udi_debug_printf: ");
    super::libc::snprintf_inner(&mut sink, fmt, args.as_va_list());
    ::std::io::stdout().write_all(&sink.0[..sink.1]).unwrap();
    println!("");
}