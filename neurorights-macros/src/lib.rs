#![forbid(unsafe_code)]

mod handler_attr;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn neurorights_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    handler_attr::expand(attr, item)
}
