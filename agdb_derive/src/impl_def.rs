use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::Generics;
use syn::ImplItem;
use syn::ItemImpl;
use syn::PathArguments;
use syn::ReturnType;
use syn::TypeParamBound;

pub fn impl_def(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ret = item.clone();

    let impl_block = syn::parse_macro_input!(item as ItemImpl);

    let ty = impl_block.self_ty;
    let ty_g = impl_block.generics;
    let mut funcs = vec![];

    for i in impl_block.items {
        if let ImplItem::Fn(f) = i {
            funcs.push(parse_function(f));
        }
    }

    let t = quote! {
        impl #ty_g ::agdb::api::ApiFunctions for #ty {
            fn functions() -> Vec<::agdb::api::Function> {
                vec![#(#funcs),*]
            }
         }
    };

    println!("{t}");

    ret.extend(Into::<TokenStream>::into(t));
    ret
}

fn parse_function(f: syn::ImplItemFn) -> proc_macro2::TokenStream {
    let name = f.sig.ident.to_string();
    let ret_ty = return_type(&f);
    let mut args = vec![];
    for a in f.sig.inputs {
        if let syn::FnArg::Typed(t) = a {
            args.push(parse_arg(t, &f.sig.generics));
        }
    }

    let api_func = quote! {
        ::agdb::api::Function {
            name: #name,
            args: vec![#(#args),*],
            ret: #ret_ty,
            expressions: vec![],
        }
    };
    api_func
}

fn parse_arg(t: syn::PatType, generics: &Generics) -> proc_macro2::TokenStream {
    let name = t.pat.to_token_stream().to_string();
    let ty = arg_type(&t.ty, generics);
    quote! { ::agdb::api::NamedType { name: #name, ty: #ty } }
}

fn arg_type(t: &syn::Type, generics: &Generics) -> proc_macro2::TokenStream {
    let t_str = t.to_token_stream().to_string();
    let ty = generics
        .type_params()
        .find_map(|g| {
            if g.ident.to_string() == t_str {
                if let Some(TypeParamBound::Trait(bound)) = g.bounds.first() {
                    if let Some(bound) = bound.path.segments.first() {
                        if bound.ident.to_string() == "Into" {
                            if let PathArguments::AngleBracketed(ty) = &bound.arguments {
                                if let syn::GenericArgument::Type(ty) = &ty.args[0] {
                                    return Some(quote! { ::agdb::#ty });
                                }
                            }
                        }
                    }
                }
            }

            None
        })
        .unwrap_or(t.to_token_stream());

    let ty = quote! { <#ty as ::agdb::api::ApiDefinition>::def };
    ty
}

fn return_type(f: &syn::ImplItemFn) -> proc_macro2::TokenStream {
    let ret_ty = if let ReturnType::Type(_, t) = &f.sig.output {
        quote! { <#t as ::agdb::api::ApiDefinition>::def }
    } else {
        quote! { || ::agdb::api::Type::None }
    };
    ret_ty
}
