use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

/// The derive macro to add `agdb` compatibility
/// to user defined types. It implements [`agdb::DbUserValue`]
/// for the type automatically to allow your type to be read and
/// stored from/to the database. If your type contains a field
/// `db_id: Option<agdb::DbId>` it will be treated specially
/// and will additionally allow shorthand inserts/updates
/// of the elements directly.
///
/// # Examples
///
/// ```ignore
/// #[derive(agdb_derive::UserValue)]
/// struct MyValue {
///     db_id: Option<agdb::DbId>, //this field is useful but not mandatory
///     num_value: i64,
///     string_value: String,
///     vec_value: Vec<u64>,
/// }
/// ```
#[proc_macro_derive(UserValue)]
pub fn db_user_value_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;
    let syn::Data::Struct(data) = input.data else {
        unimplemented!()
    };
    let db_id = data
        .fields
        .iter()
        .find_map(|f| {
            if let Some(name) = &f.ident {
                if name == "db_id" {
                    return Some(quote! {
                        self.db_id
                    });
                }
            }

            None
        })
        .unwrap_or(quote! {
            None
        });
    let mut counter: usize = 0;
    let from_db_element = data.fields.iter().filter_map(|f| {
        if let Some(name) = &f.ident {
            if name == "db_id" {
                return Some(quote! {
                    #name: Some(element.id)
                });
            } else {
                let i = counter;
                counter += 1;
                return Some(quote! {
                    #name: element.values.get(#i).ok_or(agdb::DbError::from("Not enough keys"))?.value.clone().try_into()?
                });
            }
        }

        None
    });
    let db_values = data.fields.iter().filter_map(|f| {
        if let Some(name) = &f.ident {
            if name != "db_id" {
                let key = name.to_string();

                return Some(quote! {
                    (#key, self.#name.clone()).into()
                });
            }
        }

        None
    });
    let db_keys = data.fields.iter().filter_map(|f| {
        if let Some(name) = &f.ident {
            if name != "db_id" {
                return Some(name.to_string());
            }
        }
        None
    });
    let tokens = quote! {
        impl agdb::DbUserValue for #name {
            fn db_id(&self) -> Option<agdb::DbId> {
                #db_id
            }

            fn db_keys() -> Vec<agdb::DbValue> {
                vec![#(#db_keys.into()),*]
            }

            fn from_db_element(element: &agdb::DbElement) -> Result<Self, agdb::DbError> {
                Ok(Self {
                    #(#from_db_element),*
                })
            }

            fn to_db_values(&self) -> Vec<agdb::DbKeyValue> {
                vec![#(#db_values),*]
            }
        }

        impl TryFrom<&agdb::DbElement> for #name {
            type Error = agdb::DbError;

            fn try_from(value: &agdb::DbElement) -> Result<Self, Self::Error> {
                use agdb::DbUserValue;
                #name::from_db_element(value)
            }
        }

        impl TryFrom<agdb::QueryResult> for #name {
            type Error = agdb::DbError;

            fn try_from(value: agdb::QueryResult) -> Result<Self, Self::Error> {
                use agdb::DbUserValue;
                value
                    .elements
                    .first()
                    .ok_or(Self::Error::from("No element found"))?
                    .try_into()
            }
        }
    };

    tokens.into()
}
