fn main() {
    ::cc::Build::new()
        .file("c_shim.c")
        .compile("c_shim")
        ;
}