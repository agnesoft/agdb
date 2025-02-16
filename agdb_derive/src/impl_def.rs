mod definitions;

use core::panic;
use definitions::Function;
use definitions::Functions;
use definitions::NamedType;
use definitions::NamedTypes;
use definitions::Type;
use proc_macro::token_stream::IntoIter;
use proc_macro::Delimiter;
use proc_macro::Group;
use proc_macro::Ident;
use proc_macro::Punct;
use proc_macro::TokenStream;
use proc_macro::TokenTree;
use std::fmt::Display;

pub fn impl_def(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let i2 = item.clone();
    let mut iter = item.into_iter();

    let impl_ident = next_ident(&mut iter);

    if impl_ident.to_string() == "impl" {
        let impl_generics = get_generics(&mut iter);
        let name = next_ident(&mut iter);
        let _name_generics = get_generics(&mut iter);
        let data = next_group(&mut iter);

        let mut functions = parse_impl_block(name, impl_generics, data.stream());
        functions.embed_generics();
        println!("{functions}")
    }

    return i2;
}

fn parse_impl_block(name: Ident, generics: NamedTypes, item: TokenStream) -> Functions {
    let mut iter = item.into_iter();
    let mut functions = Vec::new();

    while let Some(token) = iter.next() {
        if let TokenTree::Ident(accesor) = token {
            if accesor.to_string() == "pub" {
                functions.push(parse_function(&mut iter));
            }
        }
    }

    Functions {
        ty: name.to_string(),
        generics,
        functions,
    }
}

fn parse_function(iter: &mut IntoIter) -> Function {
    let _fn = next_ident(iter);
    assert!(_fn.to_string() == "fn");

    let name = next_ident(iter);
    let generics = get_generics(iter);
    let args = next_group(iter);
    let args = parse_args(args.stream());
    let ret = get_ret(iter);
    //let body = next_group(iter);

    Function {
        name: name.to_string(),
        generics,
        args,
        ret,
    }
}

fn parse_args(item: TokenStream) -> NamedTypes {
    let mut iter = item.into_iter();
    let mut args = Vec::new();

    while let Some(token) = iter.next() {
        match token {
            TokenTree::Group(_) => panic!("Unexpected group in arguments"),
            TokenTree::Ident(ident) => {
                let name = ident.to_string();

                if name == "mut" {
                    continue;
                }

                args.push(get_arg(ident, &mut iter));
            }
            TokenTree::Punct(_) => {}
            TokenTree::Literal(_) => panic!("Unexpected literals in arguments"),
        }
    }

    NamedTypes(args)
}

fn next_ident(iter: &mut IntoIter) -> Ident {
    while let Some(token) = iter.next() {
        if let TokenTree::Ident(ident) = token {
            return ident;
        }
    }

    panic!("Expected an identifier");
}

fn next_group(iter: &mut IntoIter) -> Group {
    while let Some(token) = iter.next() {
        if let TokenTree::Group(group) = token {
            return group;
        }
    }

    panic!("Expected a group");
}

fn get_generics(iter: &mut IntoIter) -> NamedTypes {
    let mut depth = 0;
    let mut new_iter = iter.clone();
    let mut generics = Vec::new();

    if let Some(TokenTree::Punct(p)) = new_iter.next() {
        if p.as_char() == '<' {
            depth += 1;

            while depth > 0 {
                if let Some(token) = new_iter.next() {
                    match token {
                        TokenTree::Group(_) => panic!("Unexpected group in generics"),
                        TokenTree::Ident(ident) => generics.push(get_arg(ident, &mut new_iter)),
                        TokenTree::Punct(punct) => {
                            if punct.as_char() == '<' {
                                depth += 1;
                            } else if punct.as_char() == '>' {
                                depth -= 1;
                            }
                        }
                        TokenTree::Literal(_) => panic!("Unexpected literal in generics"),
                    }
                }
            }

            *iter = new_iter;
        }
    }

    return NamedTypes(generics);
}

fn get_arg(ident: Ident, iter: &mut IntoIter) -> NamedType {
    let mut it = iter.clone();

    if let Some(TokenTree::Punct(p)) = it.next() {
        if p.as_char() == ':' {
            *iter = it;
            return NamedType {
                name: ident.to_string(),
                ty: get_type(iter),
            };
        }
    }

    return NamedType {
        name: ident.to_string(),
        ty: Type::None,
    };
}

fn get_generic_type(iter: &mut IntoIter) -> Type {
    let mut it = iter.clone();

    if let Some(TokenTree::Punct(p)) = it.next() {
        if p.as_char() == '<' {
            *iter = it;

            let t = get_type(iter);

            if let Some(TokenTree::Punct(p)) = iter.next() {
                if p.as_char() != '>' {
                    panic!("Expected '>', futher nesting not supported");
                }
            }

            return t;
        }
    }

    Type::None
}

fn get_type(iter: &mut IntoIter) -> Type {
    let ty = next_ident(iter);
    let ty_string = ty.to_string();

    let t = if ty_string == "Into" {
        let sub_ty = get_generic_type(iter);
        Type::Into(Box::new(sub_ty))
    } else if ty_string == "Vec" {
        let sub_ty = get_generic_type(iter);
        Type::Vec(Box::new(sub_ty))
    } else {
        Type::Named(ty_string)
    };

    t
}

fn get_ret(iter: &mut IntoIter) -> Type {
    let mut it = iter.clone();

    if let Some(TokenTree::Punct(p)) = it.next() {
        if p.as_char() == '-' {
            if let Some(TokenTree::Punct(p)) = it.next() {
                if p.as_char() == '>' {
                    *iter = it;
                    return get_type(iter);
                }
            }
        }
    }

    Type::None
}
