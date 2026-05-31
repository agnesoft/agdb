use agdb::type_def::Enum;
use agdb::type_def::Function;
use agdb::type_def::Generic;
use agdb::type_def::GenericKind;
use agdb::type_def::Literal;
use agdb::type_def::Static;
use agdb::type_def::Struct;
use agdb::type_def::Trait;
use agdb::type_def::Type;
use agdb::type_def::Variable;

const SKIP_LIST: &[&str] = &["reqwest_Client"];

struct Typescript {
    types: Vec<Type>,
}

impl Typescript {
    pub fn new(types: Vec<Type>) -> Self {
        Self { types }
    }

    pub fn generate(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(self.generate_preamble());

        for ty in &self.types {
            buffer.push_str(&self.generate_type(ty));
        }

        buffer
    }

    fn generate_preamble(&self) -> &str {
        r#"
// GENERATED CODE - DO NOT EDIT

// PREAMBLE

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

    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Enum(e) => self.generate_enum(e),
            Type::Struct(s) => self.generate_struct(s),
            Type::Trait(t) => self.generate_trait(t),
            Type::Function(f) => self.generate_function(f),
            Type::Static(s) => self.generate_static(s),
            _ => String::new(),
        }
    }

    fn generate_enum(&self, e: &Enum) -> String {
        if self.has_enum_variant_type(e) {
            self.generate_ts_union(e)
        } else {
            self.generate_ts_enum(e)
        }
    }

    fn generate_struct(&self, s: &Struct) -> String {
        if SKIP_LIST.contains(&s.name) {
            return String::new();
        }

        if self.has_field_names(s) {
            self.generate_ts_class(s)
        } else if s.fields.len() == 1 {
            self.generate_ts_alias(s)
        } else {
            self.generate_ts_tuple(s)
        }
    }

    fn generate_ts_tuple(&self, s: &Struct) -> String {
        let mut buffer = String::new();

        let field_types = s
            .fields
            .iter()
            .map(|f| self.type_name(&(f.ty.expect("expected type function"))()))
            .collect::<Vec<_>>()
            .join(", ");

        buffer.push_str(&format!(
            "export type {}{} = [{}];\n\n",
            s.name,
            self.generate_generics(s.generics),
            field_types
        ));

        buffer
    }

    fn generate_ts_alias(&self, s: &Struct) -> String {
        let mut buffer = String::new();

        buffer.push_str(&format!(
            "export type {}{} = {};\n\n",
            s.name,
            self.generate_generics(s.generics),
            self.type_name(&(s.fields[0].ty.expect("expected type function"))())
        ));

        buffer
    }

    fn generate_ts_class(&self, s: &Struct) -> String {
        let mut buffer = String::new();

        buffer.push_str(&format!(
            "export class {}{} {{\n",
            s.name,
            self.generate_generics(s.generics)
        ));

        for field in s.fields {
            if let Some(ty) = &field.ty {
                buffer.push_str(&format!(
                    "    public {}: {};\n",
                    field.name,
                    self.type_name(&(ty)())
                ));
            }
        }
        buffer.push_str(&self.generate_constructor(s));
        buffer.push_str("}\n\n");

        buffer
    }

    fn generate_generics(&self, g: &[Generic]) -> String {
        if g.is_empty() {
            return String::new();
        }

        format!(
            "<{}>",
            g.iter()
                .filter_map(|g| if let GenericKind::Type = g.kind {
                    Some(self.generate_generic(g))
                } else {
                    None
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn generate_generic(&self, g: &Generic) -> String {
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

    fn generate_generic_arguments(&self, types: &[fn() -> Type]) -> String {
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

    fn generate_constructor(&self, s: &Struct) -> String {
        let mut buffer = String::new();

        let params = s
            .fields
            .iter()
            .map(|f| {
                format!(
                    "{}: {}",
                    f.name,
                    self.type_name(&(f.ty.expect("expected type function"))())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        buffer.push_str(&format!("\n    constructor({}) {{\n", params));

        for field in s.fields {
            if field.ty.is_some() {
                buffer.push_str(&format!("        this.{} = {};\n", field.name, field.name));
            }
        }

        buffer.push_str("    }\n");

        buffer
    }

    fn generate_ts_union(&self, e: &Enum) -> String {
        let mut buffer = String::new();

        buffer.push_str(&format!("export type {} =\n", e.name));

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

    fn generate_ts_enum(&self, e: &Enum) -> String {
        let mut buffer = String::new();

        buffer.push_str(&format!("export enum {} {{\n", e.name));

        for variant in e.variants {
            buffer.push_str(&format!("    {},\n", variant.name));
        }

        buffer.push_str("}\n\n");

        buffer
    }

    fn field_name(&self, field: &str, i: usize) -> String {
        if field.is_empty() {
            format!("{i}")
        } else {
            field.to_owned()
        }
    }

    fn type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Enum(e) => e.name.to_owned(),
            Type::Struct(s) => s.name.to_owned(),
            Type::Literal(l) => self.literal_value(l).to_string(),
            Type::Vec(inner) => format!("{}[]", self.type_name(&(inner)())),
            Type::Function(f) => f.name.to_owned(),
            Type::Test(f) => f.name.to_owned(),
            Type::Generic(g) => {
                if let GenericKind::Argument = g.kind {
                    format!("{}{}", g.name, self.generate_generic_arguments(g.bounds))
                } else {
                    g.name.to_owned()
                }
            }
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

    fn has_field_names(&self, s: &Struct) -> bool {
        s.fields.iter().any(|f| !f.name.is_empty())
    }

    fn has_enum_variant_type(&self, e: &Enum) -> bool {
        e.variants.iter().any(|v| v.ty.is_some())
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
            self.generate_generics(t.generics)
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
            "export function {}{}({}): {} {{\n    // TODO: implement\n}}\n\n",
            f.name,
            self.generate_generics(f.generics),
            self.generate_args(f.args),
            ret
        )
    }

    fn generate_args(&self, args: &[Variable]) -> String {
        args.iter()
            .map(|arg| {
                format!(
                    "{}: {}",
                    arg.name,
                    self.type_name(&(arg.ty.expect("expected type function"))())
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_static(&self, s: &Static) -> String {
        format!(
            "export const {}: {} = undefined;\n\n",
            s.name,
            self.type_name(&(s.ty)())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Api;
    use std::process::Command;

    #[test]
    fn test_generate() {
        let out = Typescript::new(Api::type_defs()).generate();
        std::fs::write("agdb_api.ts", &out).unwrap();
        let cmd = Command::new("C:\\Users\\vlach\\AppData\\Roaming\\npm\\tsc.cmd")
            .arg("--noEmit")
            .arg("--strict")
            .arg("agdb_api.ts")
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
