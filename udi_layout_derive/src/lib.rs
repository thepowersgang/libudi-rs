/*
Challenge: nested structures
*/

#[proc_macro_derive(GetLayout, attributes(layout_ignore))]
pub fn derive(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream
{
    let input = ::syn::parse_macro_input!(input as ::syn::DeriveInput);
    let fields = match input.data
        {
        syn::Data::Struct(s) => match s.fields
            {
            syn::Fields::Named(f) => f.named,
            _ => return ::proc_macro::TokenStream::from(::quote::quote!{ compile_error!("Needs a named struct") }),
            },
        _ => return ::proc_macro::TokenStream::from(::quote::quote!{ compile_error!("Needs a struct") }),
        };
    
    //let mut entries = Vec::new();
    let mut tys = Vec::new();
    for field in fields {
        if field.attrs.iter().any(|a| {
            match a.meta {
            syn::Meta::Path(ref p) => p.is_ident("layout_ignore"),
            _ => false,
            }
        }) {
            continue;
        }
        tys.push(field.ty.clone());
        #[cfg(false_)]
        entries.push(match &field.ty
        {
        ::syn::Type::Path(p) => match &p.path.segments.last().unwrap().ident.to_string()[..]
            {
            "udi_ubit8_t" => ::quote::quote_spanned!(field.ty.span() => ::udi::ffi::layout::UDI_DL_UBIT8_T),
            "udi_sbit8_t" => ::quote::quote_spanned!(field.ty.span() => ::udi::ffi::layout::UDI_DL_SBIT8_T),
            "udi_ubit16_t" => ::quote::quote_spanned!(field.ty.span() => ::udi::ffi::layout::UDI_DL_UBIT16_T),
            "udi_sbit16_t" => ::quote::quote_spanned!(field.ty.span() => ::udi::ffi::layout::UDI_DL_SBIT16_T),
            "udi_ubit32_t" => ::quote::quote_spanned!(field.ty.span() => ::udi::ffi::layout::UDI_DL_UBIT32_T),
            "udi_sbit32_t" => ::quote::quote_spanned!(field.ty.span() => ::udi::ffi::layout::UDI_DL_SBIT32_T),
            _ => ::quote::quote_spanned!(field.ty.span() => compile_error!("Unknown type")),
            },
        _ => ::quote::quote_spanned!(field.ty.span() => compile_error!("Unknown type")),
        });
    }
    let name = input.ident;

    let ret = ::quote::quote!{
        unsafe impl udi::layout::GetLayout for #name {
            const LEN: usize = {
                0 #( + < #tys as udi::layout::GetLayout>::LEN )*
            };
            const LAYOUT: &'static [u8] = &{
                let mut rv = [0; Self::LEN];
                let mut ofs = 0;
                #(
                {
                    let mut j = 0;
                    while j < < #tys as udi::layout::GetLayout>::LEN {
                        rv[ofs] = < #tys as udi::layout::GetLayout>::LAYOUT[j];
                        ofs += 1;
                        j += 1;
                    }
                }
                )*
                let _ = ofs;
                rv
            };
        }
    };
    ::proc_macro::TokenStream::from(ret)
}