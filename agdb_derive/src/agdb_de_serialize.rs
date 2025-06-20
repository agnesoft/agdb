use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Ident;
use syn::Index;
use syn::Type;
use syn::parse_macro_input;

pub fn agdb_de_serialize(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;

    let tokens = if let syn::Data::Struct(data) = input.data {
        let fields_types = data
            .fields
            .iter()
            .map(|f| (f.ident.as_ref(), &f.ty))
            .collect::<Vec<(Option<&Ident>, &Type)>>();

        if fields_types.is_empty() || fields_types[0].0.is_some() {
            serialize_struct(name, fields_types)
        } else {
            serialize_tuple(name, fields_types)
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
                quote! { #name::#variant_name { #(#names),* } => { #(size += ::agdb::AgdbSerialize::serialized_size(#names);)* } }
            } else {
                quote! { #name::#variant_name(#(#names),*) => { #(size += ::agdb::AgdbSerialize::serialized_size(#names);)* } }
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
                quote! { #name::#variant_name { #(#names),* } => { __buffer.push(#variant_index); #(__buffer.extend(::agdb::AgdbSerialize::serialize(#names));)* } }
            } else {
                quote! { #name::#variant_name(#(#names),*) => { __buffer.push(#variant_index); #(__buffer.extend(::agdb::AgdbSerialize::serialize(#names));)* } }
            }
        }
    });
    let deserializers = enum_data.variants.iter().enumerate().map(|(index, variant)| {
        let variant_index = index as u8;
        let variant_name = &variant.ident;

        if variant.fields.is_empty() {
            quote! { ::std::option::Option::Some(#variant_index) => { ::std::result::Result::Ok(#name::#variant_name) } }
        } else {
            let mut named = true;
            let fields = variant
                .fields
                .iter()
                .map(|field| {
                    let ty = &field.ty;
                    if let Some(ident) = &field.ident {
                        quote! { #ident: { let #ident = <#ty as ::agdb::AgdbSerialize>::deserialize(&buffer[__offset as usize..])?; __offset += ::agdb::AgdbSerialize::serialized_size(&#ident); #ident } }
                    } else {
                        named = false;
                        quote! { { let v = <#ty as ::agdb::AgdbSerialize>::deserialize(&buffer[__offset as usize..])?; __offset += ::agdb::AgdbSerialize::serialized_size(&v); v } }
                    }
                })
                .collect::<Vec<_>>();

            if named {
                quote! { ::std::option::Option::Some(#variant_index) => { let mut __offset = 1_u64; ::std::result::Result::Ok(#name::#variant_name { #(#fields),* } ) } }
            } else {
                quote! { ::std::option::Option::Some(#variant_index) => { let mut __offset = 1_u64; ::std::result::Result::Ok(#name::#variant_name( #(#fields),* )) } }
            }
        }
    });

    quote! {
        impl ::agdb::AgdbSerialize for #name {
            fn serialized_size(&self) -> u64 {
                let mut size = 1_u64;
                match self {
                    #(
                        #sizes
                    )*
                }
                size
            }

            fn serialize(&self) -> ::std::vec::Vec<u8> {
                let mut __buffer = ::std::vec::Vec::with_capacity(::agdb::AgdbSerialize::serialized_size(self) as usize);
                match self {
                    #(
                        #serializers
                    )*
                }
                __buffer
            }

            fn deserialize(buffer: &[u8]) -> ::std::result::Result<Self, ::agdb::DbError> {
                match buffer.first() {
                    #(
                        #deserializers
                    ),*
                    _ => ::std::result::Result::Err(::agdb::DbError::from("Invalid enum variant"))
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
                size += ::agdb::AgdbSerialize::serialized_size(&self.#num);
            }
        });
    let serializers = fields_types
        .iter()
        .enumerate()
        .map(|(index, (_name, _ty))| {
            let num = Index::from(index);
            quote! {
                __buffer.extend(::agdb::AgdbSerialize::serialize(&self.#num));
            }
        });
    let deserializers = fields_types.iter().enumerate().map(|(index, (_name, ty))| {
        let name = format_ident!("__{}", index);
        quote! {
            let #name = <#ty as ::agdb::AgdbSerialize>::deserialize(&buffer[__offset as usize..])?;
            __offset += ::agdb::AgdbSerialize::serialized_size(&#name);
        }
    });

    quote! {
        impl ::agdb::AgdbSerialize for #name {
            fn serialized_size(&self) -> u64 {
                let mut size = 0;
                #(
                    #sizes
                )*
                size
            }

            fn serialize(&self) -> ::std::vec::Vec<u8> {
                let mut __buffer = ::std::vec::Vec::with_capacity(::agdb::AgdbSerialize::serialized_size(self) as usize);
                #(
                    #serializers
                )*
                __buffer
            }

            fn deserialize(buffer: &[u8]) -> ::std::result::Result<Self, ::agdb::DbError> {
                let mut __offset = 0;
                #(
                   #deserializers
                )*
                ::std::result::Result::Ok(Self(
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
            size += ::agdb::AgdbSerialize::serialized_size(&self.#name);
        }
    });
    let serializers = fields_types.iter().map(|(name, _ty)| {
        let name = name.unwrap();
        quote! {
            __buffer.extend(::agdb::AgdbSerialize::serialize(&self.#name));
        }
    });
    let deserializers = fields_types.iter().map(|(name, ty)| {
        let name = name.unwrap();
        quote! {
            let #name = <#ty as ::agdb::AgdbSerialize>::deserialize(&buffer[__offset as usize..])?;
            __offset += ::agdb::AgdbSerialize::serialized_size(&#name);
        }
    });

    quote! {
        impl ::agdb::AgdbSerialize for #name {
            fn serialized_size(&self) -> u64 {
                let mut size = 0;
                #(
                    #sizes
                )*
                size
            }

            fn serialize(&self) -> ::std::vec::Vec<u8> {
                let mut __buffer = ::std::vec::Vec::with_capacity(::agdb::AgdbSerialize::serialized_size(self) as usize);
                #(
                    #serializers
                )*
                __buffer
            }

            fn deserialize(buffer: &[u8]) -> ::std::result::Result<Self, ::agdb::DbError> {
                let mut __offset = 0;
                #(
                   #deserializers
                )*
                ::std::result::Result::Ok(Self {
                    #(
                        #names
                    ),*
                })
            }
        }
    }
}
