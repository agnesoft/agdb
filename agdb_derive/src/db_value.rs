use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

pub fn user_db_value_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;

    quote! {
        impl From<#name> for ::agdb::DbValue {
            fn from(value: #name) -> Self {
                use ::agdb::AgdbSerialize;
                ::agdb::DbValue::Bytes(value.serialize())
            }
        }

        impl TryFrom<::agdb::DbValue> for #name {
            type Error = ::agdb::DbError;

            fn try_from(value: ::agdb::DbValue) -> Result<Self, Self::Error> {
                <#name as ::agdb::AgdbSerialize>::deserialize(value.bytes()?)
            }
        }
    }
    .into()
}
