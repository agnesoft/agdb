use crate::type_def_parser::generics_parser;
use crate::type_def_parser::generics_parser::Generic;
use crate::type_def_parser::impl_parser;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Fields;
use syn::FieldsNamed;

pub(crate) fn parse_enum(input: &DeriveInput, e: &DataEnum) -> TokenStream2 {
    let name = &input.ident;
    let current_generics = generics_parser::extract_generics(&input.generics);
    let generics = generics_parser::parse_generics(&input.generics);
    let variants = parse_variants(&e.variants, &current_generics);
    let (impl_generics, ty_generic, where_clause) = input.generics.split_for_impl();

    let type_def_attrs = impl_parser::parse_type_def_attrs(&input.attrs);
    let impl_defs_method = impl_parser::generate_impl_defs_method(
        &type_def_attrs.impl_names,
        &type_def_attrs.from_types,
        &name.to_string(),
        &input.generics,
    );

    quote! {
        impl #impl_generics ::agdb::type_def::TypeDefinition for #name #ty_generic #where_clause {
            fn type_def() -> ::agdb::type_def::Type {
                ::agdb::type_def::Type::Enum(::agdb::type_def::Enum {
                    name: stringify!(#name),
                    generics: &[#(#generics),*],
                    variants: &[#(#variants),*],
                    impl_defs: Self::impl_defs,
                })
            }
            #impl_defs_method
        }
    }
}

fn parse_variants(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    generics: &[Generic],
) -> Vec<TokenStream2> {
    variants
        .iter()
        .map(|v| {
            let name = &v.ident;
            let ty = parse_variant_fields(&v.fields, generics);
            quote! {
                ::agdb::type_def::Variable {
                    name: stringify!(#name),
                    ty: Some(#ty),
                }
            }
        })
        .collect::<Vec<_>>()
}

fn parse_variant_fields(fields: &Fields, generics: &[Generic]) -> TokenStream2 {
    match fields {
        Fields::Named(fields_named) => parse_named_fields(fields_named, generics),
        Fields::Unnamed(fields_unnamed) => parse_unnamed_fields(fields_unnamed, generics),
        Fields::Unit => {
            quote! {
                <() as ::agdb::type_def::TypeDefinition>::type_def
            }
        }
    }
}

fn parse_unnamed_fields(fields: &syn::FieldsUnnamed, generics: &[Generic]) -> TokenStream2 {
    if fields.unnamed.len() == 1 {
        let ty = fields.unnamed.first().map(|f| &f.ty).unwrap();
        generics_parser::parse_type(ty, generics)
    } else {
        let field_types = fields
            .unnamed
            .iter()
            .map(|f| generics_parser::parse_type(&f.ty, generics))
            .collect::<Vec<_>>();

        quote! {
            || ::agdb::type_def::Type::Tuple(&[#(#field_types),*])
        }
    }
}

fn parse_named_fields(fields: &FieldsNamed, generics: &[Generic]) -> TokenStream2 {
    let fields = fields
        .named
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            let ty_def = generics_parser::parse_type(&f.ty, generics);
            quote! {
                ::agdb::type_def::Variable {
                    name: stringify!(#name),
                    ty: Some(#ty_def),
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        || ::agdb::type_def::Type::Struct(::agdb::type_def::Struct {
            name: "",
            generics: &[],
            fields: &[#(#fields),*],
            impl_defs: ::std::vec::Vec::new,
        })
    }
}
