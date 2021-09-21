use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemTrait, Result};

pub(crate) fn generate(input: ItemTrait) -> Result<TokenStream2> {
    let ident = &input.ident;
    let supertraits = &input.supertraits;
    Ok(quote! { #input impl<T> #ident for T where T: #supertraits {} })
}
