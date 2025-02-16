use agdb::api::Enum;
use agdb::api::Expression;
use agdb::api::Function;
use agdb::api::NamedType;
use agdb::api::Struct;
use agdb::api::Type;

use crate::language::Language;
use crate::utilities;
use crate::CIError;
use std::path::Path;
use std::process::Command;

pub(crate) fn update_version(
    toml: &Path,
    current_version: &str,
    new_version: &str,
) -> Result<(), CIError> {
    let mut content = std::fs::read_to_string(toml)?.replace(
        &format!("\nversion = \"{current_version}\""),
        &format!("\nversion = \"{new_version}\""),
    );
    for line in content.clone().lines() {
        let line = line.trim();
        if line.starts_with("agdb") {
            content = content.replace(line, &line.replace(current_version, new_version));
        }
    }
    std::fs::write(toml, content)?;

    Ok(())
}

pub(crate) fn generate_test_queries() -> Result<(), CIError> {
    println!("Generating test_queries.json");
    utilities::run_command(
        Command::new("cargo")
            .arg("test")
            .arg("-p")
            .arg("agdb_server")
            .arg("tests::test_queries")
            .arg("--")
            .arg("--exact"),
    )?;
    Ok(())
}

pub(crate) fn generate_api() -> Result<(), CIError> {
    println!("Generating openapi.json");
    utilities::run_command(
        Command::new("cargo")
            .arg("test")
            .arg("-p")
            .arg("agdb_server")
            .arg("api::tests::openapi")
            .arg("--")
            .arg("--exact"),
    )?;
    Ok(())
}

pub struct Rust;

impl Rust {
    fn generate_unit_struct(s: &Struct) -> String {
        format!("#[derive(Default)]\npub struct {};\n", s.name)
    }

    fn generate_tuple_conversions(s: &Struct) -> String {
        let mut buf = String::new();
        let name = s.name;
        let field = &s.fields[0];

        if let Type::String = (field.ty)() {
            buf.push_str(&format!(
                    "impl From<&String> for {name} {{\n    fn from(value: &String) -> Self {{\n        Self(value.to_string())\n    }}\n}}\n",
                ));
            buf.push_str(&format!(
                    "impl From<&str> for {name} {{\n    fn from(value: &str) -> Self {{\n        Self(value.to_string())\n    }}\n}}\n",
                ));
        } else if let Type::List(l) = (field.ty)() {
            if let Type::String = *l {
                // Single String/&String/&str to Vec
                buf.push_str(&format!(
                    "impl From<String> for {name} {{\n    fn from(value: String) -> Self {{\n        Self(vec![value])\n    }}\n}}\n",
                ));
                buf.push_str(&format!(
                        "impl From<&String> for {name} {{\n    fn from(value: &String) -> Self {{\n        Self(vec![value.to_string()])\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl From<&str> for {name} {{\n    fn from(value: &str) -> Self {{\n        Self(vec![value.to_string()])\n    }}\n}}\n",
                    ));

                // Array of String/&String/&str to Vec
                buf.push_str(&format!(
                        "impl<const N: usize> From<[String; N]> for {name} {{\n    fn from(value: [String; N]) -> Self {{\n        Self(value.to_vec())\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl<const N: usize> From<[&String; N]> for {name} {{\n    fn from(value: [&String; N]) -> Self {{\n        Self(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl<const N: usize> From<[&str; N]> for {name} {{\n    fn from(value: [&str; N]) -> Self {{\n        Self(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));

                // Slice of String/&String/&str to Vec
                buf.push_str(&format!(
                        "impl From<&[String]> for {name} {{\n    fn from(value: &[String]) -> Self {{\n        Self(value.to_vec())\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl From<&[&String]> for {name} {{\n    fn from(value: &[&String]) -> Self {{\n        Self(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl From<&[&str]> for {name} {{\n    fn from(value: &[&str]) -> Self {{\n        Self(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));

                // Vec of &String/&str to Vec
                buf.push_str(&format!(
                        "impl From<Vec<&str>> for {name} {{\n    fn from(value: Vec<&str>) -> Self {{\n        Self(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl From<Vec<&String>> for {name} {{\n    fn from(value: Vec<&String>) -> Self {{\n        Self(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl From<&Vec<&str>> for {name} {{\n    fn from(value: &Vec<&str>) -> Self {{\n        Self(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl From<&Vec<&String>> for {name} {{\n    fn from(value: &Vec<&String>) -> Self {{\n        Self(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl From<&Vec<String>> for {name} {{\n    fn from(value: &Vec<String>) -> Self {{\n        Self(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
            } else {
                let ty = Self::type_name(&(field.ty)());

                buf.push_str(&format!(
                        "impl<const N: usize> From<[{ty}; N]> for {name} {{\n    fn from(value: [{ty}; N]) -> Self {{\n        Self(value.to_vec())\n    }}\n}}\n",
                    ));
                buf.push_str(&format!(
                        "impl From<&[{ty}]> for {name} {{\n    fn from(value: &[{ty}]) -> Self {{\n        Self(value.to_vec())\n    }}\n}}\n",
                    ));
            }
        }

        buf.push_str(&format!(
                "impl From<{}> for {name} {{\n    fn from(value: {}) -> Self {{\n        Self(value)\n    }}\n}}\n",
                Self::type_name(&(field.ty)()),
                Self::type_name(&(field.ty)()),
            ));

        buf
    }

    fn generate_tuple_struct(s: &Struct) -> String {
        let n = Self::type_name(&(s.fields[0].ty)());
        format!(
            "#[derive(Default)]\npub struct {}(pub {n});\nimpl {} {{\n    pub fn new(arg: {n}) -> Self {{ Self(arg) }}\n}}\n",
            s.name,
            s.name
        )
    }

    fn generate_named_struct(s: &Struct) -> String {
        let mut buf = String::new();
        buf.push_str(&format!("#[derive(Default)]\npub struct {} {{\n", s.name));
        for field in &s.fields {
            buf.push_str(&format!(
                "    pub {}: {},\n",
                field.name,
                Self::type_name(&(field.ty)())
            ));
        }
        buf.push_str("}\n");
        buf
    }

    fn generate_expressions(expressions: &[Expression]) -> String {
        let mut buf = String::new();

        for e in expressions {
            match e {
                Expression::Create { ty } => {
                    buf.push_str(
                        format!(
                            "        let mut {} = {}::default();\n",
                            ty.name,
                            Self::type_name(&(ty.ty)())
                        )
                        .as_str(),
                    );
                }
                Expression::CreateArg { ty, arg } => {
                    buf.push_str(
                        format!(
                            "        let mut {} = {}::new({});\n",
                            ty.name,
                            Self::type_name(&(ty.ty)()),
                            if *arg == "." { "self.0" } else { arg },
                        )
                        .as_str(),
                    );
                }
                Expression::Assign {
                    object,
                    fields,
                    value,
                } => {
                    buf.push_str(
                        format!(
                            "        {}.{} = {};\n",
                            if object.is_empty() { "self" } else { object },
                            fields
                                .iter()
                                .map(|f| if *f == "." { "0" } else { f })
                                .collect::<Vec<&str>>()
                                .join("."),
                            value
                        )
                        .as_str(),
                    );
                }
                Expression::Return(var) => {
                    buf.push_str(
                        format!("        {}\n", if *var == "." { "self.0" } else { var }).as_str(),
                    );
                }
                Expression::CreateReturn { ty } => {
                    buf.push_str(
                        format!("        {}::default()\n", Self::type_name(&ty())).as_str(),
                    );
                }
                Expression::CreateReturnArg { ty, arg } => {
                    buf.push_str(format!("        {}({arg})\n", Self::type_name(&ty())).as_str());
                }
                Expression::CreateReturnArgT { ty, arg } => {
                    buf.push_str(
                        format!("        {}({arg}.into())\n", Self::type_name(&ty())).as_str(),
                    );
                }
            }
        }

        buf
    }

    fn generate_function_args(args: &[NamedType]) -> String {
        args.iter()
            .map(|a| {
                let ty = (a.ty)();
                match ty {
                    Type::Enum(_) => format!("{}: T", a.name),
                    Type::Struct(s) if s.fields.len() == 1 && s.fields[0].name.is_empty() => {
                        format!("{}: T", a.name)
                    }
                    Type::List(_) => format!("{}: T", a.name),
                    Type::None => a.name.to_string(),
                    _ => format!("{}: {}", a.name, Self::type_name(&ty)),
                }
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn generate_generics(args: &[NamedType]) -> String {
        args.iter()
            .find_map(|a| {
                if let Type::Enum(_) = (a.ty)() {
                    return Some(format!("<T: Into<{}>>", Self::type_name(&(a.ty)())));
                } else if let Type::Struct(s) = (a.ty)() {
                    if let Some(f) = s.fields.first() {
                        if f.name.is_empty() {
                            return Some(format!("<T: Into<{}>>", Self::type_name(&(a.ty)())));
                        }
                    }
                } else if let Type::List(_) = (a.ty)() {
                    return Some(format!("<T: Into<{}>>", Self::type_name(&(a.ty)())));
                }

                None
            })
            .unwrap_or_default()
    }

    fn generate_functions(functions: &[Function]) -> String {
        let mut buf = String::new();

        for function in functions {
            let mut generics = Self::generate_generics(&function.args);
            let args = Self::generate_function_args(&function.args);

            if let Some(first_arg) = function.args.first() {
                if let Type::Enum(e) = (first_arg.ty)() {
                    generics.push_str(format!("<T: Into<{}>>", e.name).as_str());
                }
            }

            buf.push_str(format!("    pub fn {}{}(", function.name, generics).as_str());
            buf.push_str(args.as_str());
            buf.push_str(format!(") -> {} {{\n", Self::type_name(&(function.ret)())).as_str());
            buf.push_str(&Self::generate_expressions(&function.expressions));
            buf.push_str("    }\n");
        }

        buf
    }

    fn generate_struct(s: &Struct) -> String {
        let mut buf = if s.fields.is_empty() {
            Self::generate_unit_struct(s)
        } else if s.fields.len() == 1 && s.fields[0].name.is_empty() {
            format!(
                "{}{}",
                Self::generate_tuple_struct(s),
                Self::generate_tuple_conversions(s)
            )
        } else {
            Self::generate_named_struct(s)
        };

        if !s.functions.is_empty() {
            buf.push_str(format!("impl {} {{\n", s.name).as_str());
            buf.push_str(&Self::generate_functions(&s.functions));
            buf.push_str("}\n\n");
        }

        buf
    }

    fn enum_variant_unit(variant: &NamedType) -> String {
        format!("    {},\n", variant.name)
    }

    fn enum_variant(variant: &NamedType) -> String {
        format!(
            "    {}({}),\n",
            variant.name,
            Self::type_name(&(variant.ty)())
        )
    }

    fn generate_enum_variants(e: &Enum) -> String {
        let mut buf = String::new();
        for variant in &e.variants {
            if let Type::None = (variant.ty)() {
                buf.push_str(&Self::enum_variant_unit(variant));
            } else {
                buf.push_str(&Self::enum_variant(variant));
            }
        }
        buf
    }

    fn generate_enum_default_impl(e: &Enum) -> String {
        let mut buf = format!(
            "impl Default for {} {{\n    fn default() -> Self {{\n        Self::",
            e.name
        );

        let first_variant = e
            .variants
            .first()
            .expect("Enum must have at least one variant");

        if let Type::None = (first_variant.ty)() {
            buf.push_str(&format!("{}\n", first_variant.name));
        } else {
            buf.push_str(&format!("{}(Default::default())\n", first_variant.name));
        }

        buf.push_str("      }\n}\n");
        buf
    }

    fn generate_enum_conversions(e: &Enum) -> String {
        let mut buf = String::new();

        for variant in &e.variants {
            if let Type::None = (variant.ty)() {
                continue;
            }

            let e = e.name;
            let v = variant.name;

            if let Type::String = (variant.ty)() {
                buf.push_str(&format!(
                    "impl From<&String> for {e} {{\n    fn from(value: &String) -> Self {{\n        Self::{v}(value.to_string())\n    }}\n}}\n",
                ));
                buf.push_str(&format!(
                    "impl From<&str> for {e} {{\n    fn from(value: &str) -> Self {{\n        Self::{v}(value.to_string())\n    }}\n}}\n",
                ));
            } else if let Type::List(l) = (variant.ty)() {
                if let Type::String = *l {
                    // Single String/&String/&str to Vec
                    buf.push_str(&format!(
                        "impl From<&String> for {e} {{\n    fn from(value: &String) -> Self {{\n        Self::{v}(vec![value.to_string()])\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl From<&str> for {e} {{\n    fn from(value: &str) -> Self {{\n        Self::{v}(vec![value.to_string()])\n    }}\n}}\n",
                    ));

                    // Array of String/&String/&str to Vec
                    buf.push_str(&format!(
                        "impl<const N: usize> From<[String; N]> for {e} {{\n    fn from(value: [String; N]) -> Self {{\n        Self::{v}(value.to_vec())\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl<const N: usize> From<[&String; N]> for {e} {{\n    fn from(value: [&String; N]) -> Self {{\n        Self::{v}(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl<const N: usize> From<[&str; N]> for {e} {{\n    fn from(value: [&str; N]) -> Self {{\n        Self::{v}(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));

                    // Slice of String/&String/&str to Vec
                    buf.push_str(&format!(
                        "impl From<&[String]> for {e} {{\n    fn from(value: &[String]) -> Self {{\n        Self::{v}(value.to_vec())\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl From<&[&String]> for {e} {{\n    fn from(value: &[&String]) -> Self {{\n        Self::{v}(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl From<&[&str]> for {e} {{\n    fn from(value: &[&str]) -> Self {{\n        Self::{v}(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));

                    // Vec of &String/&str to Vec
                    buf.push_str(&format!(
                        "impl From<Vec<&str>> for {e} {{\n    fn from(value: Vec<&str>) -> Self {{\n        Self::{v}(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl From<Vec<&String>> for {e} {{\n    fn from(value: Vec<&String>) -> Self {{\n        Self::{v}(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl From<&Vec<&str>> for {e} {{\n    fn from(value: &Vec<&str>) -> Self {{\n        Self::{v}(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl From<&Vec<&String>> for {e} {{\n    fn from(value: &Vec<&String>) -> Self {{\n        Self::{v}(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl From<&Vec<String>> for {e} {{\n    fn from(value: &Vec<String>) -> Self {{\n        Self::{v}(value.iter().map(|s| s.to_string()).collect())\n    }}\n}}\n",
                    ));
                } else {
                    let ty = Self::type_name(&(variant.ty)());

                    buf.push_str(&format!(
                        "impl<const N: usize> From<[{ty}; N]> for {e} {{\n    fn from(value: [{ty}; N]) -> Self {{\n        Self::{v}(value.to_vec())\n    }}\n}}\n",
                    ));
                    buf.push_str(&format!(
                        "impl From<&[{ty}]> for {e} {{\n    fn from(value: &[{ty}]) -> Self {{\n        Self::{v}(value.to_vec())\n    }}\n}}\n",
                    ));
                }
            }

            buf.push_str(&format!(
                "impl From<{}> for {} {{\n    fn from(value: {e}) -> Self {{\n        Self::{v}(value)\n    }}\n}}\n",
                Self::type_name(&(variant.ty)()),
                Self::type_name(&(variant.ty)()),
            ));
        }

        buf
    }

    fn generate_enum(e: &Enum) -> String {
        let mut buf = format!("pub enum {} {{\n", e.name);
        buf.push_str(&Self::generate_enum_variants(e));
        buf.push_str("}\n");
        buf.push_str(&Self::generate_enum_default_impl(e));
        buf.push_str(&Self::generate_enum_conversions(e));
        buf
    }
}

impl Language for Rust {
    fn generate_type(ty: &Type) -> String {
        match ty {
            Type::Struct(s) => Self::generate_struct(s),
            Type::Enum(e) => Self::generate_enum(e),
            _ => String::new(),
        }
    }

    fn type_name(ty: &Type) -> String {
        match ty {
            Type::None => "()".to_string(),
            Type::U8 => "u8".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U64 => "u64".to_string(),
            Type::F64 => "f64".to_string(),
            Type::String => "String".to_string(),
            Type::Enum(e) => e.name.to_string(),
            Type::Struct(s) => s.name.to_string(),
            Type::List(l) => format!("Vec<{}>", Self::type_name(l)),
            Type::Option(o) => format!("Option<{}>", Self::type_name(o)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::api::ApiDefinition;
    use agdb::DbId;
    use agdb::DbKeyValue;
    use agdb::QueryBuilder;
    use agdb::QueryConditionLogic;
    use agdb::QueryId;

    #[test]
    fn test_generate_empty_struct() {
        let generated = Rust::generate_type(&QueryBuilder::def());
        let expected = "pub struct QueryBuilder;\n";
        assert_eq!(generated, expected);
    }

    #[test]
    fn test_generate_unit_struct() {
        let generated = Rust::generate_type(&DbId::def());
        let expected = "pub struct DbId(i64);\n";
        assert_eq!(generated, expected);
    }

    #[test]
    fn test_generate_struct() {
        let generated = Rust::generate_type(&DbKeyValue::def());
        let expected =
            "pub struct DbKeyValue {\n    pub key: DbValue,\n    pub value: DbValue,\n}\n";
        assert_eq!(generated, expected);
    }

    #[test]
    fn test_generate_enum() {
        let generated = Rust::generate_type(&QueryId::def());
        let expected = "pub enum QueryId {\n    Id(DbId),\n    Alias(String),\n}\n";
        assert_eq!(generated, expected);
    }

    #[test]
    fn test_generate_enum_no_types() {
        let generated = Rust::generate_type(&QueryConditionLogic::def());
        let expected = "pub enum QueryConditionLogic {\n    And,\n    Or,\n}\n";
        assert_eq!(generated, expected);
    }

    #[test]
    fn query_builder() {
        let rust = crate::language::rust::Rust::generate_type(&crate::api::QueryBuilder::def());
        assert_eq!(
            rust,
            "#[derive(Default)]
pub struct QueryBuilder;
impl QueryBuilder {
    pub fn insert() -> Insert {
        Insert::default()
    }
}

"
        );
    }

    #[test]
    fn insert() {
        let rust = crate::language::rust::Rust::generate_type(&crate::api::Insert::def());
        assert_eq!(
            rust,
            "#[derive(Default)]
pub struct Insert;
impl Insert {
    pub fn aliases<T: Into<QueryAliases>>(self, names: T) -> InsertAliases {
        InsertAliases(names.into())
    }
}

"
        );
    }

    #[test]
    fn api() {
        use crate::api;

        let types = vec![
            api::QueryBuilder::def(),
            api::Insert::def(),
            api::QueryAliases::def(),
            api::InsertAliases::def(),
        ];

        let mut buf = String::new();

        for ty in types {
            buf.push_str(&crate::language::rust::Rust::generate_type(&ty));
        }

        std::fs::write("src/api_gen.rs", buf).unwrap();
    }
}
