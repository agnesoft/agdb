use agdb::type_def::Enum;
use agdb::type_def::Function;
use agdb::type_def::GenericKind;
use agdb::type_def::Impl;
use agdb::type_def::Literal;
use agdb::type_def::Static;
use agdb::type_def::Struct;
use agdb::type_def::Trait;
use agdb::type_def::Type;
use agdb::type_def::Variable;

use super::TranspileConfig;
use super::expressions::emit_body;
use super::expressions::emit_expression;
use super::format::IndentWriter;
use super::normalize::NormalizedType;
use super::normalize::normalize_type;
use super::types::emit_normalized;
use super::types::type_annotation;

pub fn emit_type(ty: &Type, config: &TranspileConfig, w: &mut IndentWriter) {
    match ty {
        Type::Struct(s) => emit_struct(s, config, w),
        Type::Enum(e) => emit_enum(e, config, w),
        Type::Trait(t) => emit_trait(t, config, w),
        Type::Function(f) => emit_function(f, config, w),
        Type::Test(f) => emit_test_function(f, config, w),
        Type::Impl(i) => emit_impl_standalone(i, config, w),
        Type::Static(s) => emit_static(s, config, w),
        _ => {}
    }
}

fn emit_struct(s: &Struct, config: &TranspileConfig, w: &mut IndentWriter) {
    let export = if config.export_declarations {
        "export "
    } else {
        ""
    };
    let generics = generic_params_from_slice(s.generics);

    let impls = (s.impl_defs)();
    let trait_impls: Vec<&Impl> = impls.iter().filter(|i| i.trait_.is_some()).collect();

    let implements = if trait_impls.is_empty() {
        String::new()
    } else {
        let names: Vec<String> = trait_impls
            .iter()
            .filter_map(|i| {
                i.trait_.map(|t| {
                    let ty = t();
                    ty.name().to_string()
                })
            })
            .collect();
        format!(" implements {}", names.join(", "))
    };

    w.write_line(&format!(
        "{export}class {}{generics}{implements} {{",
        s.name
    ));
    w.indent();

    for field in s.fields {
        emit_field(field, w);
    }

    if !s.fields.is_empty() {
        w.newline();
        emit_constructor(s.fields, w);
    }

    for imp in &impls {
        for func in imp.functions {
            w.newline();
            emit_method(func, w);
        }
    }

    w.dedent();
    w.write_line("}");
}

fn emit_field(field: &Variable, w: &mut IndentWriter) {
    if let Some(ty_fn) = field.ty {
        let ty = ty_fn();
        let ann = type_annotation(&ty);
        if field.name.is_empty() {
            w.write_line(&format!("value: {ann};"));
        } else {
            w.write_line(&format!("{}: {ann};", field.name));
        }
    }
}

fn emit_constructor(fields: &[Variable], w: &mut IndentWriter) {
    let params: Vec<String> = fields
        .iter()
        .filter_map(|f| {
            let name = if f.name.is_empty() { "value" } else { f.name };
            f.ty.map(|ty_fn| {
                let ann = type_annotation(&ty_fn());
                format!("{name}: {ann}")
            })
        })
        .collect();

    w.write_line(&format!("constructor({}) {{", params.join(", ")));
    w.indent();
    for field in fields {
        let name = if field.name.is_empty() {
            "value"
        } else {
            field.name
        };
        if field.ty.is_some() {
            w.write_line(&format!("this.{name} = {name};"));
        }
    }
    w.dedent();
    w.write_line("}");
}

fn emit_method(func: &Function, w: &mut IndentWriter) {
    let args_to_emit: Vec<&Variable> = func
        .args
        .iter()
        .filter(|a| a.name != "self" && a.name != "&self")
        .filter(|a| {
            if let Some(ty_fn) = a.ty {
                !matches!(ty_fn(), Type::SelfType(_) | Type::Reference(_) if a.name.is_empty())
            } else {
                true
            }
        })
        .collect();

    let is_self_param = func.args.first().is_some_and(|a| {
        a.name == "self"
            || a.name == "&self"
            || a.ty
                .is_some_and(|ty_fn| matches!(ty_fn(), Type::SelfType(_) | Type::Reference(_)))
    });
    let is_static = !is_self_param;

    let async_prefix = if func.async_fn { "async " } else { "" };
    let static_prefix = if is_static { "static " } else { "" };

    let params = format_params(&args_to_emit);
    let ret_annotation = format_return_type(func);
    let generics = generic_params_from_slice(func.generics);

    w.write_line(&format!(
        "{static_prefix}{async_prefix}{}{generics}({params}){ret_annotation} {{",
        func.name
    ));
    w.indent();
    emit_body(func.body, w);
    w.dedent();
    w.write_line("}");
}

fn emit_enum(e: &Enum, config: &TranspileConfig, w: &mut IndentWriter) {
    let export = if config.export_declarations {
        "export "
    } else {
        ""
    };
    let generics = generic_params_from_slice(e.generics);

    let all_unit = e.variants.iter().all(|v| {
        v.ty.is_none_or(|ty_fn| matches!(ty_fn(), Type::Literal(Literal::Unit)))
    });

    if all_unit {
        w.write_line(&format!("{export}enum {}{generics} {{", e.name));
        w.indent();
        for variant in e.variants {
            w.write_line(&format!("{} = \"{}\",", variant.name, variant.name));
        }
        w.dedent();
        w.write_line("}");
    } else {
        w.write(&format!("{export}type {}{generics} =", e.name));
        w.newline();
        w.indent();
        for (i, variant) in e.variants.iter().enumerate() {
            let prefix = if i == 0 { "" } else { " | " };
            if let Some(ty_fn) = variant.ty {
                let ty = ty_fn();
                match &ty {
                    Type::Literal(Literal::Unit) => {
                        w.write_line(&format!("{prefix}{{ type: \"{}\"; }}", variant.name));
                    }
                    Type::Struct(inner) => {
                        w.write(&format!("{prefix}{{ type: \"{}\"", variant.name));
                        for field in inner.fields {
                            if let Some(fty_fn) = field.ty {
                                let ann = type_annotation(&fty_fn());
                                w.write(&format!("; {}: {ann}", field.name));
                            }
                        }
                        w.write_line("; }");
                    }
                    Type::Tuple(elements) => {
                        let types: Vec<String> =
                            elements.iter().map(|f| type_annotation(&f())).collect();
                        w.write_line(&format!(
                            "{prefix}{{ type: \"{}\"; value: [{}]; }}",
                            variant.name,
                            types.join(", ")
                        ));
                    }
                    _ => {
                        let ann = type_annotation(&ty);
                        w.write_line(&format!(
                            "{prefix}{{ type: \"{}\"; value: {ann}; }}",
                            variant.name
                        ));
                    }
                }
            } else {
                w.write_line(&format!("{prefix}{{ type: \"{}\"; }}", variant.name));
            }
        }
        w.dedent();
        w.newline();
    }
}

fn emit_trait(t: &Trait, config: &TranspileConfig, w: &mut IndentWriter) {
    let export = if config.export_declarations {
        "export "
    } else {
        ""
    };
    let generics = generic_params_from_slice(t.generics);

    let extends = if t.bounds.is_empty() {
        String::new()
    } else {
        let bound_names: Vec<String> = t
            .bounds
            .iter()
            .map(|b| {
                let ty = b();
                ty.name().to_string()
            })
            .collect();
        format!(" extends {}", bound_names.join(", "))
    };

    w.write_line(&format!(
        "{export}interface {}{generics}{extends} {{",
        t.name
    ));
    w.indent();

    for func in t.functions {
        emit_interface_method(func, w);
    }

    w.dedent();
    w.write_line("}");
}

fn emit_interface_method(func: &Function, w: &mut IndentWriter) {
    let args_to_emit: Vec<&Variable> = func
        .args
        .iter()
        .filter(|a| a.name != "self" && a.name != "&self")
        .filter(|a| {
            !a.ty
                .is_some_and(|ty_fn| matches!(ty_fn(), Type::SelfType(_) | Type::Reference(_)))
                || !a.name.is_empty()
        })
        .filter(|a| {
            if let Some(ty_fn) = a.ty {
                !matches!(ty_fn(), Type::SelfType(_))
            } else {
                true
            }
        })
        .collect();

    let params = format_params(&args_to_emit);
    let ret_annotation = format_return_type(func);
    let generics = generic_params_from_slice(func.generics);

    if func.body.is_empty() {
        w.write_line(&format!(
            "{}{}({params}){ret_annotation};",
            func.name, generics
        ));
    } else {
        w.write_line(&format!(
            "{}{}({params}){ret_annotation} {{",
            func.name, generics
        ));
        w.indent();
        emit_body(func.body, w);
        w.dedent();
        w.write_line("}");
    }
}

fn emit_function(func: &Function, config: &TranspileConfig, w: &mut IndentWriter) {
    let export = if config.export_declarations {
        "export "
    } else {
        ""
    };
    let async_prefix = if func.async_fn { "async " } else { "" };
    let generics = generic_params_from_slice(func.generics);

    let args_to_emit: Vec<&Variable> = func.args.iter().collect();
    let params = format_params(&args_to_emit);
    let ret_annotation = format_return_type(func);

    w.write_line(&format!(
        "{export}{async_prefix}function {}{generics}({params}){ret_annotation} {{",
        func.name
    ));
    w.indent();
    emit_body(func.body, w);
    w.dedent();
    w.write_line("}");
}

fn emit_test_function(func: &Function, config: &TranspileConfig, w: &mut IndentWriter) {
    let export = if config.export_declarations {
        "export "
    } else {
        ""
    };

    w.write_line(&format!("{export}function {}() {{", func.name));
    w.indent();
    emit_body(func.body, w);
    w.dedent();
    w.write_line("}");
}

fn emit_impl_standalone(imp: &Impl, config: &TranspileConfig, w: &mut IndentWriter) {
    for func in imp.functions {
        emit_function(func, config, w);
        w.newline();
    }
}

fn emit_static(s: &Static, config: &TranspileConfig, w: &mut IndentWriter) {
    let export = if config.export_declarations {
        "export "
    } else {
        ""
    };
    let ty = (s.ty)();
    let ann = type_annotation(&ty);

    w.write(&format!("{export}const {}: {ann}", s.name));
    if !s.value.is_empty() {
        w.write(" = ");
        if s.value.len() == 1 {
            emit_expression(&s.value[0], w);
        } else {
            let mut body_w = IndentWriter::new(config.indent);
            emit_body(s.value, &mut body_w);
            w.write(&body_w.into_string());
        }
    }
    w.write(";");
    w.newline();
}

fn generic_params_from_slice(generics: &[agdb::type_def::Generic]) -> String {
    let type_params: Vec<&str> = generics
        .iter()
        .filter(|g| !matches!(g.kind, GenericKind::Lifetime))
        .map(|g| g.name)
        .collect();

    if type_params.is_empty() {
        String::new()
    } else {
        format!("<{}>", type_params.join(", "))
    }
}

fn format_params(args: &[&Variable]) -> String {
    args.iter()
        .filter_map(|a| {
            if let Some(ty_fn) = a.ty {
                let ty = ty_fn();
                if matches!(ty, Type::SelfType(_)) {
                    return None;
                }
                let normalized = normalize_type(&ty);
                if matches!(normalized, NormalizedType::Named(ref n) if n == "this") {
                    return None;
                }
                let ann = emit_normalized(&normalized);
                Some(format!("{}: {ann}", a.name))
            } else {
                Some(a.name.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_return_type(func: &Function) -> String {
    let ret_ty = (func.ret)();
    if matches!(ret_ty, Type::Literal(Literal::Unit)) {
        return String::new();
    }
    let ann = type_annotation(&ret_ty);
    if func.async_fn {
        format!(": Promise<{ann}>")
    } else {
        format!(": {ann}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::type_def::TypeDefinition;

    fn default_config() -> TranspileConfig {
        TranspileConfig {
            indent: "    ",
            export_declarations: true,
        }
    }

    fn transpile(ty: &Type) -> String {
        let config = default_config();
        let mut w = IndentWriter::new(config.indent);
        emit_type(ty, &config, &mut w);
        w.into_string()
    }

    #[test]
    fn empty_struct_to_class() {
        #[derive(agdb::TypeDef)]
        struct Empty {}

        let output = transpile(&Empty::type_def());
        assert!(output.contains("export class Empty {"), "Got: {output}");
        assert!(output.contains("}"), "Got: {output}");
    }

    #[test]
    fn struct_with_fields_to_class() {
        #[derive(agdb::TypeDef)]
        struct Point {
            x: i32,
            y: i32,
        }

        let output = transpile(&Point::type_def());
        assert!(output.contains("x: number;"), "Got: {output}");
        assert!(output.contains("y: number;"), "Got: {output}");
        assert!(
            output.contains("constructor(x: number, y: number)"),
            "Got: {output}"
        );
        assert!(output.contains("this.x = x;"), "Got: {output}");
        assert!(output.contains("this.y = y;"), "Got: {output}");
    }

    #[test]
    fn generic_struct_to_class() {
        #[derive(agdb::TypeDef)]
        struct Container<T> {
            _value: T,
        }

        let output = transpile(&Container::<i32>::type_def());
        assert!(
            output.contains("export class Container<T>"),
            "Got: {output}"
        );
    }

    #[test]
    fn unit_enum_to_ts_enum() {
        #[derive(agdb::TypeDef)]
        #[allow(dead_code)]
        enum Direction {
            North,
            South,
            East,
            West,
        }

        let output = transpile(&Direction::type_def());
        assert!(output.contains("export enum Direction {"), "Got: {output}");
        assert!(output.contains("North = \"North\","), "Got: {output}");
        assert!(output.contains("South = \"South\","), "Got: {output}");
    }

    #[test]
    fn tagged_enum_to_discriminated_union() {
        #[derive(agdb::TypeDef)]
        #[allow(dead_code)]
        enum Shape {
            Circle(f64),
            Point,
        }

        let output = transpile(&Shape::type_def());
        assert!(output.contains("export type Shape ="), "Got: {output}");
        assert!(output.contains("type: \"Circle\""), "Got: {output}");
        assert!(output.contains("type: \"Point\""), "Got: {output}");
    }

    #[test]
    fn trait_to_interface() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait Serializable {
            fn serialize() -> String;
        }

        let output = transpile(&SerializableDef::type_def());
        assert!(
            output.contains("export interface Serializable {"),
            "Got: {output}"
        );
        assert!(output.contains("serialize(): string;"), "Got: {output}");
    }

    #[test]
    fn trait_with_supertraits() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait Base {}

        #[agdb::trait_def]
        #[allow(dead_code)]
        trait Extended: agdb::type_def::TypeDefinition {}

        let output = transpile(&ExtendedDef::type_def());
        assert!(output.contains("extends"), "Got: {output}");
    }

    #[test]
    fn function_declaration() {
        #[agdb::fn_def]
        #[allow(unused)]
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        let Type::Function(func) = __add_type_def() else {
            panic!("Expected function");
        };
        let config = default_config();
        let mut w = IndentWriter::new(config.indent);
        emit_function(&func, &config, &mut w);
        let output = w.into_string();

        assert!(
            output.contains("export function add(a: number, b: number): number {"),
            "Got: {output}"
        );
        assert!(output.contains("return a + b;"), "Got: {output}");
    }

    #[test]
    fn async_function() {
        #[agdb::fn_def]
        #[allow(unused)]
        async fn fetch_data(url: &str) -> String {
            String::new()
        }

        let Type::Function(func) = __fetch_data_type_def() else {
            panic!("Expected function");
        };
        let config = default_config();
        let mut w = IndentWriter::new(config.indent);
        emit_function(&func, &config, &mut w);
        let output = w.into_string();

        assert!(
            output.contains("export async function fetch_data"),
            "Got: {output}"
        );
        assert!(output.contains("Promise<string>"), "Got: {output}");
    }

    #[test]
    fn no_export_when_disabled() {
        #[derive(agdb::TypeDef)]
        struct Private {
            _x: i32,
        }

        let config = TranspileConfig {
            indent: "    ",
            export_declarations: false,
        };
        let mut w = IndentWriter::new(config.indent);
        emit_type(&Private::type_def(), &config, &mut w);
        let output = w.into_string();

        assert!(!output.contains("export"), "Got: {output}");
        assert!(output.contains("class Private"), "Got: {output}");
    }

    // --- Struct with impl block → class with methods ---

    #[test]
    fn struct_with_impl_methods() {
        #[derive(agdb::TypeDef)]
        #[type_def(inherent)]
        struct Counter {
            value: i32,
        }

        #[agdb::impl_def]
        #[allow(dead_code)]
        impl Counter {
            fn get(&self) -> i32 {
                self.value
            }

            fn increment(&mut self) {
                self.value += 1;
            }
        }

        let output = transpile(&Counter::type_def());
        assert!(output.contains("export class Counter {"), "Got:\n{output}");
        assert!(output.contains("value: number;"), "Got:\n{output}");
        assert!(
            output.contains("constructor(value: number)"),
            "Got:\n{output}"
        );
        assert!(output.contains("get(): number {"), "Got:\n{output}");
        assert!(output.contains("increment()"), "Got:\n{output}");
    }

    // --- Struct implementing trait → class implements interface ---

    #[test]
    fn struct_implements_trait() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait Describable {
            fn describe() -> String;
        }

        #[derive(agdb::TypeDef)]
        #[type_def(Describable)]
        struct Item {
            name: String,
        }

        #[agdb::impl_def]
        #[allow(dead_code)]
        impl Describable for Item {
            fn describe() -> String {
                String::new()
            }
        }

        let output = transpile(&Item::type_def());
        assert!(
            output.contains("class Item implements Describable"),
            "Got:\n{output}"
        );
        assert!(output.contains("describe(): string {"), "Got:\n{output}");
    }

    // --- Enum with struct variant → discriminated union with fields ---

    #[test]
    fn enum_struct_variant_to_discriminated_union() {
        #[derive(agdb::TypeDef)]
        #[allow(dead_code)]
        enum Event {
            Click { x: i32, y: i32 },
            KeyPress(String),
            Close,
        }

        let output = transpile(&Event::type_def());
        assert!(output.contains("export type Event ="), "Got:\n{output}");
        assert!(output.contains("type: \"Click\""), "Got:\n{output}");
        assert!(output.contains("x: number"), "Got:\n{output}");
        assert!(output.contains("y: number"), "Got:\n{output}");
        assert!(output.contains("type: \"KeyPress\""), "Got:\n{output}");
        assert!(output.contains("type: \"Close\""), "Got:\n{output}");
    }

    // --- Generic enum ---

    #[test]
    fn generic_enum_to_discriminated_union() {
        #[derive(agdb::TypeDef)]
        #[allow(dead_code)]
        enum Response<T> {
            Success(T),
            Error(String),
        }

        let output = transpile(&Response::<i32>::type_def());
        assert!(
            output.contains("export type Response<T> ="),
            "Got:\n{output}"
        );
        assert!(output.contains("type: \"Success\""), "Got:\n{output}");
        assert!(output.contains("type: \"Error\""), "Got:\n{output}");
    }

    // --- Trait with multiple methods ---

    #[test]
    fn trait_multiple_methods_to_interface() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait Repository {
            fn find(id: i32) -> String;
            fn save(data: String) -> bool;
            fn delete(id: i32);
        }

        let output = transpile(&RepositoryDef::type_def());
        assert!(
            output.contains("export interface Repository {"),
            "Got:\n{output}"
        );
        assert!(
            output.contains("find(id: number): string;"),
            "Got:\n{output}"
        );
        assert!(
            output.contains("save(data: string): boolean;"),
            "Got:\n{output}"
        );
        assert!(output.contains("delete(id: number);"), "Got:\n{output}");
    }

    // --- Trait with generic methods ---

    #[test]
    fn trait_with_generic_method() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait Converter {
            fn convert<T: agdb::type_def::TypeDefinition>(input: T) -> T;
        }

        let output = transpile(&ConverterDef::type_def());
        assert!(
            output.contains("export interface Converter {"),
            "Got:\n{output}"
        );
        assert!(
            output.contains("convert<T>(input: T): T;"),
            "Got:\n{output}"
        );
    }

    // --- Trait with default implementation ---

    #[test]
    fn trait_default_method_has_body() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait Greetable {
            fn greet() -> String {
                let i = "hello";
                format!("{}", i.len())
            }
        }

        let output = transpile(&GreetableDef::type_def());
        assert!(
            output.contains("export interface Greetable {"),
            "Got:\n{output}"
        );
        assert!(output.contains("greet(): string {"), "Got:\n{output}");
    }

    // --- Async trait method ---

    #[test]
    fn trait_with_async_method() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait Fetcher {
            async fn fetch(url: &str) -> String;
        }

        let output = transpile(&FetcherDef::type_def());
        assert!(
            output.contains("export interface Fetcher {"),
            "Got:\n{output}"
        );
        assert!(
            output.contains("fetch(url: string): Promise<string>;"),
            "Got:\n{output}"
        );
    }

    // --- Function with various types ---

    #[test]
    fn function_option_param_and_return() {
        #[agdb::fn_def]
        #[allow(unused)]
        fn find(name: Option<String>) -> Option<i32> {
            None
        }

        let Type::Function(func) = __find_type_def() else {
            panic!("Expected function");
        };
        let config = default_config();
        let mut w = IndentWriter::new(config.indent);
        emit_function(&func, &config, &mut w);
        let output = w.into_string();

        assert!(
            output.contains("find(name: string | null): number | null"),
            "Got:\n{output}"
        );
    }

    #[test]
    fn function_vec_param() {
        #[agdb::fn_def]
        #[allow(unused)]
        fn sum(values: Vec<i32>) -> i32 {
            0
        }

        let Type::Function(func) = __sum_type_def() else {
            panic!("Expected function");
        };
        let config = default_config();
        let mut w = IndentWriter::new(config.indent);
        emit_function(&func, &config, &mut w);
        let output = w.into_string();

        assert!(
            output.contains("sum(values: number[]): number"),
            "Got:\n{output}"
        );
    }

    // --- Static declaration ---

    #[test]
    fn static_const_declaration() {
        #[agdb::static_def]
        #[allow(dead_code)]
        static MAX_SIZE: i32 = 100;

        let ty = __MAX_SIZE_type_def();
        let output = transpile(&ty);
        assert!(
            output.contains("export const MAX_SIZE: number"),
            "Got:\n{output}"
        );
        assert!(output.contains("= 100;"), "Got:\n{output}");
    }

    // --- Struct with various field types ---

    #[test]
    fn struct_with_option_fields() {
        #[derive(agdb::TypeDef)]
        struct UserProfile {
            name: String,
            email: Option<String>,
            age: Option<i32>,
        }

        let output = transpile(&UserProfile::type_def());
        assert!(output.contains("name: string;"), "Got:\n{output}");
        assert!(output.contains("email: string | null;"), "Got:\n{output}");
        assert!(output.contains("age: number | null;"), "Got:\n{output}");
    }

    #[test]
    fn struct_with_vec_fields() {
        #[derive(agdb::TypeDef)]
        struct Database {
            records: Vec<String>,
            ids: Vec<i32>,
        }

        let output = transpile(&Database::type_def());
        assert!(output.contains("records: string[];"), "Got:\n{output}");
        assert!(output.contains("ids: number[];"), "Got:\n{output}");
    }

    #[test]
    fn struct_with_nested_types() {
        #[derive(agdb::TypeDef)]
        struct Config {
            values: Vec<Option<String>>,
        }

        let output = transpile(&Config::type_def());
        assert!(
            output.contains("values: (string | null)[];"),
            "Got:\n{output}"
        );
    }

    // --- Multi-generic struct ---

    #[test]
    fn multi_generic_struct() {
        #[derive(agdb::TypeDef)]
        struct Pair<A, B> {
            _first: A,
            _second: B,
        }

        let output = transpile(&Pair::<i32, String>::type_def());
        assert!(output.contains("export class Pair<A, B>"), "Got:\n{output}");
    }

    // --- Function with body containing multiple statement types ---

    #[test]
    fn function_complex_body() {
        #[agdb::fn_def]
        #[allow(unused)]
        fn process(items: Vec<i32>) -> i32 {
            let mut total = 0;
            for item in items {
                total += item;
            }
            total
        }

        let Type::Function(func) = __process_type_def() else {
            panic!("Expected function");
        };
        let config = default_config();
        let mut w = IndentWriter::new(config.indent);
        emit_function(&func, &config, &mut w);
        let output = w.into_string();

        assert!(
            output.contains("export function process(items: number[]): number {"),
            "Got:\n{output}"
        );
        assert!(output.contains("let total = 0;"), "Got:\n{output}");
        assert!(
            output.contains("for (const item of items)"),
            "Got:\n{output}"
        );
        assert!(output.contains("total += item;"), "Got:\n{output}");
        assert!(output.contains("return total;"), "Got:\n{output}");
    }

    // --- Generic function ---

    #[test]
    fn generic_function_declaration() {
        #[agdb::fn_def]
        #[allow(unused)]
        fn identity<T: agdb::type_def::TypeDefinition>(value: T) -> T {
            value
        }

        let Type::Function(func) = __identity_type_def() else {
            panic!("Expected function");
        };
        let config = default_config();
        let mut w = IndentWriter::new(config.indent);
        emit_function(&func, &config, &mut w);
        let output = w.into_string();

        assert!(
            output.contains("export function identity<T>(value: T): T {"),
            "Got:\n{output}"
        );
        assert!(output.contains("return value;"), "Got:\n{output}");
    }

    // --- Struct with lifetime (lifetime stripped) ---

    #[test]
    fn struct_lifetime_stripped() {
        #[derive(agdb::TypeDef)]
        struct Borrowed<'a> {
            _data: &'a str,
        }

        let output = transpile(&Borrowed::type_def());
        assert!(!output.contains("'a"), "Got:\n{output}");
        assert!(output.contains("export class Borrowed {"), "Got:\n{output}");
    }
}
