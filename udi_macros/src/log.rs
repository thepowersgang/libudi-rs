
pub fn trace_write(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream
{
    struct Input {
        context: ::syn::Expr,
        trace_event: ::syn::Expr,
        meta_idx: ::syn::Expr,
        message: ::syn::Type,
        arg_values: Vec<::syn::Expr>,
    }
    impl ::syn::parse::Parse for Input {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let context = input.parse()?;
            let _: ::syn::Token![,] = input.parse()?;
            let trace_event = input.parse()?;
            let _: ::syn::Token![,] = input.parse()?;
            let meta_idx = input.parse()?;
            let _: ::syn::Token![,] = input.parse()?;
            let message = input.parse()?;
            let arg_values = if input.peek(::syn::Token![,]) {
                let _: ::syn::Token![,] = input.parse()?;
                ::syn::punctuated::Punctuated::<::syn::Expr, ::syn::Token![,]>::parse_terminated(input)?
                    .into_iter().collect()
            }
            else {
                vec![]
            };
            Ok(Self {
                context,
                trace_event,
                meta_idx,
                message,
                arg_values,
            })
        }
    }

    let Input { context, trace_event, meta_idx, message, arg_values } = ::syn::parse_macro_input!(input as Input);

    let args_exploded = (0..arg_values.len()).map(|i| ::quote::quote!(args.#i));

    ::proc_macro::TokenStream::from(::quote::quote!{
        unsafe {
            let context: &::udi::init::RData<_> = #context;
            let trace_event: ::udi::log::TraceEvent = #trace_event;
            let meta_idx: ::udi::ffi::udi_index_t = #meta_idx;
            let args: <#message as ::udi::log::Message>::Args = ( #( #arg_values, )* );
            ::udi::ffi::log::udi_trace_write(
                context as *const ::udi::init::RData<_> as *const _,
                trace_event.to_raw(),
                meta_idx,
                <#message as ::udi::log::Message>::IDX,
                #( #args_exploded, )*
            );
        }
    })
}