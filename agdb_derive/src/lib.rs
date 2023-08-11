use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

#[proc_macro_derive(UserValue)]
#[allow(clippy::manual_map)]
pub fn db_user_value_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;
    let syn::Data::Struct(data) = input.data else { unimplemented!() };
    let mut counter: usize = 0;
    let from_db_fields = data.fields.iter().filter_map(|f| {
        if let Some(name) = &f.ident {
            let i = counter;
            counter += 1;
            return Some(quote! {
                #name: values[#i].value.clone().try_into()?
            });
        }

        None
    });
    let db_values = data.fields.iter().filter_map(|f| {
        if let Some(name) = &f.ident {
            let key = name.to_string();

            return Some(quote! {
                (#key, self.#name.clone()).into()
            });
        }

        None
    });
    let db_keys = data.fields.iter().filter_map(|f| {
        if let Some(name) = &f.ident {
            Some(name.to_string())
        } else {
            None
        }
    });
    let tokens = quote! {
        impl agdb::DbUserValue for #name {
            fn from_db_values(values: &[agdb::DbKeyValue]) -> Result<Self, agdb::DbError> {
                Ok(Self {
                    #(#from_db_fields),*
                })
            }

            fn db_values(&self) -> Vec<agdb::DbKeyValue> {
                vec![#(#db_values),*]
            }

            fn db_keys() -> Vec<agdb::DbKey> {
                vec![#(#db_keys.into()),*]
            }
        }

        impl TryFrom<agdb::QueryResult> for #name {
            type Error = agdb::DbError;

            fn try_from(value: agdb::QueryResult) -> Result<Self, Self::Error> {
                #name::from_db_values(
                    &value
                        .elements
                        .get(0)
                        .ok_or(Self::Error::from("No element found"))?
                        .values,
                )
            }
        }
    };

    tokens.into()
}
