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
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let ty_str = segment.ident.to_string();

        // If this is a known generic parameter (like T), create a GenericArg with empty args
        if list_generics.contains(&ty_str) {
            return quote! { || ::agdb::api_def::Type::GenericArg(::agdb::api_def::GenericArg {
                name: #ty_str,
                args: &[],
            }) };
        }

        // If it has angle-bracketed arguments (like Vec<i32> or GenericReturn<T>),
        // create a GenericArg with the inner type args
        if let PathArguments::AngleBracketed(ab) = &segment.arguments {
            let args: Vec<TokenStream> = ab
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

            if !args.is_empty() {
                return quote! { || ::agdb::api_def::Type::GenericArg(::agdb::api_def::GenericArg {
                    name: #ty_str,
                    args: &[#(#args),*],
                }) };
            }
        }
    }

    // Check if the type contains any generic parameter in nested positions (like &[T])
    // If so, we need to use the closure form to avoid requiring T: TypeDefinition
    if type_contains_generic(ty, list_generics) {
        let ty_str = quote!(#ty).to_string();
        return quote! { || ::agdb::api_def::Type::GenericArg(::agdb::api_def::GenericArg {
            name: #ty_str,
            args: &[],
        }) };
    }

    // No generic parameters - use TypeDefinition::type_def
    quote! { <#ty as ::agdb::api_def::TypeDefinition>::type_def }
}

fn type_contains_generic(ty: &Type, generics: &[String]) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let ident_str = segment.ident.to_string();
                if generics.contains(&ident_str) {
                    return true;
                }

                if let PathArguments::AngleBracketed(ab) = &segment.arguments {
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
