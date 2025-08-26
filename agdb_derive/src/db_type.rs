use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

const DB_ID: &str = "db_id";
const AGDB: &str = "agdb";
const FLATTEN: &str = "flatten";
const SKIP: &str = "skip";
const RENAME: &str = "rename";

pub fn db_type_marker_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;

    let tokens = quote! {
        impl ::agdb::DbTypeMarker for #name {}
    };

    tokens.into()
}

pub fn db_type_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;
    let syn::Data::Struct(data) = input.data else {
        unimplemented!()
    };
    let has_option = data.fields.iter().any(|f| {
        if let Some(ident) = &f.ident
            && ident != DB_ID
        {
            return is_option_type(f);
        }

        false
    });

    let db_id = impl_db_id(&data);
    let from_db_element = data.fields.iter().filter_map(impl_from_db_element);
    let db_values = to_db_values(&data);
    let keys = if has_option {
        quote! { Vec::new() }
    } else {
        db_keys(&data)
    };

    let tokens = quote! {
        impl ::agdb::DbType for #name {
            type ValueType = #name;

            #[track_caller]
            fn db_id(&self) -> ::std::option::Option<::agdb::QueryId> {
                #db_id
            }

            #[track_caller]
            fn db_keys() -> ::std::vec::Vec<::agdb::DbValue> {
                #keys
            }

            #[track_caller]
            fn from_db_element(element: &::agdb::DbElement) -> std::result::Result<Self::ValueType, ::agdb::DbError> {
                Ok(Self {
                    #(#from_db_element),*
                })
            }

            #[track_caller]
            fn to_db_values(&self) -> ::std::vec::Vec<::agdb::DbKeyValue> {
                #db_values
            }
        }

        impl ::agdb::DbType for &#name {
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
                use ::agdb::DbType;
                #name::from_db_element(value)
            }
        }

        impl TryFrom<agdb::QueryResult> for #name {
            type Error = agdb::DbError;

            #[track_caller]
            fn try_from(value: ::agdb::QueryResult) -> ::std::result::Result<Self, Self::Error> {
                use ::agdb::DbType;
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

fn to_db_values(data: &syn::DataStruct) -> proc_macro2::TokenStream {
    let fields = data.fields.iter().filter_map(impl_to_db_value);

    quote! {
        let mut values = vec![];
        #(#fields)*
        values
    }
}

fn db_keys(data: &syn::DataStruct) -> proc_macro2::TokenStream {
    let keys = data.fields.iter().filter_map(impl_db_key);

    let q = quote! {
        let mut keys = vec![];
        #(#keys)*
        keys
    };

    q
}

fn impl_db_key(f: &syn::Field) -> Option<proc_macro2::TokenStream> {
    if let Some(name) = &f.ident
        && name != DB_ID
        && !is_skip_type(f)
    {
        if is_flatten_type(f) {
            let ty = &f.ty;
            return Some(quote! {
                keys.extend(#ty::db_keys());
            });
        }

        let name_str = field_name(f);
        return Some(quote! {
            keys.push(#name_str.into());
        });
    }

    None
}

fn impl_to_db_value(f: &syn::Field) -> Option<proc_macro2::TokenStream> {
    if let Some(name) = &f.ident
        && name != DB_ID
        && !is_skip_type(f)
    {
        let key = field_name(f);

        if is_option_type(f) {
            if is_flatten_type(f) {
                return Some(quote! {
                    if let ::std::option::Option::Some(value) = &self.#name {
                        values.extend(value.to_db_values());
                    }
                });
            }

            return Some(quote! {
                if let ::std::option::Option::Some(value) = &self.#name {
                    values.push((#key, value.clone()).into());
                }
            });
        }

        if is_flatten_type(f) {
            return Some(quote! {
                values.extend(self.#name.to_db_values());
            });
        }

        return Some(quote! {
            values.push((#key, self.#name.clone()).into());
        });
    }

    None
}

fn impl_from_db_element(f: &syn::Field) -> Option<proc_macro2::TokenStream> {
    if let Some(name) = &f.ident {
        if name == DB_ID {
            return Some(quote! {
                #name: ::std::option::Option::Some(element.id.into())
            });
        }

        let str_name = field_name(f);
        let ty = &f.ty;

        if is_option_type(f) {
            if is_flatten_type(f) {
                return Some(quote! {
                    #name: if let ::std::result::Result::Ok(value) = #ty::from_db_element(element) {
                        ::std::option::Option::Some(value)
                    } else {
                        ::std::option::Option::None
                    }
                });
            }

            if is_skip_type(f) {
                return Some(quote! {
                    #name: None
                });
            }

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
        }

        if is_flatten_type(f) {
            return Some(quote! {
                #name: #ty::from_db_element(element)?
            });
        }

        if is_skip_type(f) {
            return Some(quote! {
                #name: #ty::default()
            });
        }

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

    None
}

fn impl_db_id(data: &syn::DataStruct) -> proc_macro2::TokenStream {
    data.fields
        .iter()
        .find_map(|f| {
            if let Some(name) = &f.ident
                && name == DB_ID
            {
                return Some(quote! {
                    if let ::std::option::Option::Some(id) = &self.db_id {
                        return ::std::option::Option::Some(id.clone().into());
                    } else {
                        return ::std::option::Option::None;
                    }
                });
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
            if seg.ident == "Option"
                && let syn::PathArguments::AngleBracketed(ref args) = seg.arguments
                && let Some(syn::GenericArgument::Type(_)) = args.args.first()
            {
                return true;
            }

            false
        });
    }

    false
}

fn is_flatten_type(f: &syn::Field) -> bool {
    f.attrs
        .iter()
        .find(|attr| attr.path().is_ident(AGDB))
        .and_then(|attr| {
            let mut found = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(FLATTEN) {
                    found = Some(());
                };
                Ok(())
            });
            found
        })
        .is_some()
}

fn is_skip_type(f: &syn::Field) -> bool {
    f.attrs
        .iter()
        .find(|attr| attr.path().is_ident(AGDB))
        .and_then(|attr| {
            let mut found = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(SKIP) {
                    found = Some(());
                };
                Ok(())
            });
            found
        })
        .is_some()
}

fn field_name(f: &syn::Field) -> String {
    if let Some(attrs) = f.attrs.iter().find(|attr| attr.path().is_ident(AGDB)) {
        let mut name = None;
        let _ = attrs.parse_nested_meta(|meta| {
            if meta.path.is_ident(RENAME)
                && let Ok(lit_str) = meta.value()
                && let Ok(syn::Lit::Str(lit_str)) = lit_str.parse()
            {
                name = Some(lit_str.value());
            }
            Ok(())
        });
        if let Some(name) = name {
            return name;
        }
    }

    f.ident.as_ref().expect("named field").to_string()
}
