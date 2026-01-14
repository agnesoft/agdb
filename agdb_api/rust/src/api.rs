use crate::AdminStatus;
use crate::AgdbApi;
use crate::AgdbApiError;
use crate::ClusterStatus;
use crate::DbAudit;
use crate::DbUser;
use crate::ReqwestClient;
use crate::ServerDatabase;
use crate::UserStatus;
use agdb::DbKeyOrder;
use agdb::DbKeyValue;
use agdb::DbValues;
use agdb::MultiValues;
use agdb::QueryAliases;
use agdb::QueryBuilder;
use agdb::QueryCondition;
use agdb::QueryResult;
use agdb::QueryType;
use agdb::SingleValues;
use agdb::api_def::Enum;
use agdb::api_def::Function;
use agdb::api_def::Struct;
use agdb::api_def::TupleStruct;
use agdb::api_def::Type;
use agdb::api_def::TypeDefinition;
use std::collections::HashSet;

pub struct Api {
    type_names: HashSet<String>,
}

impl Api {
    pub fn types() -> Vec<Type> {
        let top_level_types = vec![
            QueryBuilder::type_def(),
            AgdbApi::<ReqwestClient>::type_def(),
            AgdbApiError::type_def(),
            DbKeyOrder::type_def(),
            DbKeyValue::type_def(),
            QueryCondition::type_def(),
            QueryAliases::type_def(),
            MultiValues::type_def(),
            SingleValues::type_def(),
            DbValues::type_def(),
            ServerDatabase::type_def(),
            QueryType::type_def(),
            QueryResult::type_def(),
            AdminStatus::type_def(),
            UserStatus::type_def(),
            ClusterStatus::type_def(),
            DbAudit::type_def(),
            DbUser::type_def(),
        ];

        let mut types = vec![];
        let mut api = Api {
            type_names: HashSet::new(),
        };

        for t in top_level_types {
            types.extend(api.extract_types(t));
        }

        types
    }

    fn extract_types(&mut self, t: Type) -> Vec<Type> {
        if self.type_names.contains(t.name()) {
            return Vec::new();
        } else {
            self.type_names.insert(t.name().to_string());
        }

        match &t {
            Type::Enum(e) => {
                let mut types = self.enum_types(e);
                types.push(t);
                types
            }
            Type::Struct(s) => {
                let mut types = self.struct_types(s);
                types.push(t);
                types
            }
            Type::TupleStruct(ts) => {
                let mut types = self.tuple_types(ts);
                types.push(t);
                types
            }
            Type::Literal(_) => Vec::new(),
            Type::Tuple(items) => items
                .iter()
                .flat_map(|ty| self.extract_types(ty()))
                .collect(),
            Type::Slice(t) => self.extract_types(t()),
            Type::Vec(t) => self.extract_types(t()),
            Type::Option(t) => self.extract_types(t()),
            Type::Result(t1, t2) => {
                let mut types = self.extract_types(t1());
                types.extend(self.extract_types(t2()));
                types
            }
            Type::Generic(_) => Vec::new(),
        }
    }

    fn enum_types(&mut self, e: &Enum) -> Vec<Type> {
        let mut types = vec![];

        for variant in e.variants {
            if let Some(ty_fn) = variant.ty {
                types.extend(self.extract_types(ty_fn()));
            }
        }

        for f in e.functions {
            types.extend(self.function_types(f));
        }

        types
    }

    fn struct_types(&mut self, s: &Struct) -> Vec<Type> {
        let mut types = vec![];

        for field in s.fields {
            if let Some(ty_fn) = field.ty {
                types.extend(self.extract_types(ty_fn()));
            }
        }

        for f in s.functions {
            types.extend(self.function_types(f));
        }

        types
    }

    fn tuple_types(&mut self, t: &TupleStruct) -> Vec<Type> {
        let mut types = vec![];

        for field_fn in t.fields {
            types.extend(self.extract_types(field_fn()));
        }

        for f in t.functions {
            types.extend(self.function_types(f));
        }

        types
    }

    fn function_types(&mut self, f: &Function) -> Vec<Type> {
        let mut types = vec![];

        for arg in f.args {
            if let Some(ty_fn) = arg.ty {
                types.extend(self.extract_types(ty_fn()));
            }
        }

        if let Some(ret_ty_fn) = &f.ret {
            types.extend(self.extract_types(ret_ty_fn()));
        }

        types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::api_def::Generic;
    use agdb::api_def::NamedType;

    struct RustApi;

    impl RustApi {
        fn generate() -> String {
            let mut buffer = String::new();
            buffer.push_str(&Self::preamble());

            Api::types().iter().for_each(|ty| {
                buffer.push_str(&Self::write_type(ty));
                buffer.push('\n');
            });

            buffer
        }

        fn write_type(ty: &Type) -> String {
            match ty {
                Type::Enum(e) => Self::write_enum(e),
                Type::Struct(s) => Self::write_struct(s),
                Type::TupleStruct(t) => Self::write_tuple_struct(t),
                Type::Literal(_)
                | Type::Tuple(_)
                | Type::Slice(_)
                | Type::Vec(_)
                | Type::Option(_)
                | Type::Result(_, _)
                | Type::Generic(_) => String::new(),
            }
        }

        fn generic_decl(generic: &Generic) -> String {
            if !generic.bounds.is_empty() {
                return format!(
                    "{}: {}",
                    generic.name,
                    generic
                        .bounds
                        .iter()
                        .map(|g| Self::generic_decl(g))
                        .collect::<Vec<String>>()
                        .join(" + ")
                );
            }

            if !generic.args.is_empty() {
                let args = generic
                    .args
                    .iter()
                    .map(|arg| Self::type_name(&arg()))
                    .collect::<Vec<String>>()
                    .join(", ");
                return format!("{}<{}>", generic.name, args);
            }

            generic.name.to_string()
        }

        fn generics_decl(generics: &[Generic]) -> String {
            if generics.is_empty() {
                return String::new();
            }

            let generic_decls: Vec<String> = generics.iter().map(Self::generic_decl).collect();

            format!("<{}>", generic_decls.join(", "))
        }

        fn generics(generics: &[Generic]) -> String {
            if generics.is_empty() {
                return String::new();
            }

            let generic_names: Vec<&str> = generics.iter().map(|g| g.name).collect();
            format!("<{}>", generic_names.join(", "))
        }

        fn type_name(ty: &Type) -> String {
            match ty {
                Type::Literal(lit) => match lit {
                    agdb::api_def::LiteralType::Bool => "bool".to_string(),
                    agdb::api_def::LiteralType::I8 => "i8".to_string(),
                    agdb::api_def::LiteralType::I16 => "i16".to_string(),
                    agdb::api_def::LiteralType::I32 => "i32".to_string(),
                    agdb::api_def::LiteralType::I64 => "i64".to_string(),
                    agdb::api_def::LiteralType::U8 => "u8".to_string(),
                    agdb::api_def::LiteralType::U16 => "u16".to_string(),
                    agdb::api_def::LiteralType::U32 => "u32".to_string(),
                    agdb::api_def::LiteralType::U64 => "u64".to_string(),
                    agdb::api_def::LiteralType::F32 => "f32".to_string(),
                    agdb::api_def::LiteralType::F64 => "f64".to_string(),
                    agdb::api_def::LiteralType::String => "String".to_string(),
                    agdb::api_def::LiteralType::Str => "&str".to_string(),
                    agdb::api_def::LiteralType::Unit => "()".to_string(),
                },
                Type::Enum(e) => format!("{}{}", e.name, Self::generics(e.generics)),
                Type::Struct(s) => format!("{}{}", s.name, Self::generics(s.generics)),
                Type::TupleStruct(t) => format!("{}{}", t.name, Self::generics(t.generics)),
                Type::Tuple(t) => format!(
                    "({})",
                    t.iter()
                        .map(|ty| {
                            let ty = ty();
                            Self::type_name(&ty)
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                Type::Slice(s) => format!("&[{}]", Self::type_name(&s())),
                Type::Vec(v) => format!("Vec<{}>", Self::type_name(&v())),
                Type::Option(o) => format!("Option<{}>", Self::type_name(&o())),
                Type::Result(ok, err) => format!(
                    "Result<{}, {}>",
                    Self::type_name(&ok()),
                    Self::type_name(&err())
                ),
                Type::Generic(name) => name.to_string(),
            }
        }

        fn write_enum(e: &Enum) -> String {
            let mut buffer = String::new();
            buffer.push_str(&format!(
                "enum {}{} {{\n",
                e.name,
                Self::generics_decl(e.generics)
            ));
            for variant in e.variants {
                buffer.push_str(&Self::write_enum_variant(variant));
            }
            buffer.push_str("}\n");
            buffer
        }

        fn write_enum_variant(variant: &NamedType) -> String {
            if let Some(ty_fn) = &variant.ty {
                let ty = ty_fn();
                match ty {
                    Type::Struct(s) => {
                        return format!(
                            "    {} {{ {} }},\n",
                            variant.name,
                            s.fields
                                .iter()
                                .map(|f| {
                                    let field_ty =
                                        f.ty.expect("enum struct field must have a type");
                                    format!("{}: {}", f.name, Self::type_name(&field_ty()))
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                    Type::TupleStruct(ts) => {
                        return format!(
                            "    {}({}),\n",
                            variant.name,
                            ts.fields
                                .iter()
                                .map(|f| { Self::type_name(&f()) })
                                .collect::<Vec<String>>()
                                .join(", ")
                        );
                    }
                    _ => {
                        return format!("    {}({}),\n", variant.name, Self::type_name(&ty));
                    }
                }
            }

            format!("    {},\n", variant.name)
        }

        fn write_struct(s: &Struct) -> String {
            let mut buffer = String::new();
            buffer.push_str(&format!(
                "struct {}{} {{\n{}}}\n",
                s.name,
                Self::generics_decl(s.generics),
                s.fields
                    .iter()
                    .map(|f| {
                        let ty = f.ty.expect("struct fields must have a type");
                        format!("    {}: {},\n", f.name, Self::type_name(&ty()))
                    })
                    .collect::<Vec<String>>()
                    .join("")
            ));

            buffer.push_str(&Self::write_functions(s.functions, s.name, s.generics));

            buffer
        }

        fn write_functions(functions: &[Function], ty: &str, generics: &[Generic]) -> String {
            let mut buffer = String::new();

            if !functions.is_empty() {
                buffer.push_str(&format!(
                    "impl{} {}{} {{\n",
                    Self::generics_decl(generics),
                    ty,
                    Self::generics(generics)
                ));

                for f in functions {
                    buffer.push_str(&format!(
                        "    pub {}fn {}{}({}){} {{ todo!() }}\n",
                        if f.async_fn { "async " } else { "" },
                        f.name,
                        Self::generics_decl(f.generics),
                        f.args
                            .iter()
                            .map(|arg| {
                                let ty = if let Some(ty_fn) = &arg.ty {
                                    format!(": {}", Self::type_name(&ty_fn()))
                                } else {
                                    String::new()
                                };
                                format!("{}{}", arg.name, ty)
                            })
                            .collect::<Vec<String>>()
                            .join(", "),
                        if let Some(ret_ty_fn) = &f.ret {
                            if f.name == "search" {
                                let x = ret_ty_fn().name();
                                println!("Function {ty}::{} returns {}", f.name, x);
                            }

                            format!(" -> {}", Self::type_name(&ret_ty_fn()))
                        } else {
                            String::new()
                        }
                    ));
                }

                buffer.push_str("}\n");
            }

            buffer
        }

        fn write_tuple_struct(t: &TupleStruct) -> String {
            let mut buffer = String::new();
            buffer.push_str(&format!(
                "struct {}{}({});\n",
                t.name,
                Self::generics_decl(t.generics),
                t.fields
                    .iter()
                    .map(|f| Self::type_name(&f()))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
            buffer
        }

        fn preamble() -> String {
            r#"use serde::Serialize;
use serde::de::DeserializeOwned;
use agdb::DbType;
use std::borrow::Borrow;

type AgdbApiResult<T> = Result<T, AgdbApiError>;

pub trait AgdbApiClient {}

pub trait HttpClient {
    fn delete(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<u16>> + Send;
    fn get<T: DeserializeOwned + Send>(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<(u16, T)>> + Send;
    fn post<T: Serialize + Send, R: DeserializeOwned + Send>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<(u16, R)>> + Send;
    fn put<T: Serialize + Send>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<u16>> + Send;
}

pub trait SearchQueryBuilder: agdb::api_def::TypeDefinition {
    fn search_mut(&mut self) -> &mut SearchQuery;
}

"#
            .to_string()
        }
    }

    #[test]
    fn rust_api() {
        let code = RustApi::generate();
        std::fs::write("../../examples/generated_api/src/lib.rs", code).unwrap();
    }
}
