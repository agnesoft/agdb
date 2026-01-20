use super::enum_def;
use super::struct_def;
use super::tuple_def;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::Fields;
use syn::GenericArgument;
use syn::PathArguments;
use syn::Type;

fn type_contains_generic(ty: &Type, generics: &[String]) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(ident_str) = type_path.path.segments.last().map(|s| s.ident.to_string())
                && generics.contains(&ident_str)
            {
                return true;
            }

            for seg in &type_path.path.segments {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    for arg in &ab.args {
                        if let GenericArgument::Type(inner_ty) = arg
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

pub(crate) fn type_def(input: DeriveInput) -> TokenStream {
    match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            Fields::Named(fields) => struct_def::parse_struct(Some(fields), &input),
            Fields::Unnamed(fields) => tuple_def::parse_tuple(Some(fields), &input),
            Fields::Unit => struct_def::parse_struct(None, &input),
        },
        syn::Data::Enum(e) => enum_def::parse_enum(e, &input),
        syn::Data::Union(_) => {
            panic!("{}: Union types are not supported", input.ident);
        }
    }
}

pub(crate) fn parse_type(ty: &Type, list_generics: &[String]) -> TokenStream {
    if type_contains_generic(ty, list_generics) {
        let type_args = extract_generic_args(ty, list_generics);
        quote! { || ::agdb::api_def::Type::GenericArg(::agdb::api_def::GenericArg {
            name: stringify!(#ty),
            args: &[#(#type_args),*],
        }) }
    } else {
        quote! { <#ty as ::agdb::api_def::TypeDefinition>::type_def }
    }
}

fn extract_generic_args(ty: &Type, list_generics: &[String]) -> Vec<TokenStream> {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last()
                && let PathArguments::AngleBracketed(ab) = &segment.arguments
            {
                return ab
                    .args
                    .iter()
                    .filter_map(|arg| {
                        if let GenericArgument::Type(inner_ty) = arg {
                            Some(parse_type(inner_ty, list_generics))
                        } else {
                            None
                        }
                    })
                    .collect();
            }
            vec![]
        }
        _ => vec![],
    }
}
