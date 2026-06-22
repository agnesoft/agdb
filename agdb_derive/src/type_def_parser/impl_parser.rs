use crate::type_def_parser::function_parser;
use crate::type_def_parser::generics_parser;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::quote;
use syn::GenericParam;
use syn::ImplItem;

/// Generates a helper free function for an impl block annotated with `#[agdb::impl_def]`.
///
/// - Inherent impl (`impl S { ... }`) → `fn __S_impl_def() -> Impl`
/// - Trait impl (`impl Drop for S { ... }`) → `fn __S_Drop_impl_def() -> Impl`
///
/// The `#[type_def(inherent, Drop)]` attribute on the struct/enum tells the
/// `#[derive(TypeDef)]` macro to call these helpers from `impl_defs()`.
pub(crate) fn parse_impl(input: &syn::ItemImpl) -> TokenStream2 {
    let ty = &input.self_ty;
    let generics = generics_parser::parse_generics(&input.generics);
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();

    let impl_generics_list = generics_parser::extract_generics(&input.generics);
    let methods = input
        .items
        .iter()
        .map(|item| match item {
            ImplItem::Fn(impl_fn) => {
                function_parser::parse_impl_fn(impl_fn, &impl_generics_list)
            }
            _ => crate::compile_error(item, "Only function items are supported in impl blocks"),
        })
        .collect::<Vec<_>>();

    let type_base_name = extract_type_base_name(ty);

    if let Some((_, path, _)) = &input.trait_ {
        // TRAIT IMPL → __TypeName_TraitName_impl_def()
        let trait_name = path.segments.last().unwrap().ident.to_string();
        let helper_fn_name = format_ident!("__{}_{}_impl_def", type_base_name, trait_name);

        let trait_part = quote! {
            Some(|| ::agdb::type_def::Type::Trait(::agdb::type_def::Trait {
                name: #trait_name.to_owned(),
                generics: vec![],
                bounds: vec![],
                functions: vec![],
            }))
        };

        quote! {
            #[allow(non_snake_case)]
            fn #helper_fn_name #impl_generics () -> ::agdb::type_def::Impl #where_clause {
                ::agdb::type_def::Impl {
                    name: stringify!(#ty).to_owned(),
                    generics: vec![#(#generics),*],
                    trait_: #trait_part,
                    ty: <#ty as ::agdb::type_def::TypeDefinition>::type_def,
                    functions: vec![#(#methods),*],
                }
            }
        }
    } else {
        // INHERENT IMPL → __TypeName_impl_def()
        let helper_fn_name = format_ident!("__{}_impl_def", type_base_name);

        quote! {
            #[allow(non_snake_case)]
            fn #helper_fn_name #impl_generics () -> ::agdb::type_def::Impl #where_clause {
                ::agdb::type_def::Impl {
                    name: stringify!(#ty).to_owned(),
                    generics: vec![#(#generics),*],
                    trait_: None,
                    ty: <#ty as ::agdb::type_def::TypeDefinition>::type_def,
                    functions: vec![#(#methods),*],
                }
            }
        }
    }
}

/// Result of parsing `#[type_def(...)]` attributes.
pub(crate) struct TypeDefAttrs {
    /// Impl block names: `"inherent"`, `"Drop"`, etc.
    pub impl_names: Vec<String>,
    /// Types listed in `from(...)`: source types for From impls.
    pub from_types: Vec<syn::Type>,
}

/// Parses `#[type_def(inherent, Drop, from(str, String, i64))]` from item attributes.
pub(crate) fn parse_type_def_attrs(attrs: &[syn::Attribute]) -> TypeDefAttrs {
    let mut result = TypeDefAttrs {
        impl_names: vec![],
        from_types: vec![],
    };

    for attr in attrs {
        if !attr.path().is_ident("type_def") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("from") {
                let content;
                syn::parenthesized!(content in meta.input);
                let types =
                    syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated(
                        &content,
                    )?;
                result.from_types = types.into_iter().collect();
            } else if let Some(ident) = meta.path.get_ident() {
                result.impl_names.push(ident.to_string());
            }
            Ok(())
        });
    }

    result
}

/// Generates the `impl_defs()` override for `impl TypeDefinition`.
/// `impl_names` comes from `parse_type_def_attrs`.
pub(crate) fn generate_impl_defs_method(
    impl_names: &[String],
    from_types: &[syn::Type],
    type_name: &str,
    generics: &syn::Generics,
) -> TokenStream2 {
    if impl_names.is_empty() && from_types.is_empty() {
        return quote! {};
    }

    let ty_params = ty_params_for_call(generics);

    let calls: Vec<TokenStream2> = impl_names
        .iter()
        .map(|name| {
            let fn_name = if name == "inherent" {
                format_ident!("__{}_impl_def", type_name)
            } else {
                format_ident!("__{}_{}_impl_def", type_name, name)
            };
            if ty_params.is_empty() {
                quote! { #fn_name() }
            } else {
                quote! { #fn_name::<#(#ty_params),*>() }
            }
        })
        .collect();

    let from_entries: Vec<TokenStream2> = from_types
        .iter()
        .map(|ty| {
            let ty_str = quote!(#ty).to_string();
            quote! {
                ::agdb::type_def::Impl {
                    name: stringify!(From).to_owned(),
                    generics: vec![],
                    trait_: Some(|| ::agdb::type_def::Type::Trait(::agdb::type_def::Trait {
                        name: "From".to_owned(),
                        generics: vec![::agdb::type_def::Generic {
                            kind: ::agdb::type_def::GenericKind::Type,
                            name: #ty_str.to_owned(),
                            bounds: vec![<#ty as ::agdb::type_def::TypeDefinition>::type_def],
                        }],
                        bounds: vec![],
                        functions: vec![],
                    })),
                    ty: || ::agdb::type_def::Type::Literal(::agdb::type_def::Literal::Unit),
                    functions: vec![],
                }
            }
        })
        .collect();

    if from_entries.is_empty() {
        quote! {
            fn impl_defs() -> ::std::vec::Vec<::agdb::type_def::Impl> {
                ::std::vec![#(#calls),*]
            }
        }
    } else if calls.is_empty() {
        quote! {
            fn impl_defs() -> ::std::vec::Vec<::agdb::type_def::Impl> {
                ::std::vec![#(#from_entries),*]
            }
        }
    } else {
        quote! {
            fn impl_defs() -> ::std::vec::Vec<::agdb::type_def::Impl> {
                let mut v = ::std::vec![#(#calls),*];
                v.extend(::std::vec![#(#from_entries),*]);
                v
            }
        }
    }
}

/// Returns the bare type-parameter tokens needed for a turbofish call.
/// Lifetimes are excluded since they cannot always be given explicitly.
pub(crate) fn ty_params_for_call(generics: &syn::Generics) -> Vec<TokenStream2> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(t) => {
                let ident = &t.ident;
                Some(quote! { #ident })
            }
            GenericParam::Const(c) => {
                let ident = &c.ident;
                Some(quote! { #ident })
            }
            GenericParam::Lifetime(_) => None,
        })
        .collect()
}

fn extract_type_base_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .first()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
        _ => "Unknown".to_string(),
    }
}
