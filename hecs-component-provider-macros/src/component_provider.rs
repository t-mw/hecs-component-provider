use itertools::izip;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Error, Ident, Member, PathArguments, Result, Type, TypeReference};
use unzip_n::unzip_n;

unzip_n!(3);

pub(crate) fn derive(input: DeriveInput) -> Result<TokenStream2> {
    let stream_refs = derive_refs(input.clone())?;
    let stream_muts = derive_muts(input.clone())?;
    let stream_option_refs = derive_option_refs(input.clone())?;
    let stream_option_muts = derive_option_muts(input)?;

    Ok(stream_refs
        .into_iter()
        .chain(stream_muts)
        .chain(stream_option_refs)
        .chain(stream_option_muts)
        .collect::<TokenStream2>())
}

fn derive_refs(input: DeriveInput) -> Result<TokenStream2> {
    let InputDecomposition {
        ident,
        fields,
        types,
        ref_types,
        struct_type,
        ..
    } = decompose_derive_input(input)?;

    let tokens = match struct_type {
        StructType::Bundle => quote! {
            #(
                impl ::hecs_component_provider::ComponentProvider<#types> for #ident {
                    fn get(&self) -> &#types {
                        &self.#fields
                    }
                }
            )*
        },
        StructType::Query => {
            let (fields, types, ref_types): (Vec<_>, Vec<_>, Vec<_>) =
                izip!(fields.into_iter(), types.into_iter(), ref_types.into_iter())
                    .filter_map(|(f, t, p)| {
                        Some((f, remove_type_lifetime(&remove_type_mutability(&t)), p?))
                    })
                    .unzip_n();
            quote! {
                        #(
                            impl<'a> ::hecs_component_provider::ComponentProvider<#ref_types> for #ident<'a> {
                                fn get(&self) -> #types {
                                    self.#fields
                                }
                            }
                        )*
            }
        }
    };

    Ok(tokens)
}

fn derive_muts(input: DeriveInput) -> Result<TokenStream2> {
    let InputDecomposition {
        ident,
        fields,
        types,
        ref_types,
        struct_type,
        ..
    } = decompose_derive_input(input)?;

    let tokens = match struct_type {
        StructType::Bundle => quote! {
            #(
                impl ::hecs_component_provider::ComponentProviderMut<#types> for #ident {
                    fn get_mut(&mut self) -> &mut #types {
                        &mut self.#fields
                    }
                }
            )*
        },
        StructType::Query => {
            let (fields, types, ref_types): (Vec<_>, Vec<_>, Vec<_>) =
                izip!(fields.into_iter(), types.into_iter(), ref_types.into_iter())
                    .filter_map(|(f, t, p)| {
                        if is_mutable_type_ref(&t) {
                            Some((f, remove_type_lifetime(&t), p?))
                        } else {
                            None
                        }
                    })
                    .unzip_n();
            quote! {
                        #(
                            impl<'a> ::hecs_component_provider::ComponentProviderMut<#ref_types> for #ident<'a> {
                                fn get_mut(&mut self) -> #types {
                                    self.#fields
                                }
                            }
                        )*
            }
        }
    };

    Ok(tokens)
}

fn derive_option_refs(input: DeriveInput) -> Result<TokenStream2> {
    let InputDecomposition {
        ident,
        fields,
        types,
        option_types,
        struct_type,
        ..
    } = decompose_derive_input(input)?;

    let tokens = match struct_type {
        StructType::Bundle => quote! {},
        StructType::Query => {
            let (fields, types, option_types): (Vec<_>, Vec<_>, Vec<_>) = izip!(
                fields.into_iter(),
                types.into_iter(),
                option_types.into_iter()
            )
            .filter_map(|(f, t, p)| {
                Some((f, remove_type_lifetime(&remove_type_mutability(&t)), p?))
            })
            .unzip_n();
            quote! {
                    #(
                        impl<'a> ::hecs_component_provider::ComponentProviderOptional<#option_types> for #ident<'a> {
                            fn get_optional(&self) -> #types {
                                // convert Option<&mut T> to Option<&T>
                                if let Some(v) = &self.#fields {
                                    Some(&*v)
                                } else {
                                    None
                                }
                            }
                        }
                    )*
            }
        }
    };

    Ok(tokens)
}

fn derive_option_muts(input: DeriveInput) -> Result<TokenStream2> {
    let InputDecomposition {
        ident,
        fields,
        types,
        option_types,
        struct_type,
        ..
    } = decompose_derive_input(input)?;

    let tokens = match struct_type {
        StructType::Bundle => quote! {},
        StructType::Query => {
            let (fields, types, option_types): (Vec<_>, Vec<_>, Vec<_>) = izip!(
                fields.into_iter(),
                types.into_iter(),
                option_types.into_iter()
            )
            .filter_map(|(f, t, p)| {
                if is_mutable_type_ref(&t) {
                    Some((f, remove_type_lifetime(&t), p?))
                } else {
                    None
                }
            })
            .unzip_n();
            quote! {
                        #(
                            impl<'a> ::hecs_component_provider::ComponentProviderOptionalMut<#option_types> for #ident<'a> {
                                fn get_optional_mut(&mut self) -> #types {
                                    // fix Copy error when returning self.#fields directly
                                    if let Some(v) = &mut self.#fields {
                                        Some(&mut *v)
                                    } else {
                                        None
                                    }
                                }
                            }
                        )*
            }
        }
    };

    Ok(tokens)
}

struct InputDecomposition {
    ident: Ident,
    fields: Vec<Member>,
    types: Vec<Type>,
    ref_types: Vec<Option<Type>>,
    option_types: Vec<Option<Type>>,
    struct_type: StructType,
}

enum StructType {
    Bundle,
    Query,
}

fn decompose_derive_input(input: DeriveInput) -> Result<InputDecomposition> {
    let ident = input.ident;
    let data = match input.data {
        syn::Data::Struct(s) => s,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "derive(ComponentProvider) may only be applied to structs",
            ))
        }
    };

    let lifetimes: Vec<_> = input.generics.lifetimes().cloned().collect();
    if lifetimes.len() > 1 {
        return Err(Error::new_spanned(
            input.generics,
            "must have <= 1 lifetime parameter",
        ));
    };

    if input.generics.params.len() != lifetimes.len() {
        return Err(Error::new_spanned(ident, "must have no type parameters"));
    }

    let (fields, types) = match data.fields {
        syn::Fields::Named(ref fields) => fields
            .named
            .iter()
            .map(|f| (Member::Named(f.ident.clone().unwrap()), f.ty.clone()))
            .unzip(),
        syn::Fields::Unnamed(ref fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                (
                    Member::Unnamed(syn::Index {
                        index: i as u32,
                        span: Span::call_site(),
                    }),
                    f.ty.clone(),
                )
            })
            .unzip(),
        syn::Fields::Unit => (Vec::new(), Vec::new()),
    };

    let ref_types: Vec<_> = types.iter().map(extract_ref_type).collect();
    let option_types: Vec<_> = types.iter().map(extract_option_type).collect();

    Ok(InputDecomposition {
        ident,
        fields,
        types,
        ref_types,
        option_types,
        struct_type: if lifetimes.len() == 0 {
            StructType::Bundle
        } else {
            StructType::Query
        },
    })
}

fn extract_ref_type(t: &Type) -> Option<Type> {
    match t {
        Type::Reference(type_reference) => Some(type_reference.elem.as_ref().clone()),
        _ => None,
    }
}

fn extract_option_type(t: &Type) -> Option<Type> {
    if let Type::Path(type_path) = t {
        let segment = type_path.path.segments.first()?;
        if segment.ident == "Option" {
            if let PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args, ..
            }) = &segment.arguments
            {
                if let Some(syn::GenericArgument::Type(t)) = args.first() {
                    return extract_ref_type(t);
                }
            }
        }
    }
    None
}

fn is_mutable_type_ref(ty: &Type) -> bool {
    struct Visitor {
        is_mut: bool,
    }
    impl syn::visit::Visit<'_> for Visitor {
        fn visit_type_reference(&mut self, l: &TypeReference) {
            if l.mutability.is_some() {
                self.is_mut = true;
            }
        }
    }

    let mut visitor = Visitor { is_mut: false };
    syn::visit::visit_type(&mut visitor, &ty);
    visitor.is_mut
}

fn remove_type_mutability(ty: &Type) -> Type {
    struct Visitor;
    impl syn::visit_mut::VisitMut for Visitor {
        fn visit_type_reference_mut(&mut self, l: &mut TypeReference) {
            l.mutability = None;
        }
    }

    let mut ty = ty.clone();
    syn::visit_mut::visit_type_mut(&mut Visitor, &mut ty);
    ty
}

fn remove_type_lifetime(ty: &Type) -> Type {
    struct Visitor;
    impl syn::visit_mut::VisitMut for Visitor {
        fn visit_type_reference_mut(&mut self, l: &mut TypeReference) {
            l.lifetime = None;
        }
    }

    let mut ty = ty.clone();
    syn::visit_mut::visit_type_mut(&mut Visitor, &mut ty);
    ty
}
