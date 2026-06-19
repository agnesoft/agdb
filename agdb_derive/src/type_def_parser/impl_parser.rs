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

    let methods = input
        .items
        .iter()
        .map(|item| match item {
            ImplItem::Fn(impl_fn) => function_parser::parse_impl_fn(impl_fn),
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

/// Parses `#[type_def(inherent, Drop, Display)]` from item attributes.
/// Returns the list of names. `"inherent"` means the inherent impl block.
pub(crate) fn parse_type_def_impls(attrs: &[syn::Attribute]) -> Vec<String> {
    for attr in attrs {
        if !attr.path().is_ident("type_def") {
            continue;
        }
        if let Ok(names) = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated,
        ) {
            return names.iter().map(|i| i.to_string()).collect();
        }
    }
    vec![]
}

/// Generates the `impl_defs()` override for `impl TypeDefinition`.
/// `impl_names` comes from `parse_type_def_impls`.
pub(crate) fn generate_impl_defs_method(
    impl_names: &[String],
    type_name: &str,
    generics: &syn::Generics,
) -> TokenStream2 {
    if impl_names.is_empty() {
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

    quote! {
        fn impl_defs() -> ::std::vec::Vec<::agdb::type_def::Impl> {
            ::std::vec![#(#calls),*]
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
