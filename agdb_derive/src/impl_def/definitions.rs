use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Type {
    None,
    Named(String),
    Into(Box<Type>),
    Vec(Box<Type>),
}

#[derive(Debug)]
pub struct NamedType {
    pub name: String,
    pub ty: Type,
}

pub struct NamedTypes(pub Vec<NamedType>);

pub struct Function {
    pub name: String,
    pub generics: NamedTypes,
    pub args: NamedTypes,
    pub ret: Type,
}

pub struct Functions {
    pub ty: String,
    pub generics: NamedTypes,
    pub functions: Vec<Function>,
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::None => String::new(),
            Type::Named(name) => name.clone(),
            Type::Into(ty) => format!("::std::convert::Into<{}>", ty.name()),
            Type::Vec(ty) => format!("::std::vec::Vec<{}>", ty.name()),
        }
    }
}

impl Functions {
    pub fn embed_generics(&mut self) {
        for f in self.functions.iter_mut() {
            for arg in &mut f.args.0 {
                if let Type::Named(ty) = &arg.ty {
                    if let Some(g) = self.generics.0.iter().find(|g| g.name == *ty) {
                        arg.ty = g.ty.clone();
                    } else if let Some(g) = f.generics.0.iter().find(|g| g.name == *ty) {
                        arg.ty = g.ty.clone();
                    }
                }
            }
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::None => Ok(()),
            Type::Named(name) => f.write_str(name),
            Type::Into(ty) => f.write_fmt(format_args!("Into<{}>", ty)),
            Type::Vec(ty) => f.write_fmt(format_args!("Vec<{}>", ty)),
        }
    }
}

impl Display for NamedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Type::None = self.ty {
            return f.write_str(&self.name);
        } else {
            return f.write_fmt(format_args!("{}: {}", self.name, self.ty));
        }
    }
}

impl Display for NamedTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self
            .0
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        if !args.is_empty() {
            return f.write_str(&args);
        }

        Ok(())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_fmt(format_args!(
            "fn {name}({args}) -> {ret};",
            name = self.name,
            args = self.args.to_string(),
            ret = self.ret,
        ));
    }
}

impl Display for Functions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let functions = self
            .functions
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        return f.write_fmt(format_args!(
            "impl {ty}\n{functions}\n",
            ty = self.ty,
            functions = functions
        ));
    }
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let n = &self.name;
        let r = format_ident!("{}", self.ret.name());
        let args = self.args.0.iter();
        let t = quote! {
            ::agdb::api::Function {
                name: #n,
                args: vec![#(#args),*],
                expressions: vec![],
                ret: #r ::def,
            }
        };
        *tokens = quote! {
            #tokens
            #t
        };
    }
}

impl ToTokens for NamedTypes {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let types = self.0.iter();
        *tokens = quote! {
            #tokens
            #(#types),*
        };
    }
}

impl ToTokens for NamedType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty_name = self.ty.name();

        let t = if self.name.is_empty() {
            let ty = format_ident!("{}", &ty_name);
            quote! {
                #ty
            }
        } else if ty_name.is_empty() {
            let name = format_ident!("{}", &self.name);
            quote! {
                #name
            }
        } else {
            let name = format_ident!("{}", &self.name);
            let ty = format_ident!("{}", &ty_name);
            quote! {
                #name: #ty
            }
        };

        quote! {
            #tokens
            #t
        };
    }
}
