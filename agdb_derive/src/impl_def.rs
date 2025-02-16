mod definitions;

use core::panic;
use definitions::{Function, Functions, NamedType, NamedTypes};
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

        let functions = parse_impl_block(name, impl_generics, data.stream());
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
                functions.push(parse_function(&name, &mut iter));
            }
        }
    }

    Functions {
        ty: name.to_string(),
        generics,
        functions,
    }
}

fn parse_function(ty: &Ident, iter: &mut IntoIter) -> Function {
    let _fn = next_ident(iter);
    assert!(_fn.to_string() == "fn");

    let name = next_ident(iter);
    let generics = get_generics(iter);
    let args = next_group(iter);
    let args = parse_args(args.stream());
    let ret = next_ident(iter);
    let ret_generics = get_generics(iter);
    //let body = next_group(iter);

    Function {
        name: name.to_string(),
        generics,
        args,
        ret: ret.to_string(),
        ret_generics,
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

                if name == "self" || name == "Self" {
                    args.push(NamedType {
                        name: "self".to_string(),
                        ty: String::new(),
                    });
                } else {
                    let ty = next_ident(&mut iter);
                    args.push(NamedType {
                        name,
                        ty: ty.to_string(),
                    });
                }
            }
            TokenTree::Punct(punct) => {}
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
                        TokenTree::Ident(ident) => generics.push(get_generic(ident, &mut new_iter)),
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

fn get_generic(ident: Ident, iter: &mut IntoIter) -> NamedType {
    let mut it = iter.clone();

    if let Some(TokenTree::Punct(p)) = it.next() {
        if p.as_char() == ':' {
            let ty = next_ident(&mut it);
            let sub_generic = get_sub_generic(&mut it);
            *iter = it;

            return NamedType {
                name: ident.to_string(),
                ty: if sub_generic.is_empty() {
                    ty.to_string()
                } else {
                    format!("{}<{}>", ty, sub_generic)
                },
            };
        }
    }

    return NamedType {
        name: ident.to_string(),
        ty: String::new(),
    };
}

fn get_sub_generic(iter: &mut IntoIter) -> String {
    let mut it = iter.clone();

    if let Some(TokenTree::Punct(p)) = it.next() {
        if p.as_char() == '<' {
            let ty = next_ident(&mut it);

            if let Some(TokenTree::Punct(p)) = it.next() {
                if p.as_char() == '>' {
                    *iter = it;
                    return ty.to_string();
                } else {
                    panic!("Further nesting not supported, expected '>'");
                }
            }
        }
    }

    return String::new();
}
