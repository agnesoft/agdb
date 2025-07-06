use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

const DB_ID: &str = "db_id";

pub fn db_user_value_marker_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;

    let tokens = quote! {
        impl ::agdb::DbUserValueMarker for #name {}
    };

    tokens.into()
}

pub fn db_user_value_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;
    let syn::Data::Struct(data) = input.data else {
        unimplemented!()
    };
    let has_option = data.fields.iter().any(|f| {
        if let Some(ident) = &f.ident {
            if ident != DB_ID {
                return is_option_type(f);
            }
        }

        false
    });

    let db_id = impl_db_id(&data);
    let mut counter: usize = 0;
    let from_db_element = data
        .fields
        .iter()
        .filter_map(|f| impl_from_db_element(f, &mut counter, has_option));
    let db_values = data
        .fields
        .iter()
        .filter_map(|f| impl_db_values(f, has_option));
    let db_keys = data.fields.iter().filter_map(|f| {
        if !has_option {
            if let Some(name) = &f.ident {
                if name != DB_ID {
                    return Some(name.to_string());
                }
            }
        }
        None
    });

    let tokens = quote! {
        impl ::agdb::DbUserValue for #name {
            type ValueType = #name;

            #[track_caller]
            fn db_id(&self) -> ::std::option::Option<::agdb::QueryId> {
                #db_id
            }

            #[track_caller]
            fn db_keys() -> ::std::vec::Vec<::agdb::DbValue> {
                vec![#(#db_keys.into()),*]
            }

            #[track_caller]
            fn from_db_element(element: &::agdb::DbElement) -> std::result::Result<Self::ValueType, ::agdb::DbError> {
                Ok(Self {
                    #(#from_db_element),*
                })
            }

            #[track_caller]
            fn to_db_values(&self) -> ::std::vec::Vec<::agdb::DbKeyValue> {
                let mut values = ::std::vec::Vec::with_capacity(#counter);
                #(#db_values)*
                values
            }
        }

        impl ::agdb::DbUserValue for &#name {
            type ValueType = #name;

            #[track_caller]
            fn db_id(&self) -> ::std::option::Option<::agdb::QueryId> {
                #name::db_id(*self)
            }

            #[track_caller]
            fn db_keys() -> ::std::vec::Vec<::agdb::DbValue> {
                #name::db_keys()
            }

            #[track_caller]
            fn from_db_element(element: &::agdb::DbElement) -> ::std::result::Result<Self::ValueType, ::agdb::DbError> {
                #name::from_db_element(element)
            }

            #[track_caller]
            fn to_db_values(&self) -> ::std::vec::Vec<::agdb::DbKeyValue> {
                #name::to_db_values(*self)
            }
        }

        impl TryFrom<&::agdb::DbElement> for #name {
            type Error = ::agdb::DbError;

            #[track_caller]
            fn try_from(value: &::agdb::DbElement) -> ::std::result::Result<Self, Self::Error> {
                use ::agdb::DbUserValue;
                #name::from_db_element(value)
            }
        }

        impl TryFrom<agdb::QueryResult> for #name {
            type Error = agdb::DbError;

            #[track_caller]
            fn try_from(value: ::agdb::QueryResult) -> ::std::result::Result<Self, Self::Error> {
                use ::agdb::DbUserValue;
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

fn impl_db_values(f: &syn::Field, has_option: bool) -> Option<proc_macro2::TokenStream> {
    if let Some(name) = &f.ident {
        if name != DB_ID {
            let key = name.to_string();

            if has_option && is_option_type(f) {
                return Some(quote! {
                    if let ::std::option::Option::Some(value) = &self.#name {
                        values.push((#key, value.clone()).into());
                    }
                });
            } else {
                return Some(quote! {
                    values.push((#key, self.#name.clone()).into());
                });
            }
        }
    }

    None
}

fn impl_from_db_element(
    f: &syn::Field,
    counter: &mut usize,
    has_option: bool,
) -> Option<proc_macro2::TokenStream> {
    if let Some(name) = &f.ident {
        if name == DB_ID {
            return Some(quote! {
                #name: ::std::option::Option::Some(element.id.into())
            });
        } else if has_option {
            let str_name = name.to_string();
            if is_option_type(f) {
                return Some(quote! {
                    #name: element.values.iter().find_map(|kv| {
                            if let ::std::result::Result::Ok(key) = kv.key.string() {
                                if key == #str_name { return ::std::option::Option::Some(kv.value.clone().try_into());
                            }
                        }
                        ::std::option::Option::None })
                            .map_or_else(|| ::std::result::Result::Ok(::std::option::Option::None), |v| {
                                if let ::std::result::Result::Ok(v) = v {
                                    ::std::result::Result::Ok(::std::option::Option::Some(v))
                                } else {
                                    ::std::result::Result::Err(::agdb::DbError::from(format!("Failed to convert value of '{}': {}", #str_name, v.unwrap_err())))
                                }
                            })?
                });
            } else {
                return Some(quote! {
                    #name: element.values.iter().find_map(|kv| { if let ::std::result::Result::Ok(key) = kv.key.string() {
                            if key == #str_name {
                                return ::std::option::Option::Some(kv.value.clone().try_into());
                            }
                        } ::std::option::Option::None
                    })
                    .ok_or(::agdb::DbError::from(format!("Key '{}' not found", #str_name)))?
                    .map_err(|e| ::agdb::DbError::from(format!("Failed to convert value of '{}': {}", #str_name, e)))?
                });
            }
        } else {
            let str_name = name.to_string();
            let i = *counter;
            *counter += 1;
            return Some(quote! {
                #name: element.values.get(#i)
                    .ok_or(::agdb::DbError::from(format!("Not enough keys: '{}' not found at position {}", #str_name, #i)))?
                        .value.clone().try_into().map_err(|e| ::agdb::DbError::from(format!("Failed to convert value of '{}': {}", #str_name, e)))?
            });
        }
    }

    None
}

fn impl_db_id(data: &syn::DataStruct) -> proc_macro2::TokenStream {
    data.fields
        .iter()
        .find_map(|f| {
            if let Some(name) = &f.ident {
                if name == DB_ID {
                    return Some(quote! {
                        if let ::std::option::Option::Some(id) = &self.db_id {
                            return ::std::option::Option::Some(id.clone().into());
                        } else {
                            return ::std::option::Option::None;
                        }
                    });
                }
            }

            None
        })
        .unwrap_or(quote! {
            ::std::option::Option::None
        })
}

fn is_option_type(f: &syn::Field) -> bool {
    if let syn::Type::Path(type_path) = &f.ty {
        return type_path.path.segments.iter().any(|seg| {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ref args) = seg.arguments {
                    if let Some(syn::GenericArgument::Type(_)) = args.args.first() {
                        return true;
                    }
                }
            }

            false
        });
    }

    false
}
