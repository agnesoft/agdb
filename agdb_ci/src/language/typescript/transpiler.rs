mod declarations;
mod expressions;
mod format;
mod normalize;
mod types;

use agdb::type_def::Expression;
use agdb::type_def::Type;

use format::IndentWriter;

pub struct TranspileConfig {
    pub indent: &'static str,
    pub export_declarations: bool,
    pub preamble: &'static str,
    pub skip_types: &'static [&'static str],
}

impl Default for TranspileConfig {
    fn default() -> Self {
        Self {
            indent: "    ",
            export_declarations: true,
            preamble: "",
            skip_types: &[],
        }
    }
}

pub fn transpile_type(ty: &Type, config: &TranspileConfig) -> String {
    let mut w = IndentWriter::new(config.indent);
    declarations::emit_type(ty, config, &mut w);
    w.into_string()
}

pub fn transpile_module(types: &[Type], config: &TranspileConfig) -> String {
    let mut w = IndentWriter::new(config.indent);
    if !config.preamble.is_empty() {
        w.write(config.preamble);
        w.newline();
        w.newline();
    }
    let mut first = true;
    for ty in types {
        if config.skip_types.contains(&ty.name()) {
            continue;
        }
        if matches!(ty, Type::Pointer(_)) {
            continue;
        }
        if !first {
            w.newline();
        }
        first = false;
        declarations::emit_type(ty, config, &mut w);
    }
    w.into_string()
}

pub fn transpile_body(body: &[Expression], config: &TranspileConfig) -> String {
    let mut w = IndentWriter::new(config.indent);
    expressions::emit_body(body, &mut w);
    w.into_string()
}

pub fn type_annotation(ty: &Type) -> String {
    types::type_annotation(ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::type_def::TypeDefinition;

    fn config() -> TranspileConfig {
        TranspileConfig::default()
    }

    #[test]
    fn transpile_simple_struct() {
        #[derive(agdb::TypeDef)]
        struct Item {
            id: i32,
            name: String,
        }

        let output = transpile_type(&Item::type_def(), &config());
        assert!(output.contains("export class Item {"), "Got:\n{output}");
        assert!(output.contains("id: number;"), "Got:\n{output}");
        assert!(output.contains("name: string;"), "Got:\n{output}");
        assert!(
            output.contains("constructor(id: number = 0, name: string = \"\")"),
            "Got:\n{output}"
        );
    }

    #[test]
    fn transpile_function_with_body() {
        #[agdb::fn_def]
        #[allow(unused)]
        fn multiply(a: i32, b: i32) -> i32 {
            a * b
        }

        let ty = __multiply_type_def();
        let output = transpile_type(&ty, &config());
        assert!(
            output.contains("export function multiply(a: number, b: number): number"),
            "Got:\n{output}"
        );
        assert!(output.contains("return a * b;"), "Got:\n{output}");
    }

    #[test]
    fn transpile_enum_unit_variants() {
        #[derive(agdb::TypeDef)]
        #[allow(dead_code)]
        enum Status {
            Active,
            Inactive,
            Pending,
        }

        let output = transpile_type(&Status::type_def(), &config());
        assert!(output.contains("export enum Status {"), "Got:\n{output}");
        assert!(output.contains("Active = \"Active\","), "Got:\n{output}");
        assert!(
            output.contains("Inactive = \"Inactive\","),
            "Got:\n{output}"
        );
        assert!(output.contains("Pending = \"Pending\","), "Got:\n{output}");
    }

    #[test]
    fn transpile_trait_to_interface() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait Printable {
            fn print() -> String;
            fn display(prefix: &str) -> String;
        }

        let output = transpile_type(&PrintableDef::type_def(), &config());
        assert!(
            output.contains("export interface Printable {"),
            "Got:\n{output}"
        );
        assert!(output.contains("print(): string;"), "Got:\n{output}");
        assert!(
            output.contains("display(prefix: string): string;"),
            "Got:\n{output}"
        );
    }

    #[test]
    fn transpile_module_multiple_types() {
        #[derive(agdb::TypeDef)]
        #[allow(dead_code)]
        struct User {
            id: i32,
            email: String,
        }

        #[derive(agdb::TypeDef)]
        #[allow(dead_code)]
        enum Role {
            Admin,
            Member,
        }

        let types = vec![User::type_def(), Role::type_def()];
        let output = transpile_module(&types, &config());
        assert!(output.contains("class User"), "Got:\n{output}");
        assert!(output.contains("enum Role"), "Got:\n{output}");
    }

    #[test]
    fn transpile_body_statements() {
        #[agdb::fn_def]
        #[allow(unused)]
        fn example() {
            let x = 10;
            let y = 20;
            let _sum = x + y;
        }

        let Type::Function(f) = __example_type_def() else {
            panic!("Expected function");
        };
        let output = transpile_body(f.body, &config());
        assert!(output.contains("let x = 10;"), "Got:\n{output}");
        assert!(output.contains("let y = 20;"), "Got:\n{output}");
        assert!(output.contains("let _sum = x + y;"), "Got:\n{output}");
    }

    #[test]
    fn type_annotation_api() {
        assert_eq!(type_annotation(&i32::type_def()), "number");
        assert_eq!(type_annotation(&String::type_def()), "string");
        assert_eq!(type_annotation(&bool::type_def()), "boolean");
        assert_eq!(type_annotation(&Vec::<i32>::type_def()), "number[]");
        assert_eq!(
            type_annotation(&Option::<String>::type_def()),
            "string | null"
        );
    }

    #[test]
    fn option_type_in_struct() {
        #[derive(agdb::TypeDef)]
        struct Config {
            name: String,
            timeout: Option<i32>,
        }

        let output = transpile_type(&Config::type_def(), &config());
        assert!(output.contains("name: string;"), "Got:\n{output}");
        assert!(output.contains("timeout: number | null;"), "Got:\n{output}");
    }

    #[test]
    fn vec_type_in_struct() {
        #[derive(agdb::TypeDef)]
        struct Collection {
            items: Vec<String>,
        }

        let output = transpile_type(&Collection::type_def(), &config());
        assert!(output.contains("items: string[];"), "Got:\n{output}");
    }

    #[test]
    fn result_type_transparent() {
        assert_eq!(
            type_annotation(&Result::<i32, String>::type_def()),
            "number"
        );
    }

    #[test]
    fn reference_types_stripped() {
        assert_eq!(type_annotation(&<&str>::type_def()), "string");
        assert_eq!(type_annotation(&<&i32>::type_def()), "number");
    }

    #[test]
    fn box_types_stripped() {
        assert_eq!(type_annotation(&Box::<String>::type_def()), "string");
    }
}
