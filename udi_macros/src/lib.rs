/*
Challenge: nested structures
*/

/// Call the `udi_debug_printf` function without needing unsafe
#[proc_macro]
pub fn debug_printf(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream
{
    fn parse_format_args(input: &::syn::LitStr) -> ::syn::Result< (Vec<::syn::Type>, bool) > {
        fn nextc(input: &::syn::LitStr, it: &mut ::core::str::Chars<'_>) -> ::syn::Result<char> {
            it.next().ok_or(::syn::Error::new(input.span(), "Unexpected end of format string"))
        }
        let mut rv = Vec::new();
        let mut require_unsafe = false;
        let input_s = input.value();
        let mut it = input_s.chars();
        while let Some(c) = it.next()
        {
            if c != '%' {
                continue;
            }

            let mut c = nextc(input, &mut it)?;
            if c == '%' {
                // Literal `%` - No argument type
                continue ;
            }

            if c == '0' {
                // Leading zero pad requested
                c = nextc(input, &mut it)?;
            }
            else if c == '-' {
                // Left pad, not allowed with `0`
                c = nextc(input, &mut it)?;
            }
            else {
            }

            // Width
            while let Some(_) = c.to_digit(10) {
                c = nextc(input, &mut it)?;
            }

            rv.push(match c {
                'X'|'x' => ::syn::parse_str("::udi::ffi::udi_ubit32_t").unwrap(),
                'd' => ::syn::parse_str("::udi::ffi::udi_sbit32_t").unwrap(),
                'u' => ::syn::parse_str("::udi::ffi::udi_ubit32_t").unwrap(),
                'h' => match nextc(input, &mut it)?
                    {
                    'X'|'x' => ::syn::parse_str("::udi::ffi::udi_ubit16_t").unwrap(),
                    'd' => ::syn::parse_str("::udi::ffi::udi_sbit16_t").unwrap(),
                    'u' => ::syn::parse_str("::udi::ffi::udi_ubit16_t").unwrap(),
                    _ => return Err(::syn::Error::new(input.span(), format!("Invalid formatting fragment `%h{}`", c))),
                    },
                'b' => match nextc(input, &mut it)?
                    {
                    'X'|'x' => ::syn::parse_str("::udi::ffi::udi_ubit8_t").unwrap(),
                    'd' => ::syn::parse_str("::udi::ffi::udi_sbit8_t").unwrap(),
                    'u' => ::syn::parse_str("::udi::ffi::udi_ubit8_t").unwrap(),
                    _ => return Err(::syn::Error::new(input.span(), format!("Invalid formatting fragment `%b{}`", c))),
                    },
                'p'|'P' => ::syn::parse_str("*const impl Sized").unwrap(),
                'a'|'A' => ::syn::parse_str("::udi::ffi::udi_busaddr64_t").unwrap(),
                'c' => ::syn::parse_str("u8").unwrap(),
                's' => {
                    require_unsafe = true;
                    ::syn::parse_str("*const i8").unwrap()
                    },
                '<' => {
                    while c != '>' {
                        c = nextc(input, &mut it)?;
                    }
                    ::syn::parse_str("::udi::ffi::udi_ubit32_t").unwrap()
                    }
                _ => return Err(::syn::Error::new(input.span(), format!("Invalid formatting fragment `%{}`", c))),
                })
        }
        Ok((rv, require_unsafe))
    }
    struct Input {
        format_string: String,
        require_unsafe: bool,
        exp_arg_tys: Vec<::syn::Type>,
        arg_values: Vec<::syn::Expr>,
    }
    impl ::syn::parse::Parse for Input {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let lit: ::syn::LitStr = input.parse()?;
            let (exp_arg_tys,require_unsafe) = parse_format_args(&lit)?;
            let arg_values = if input.peek(::syn::Token![,]) {
                    let _: ::syn::Token![,] = input.parse()?;
                    ::syn::punctuated::Punctuated::<::syn::Expr, ::syn::Token![,]>::parse_terminated(input)?
                        .into_iter().collect()
                }
                else {
                    vec![]
                };
            Ok(Self {
                format_string: lit.value(),
                require_unsafe,
                exp_arg_tys,
                arg_values,
            })
        }
    }

    let input = ::syn::parse_macro_input!(input as Input);
    let format = &input.format_string;
    let exp_arg_tys = &input.exp_arg_tys;
    let arg_name: Vec<_> = exp_arg_tys.iter().enumerate()
        .map(|(i,ty)| ::syn::Ident::new(format!("arg{}", i).as_str(), ::syn::spanned::Spanned::span(&ty)))
        .collect();
    let input_arg = &input.arg_values;
    let unsafe_tok = if input.require_unsafe { Some( ::syn::Ident::new("unsafe", ::syn::spanned::Spanned::span(&exp_arg_tys[0])) ) } else { None };
    ::proc_macro::TokenStream::from(::quote::quote!{
        {
            #unsafe_tok fn udi_debug_printf( #( #arg_name: #exp_arg_tys ),* ) {
                unsafe {
                    ::udi::ffi::log::udi_debug_printf( concat!(#format,"\0").as_ptr() as _ #(, #arg_name )* );
                }
            }
            udi_debug_printf( #(#input_arg),* );
        }
    })
}

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