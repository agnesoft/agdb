use agdb::type_def::Enum;
use agdb::type_def::Function;
use agdb::type_def::Generic;
use agdb::type_def::GenericKind;
use agdb::type_def::Impl;
use agdb::type_def::Literal;
use agdb::type_def::Static;
use agdb::type_def::Struct;
use agdb::type_def::Trait;
use agdb::type_def::Type;
use agdb::type_def::Variable;

const SKIP_LIST: &[&str] = &["reqwest_Client"];

struct Typescript {
    types: Vec<Type>,
    tests: Vec<(String, Vec<Type>)>,
}

impl Typescript {
    pub fn new(types: Vec<Type>, tests: Vec<(String, Vec<Type>)>) -> Self {
        Self { types, tests }
    }

    pub fn generate(&self) -> String {
        self.generate_types(self.generate_preamble(), &self.types)
    }

    pub fn generate_tests(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(self.test_preamble());

        for (namespace, types) in &self.tests {
            if !namespace.is_empty() {
                buffer.push_str(&format!("namespace {namespace} {{\n"));
            }

            buffer.push_str(&self.generate_types("", types));

            if !namespace.is_empty() {
                buffer.push_str("}\n\n");
            }
        }

        buffer
    }

    fn generate_types(&self, preamble: &str, types: &[Type]) -> String {
        let mut buffer = String::new();
        buffer.push_str(preamble);

        for ty in types {
            buffer.push_str(&self.generate_type(ty));
        }

        buffer
    }

    fn generate_preamble(&self) -> &str {
        r#"
// GENERATED CODE - DO NOT EDIT

// PREAMBLE

export interface Into {
    into<T>(): T;
}

export class Option<T> {
  public value: T | null;

  constructor(value: T | null) {
    this.value = value;
  }
}

export function Some<T>(value: T): Option<T> {
  return new Option(value);
}

export function None<T>(): Option<T> {
  return new Option<T>(null);
}

export class Result<T, E> {
  public value: T | E;

  constructor(value: T | E) {
    this.value = value;
  }
}

export function Ok<T, E>(value: T): Result<T, E> {
  return new Result<T, E>(value);
}

export function Err<T, E>(error: E): Result<T, E> {
  return new Result<T, E>(error);
}

export class reqwest_Client {
    // This is a placeholder for the actual reqwest.Client type
}

// END OF PREAMBLE

"#
    }

    fn test_preamble(&self) -> &str {
        r#"
// GENERATED TESTS - DO NOT EDIT

// PREAMBLE
import { Option, Some, None, Result, Ok, Err, reqwest_Client } from "./agdb_api";

// END OF PREAMBLE

"#
    }

    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Enum(e) => self.generate_enum(e, &self.type_name(ty)),
            Type::Struct(s) => self.generate_struct(s, &self.type_name(ty)),
            Type::Trait(t) => self.generate_trait(t),
            Type::Function(f) => self.generate_function(f),
            Type::Static(s) => self.generate_static(s),
            Type::Test(t) => self.generate_function(t),
            _ => panic!("Unsupported top level type: {:?}", ty),
        }
    }

    fn generate_enum(&self, e: &Enum, e_name: &str) -> String {
        let mut buffer = self.generate_enum_type(e);
        let name = e.name;
        let full_name = format!("{}{}", e.name, self.generate_generics_decl(e.generics));

        buffer.push_str(&format!(
            "export class {full_name} {{\n    value: {name}Type;\n\n",
        ));

        buffer.push_str(&format!(
            "    constructor({name}Type: {name}Type) {{\n        this.value = {name}Type;\n    }}\n\n",
        ));

        for variant in e.variants {
            let variant_name = &variant.name;
            let variant_type = self.type_name(&(variant.ty.expect("expected a type function"))());

            buffer.push_str(&format!(
                "    static {variant_name}(value: {variant_type}): {name} {{\n        return new {name}({{ {variant_name}: value }});\n    }}\n\n",
            ));
        }

        for i in &(e.impl_defs)() {
            for f in i.functions {
                buffer.push_str(&self.generate_member_function(f, i, e_name));
            }
        }

        buffer.push_str("}\n\n");
        buffer
    }

    fn generate_enum_type(&self, e: &Enum) -> String {
        let mut buffer = String::new();

        buffer.push_str(&format!("type {}Type =\n", e.name));

        let variants = e
            .variants
            .iter()
            .map(|v| {
                format!(
                    "    | {{ {}: {} }}",
                    v.name,
                    self.type_name(&(v.ty.expect("expected a type function"))())
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        buffer.push_str(&variants);
        buffer.push_str(";\n\n");

        buffer
    }

    fn generate_struct(&self, s: &Struct, s_name: &str) -> String {
        if SKIP_LIST.contains(&s.name) {
            return String::new();
        }

        let mut buffer = String::new();
        let full_name = format!("{}{}", s.name, self.generate_generics_decl(s.generics));

        buffer.push_str(&format!("export class {full_name} {{\n",));

        for (i, field) in s.fields.iter().enumerate() {
            if let Some(ty) = &field.ty {
                buffer.push_str(&format!(
                    "    public {}: {};\n",
                    self.field_name(field.name, i),
                    self.type_name(&(ty)())
                ));
            }
        }

        buffer.push_str(&self.generate_constructor(s));

        for i in &(s.impl_defs)() {
            for f in i.functions {
                buffer.push_str(&self.generate_member_function(f, i, s_name));
            }
        }

        buffer.push_str("}\n\n");

        buffer
    }

    fn generate_generics_decl(&self, g: &[Generic]) -> String {
        if g.is_empty() {
            return String::new();
        }

        format!(
            "<{}>",
            g.iter()
                .filter_map(|g| if let GenericKind::Type = g.kind {
                    Some(self.generate_generic_decl(g))
                } else {
                    None
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn generate_generic_decl(&self, g: &Generic) -> String {
        let bounds = g
            .bounds
            .iter()
            .map(|b| self.type_name(&(b)()))
            .collect::<Vec<_>>()
            .join(" & ");
        if bounds.is_empty() {
            g.name.to_owned()
        } else {
            format!("{} extends {}", g.name, bounds)
        }
    }

    fn generate_generic_args_from_types(&self, types: &[fn() -> Type]) -> String {
        if types.is_empty() {
            return String::new();
        }

        format!(
            "<{}>",
            types
                .iter()
                .map(|t| self.type_name(&t()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn generate_generic_args_from_generics(&self, types: &[Generic]) -> String {
        if types.is_empty() {
            return String::new();
        }

        format!(
            "<{}>",
            types
                .iter()
                .map(|g| self.type_name_generic(g))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn generate_constructor(&self, s: &Struct) -> String {
        let mut buffer = String::new();

        let params = s
            .fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                format!(
                    "{}: {}",
                    self.field_name(field.name, i),
                    self.type_name(&(field.ty.expect("expected type function"))())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        buffer.push_str(&format!("\n    constructor({}) {{\n", params));

        for (i, field) in s.fields.iter().enumerate() {
            if field.ty.is_some() {
                buffer.push_str(&format!(
                    "        this.{} = {};\n",
                    self.field_name(field.name, i),
                    self.field_name(field.name, i)
                ));
            }
        }

        buffer.push_str("    }\n");

        buffer
    }

    fn field_name(&self, field: &str, i: usize) -> String {
        if field.is_empty() {
            format!("_{i}")
        } else {
            field.to_owned()
        }
    }

    fn type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Enum(e) => format!(
                "{}{}",
                e.name.to_owned(),
                self.generate_generic_args_from_generics(e.generics)
            ),
            Type::Struct(s) => format!(
                "{}{}",
                s.name.to_owned(),
                self.generate_generic_args_from_generics(s.generics)
            ),
            Type::Literal(l) => self.literal_value(l).to_string(),
            Type::Vec(inner) => format!("{}[]", self.type_name(&(inner)())),
            Type::Function(f) => f.name.to_owned(),
            Type::Test(f) => f.name.to_owned(),
            Type::Generic(g) => self.type_name_generic(g),
            Type::Impl(_) => panic!("impl block does not have a name"),
            Type::Option(inner) => format!("Option<{}>", self.type_name(&(inner)())),
            Type::Pointer(p) => self.type_name(&(p.ty)()),
            Type::Reference(r) => self.type_name(&(r.ty)()),
            Type::Result { ok, err } => format!(
                "Result<{}, {}>",
                self.type_name(&(ok)()),
                self.type_name(&(err)())
            ),
            Type::SelfType(_) => "this".to_owned(),
            Type::Slice(s) => format!("{}[]", self.type_name(&(s)())),
            Type::Static(s) => self.type_name(&(s.ty)()),
            Type::Trait(t) => t.name.to_owned(),
            Type::Tuple(items) => {
                let types = items
                    .iter()
                    .map(|item| self.type_name(&(item)()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", types)
            }
        }
    }

    fn type_name_generic(&self, g: &Generic) -> String {
        if let GenericKind::Argument = g.kind {
            format!(
                "{}{}",
                g.name,
                self.generate_generic_args_from_types(g.bounds)
            )
        } else {
            g.name.to_owned()
        }
    }

    fn has_field_names(&self, s: &Struct) -> bool {
        s.fields.iter().any(|f| !f.name.is_empty())
    }

    fn literal_value(&self, l: &Literal) -> &str {
        match l {
            Literal::Bool => "boolean",
            Literal::F32
            | Literal::F64
            | Literal::I8
            | Literal::I16
            | Literal::I32
            | Literal::I64
            | Literal::I128
            | Literal::Isize
            | Literal::U8
            | Literal::U16
            | Literal::U32
            | Literal::U64
            | Literal::U128
            | Literal::Usize => "number",
            Literal::Str | Literal::String => "string",
            Literal::Unit => "void",
        }
    }

    fn generate_trait(&self, t: &Trait) -> String {
        format!(
            "export interface {}{} {{\n}}\n\n",
            t.name,
            self.generate_generics_decl(t.generics)
        )
    }

    fn generate_function(&self, f: &Function) -> String {
        let ret = self.type_name(&(f.ret)());
        let ret = if f.async_fn {
            format!("Promise<{}>", ret)
        } else {
            ret
        };

        format!(
            "function {}{}({}): {} {{\n    // TODO: implement\n}}\n\n",
            f.name,
            self.generate_generics_decl(f.generics),
            self.generate_args(f.args),
            ret
        )
    }

    fn generate_member_function(&self, f: &Function, _i: &Impl, class_name: &str) -> String {
        let ret = self.type_name(&(f.ret)());
        let ret = if f.async_fn {
            format!(": Promise<{ret}>")
        } else if let Type::SelfType(_) = (f.ret)() {
            format!(": {class_name}")
        } else {
            format!(": {ret}")
        };

        format!(
            "    {}{}({}){} {{\n        // TODO: implement\n    }}\n\n",
            self.ts_name(f.name),
            self.generate_generics_decl(f.generics),
            self.generate_args(f.args),
            ret
        )
    }

    fn generate_args(&self, args: &[Variable]) -> String {
        args.iter()
            .filter_map(|arg| {
                if let Type::SelfType(_) = arg.ty.expect("expected type function")() {
                    return None;
                }

                let ty = self.type_name(&(arg.ty.expect("expected type function"))());
                Some(format!("{}: {ty}", arg.name))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_static(&self, s: &Static) -> String {
        format!(
            "export const {}: {} = undefined;\n\n",
            self.ts_name(s.name),
            self.type_name(&(s.ty)())
        )
    }

    fn ts_name(&self, name: &str) -> String {
        if name == "delete" {
            "delete_".to_owned()
        } else {
            name.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Api;
    use std::process::Command;

    #[test]
    fn test_generate() {
        let ts = Typescript::new(Api::types(), Api::tests());
        let out = ts.generate();
        std::fs::write("agdb_api.ts", &out).unwrap();
        let tests = ts.generate_tests();
        std::fs::write("agdb_api.spec.ts", &tests).unwrap();

        let cmd = Command::new("C:\\Users\\vlach\\AppData\\Roaming\\npm\\tsc.cmd")
            .arg("--noEmit")
            .arg("--strict")
            .arg("agdb_api.ts")
            .arg("agdb_api.spec.ts")
            .output();

        match cmd {
            Ok(output) => {
                let mut out = String::from_utf8_lossy(&output.stdout).to_string();
                out.push_str(&String::from_utf8_lossy(&output.stderr));
                std::fs::write("agdb_api.tsc.log", &out).unwrap();

                assert!(
                    output.status.success(),
                    "TypeScript compilation failed with status: {}",
                    output.status
                );
            }
            Err(e) => {
                panic!("Failed to execute tsc: {}", e);
            }
        }
    }
}
