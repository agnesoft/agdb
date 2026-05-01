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
    match extract_type_param_bound(bound) {
        Ok(name) => quote! {
            || ::agdb::type_def::Type::Trait(::agdb::type_def::Trait {
                name: #name,
                generics: &[],
                bounds: &[],
                functions: &[],
            })
        },
        Err(e) => e,
    }
}

fn extract_type_param_bound(bound: &TypeParamBound) -> Result<String, TokenStream2> {
    match bound {
        TypeParamBound::Trait(trait_bound) => {
            Ok(trait_bound.path.segments.last().unwrap().ident.to_string())
        }
        _ => Err(crate::compile_error(
            proc_macro2::TokenStream::new(),
            "Only trait bounds are supported",
        )),
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
    } else if let syn::Type::Tuple(tuple) = ty {
        let fields = tuple
            .elems
            .iter()
            .map(|elem| parse_type(elem, generics))
            .collect::<Vec<_>>();
        quote! {
            || ::agdb::type_def::Type::Tuple(&[#(#fields),*])
        }
    } else if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_none()
            && let Some(last) = type_path.path.segments.last()
        {
            let ident = last.ident.to_string();

            if ident == "Option"
                && let syn::PathArguments::AngleBracketed(args) = &last.arguments
                && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
            {
                let inner = parse_type(inner_ty, generics);
                return quote! {
                    || ::agdb::type_def::Type::Option(#inner)
                };
            }

            if ident == "Vec"
                && let syn::PathArguments::AngleBracketed(args) = &last.arguments
                && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
            {
                let inner = parse_type(inner_ty, generics);
                return quote! {
                    || ::agdb::type_def::Type::Vec(#inner)
                };
            }

            if ident == "Box"
                && let syn::PathArguments::AngleBracketed(args) = &last.arguments
                && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
            {
                let inner = parse_type(inner_ty, generics);
                return quote! {
                    || ::agdb::type_def::Type::Pointer(::agdb::type_def::Pointer {
                        kind: ::agdb::type_def::PointerKind::Box,
                        ty: #inner,
                    })
                };
            }

            if ident == "Result"
                && let syn::PathArguments::AngleBracketed(args) = &last.arguments
            {
                let mut type_args = args.args.iter().filter_map(|arg| {
                    if let syn::GenericArgument::Type(ty) = arg {
                        Some(ty)
                    } else {
                        None
                    }
                });

                if let (Some(ok_ty), Some(err_ty)) = (type_args.next(), type_args.next()) {
                    let ok = parse_type(ok_ty, generics);
                    let err = parse_type(err_ty, generics);
                    return quote! {
                        || ::agdb::type_def::Type::Result {
                            ok: #ok,
                            err: #err,
                        }
                    };
                }
            }
        }

        if type_contains_generic(ty, generics) {
            quote! {
                || ::agdb::type_def::Type::Generic(::agdb::type_def::Generic {
                    kind: ::agdb::type_def::GenericKind::Type,
                    name: stringify!(#ty),
                    bounds: &[],
                })
            }
        } else if let syn::Type::Path(type_path) = ty
            && type_path.qself.is_none()
            && type_path.path.is_ident("Self")
        {
            quote! { || ::agdb::type_def::Type::SelfType(false) }
        } else {
            quote! { <#ty as ::agdb::type_def::TypeDefinition>::type_def }
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

fn type_contains_generic(ty: &syn::Type, generics: &[Generic]) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(ident_str) = type_path.path.segments.last().map(|s| s.ident.to_string())
                && generics.iter().any(|g| g.name == ident_str)
            {
                return true;
            }

            for seg in &type_path.path.segments {
                if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                    for arg in &ab.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg
                            && type_contains_generic(inner_ty, generics)
                        {
                            return true;
                        }
                    }
                }
            }

            false
        }
        syn::Type::Reference(tr) => type_contains_generic(&tr.elem, generics),
        syn::Type::Slice(ts) => type_contains_generic(&ts.elem, generics),
        syn::Type::Array(ta) => type_contains_generic(&ta.elem, generics),
        syn::Type::Tuple(tt) => tt.elems.iter().any(|e| type_contains_generic(e, generics)),
        syn::Type::Paren(tp) => type_contains_generic(&tp.elem, generics),
        syn::Type::Group(tg) => type_contains_generic(&tg.elem, generics),
        _ => false,
    }
}
