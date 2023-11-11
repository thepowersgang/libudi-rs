
pub fn udiprops(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream
{
    let lit: ::syn::LitStr = ::syn::parse_macro_input!(input as ::syn::LitStr);
    let input_string = lit.value();
    let props_lines = ::udiprops_parse::from_reader(&input_string.into_bytes()[..]).expect("Parse of udiprops string failed?");

    let mut body = Vec::new();
    ::udiprops_parse::create_module_body(&mut body, &props_lines, false).unwrap();

    String::from_utf8(body).unwrap().parse().unwrap()
}