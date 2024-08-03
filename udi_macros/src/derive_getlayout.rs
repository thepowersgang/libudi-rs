use syn::spanned::Spanned as _;

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
    
    let mut field_idents = Vec::new();
    let mut lengths = Vec::new();
    let mut buffers = Vec::new();
    for field in fields {
        enum Attr {
            Ignore,
            Inline,
            InlineDriverTyped,
            Movable,
            ChainCb,
            Buf(BufAttr),
        }
        impl ::syn::parse::Parse for Attr {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let i: ::syn::Ident = input.parse()?;
                Ok(match i.to_string().as_str() {
                "ignore" => Attr::Ignore,
                "inline" => Attr::Inline,
                "inline_driver_typed" => Attr::InlineDriverTyped,
                "movable" => Attr::Movable,
                "chain_cb" => Attr::ChainCb,
                "buf" => {
                    let a;
                    ::syn::parenthesized!(a in input);
                    Attr::Buf(a.parse()?)
                    },
                _ => return Err(::syn::Error::new(i.span(), "Unknown attribute value")),
                })
            }
        }
        struct BufAttr {
            field: ::syn::Ident,
            val_mask: u8,
            val_match: u8,
        }
        impl ::syn::parse::Parse for BufAttr {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let field: ::syn::Ident = input.parse()?;
                let _: ::syn::Token![&] = input.parse()?;
                let val_mask: ::syn::LitInt = input.parse()?;
                let _: ::syn::Token![==] = input.parse()?;
                let val_match: ::syn::LitInt = input.parse()?;
                Ok(BufAttr {
                    field,
                    val_mask: val_mask.base10_parse()?,
                    val_match: val_match.base10_parse()?,
                })
            }
        }
        let a = field.attrs.iter().filter_map(|a| 
            match a.meta {
            syn::Meta::List(ref l) if l.path.is_ident("layout") => Some(l),
            _ => None
            })
            .next();
        let a = match a {
            None => None,
            Some(a) => match a.parse_args::<Attr>()
                {
                Err(e) => {
                    let e = e.to_string();
                    lengths.push(::quote::quote!( 0 ));
                    buffers.push(::quote::quote_spanned!( a.span() => compile_error!( #e ) ));
                    continue;
                },
                Ok(v) => Some(v),
                },
        };

        if let Some(Attr::Ignore) = a {
            assert!(lengths.is_empty(), "Can only ignore the first field");
            continue;
        }
        field_idents.push((field.ident.unwrap(), lengths.len()));
        let mut ty = &field.ty;
        let mut n_arrays = 0;
        while let ::syn::Type::Array(a) = ty {
            let len = &a.len;
            lengths.push(::quote::quote!( 1 ));
            buffers.push(::quote::quote!( [#len] ));
            ty = &*a.elem;
            n_arrays += 1;
        }
        let ovr = match ty
            {
            ::syn::Type::Ptr(p) => match &*p.elem
                {
                // A buffer needs three bytes to indicate where its preserve flag is
                // #[layout_buffer(field_name & 0x01 == 0x01)]
                ::syn::Type::Path(p) if p.path.segments.last().unwrap().ident == "udi_buf_t" => {
                    let (fld_idx, val_mask, val_match) = if let Some(a) = a {
                        let a = match a {
                            Attr::Buf(a) => a,
                            _ => {
                                lengths.push(::quote::quote!( 0 ));
                                buffers.push(::quote::quote_spanned!( ty.span() => compile_error!( "`*mut udi_buf_t` must have `#[layout(buf(...))]`") ));
                                continue ;
                            },
                        };
                        let &(_,idx) = match field_idents.iter().find(|(ident,_)| *ident == a.field)
                            {
                            Some(v) => v,
                            None => panic!("Unable to find field {} for buf flag", a.field),
                            };
                        let lens = &lengths[..idx];
                        (::quote::quote!( 0 #( + #lens)* ), a.val_mask, a.val_match)
                    }
                    else {
                        // Default to non-preserved
                        (::quote::quote!(0),0,1)
                    };
                    buffers.push(::quote::quote!( [udi::ffi::layout::UDI_DL_BUF,#fld_idx,#val_mask,#val_match] ));
                    lengths.push(::quote::quote!( 4 ));
                    true
                },
                // c_void => UDI_DL_INLINE_UNTYPED, UDI_DL_INLINE_DRIVER_TYPED, UDI_DL_MOVABLE_UNTYPED, depending on attribute
                ::syn::Type::Path(p) if p.path.segments.last().unwrap().ident == "c_void" => {
                    match a {
                    None => {
                        lengths.push(::quote::quote!( 0 ));
                        buffers.push(::quote::quote_spanned!( ty.span() => compile_error!( "Require an attribute on *mut c_void") ));
                        continue;
                        },
                    |Some(Attr::Ignore)
                    |Some(Attr::ChainCb)
                    |Some(Attr::Buf(_))
                        => {
                        lengths.push(::quote::quote!( 0 ));
                        buffers.push(::quote::quote_spanned!( ty.span() => compile_error!( "Invalid attribute on *mut c_void") ));
                        continue;
                        },
                    Some(Attr::Inline)
                        => buffers.push(::quote::quote!( [udi::ffi::layout::UDI_DL_INLINE_UNTYPED] )),
                    Some(Attr::InlineDriverTyped)
                        => buffers.push(::quote::quote!( [udi::ffi::layout::UDI_DL_INLINE_DRIVER_TYPED] )),
                    Some(Attr::Movable)
                        => buffers.push(::quote::quote!( [udi::ffi::layout::UDI_DL_MOVABLE_UNTYPED] )),
                    }
                    lengths.push(::quote::quote!( 1 ));
                    true
                    },
                _ if matches!(a, Some(Attr::ChainCb)) => {
                    lengths.push(::quote::quote!( 1 ));
                    buffers.push(::quote::quote!( [ udi::ffi::layout::UDI_DL_CB ] ));
                    true
                },
                // any others => UDI_DL_INLINE_TYPED or UDI_DL_MOVABLE_TYPED
                _ => {
                    match a {
                    None => {
                        lengths.push(::quote::quote!( 0 ));
                        buffers.push(::quote::quote_spanned!( ty.span() => compile_error!( "Require an attribute on *mut T") ));
                        continue;
                        },
                    |Some(Attr::Ignore)
                    |Some(Attr::Buf(_))
                    |Some(Attr::InlineDriverTyped)
                        => {
                        lengths.push(::quote::quote!( 0 ));
                        buffers.push(::quote::quote_spanned!( ty.span() => compile_error!( "Invalid attribute on *mut T") ));
                        continue;
                        },
                    Some(Attr::ChainCb) => unreachable!(),
                    Some(Attr::Inline)
                        => buffers.push(::quote::quote!( [udi::ffi::layout::UDI_DL_INLINE_TYPED] )),
                    Some(Attr::Movable)
                        => buffers.push(::quote::quote!( [udi::ffi::layout::UDI_DL_MOVABLE_TYPED] )),
                    }
                    lengths.push(::quote::quote!( 1 ));

                    lengths.push(::quote::quote!( < #ty as udi::layout::GetLayout>::LEN ));
                    buffers.push(::quote::quote!( < #ty as udi::layout::GetLayout>::LAYOUT ));

                    lengths.push(::quote::quote!( 1 ));
                    buffers.push(::quote::quote!( [udi::ffi::layout::UDI_DL_END] ));
                    true
                },
                },
            _ => false,
            };
        
        if ovr {
            // Already emitted
        }
        else {
            lengths.push(::quote::quote!( < #ty as udi::layout::GetLayout>::LEN ));
            buffers.push(::quote::quote!( < #ty as udi::layout::GetLayout>::LAYOUT ));
        }
        
        for _ in 0 .. n_arrays {
            lengths.push(::quote::quote!( 1 ));
            buffers.push(::quote::quote!( udi::ffi::layout::UDI_DL_END ));
        }
    }
    let name = input.ident;

    let ret = ::quote::quote!{
        unsafe impl udi::layout::GetLayout for #name {
            const LEN: usize = {
                0 #( + #lengths )*
            };
            const LAYOUT: &'static [u8] = &{
                let mut rv = [0; Self::LEN];
                let mut ofs = 0;
                #(
                {
                    let mut j = 0;
                    while j < #lengths {
                        rv[ofs] = (#buffers)[j];
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
