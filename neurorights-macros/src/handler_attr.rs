use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn};

pub fn expand(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let name = &sig.ident;

    let wrapped_name = format_ident!("{}_inner", name);

    // Original logic moved into `_inner`.
    let gen = quote! {
        #vis fn #wrapped_name(
            bound: ::neurorights_core::NeurorightsBound<
                crate::PromptEnvelope,
                ::neurorights_core::NeurorightsEnvelope
            >
        ) #block

        #vis fn #name(
            bound: ::neurorights_core::NeurorightsBound<
                crate::PromptEnvelope,
                ::neurorights_core::NeurorightsEnvelope
            >
        ) {
            #wrapped_name(bound)
        }
    };

    gen.into()
}
