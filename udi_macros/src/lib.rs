/*
Challenge: nested structures
*/

mod udiprops;
mod printf;
mod derive_getlayout;

/// Parse a `udiprops.txt` body from a string, and generate a `udiprops` module
/// 
/// See `udiprops_parse` for how this is done.
#[proc_macro]
pub fn udiprops(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    udiprops::udiprops(input)
}

/// Call the `udi_debug_printf` function without needing unsafe
#[proc_macro]
pub fn debug_printf(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    printf::debug_printf(input)
}
/// Call the `udi_snprintf` function without needing unsafe
#[proc_macro]
pub fn snprintf(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    printf::snprintf(input)
}

/// Derive macro for the `GetLayout` trait
#[proc_macro_derive(GetLayout, attributes(layout_ignore))]
pub fn derive(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    derive_getlayout::derive(input)
}