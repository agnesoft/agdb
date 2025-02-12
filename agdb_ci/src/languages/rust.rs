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
                    buf = format!("#[derive(Default)]\npub struct {};\n", s.name);
                } else if s.fields.len() == 1 && s.fields[0].name.is_empty() {
                    let n = Self::type_name(&(s.fields[0].ty)());
                    buf = format!(
                        "#[derive(Default)]\npub struct {}({n});\nimpl {} {{\n    pub fn new(arg: {n}) -> Self {{ Self(arg) }}\n}}\n",
                        s.name,
                        s.name
                    );
                } else {
                    buf.push_str(&format!("#[derive(Default)]\npub struct {} {{\n", s.name));
                    for field in &s.fields {
                        buf.push_str(&format!(
                            "    pub {}: {},\n",
                            field.name,
                            Self::type_name(&(field.ty)())
                        ));
                    }
                    buf.push_str("}\n");
                }

                if !s.functions.is_empty() {
                    buf.push_str(format!("impl {} {{\n", s.name).as_str());

                    for function in &s.functions {
                        buf.push_str(format!("    pub fn {}(", function.name).as_str());

                        let mut args = vec!["self".to_string()];
                        args.extend(
                            function
                                .args
                                .iter()
                                .map(|a| format!("{}: {},", a.name, Self::type_name(&(a.ty)()))),
                        );
                        buf.push_str(args.join(", ").as_str());
                        buf.push_str(
                            format!(") -> {} {{\n", Self::type_name(&(function.ret)())).as_str(),
                        );

                        for e in &function.expressions {
                            match e {
                                agdb::api::Expression::Create { ty } => {
                                    buf.push_str(
                                        format!(
                                            "        let mut {} = {}::default();\n",
                                            ty.name,
                                            Self::type_name(&(ty.ty)())
                                        )
                                        .as_str(),
                                    );
                                }
                                agdb::api::Expression::CreateArg { ty, arg } => {
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
                                agdb::api::Expression::Assign {
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
                                agdb::api::Expression::Return(var) => {
                                    buf.push_str(
                                        format!(
                                            "        {}\n",
                                            if *var == "." { "self.0" } else { var }
                                        )
                                        .as_str(),
                                    );
                                }
                                agdb::api::Expression::CreateReturn { ty } => {
                                    buf.push_str(
                                        format!("        {}::default()\n", Self::type_name(&ty()))
                                            .as_str(),
                                    );
                                }
                                agdb::api::Expression::CreateReturnArg { ty, arg } => {
                                    buf.push_str(
                                        format!("        {}::new({arg})\n", Self::type_name(&ty()))
                                            .as_str(),
                                    );
                                }
                            }
                        }

                        buf.push_str("    }\n");
                    }

                    buf.push_str("}\n\n");
                }
            }
            Type::Enum(e) => {
                buf.push_str(&format!("pub enum {} {{\n", e.name));
                for variant in &e.variants {
                    if let Type::None = (variant.ty)() {
                        buf.push_str(&format!("    {},\n", variant.name));
                    } else {
                        buf.push_str(&format!(
                            "    {}({}),\n",
                            variant.name,
                            Self::type_name(&(variant.ty)())
                        ));
                    }
                }
                buf.push_str("}\n");
                buf.push_str(
                    format!(
                        "impl Default for {} {{\n    fn default() -> Self {{\n        Self::",
                        e.name
                    )
                    .as_str(),
                );

                let variant = e.variants.first().unwrap();
                if let Type::None = (variant.ty)() {
                    buf.push_str(&format!("{}\n", variant.name));
                } else {
                    buf.push_str(&format!("{}(Default::default())\n", variant.name));
                }
                buf.push_str("      }\n}\n");
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

    #[test]
    fn proof_of_generator() {
        let types = vec![
            agdb::DbValue::def(),
            agdb::DbId::def(),
            agdb::DbF64::def(),
            agdb::Comparison::def(),
            agdb::CountComparison::def(),
            agdb::KeyValueComparison::def(),
            agdb::QueryConditionLogic::def(),
            agdb::QueryConditionModifier::def(),
            agdb::QueryConditionData::def(),
            agdb::SearchQueryAlgorithm::def(),
            agdb::DbKeyOrder::def(),
            agdb::QueryCondition::def(),
            agdb::QueryId::def(),
            agdb::SearchQuery::def(),
            agdb::QueryIds::def(),
            agdb::InsertAliasesQuery::def(),
            agdb::api::builder::QueryBuilder::def(),
            agdb::api::builder::Insert::def(),
            agdb::api::builder::InsertAliases::def(),
            agdb::api::builder::InsertAliasesIds::def(),
        ];

        let mut buf = "mod builder {\n".to_string();

        for ty in types {
            buf.push_str(&Rust::generate_type(&ty));
        }

        buf.push_str("}\n");

        std::fs::write("src/languages/builder.rs", buf).unwrap();
    }
}
