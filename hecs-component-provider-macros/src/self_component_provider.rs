use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error, Result};

pub(crate) fn derive(input: DeriveInput) -> Result<TokenStream2> {
    let ident = input.ident;
    match input.data {
        syn::Data::Struct(_) => (),
        _ => {
            return Err(Error::new_spanned(
                ident,
                "derive(SelfComponentProvider) may only be applied to structs",
            ))
        }
    };

    let lifetimes: Vec<_> = input
        .generics
        .lifetimes()
        .map(|x| x.lifetime.clone())
        .collect();
    if lifetimes.len() > 0 {
        return Err(Error::new_spanned(
            input.generics,
            "must have no lifetime parameters",
        ));
    };

    if input.generics.params.len() > 0 {
        return Err(Error::new_spanned(ident, "must have no type parameters"));
    }

    Ok(quote! {
        impl ::hecs_component_provider::ComponentProvider<#ident> for #ident {
            fn get(&self) -> &#ident {
                self
            }
        }

        impl ::hecs_component_provider::ComponentProviderMut<#ident> for #ident {
            fn get_mut(&mut self) -> &mut #ident {
                self
            }
        }
    })
}
