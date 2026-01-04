use crate::AgdbApi;
use crate::ReqwestClient;
use agdb::QueryBuilder;
use agdb::api_def::Enum;
use agdb::api_def::Expression;
use agdb::api_def::Function;
use agdb::api_def::Generic;
use agdb::api_def::NamedType;
use agdb::api_def::Struct;
use agdb::api_def::Tuple;
use agdb::api_def::Type;
use agdb::api_def::TypeDefinition;

pub trait Language {
    fn preamble() -> String {
        String::new()
    }

    fn enum_begin(ty: &str) -> String;
    fn enum_variant(name: &str, ty: Option<String>) -> String;
    fn enum_end() -> String;

    fn struct_begin(ty: &str) -> String;
    fn struct_end() -> String;

    fn tuple_begin(ty: &str) -> String;
    fn tuple_end() -> String;

    fn impl_begin(ty: &str) -> String;
    fn function_begin(
        name: &str,
        generics: &[Generic],
        args: &[(String, String)],
        ret_ty: Option<String>,
    ) -> String;
    fn expression(expr: &Expression) -> String;
    fn impl_end() -> String;

    fn named_type(name: &str, ty: &str) -> String;
    fn field_separator() -> &'static str {
        ", "
    }
    fn ty(name: &str, generics: &[Generic]) -> String;
    fn ty_declaration(name: &str, generics: &[Generic]) -> String {
        Self::ty(name, generics)
    }

    // --- default implementations ---

    fn write_struct(s: &Struct) -> String {
        let mut buffer = String::new();
        buffer.push_str(&Self::struct_begin(&Self::ty_declaration(
            s.name, s.generics,
        )));
        buffer.push_str(
            &s.fields
                .iter()
                .map(|f| Self::write_struct_field(f))
                .collect::<Vec<_>>()
                .join(&format!("{}\n", Self::field_separator())),
        );
        buffer.push_str(&Self::struct_end());
        buffer.push_str(&Self::impl_begin(&Self::ty_declaration(s.name, s.generics)));

        for f in s.functions {
            buffer.push_str(&Self::write_function(f));
        }

        buffer.push_str(&Self::impl_end());
        buffer
    }

    fn write_struct_field(field: &NamedType) -> String {
        let mut buffer = String::new();
        let ty = field.ty.expect("struct field type cannot be None")();
        buffer.push_str("    ");
        buffer.push_str(&Self::named_type(
            field.name,
            &Self::ty(ty.name(), ty.generics()),
        ));
        buffer
    }

    fn write_enum(e: &Enum) -> String {
        let mut buffer = String::new();
        buffer.push_str(&Self::enum_begin(&Self::ty_declaration(e.name, e.generics)));
        buffer.push_str(
            &e.variants
                .iter()
                .map(|v| Self::write_enum_variant(v))
                .collect::<Vec<_>>()
                .join(&format!("{}\n", Self::field_separator())),
        );

        buffer.push_str(&Self::enum_end());
        buffer
    }

    fn write_enum_variant(variant: &NamedType) -> String {
        let mut buffer = String::new();
        let ty = variant.ty.map(|ty| {
            let ty = ty();
            Self::ty(ty.name(), ty.generics())
        });
        buffer.push_str(&Self::enum_variant(variant.name, ty));
        buffer
    }

    fn write_tuple(t: &Tuple) -> String {
        let mut buffer = String::new();
        buffer.push_str(&Self::tuple_begin(&Self::ty_declaration(
            t.name, t.generics,
        )));

        buffer.push_str(
            &t.fields
                .iter()
                .map(|f| f().name())
                .collect::<Vec<_>>()
                .join(Self::field_separator()),
        );

        buffer.push_str(&Self::tuple_end());
        buffer.push_str(&Self::impl_begin(&Self::ty_declaration(t.name, t.generics)));

        for f in t.functions {
            buffer.push_str(&Self::write_function(f));
        }

        buffer.push_str(&Self::impl_end());
        buffer
    }

    fn write_function(f: &Function) -> String {
        let mut buffer = String::new();
        let args: Vec<(String, String)> = f
            .args
            .iter()
            .map(|arg| {
                let ty = arg.ty.expect("function argument type cannot be None")();
                (arg.name.to_string(), Self::ty(ty.name(), ty.generics()))
            })
            .collect();

        let ret_ty = if let Some(ret_ty_fn) = &f.ret {
            let ty = ret_ty_fn();
            Some(Self::ty(ty.name(), ty.generics()))
        } else {
            None
        };

        buffer.push_str(&Self::function_begin(f.name, f.generics, &args, ret_ty));

        for e in f.expressions {
            buffer.push_str(&Self::expression(e));
            buffer.push('\n');
        }

        buffer.push_str("    ");
        buffer.push_str(&Self::impl_end());
        buffer.push('\n');
        buffer
    }

    fn write_type(ty: &Type) -> String {
        match ty {
            Type::Enum(e) => Self::write_enum(e),
            Type::Struct(s) => Self::write_struct(s),
            Type::Tuple(t) => Self::write_tuple(t),
        }
    }
}

pub struct Api {
    pub buffer: String,
}

impl Api {
    pub fn generate<T: Language>() -> Api {
        let mut api = Api {
            buffer: String::new(),
        };

        api.buffer.push_str(&T::preamble());

        for ty in Api::types() {
            api.buffer.push_str(&T::write_type(&ty));
            api.buffer.push('\n');
        }

        api
    }

    fn types() -> Vec<Type> {
        let top_level_types = vec![
            QueryBuilder::type_def(),
            AgdbApi::<ReqwestClient>::type_def(),
        ];
        top_level_types
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RustApi;

    impl RustApi {
        fn generics_raw(generics: &[Generic]) -> String {
            if generics.is_empty() {
                String::new()
            } else {
                let generics = generics
                    .iter()
                    .map(|g| g.name)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{generics}")
            }
        }

        fn generics(generics: &[Generic]) -> String {
            let raw = Self::generics_raw(generics);
            if raw.is_empty() {
                String::new()
            } else {
                format!("<{raw}>")
            }
        }

        fn generics_decl(generics: &[Generic]) -> String {
            if generics.is_empty() {
                String::new()
            } else {
                let decls = generics
                    .iter()
                    .map(|g| {
                        format!(
                            "{}{}",
                            g.name,
                            if g.bounds.is_empty() {
                                String::new()
                            } else {
                                format!(": {}", g.bounds.join(" + "))
                            }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("<{decls}>")
            }
        }
    }

    impl Language for RustApi {
        fn preamble() -> String {
            r#"
use serde::Serialize;
use serde::de::DeserializeOwned;

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

"#
            .to_string()
        }

        fn enum_begin(ty: &str) -> String {
            format!("enum {ty} {{\n")
        }

        fn enum_variant(name: &str, ty: Option<String>) -> String {
            format!(
                "    {name}{},\n",
                ty.map_or("".to_string(), |t| format!("({})", t))
            )
        }

        fn enum_end() -> String {
            "}\n\n".to_string()
        }

        fn struct_begin(ty: &str) -> String {
            format!("struct {ty} {{\n")
        }

        fn struct_end() -> String {
            "}\n\n".to_string()
        }

        fn tuple_begin(ty: &str) -> String {
            format!("struct {ty}(\n")
        }

        fn tuple_end() -> String {
            ");\n\n".to_string()
        }

        fn impl_begin(ty: &str) -> String {
            format!("impl {ty} {{\n")
        }

        fn impl_end() -> String {
            "}\n".to_string()
        }

        fn expression(expr: &Expression) -> String {
            String::new()
        }

        fn named_type(name: &str, ty: &str) -> String {
            format!("{name}: {ty}")
        }

        fn function_begin(
            name: &str,
            generics: &[Generic],
            args: &[(String, String)],
            ret_ty: Option<String>,
        ) -> String {
            let args = args
                .iter()
                .map(|(name, ty)| Self::named_type(name, ty))
                .collect::<Vec<_>>()
                .join(", ");
            let ret = ret_ty.map(|ty| format!(" -> {ty}")).unwrap_or_default();
            format!(
                "    pub fn {name}{}({args}){ret} {{\n",
                Self::generics_decl(generics)
            )
        }

        fn ty(name: &str, generics: &[Generic]) -> String {
            if name == "Slice" {
                format!("&[{}]", Self::generics_raw(generics))
            } else {
                format!("{name}{}", Self::generics(generics))
            }
        }

        fn ty_declaration(name: &str, generics: &[Generic]) -> String {
            format!("{name}{}", Self::generics_decl(generics))
        }
    }

    #[test]
    fn rust_api() {
        let code = Api::generate::<RustApi>();
        std::fs::write("../../examples/generated_api/src/lib.rs", code.buffer).unwrap();
        //assert!(!code.buffer.is_empty());
    }
}
