
pub fn debug_printf(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream
{
    struct Input {
        format_string: String,
        exp_arg_tys: Vec<::syn::Type>,
        arg_values: Vec<::syn::Expr>,
    }
    impl ::syn::parse::Parse for Input {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let lit: ::syn::LitStr = input.parse()?;
            let exp_arg_tys = parse_format_args(&lit)?;
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
    ::proc_macro::TokenStream::from(::quote::quote!{
        {
            fn udi_debug_printf( #( #arg_name: #exp_arg_tys ),* ) {
                if cfg!(miri) {
                    return ;
                }
                unsafe {
                    ::udi::ffi::log::udi_debug_printf( concat!(#format,"\0").as_ptr() as _ #(, ::udi::libc::SnprintfArg::into_arg(#arg_name) )* );
                }
            }
            udi_debug_printf( #(#input_arg),* );
        }
    })
}

pub fn snprintf(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream
{
    struct Input {
        buf: ::syn::Expr,

        format_string: String,
        exp_arg_tys: Vec<::syn::Type>,
        arg_values: Vec<::syn::Expr>,
    }
    impl ::syn::parse::Parse for Input {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let buf = input.parse()?;
            let _: ::syn::Token![,] = input.parse()?;
            let lit: ::syn::LitStr = input.parse()?;
            let exp_arg_tys = parse_format_args(&lit)?;
            let arg_values = if input.peek(::syn::Token![,]) {
                    let _: ::syn::Token![,] = input.parse()?;
                    ::syn::punctuated::Punctuated::<::syn::Expr, ::syn::Token![,]>::parse_terminated(input)?
                        .into_iter().collect()
                }
                else {
                    vec![]
                };
            Ok(Self {
                buf,
                format_string: lit.value(),
                exp_arg_tys,
                arg_values,
            })
        }
    }

    let input = ::syn::parse_macro_input!(input as Input);

    let buf = &input.buf;
    let format = &input.format_string;
    let exp_arg_tys = &input.exp_arg_tys;
    let arg_name: Vec<_> = exp_arg_tys.iter().enumerate()
        .map(|(i,ty)| ::syn::Ident::new(format!("arg{}", i).as_str(), ::syn::spanned::Spanned::span(&ty)))
        .collect();
    let input_arg = &input.arg_values;
    ::proc_macro::TokenStream::from(::quote::quote!{
        {
            fn udi_snprintf( buf: &mut [u8], #( #arg_name: #exp_arg_tys ),* ) -> usize {
                unsafe {
                    ::udi::ffi::libc::udi_snprintf(
                        buf.as_mut_ptr() as *mut ::core::ffi::c_char,
                        buf.len(),
                        concat!(#format,"\0").as_ptr() as _
                        #(, ::udi::libc::SnprintfArg::into_arg(#arg_name) )* )
                }
            }
            udi_snprintf( #buf, #(#input_arg),* )
        }
    })
}

fn parse_format_args(input: &::syn::LitStr) -> ::syn::Result< Vec<::syn::Type> >
{
    fn err(input: &::syn::LitStr, error: ::udi_macro_helpers::printf::Error) -> ::syn::Error {
        ::syn::Error::new(input.span(), error.kind)
    }
    let mut rv = Vec::new();
    let input_s = input.value();
    let mut p = ::udi_macro_helpers::printf::Parser::new(input_s.as_bytes());
    loop {
        let v = match p.next() {
            Ok(None) => break,
            Ok(Some(v)) => v,
            Err(e) => return Err(err(input, e)),
            };
        rv.push(match v {
        udi_macro_helpers::printf::FormatArg::StringData(_) => continue,

        udi_macro_helpers::printf::FormatArg::Pointer(_) => ::syn::parse_str("*const impl Sized").unwrap(),
        udi_macro_helpers::printf::FormatArg::String(_, _) => ::syn::parse_str("&::core::ffi::CStr").unwrap(),
        udi_macro_helpers::printf::FormatArg::BusAddr(_) => ::syn::parse_str("::udi::ffi::udi_busaddr64_t").unwrap(),
        udi_macro_helpers::printf::FormatArg::Char => ::syn::parse_str("u8").unwrap(),
        udi_macro_helpers::printf::FormatArg::Integer(_, _, ty, _) => match ty
            {
            udi_macro_helpers::printf::Size::U32 => ::syn::parse_str("::udi::ffi::udi_ubit32_t").unwrap(),
            udi_macro_helpers::printf::Size::U16 => ::syn::parse_str("::udi::ffi::udi_ubit16_t").unwrap(),
            udi_macro_helpers::printf::Size::U8 => ::syn::parse_str("::udi::ffi::udi_ubit8_t").unwrap(),
            },
        udi_macro_helpers::printf::FormatArg::BitSet(mut bs) => {
            loop {
                match bs.next() {
                Err(e) => return Err(err(input, e)),
                Ok(None) => break,
                Ok(Some(udi_macro_helpers::printf::BitsetEnt::Single(_, _, _))) => {},
                Ok(Some(udi_macro_helpers::printf::BitsetEnt::Range(_, _, _, mut r))) => {
                    loop {
                        match r.next() {
                        Err(e) => return Err(err(input, e)),
                        Ok(None) => break,
                        Ok(Some((_,_))) => {},
                        }
                    }
                },
                }
            }
            ::syn::parse_str("::udi::ffi::udi_ubit32_t").unwrap()
            },
        });
    }
    Ok(rv)
}