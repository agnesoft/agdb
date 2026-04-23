use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use quote::quote;
use syn::GenericParam;
use syn::Generics;
use syn::TypeParamBound;

#[derive(Clone)]
pub(crate) struct Generic {
    pub name: String,
    pub bounds: Vec<TokenStream2>,
}

pub(crate) fn extract_generics(generics: &Generics) -> Vec<Generic> {
    let mut extracted: Vec<Generic> = Vec::new();

    if let Some(where_clause) = &generics.where_clause {
        where_clause.predicates.iter().for_each(|predicate| {
            if let syn::WherePredicate::Type(pred) = predicate {
                extracted.push(Generic {
                    name: pred.bounded_ty.to_token_stream().to_string(),
                    bounds: pred.bounds.iter().map(parse_type_param_bound).collect(),
                });
            }
        });
    }

    generics.params.iter().for_each(|param| {
        if let GenericParam::Type(ty) = param {
            let name = ty.ident.to_string();
            if !extracted.iter().any(|g| g.name == name) {
                extracted.push(Generic {
                    name,
                    bounds: ty.bounds.iter().map(parse_type_param_bound).collect(),
                });
            }
        }
    });

    extracted
}

pub(crate) fn parse_generics(generics: &Generics) -> Vec<TokenStream2> {
    let extracted = extract_generics(generics);

    generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Lifetime(lt) => {
                let name = lt.lifetime.ident.to_string();
                quote! {
                    ::agdb::type_def::Generic {
                        kind: ::agdb::type_def::GenericKind::Lifetime,
                        name: #name,
                        bounds: &[],
                    }
                }
            }
            GenericParam::Type(ty) => {
                let name = ty.ident.to_string();
                let bounds = extracted
                    .iter()
                    .find(|g| g.name == name)
                    .map(|g| g.bounds.clone())
                    .unwrap_or_default();
                quote! {
                    ::agdb::type_def::Generic {
                        kind: ::agdb::type_def::GenericKind::Type,
                        name: #name,
                        bounds: &[#(#bounds),*],
                    }
                }
            }
            GenericParam::Const(const_param) => {
                let name = const_param.ident.to_string();
                let ty = &const_param.ty;
                quote! {
                    ::agdb::type_def::Generic {
                        kind: ::agdb::type_def::GenericKind::Const,
                        name: #name,
                        bounds: &[<#ty as ::agdb::type_def::TypeDefinition>::type_def],
                    }
                }
            }
        })
        .collect()
}

pub(crate) fn parse_type_param_bound(bound: &TypeParamBound) -> TokenStream2 {
    let name = extract_type_param_bound(bound);

    quote! {
        || ::agdb::type_def::Type::Trait(::agdb::type_def::Trait {
            name: #name,
            generics: &[],
            bounds: &[],
            functions: &[],
        })
    }
}

fn extract_type_param_bound(bound: &TypeParamBound) -> String {
    match bound {
        TypeParamBound::Trait(trait_bound) => {
            trait_bound.path.segments.last().unwrap().ident.to_string()
        }
        _ => unimplemented!("Only trait bounds are supported for now"),
    }
}

pub(crate) fn parse_lifetime_params(generics: &Generics) -> Vec<&syn::LifetimeParam> {
    generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Lifetime(lt) = param {
                Some(lt)
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn parse_type(ty: &syn::Type, generics: &[Generic]) -> TokenStream2 {
    let ty_str = quote! { #ty }.to_string();

    if let Some(generic) = generics.iter().find(|g| g.name == ty_str) {
        let bounds = &generic.bounds;
        quote! {
            || ::agdb::type_def::Type::Generic(::agdb::type_def::Generic {
                kind: ::agdb::type_def::GenericKind::Type,
                name: stringify!(#ty),
                bounds: &[#(#bounds),*],
            })
        }
    } else if let syn::Type::Reference(type_ref) = ty {
        let mutable = type_ref.mutability.is_some();
        let lifetime = if let Some(lt) = &type_ref.lifetime {
            let lt_str = lt.ident.to_string();
            quote! { Some(#lt_str) }
        } else {
            quote! { None }
        };
        let inner = match type_ref.elem.as_ref() {
            syn::Type::Path(p) if p.path.is_ident("str") => {
                quote! { || ::agdb::type_def::Type::Literal(::agdb::type_def::Literal::Str) }
            }
            syn::Type::Slice(slice) => {
                let elem = parse_type(&slice.elem, generics);
                quote! { || ::agdb::type_def::Type::Slice(#elem) }
            }
            other => parse_type(other, generics),
        };
        quote! {
            || ::agdb::type_def::Type::Reference(::agdb::type_def::Reference {
                mutable: #mutable,
                lifetime: #lifetime,
                ty: #inner,
            })
        }
    } else {
        quote! { <#ty as ::agdb::type_def::TypeDefinition>::type_def }
    }
}
