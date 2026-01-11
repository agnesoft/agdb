use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

pub fn user_db_value_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::std::convert::From<#name #ty_generics> for ::agdb::DbValue #where_clause {
            fn from(value: #name #ty_generics) -> Self {
                ::agdb::DbValue::Bytes(::agdb::AgdbSerialize::serialize(&value))
            }
        }

        impl #impl_generics ::std::convert::From<&#name #ty_generics> for ::agdb::DbValue #where_clause {
            fn from(value: &#name #ty_generics) -> Self {
                ::agdb::DbValue::Bytes(::agdb::AgdbSerialize::serialize(value))
            }
        }

        impl #impl_generics ::std::convert::TryFrom<::agdb::DbValue> for #name #ty_generics #where_clause {
            type Error = ::agdb::DbError;

            fn try_from(value: ::agdb::DbValue) -> ::std::result::Result<Self, Self::Error> {
                <#name #ty_generics as ::agdb::AgdbSerialize>::deserialize(value.bytes()?)
            }
        }
    }
    .into()
}
