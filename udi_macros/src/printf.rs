
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
            fn udi_snprintf( buf: &mut [u8], #( #arg_name: #exp_arg_tys ),* ) {
                unsafe {
                    ::udi::ffi::libc::udi_snprintf( buf.as_ptr(), buf.len(), concat!(#format,"\0").as_ptr() as _ #(, ::udi::libc::SnprintfArg::into_arg(#arg_name) )* );
                }
            }
            udi_snprintf( #buf, #(#input_arg),* );
        }
    })
}

fn parse_format_args(input: &::syn::LitStr) -> ::syn::Result< Vec<::syn::Type> >
{
    fn nextc(input: &::syn::LitStr, it: &mut ::core::str::Chars<'_>) -> ::syn::Result<char> {
        it.next().ok_or(::syn::Error::new(input.span(), "Unexpected end of format string"))
    }
    let mut rv = Vec::new();
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
                ::syn::parse_str("&::core::ffi::CStr").unwrap()
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
    Ok(rv)
}