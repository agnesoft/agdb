use agdb::api::Type;

use crate::languages::Language;
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

struct Rust;

impl Language for Rust {
    fn generate_type(ty: &Type) -> String {
        let mut buf = String::new();

        match ty {
            Type::Struct(s) => {
                if s.fields.is_empty() {
                    buf = format!("pub struct {};\n", s.name);
                } else if s.fields.len() == 1 && s.fields[0].name.is_empty() {
                    buf = format!(
                        "pub struct {}({});\n",
                        s.name,
                        Self::type_name(&s.fields[0].ty)
                    );
                } else {
                    buf.push_str(&format!("pub struct {} {{\n", s.name));
                    for field in &s.fields {
                        buf.push_str(&format!(
                            "    pub {}: {},\n",
                            field.name,
                            Self::type_name(&field.ty)
                        ));
                    }
                    buf.push_str("}\n");
                }
            }
            Type::Enum(e) => {
                buf.push_str(&format!("pub enum {} {{\n", e.name));
                for variant in &e.variants {
                    if let Type::None = variant.ty {
                        buf.push_str(&format!("    {},\n", variant.name));
                    } else {
                        buf.push_str(&format!(
                            "    {}({}),\n",
                            variant.name,
                            Self::type_name(&variant.ty)
                        ));
                    }
                }
                buf.push_str("}\n");
            }
            _ => {}
        }

        buf
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
}
