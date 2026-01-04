use super::generics;
use super::struct_def;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Fields;
use syn::Ident;
use syn::Token;
use syn::Variant;
use syn::punctuated::Punctuated;

pub(crate) fn parse_enum(e: &DataEnum, input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generic_names = generics::list_generics(&input.generics);
    let generics = generics::parse_generics(name, &input.generics);
    let variants = parse_variants(name, &e.variants, &generic_names);
    let (impl_generics, ty_generic, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::agdb::api_def::TypeDefinition for #name #ty_generic #where_clause {
            fn type_def() -> ::agdb::api_def::Type {
                ::agdb::api_def::Type::Enum(::agdb::api_def::Enum {
                    name: stringify!(#name),
                    generics: &[#(#generics),*],
                    variants: &[#(#variants),*],
                    functions: <#name #ty_generic as ::agdb::api_def::ImplDefinition>::functions(),
                })
            }
        }
    }
}

fn parse_variants(
    name: &Ident,
    variants: &Punctuated<Variant, Token![,]>,
    generics: &[String],
) -> Vec<TokenStream> {
    variants
        .iter()
        .map(|v| {
            let variant_name = &v.ident;
            match &v.fields {
                Fields::Named(fields_named) => {
                    let fields = struct_def::parse_named_fields(name, Some(fields_named), generics);

                    quote! {
                        ::agdb::api_def::NamedType {
                            name: stringify!(#variant_name),
                            ty: Some(|| ::agdb::api_def::Type::Struct(::agdb::api_def::Struct {
                                name: "",
                                generics: &[],
                                fields: &[#(#fields),*],
                                functions: &[],
                            })),
                        }
                    }
                }
                Fields::Unnamed(fields_unnamed) => {
                    let field_name = fields_unnamed.unnamed.first().map(|f| &f.ty).unwrap();
                    quote! {
                        ::agdb::api_def::NamedType {
                            name: stringify!(#variant_name),
                            ty: Some(<#field_name as ::agdb::api_def::TypeDefinition>::type_def),
                        }
                    }
                }
                Fields::Unit => {
                    quote! {
                        ::agdb::api_def::NamedType {
                            name: stringify!(#variant_name),
                            ty: None,
                        }
                    }
                }
            }
        })
        .collect()
}
