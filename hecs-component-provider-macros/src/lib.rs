use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemTrait};

mod component_provider;
mod default_trait_impl;
mod self_component_provider;

#[proc_macro_derive(ComponentProvider)]
pub fn component_provider_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match component_provider::derive(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_derive(SelfComponentProvider)]
pub fn self_component_provider_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match self_component_provider::derive(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn default_trait_impl(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemTrait);

    match default_trait_impl::generate(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}
