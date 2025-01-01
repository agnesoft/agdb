use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::parse_macro_input;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Ident;
use syn::Index;
use syn::Type;

const DB_ID: &str = "db_id";

/// The helper derive macro to add `agdb` compatibility to
/// user defined types. This type provides blank implementation
/// of the `agdb::DbUserValueMarker` trait. This is needed for the
/// vectorized custom values to be compatible with the database
/// as the `From` trait implementation witohut it conflicts
/// with the blanket implementations.
///
/// # Examples
///
/// ```ignore
/// #[derive(agdb::UserValueMarker, Default, Copy, Clone, Debug)]
/// enum MyEnum {
///    #[default]
///    A,
///    B,
/// }
/// ```
#[proc_macro_derive(UserValueMarker)]
pub fn db_user_value_marker_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;

    let tokens = quote! {
        impl agdb::DbUserValueMarker for #name {}
    };

    tokens.into()
}

/// The derive macro to add `agdb` platform agnostic serialization
/// support. This is only needed if you want to serialize custom
/// complex data structures and do not want or cannot use serde.
/// It is primarily used internally to serialize the `agdb` data
/// structures.
#[proc_macro_derive(AgdbDeSerialize)]
pub fn agdb_de_serialize(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;

    let tokens = if let syn::Data::Struct(data) = input.data {
        let fields_types = data
            .fields
            .iter()
            .map(|f| (f.ident.as_ref(), &f.ty))
            .collect::<Vec<(Option<&Ident>, &Type)>>();

        if fields_types.is_empty() || fields_types[0].0.is_none() {
            serialize_tuple(name, fields_types)
        } else {
            serialize_struct(name, fields_types)
        }
    } else if let syn::Data::Enum(data) = input.data {
        serialize_enum(name, data)
    } else {
        unimplemented!()
    };

    tokens.into()
}

fn serialize_enum(name: Ident, enum_data: DataEnum) -> proc_macro2::TokenStream {
    let sizes = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        if variant.fields.is_empty() {
            quote! { #name::#variant_name => {} }
        } else {
            let mut named = false;
            let names = variant
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    if let Some(i) = &field.ident {
                        named = true;
                        i.clone()
                    } else {
                        format_ident!("__{}", index)
                    }
                })
                .collect::<Vec<_>>();

            if named {
                quote! { #name::#variant_name { #(#names),* } => { #(size += #names.serialized_size();)* } }
            } else {
                quote! { #name::#variant_name(#(#names),*) => { #(size += #names.serialized_size();)* } }
            }
        }
    });
    let serializers = enum_data.variants.iter().enumerate().map(|(index, variant)| {
        let variant_name = &variant.ident;
        let variant_index = index as u8;

        if variant.fields.is_empty() {
            quote! { #name::#variant_name => { __buffer.push(#variant_index); } }
        } else {
            let mut named = false;
            let names = variant
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    if let Some(ident) = &field.ident {
                        named = true;
                        ident.clone()
                    } else {
                        format_ident!("__{}", index)
                    }
                })
                .collect::<Vec<_>>();

            if named {
                quote! { #name::#variant_name { #(#names),* } => { __buffer.push(#variant_index); #(__buffer.extend(#names.serialize());)* } }
            } else {
                quote! { #name::#variant_name(#(#names),*) => { __buffer.push(#variant_index); #(__buffer.extend(#names.serialize());)* } }
            }
        }
    });
    let deserializers = enum_data.variants.iter().enumerate().map(|(index, variant)| {
        let variant_index = index as u8;
        let variant_name = &variant.ident;

        if variant.fields.is_empty() {
            quote! { Some(#variant_index) => { Ok(#name::#variant_name) } }
        } else {
            let mut named = true;
            let fields = variant
                .fields
                .iter()
                .map(|field| {
                    let ty = &field.ty;
                    if let Some(ident) = &field.ident {
                        quote! { #ident: { let #ident = <#ty as agdb::AgdbSerialize>::deserialize(&buffer[__offset as usize..])?; __offset += #ident.serialized_size(); #ident } }
                    } else {
                        named = false;
                        quote! { { let v = <#ty as agdb::AgdbSerialize>::deserialize(&buffer[__offset as usize..])?; __offset += v.serialized_size(); v } }
                    }
                })
                .collect::<Vec<_>>();

            if named {
                quote! { Some(#variant_index) => { let mut __offset = 1_u64; Ok(#name::#variant_name { #(#fields),* } ) } }
            } else {
                quote! { Some(#variant_index) => { let mut __offset = 1_u64; Ok(#name::#variant_name( #(#fields),* )) } }
            }
        }
    });

    quote! {
        impl agdb::AgdbSerialize for #name {
            fn serialized_size(&self) -> u64 {
                let mut size = 1_u64;
                match self {
                    #(
                        #sizes
                    )*
                }
                size
            }

            fn serialize(&self) -> Vec<u8> {
                let mut __buffer = Vec::with_capacity(self.serialized_size() as usize);
                match self {
                    #(
                        #serializers
                    )*
                }
                __buffer
            }

            fn deserialize(buffer: &[u8]) -> Result<Self, agdb::DbError> {
                match buffer.first() {
                    #(
                        #deserializers
                    ),*
                    _ => Err(agdb::DbError::from("Invalid enum variant"))
                }

            }
        }
    }
}

fn serialize_tuple(
    name: Ident,
    fields_types: Vec<(Option<&Ident>, &Type)>,
) -> proc_macro2::TokenStream {
    let names = fields_types
        .iter()
        .enumerate()
        .map(|(index, (_name, _ty))| format_ident!("__{}", index));
    let sizes = fields_types
        .iter()
        .enumerate()
        .map(|(index, (_name, _ty))| {
            let num = Index::from(index);
            quote! {
                size += self.#num.serialized_size();
            }
        });
    let serializers = fields_types
        .iter()
        .enumerate()
        .map(|(index, (_name, _ty))| {
            let num = Index::from(index);
            quote! {
                __buffer.extend(self.#num.serialize());
            }
        });
    let deserializers = fields_types.iter().enumerate().map(|(index, (_name, ty))| {
        let name = format_ident!("__{}", index);
        quote! {
            let #name = <#ty as agdb::AgdbSerialize>::deserialize(&buffer[__offset as usize..])?;
            __offset += #name.serialized_size();
        }
    });

    quote! {
        impl agdb::AgdbSerialize for #name {
            fn serialized_size(&self) -> u64 {
                let mut size = 0;
                #(
                    #sizes
                )*
                size
            }

            fn serialize(&self) -> Vec<u8> {
                let mut __buffer = Vec::with_capacity(self.serialized_size() as usize);
                #(
                    #serializers
                )*
                __buffer
            }

            fn deserialize(buffer: &[u8]) -> Result<Self, agdb::DbError> {
                let mut __offset = 0;
                #(
                   #deserializers
                )*
                Ok(Self(
                    #(
                        #names
                    ),*
                ))
            }
        }
    }
}

fn serialize_struct(
    name: Ident,
    fields_types: Vec<(Option<&Ident>, &Type)>,
) -> proc_macro2::TokenStream {
    let names = fields_types.iter().map(|(name, _ty)| name.unwrap());
    let sizes = fields_types.iter().map(|(name, _ty)| {
        let name = name.unwrap();
        quote! {
            size += self.#name.serialized_size();
        }
    });
    let serializers = fields_types.iter().map(|(name, _ty)| {
        let name = name.unwrap();
        quote! {
            __buffer.extend(self.#name.serialize());
        }
    });
    let deserializers = fields_types.iter().map(|(name, ty)| {
        let name = name.unwrap();
        quote! {
            let #name = <#ty as agdb::AgdbSerialize>::deserialize(&buffer[__offset as usize..])?;
            __offset += #name.serialized_size();
        }
    });

    quote! {
        impl agdb::AgdbSerialize for #name {
            fn serialized_size(&self) -> u64 {
                let mut size = 0;
                #(
                    #sizes
                )*
                size
            }

            fn serialize(&self) -> Vec<u8> {
                let mut __buffer = Vec::with_capacity(self.serialized_size() as usize);
                #(
                    #serializers
                )*
                __buffer
            }

            fn deserialize(buffer: &[u8]) -> Result<Self, agdb::DbError> {
                let mut __offset = 0;
                #(
                   #deserializers
                )*
                Ok(Self {
                    #(
                        #names
                    ),*
                })
            }
        }
    }
}

/// The derive macro to add `agdb` compatibility
/// to user defined types. It implements [`agdb::DbUserValue`]
/// for the type automatically to allow your type to be read and
/// stored from/to the database.
///
/// If your type contains a field `db_id: Option<agdb::DbId>`
/// it will be treated specially and will additionally allow
/// shorthand inserts/updates of the elements directly.
///
/// All database types are supported. User (custom) types must
/// `impl From<T> for agdb::DbValue` and `impl TryFrom<agdb::DbValue> for T { type Error = agdb::DbError }`.
///
/// `Option` can be used on on any supported type. If a value type is
/// an `Option` and runtime value is `None` the value will NOT be stored
/// in the database. When reading elements if a type contains
/// an `Option` all keys are always retrieved and searched for individual
/// keys as opposed to standard index based read for non-optional types.
///
/// # Examples
///
/// ## Standard
/// ```ignore
/// #[derive(agdb::UserValue)]
/// struct MyValue {
///     num_value: i64,
///     string_value: String,
///     vec_value: Vec<u64>,
/// }
/// ```
///
/// ## With db_id
/// ```ignore
/// #[derive(agdb::UserValue)]
/// struct MyValue {
///     db_id: Option<agdb::DbId>, //this field is useful but not mandatory
///     num_value: i64,
///     string_value: String,
///     vec_value: Vec<u64>,
/// }
/// ```
///
/// ## With optional
/// ```ignore
/// #[derive(agdb::UserValue)]
/// struct MyValue {
///     db_id: Option<agdb::DbId>, //this field is useful but not mandatory
///     num_value: i64,
///     string_value: Option<String>, // Optional value will not be stored if None
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
        impl agdb::DbUserValue for #name {
            type ValueType = #name;

            #[track_caller]
            fn db_id(&self) -> Option<agdb::QueryId> {
                #db_id
            }

            #[track_caller]
            fn db_keys() -> Vec<agdb::DbValue> {
                vec![#(#db_keys.into()),*]
            }

            #[track_caller]
            fn from_db_element(element: &agdb::DbElement) -> std::result::Result<Self::ValueType, agdb::DbError> {
                Ok(Self {
                    #(#from_db_element),*
                })
            }

            #[track_caller]
            fn to_db_values(&self) -> Vec<agdb::DbKeyValue> {
                let mut values = Vec::with_capacity(#counter);
                #(#db_values)*
                values
            }
        }

        impl agdb::DbUserValue for &#name {
            type ValueType = #name;

            #[track_caller]
            fn db_id(&self) -> Option<agdb::QueryId> {
                #name::db_id(*self)
            }

            #[track_caller]
            fn db_keys() -> Vec<agdb::DbValue> {
                #name::db_keys()
            }

            #[track_caller]
            fn from_db_element(element: &agdb::DbElement) -> std::result::Result<Self::ValueType, agdb::DbError> {
                #name::from_db_element(element)
            }

            #[track_caller]
            fn to_db_values(&self) -> Vec<agdb::DbKeyValue> {
                #name::to_db_values(*self)
            }
        }

        impl TryFrom<&agdb::DbElement> for #name {
            type Error = agdb::DbError;

            #[track_caller]
            fn try_from(value: &agdb::DbElement) -> std::result::Result<Self, Self::Error> {
                use agdb::DbUserValue;
                #name::from_db_element(value)
            }
        }

        impl TryFrom<agdb::QueryResult> for #name {
            type Error = agdb::DbError;

            #[track_caller]
            fn try_from(value: agdb::QueryResult) -> std::result::Result<Self, Self::Error> {
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

fn impl_db_values(f: &syn::Field, has_option: bool) -> Option<proc_macro2::TokenStream> {
    if let Some(name) = &f.ident {
        if name != DB_ID {
            let key = name.to_string();

            if has_option && is_option_type(f) {
                return Some(quote! {
                    if let Some(value) = &self.#name {
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
                #name: Some(element.id.into())
            });
        } else if has_option {
            let str_name = name.to_string();
            if is_option_type(f) {
                return Some(quote! {
                    #name: element.values.iter().find_map(|kv| { if let Ok(key) = kv.key.string() { if key == #str_name { return Some(kv.value.clone().try_into()); } } None }).map_or_else(|| Ok(None), |v| { if let Ok(v) = v { Ok(Some(v)) } else { Err(agdb::DbError::from(format!("Failed to convert value of '{}': {}", #str_name, v.unwrap_err()))) } })?
                });
            } else {
                return Some(quote! {
                    #name: element.values.iter().find_map(|kv| { if let Ok(key) = kv.key.string() { if key == #str_name { return Some(kv.value.clone().try_into()); } } None }).ok_or(agdb::DbError::from(format!("Key '{}' not found", #str_name)))?.map_err(|e| agdb::DbError::from(format!("Failed to convert value of '{}': {}", #str_name, e)))?
                });
            }
        } else {
            let str_name = name.to_string();
            let i = *counter;
            *counter += 1;
            return Some(quote! {
                #name: element.values.get(#i).ok_or(agdb::DbError::from(format!("Not enough keys: '{}' not found at position {}", #str_name, #i)))?.value.clone().try_into().map_err(|e| agdb::DbError::from(format!("Failed to convert value of '{}': {}", #str_name, e)))?
            });
        }
    }

    None
}

fn impl_db_id(data: &syn::DataStruct) -> proc_macro2::TokenStream {
    let db_id = data
        .fields
        .iter()
        .find_map(|f| {
            if let Some(name) = &f.ident {
                if name == DB_ID {
                    return Some(quote! {
                        if let Some(id) = self.db_id.clone() {
                            return Some(id.into());
                        } else {
                            return None;
                        }
                    });
                }
            }

            None
        })
        .unwrap_or(quote! {
            None
        });
    db_id
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
