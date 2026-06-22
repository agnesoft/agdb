mod rewrite_ts_api;
mod rewrite_ts_identifiers;
mod rewrite_ts_if_let;
mod rewrite_ts_into;
mod rewrite_ts_methods;
mod rewrite_ts_path_strip;
mod rewrite_ts_tuple_access;

use std::collections::HashMap;

use agdb::type_def::Enum;
use agdb::type_def::Expression;
use agdb::type_def::Function;
use agdb::type_def::Generic;
use agdb::type_def::GenericKind;
use agdb::type_def::Impl;
use agdb::type_def::Literal;
use agdb::type_def::LiteralValue;
use agdb::type_def::Op;
use agdb::type_def::Pattern;
use agdb::type_def::Static;
use agdb::type_def::Struct;
use agdb::type_def::Trait;
use agdb::type_def::Type;
use agdb::type_def::Variable;

use super::rewrite::RewriteContext;
use super::rewrite::RewritePipeline;
use super::rewrite::StripAtomics;
use super::rewrite::StripMemoryManagement;
use super::rewrite::StripPointerTypes;
use super::rewrite::StripReferences;
use super::rewrite::StripRustCalls;
use super::rewrite::StripSmartPointers;
use rewrite_ts_api::RewriteTsApi;
use rewrite_ts_identifiers::RewriteTsIdentifiers;
use rewrite_ts_if_let::RewriteTsIfLet;
use rewrite_ts_into::RewriteTsInto;
use rewrite_ts_methods::RewriteTsMethods;
use rewrite_ts_path_strip::RewriteTsPathStrip;
use rewrite_ts_tuple_access::RewriteTsTupleAccess;

const SKIP_LIST: &[&str] = &[
    "reqwest_Client",
    "reqwest::Client",
    "PathBuf",
    "Duration",
    "AtomicU16",
];
const MANUAL_IMPL: &[&str] = &[
    "collapse_conditions",
    "search_mut",
    "create_cluster",
    "wait_for_leader",
    "concurrent_logins",
];
const MANUAL_IMPL_QUALIFIED: &[(&str, &str)] = &[
    ("ReqwestClient", "delete"),
    ("ReqwestClient", "get"),
    ("ReqwestClient", "post"),
    ("ReqwestClient", "put"),
    ("ReqwestClient", "new"),
    ("TestServerImpl", "with_config"),
];
const ASSOCIATED_TYPE_TRAITS: &[&str] = &["DbType"];
const STRIP_BOUNDS: &[&str] = &["Send", "Sync", "Clone", "Copy", "Sized", "From"];
const SUBSTITUTE_BOUNDS: &[(&str, &str)] = &[];

const INTO_TYPE_MAP: &[(&str, &str)] = &[
    ("QueryAliases", "string[]"),
    ("MultiValues", "DbKeyValue[][]"),
    ("SingleValues", "DbKeyValue[]"),
    ("DbValues", "DbValueInput[]"),
    ("DbKeyOrders", "DbKeyOrder[]"),
    ("String", "string"),
    ("ReqwestClientTypeDef", "ReqwestClientTypeDef"),
];

#[derive(Default)]
struct Context {
    ret: Option<String>,
    error_type: Option<String>,
    ty: Option<String>,
    into_targets: HashMap<String, String>,
}

fn pipeline() -> RewritePipeline {
    RewritePipeline::new(vec![
        Box::new(StripSmartPointers),
        Box::new(StripAtomics),
        Box::new(StripReferences),
        Box::new(StripMemoryManagement),
        Box::new(RewriteTsInto),
        Box::new(StripRustCalls),
        Box::new(RewriteTsTupleAccess),
        Box::new(RewriteTsMethods),
        Box::new(RewriteTsApi),
        Box::new(RewriteTsIdentifiers),
        Box::new(RewriteTsIfLet),
        Box::new(RewriteTsPathStrip),
        Box::new(StripPointerTypes),
    ])
}

pub struct Typescript {
    types: Vec<Type>,
    tests: Vec<(String, Vec<Type>)>,
    pipeline: RewritePipeline,
}

impl Typescript {
    pub fn new(types: Vec<Type>, tests: Vec<(String, Vec<Type>)>) -> Self {
        Self {
            types,
            tests,
            pipeline: pipeline(),
        }
    }

    fn rewrite_body(&self, body: &[Expression], context: &Context) -> Vec<Expression> {
        let mut rewritten = body.to_vec();
        let ctx = RewriteContext {
            current_type: context.ty.clone(),
            current_function: None,
            error_type: context.error_type.clone(),
            into_targets: context.into_targets.clone(),
        };
        self.pipeline.rewrite_exprs(&mut rewritten, &ctx);
        rewritten
    }

    pub fn generate(&self) -> String {
        self.generate_types(self.generate_preamble(), &self.types)
    }

    pub fn generate_tests(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(&self.test_preamble(&self.types));

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

export interface Send {}

export interface Borrow {}

export interface Serialize {}

export interface DeserializeOwned {}

export type Option<T> = { value: T } | null;

export function Some<T>(value: T): Option<T> {
  return { value };
}

export const None: Option<never> = null;

export class Result<T, E> {
  public value: T | E;
  has_value: boolean;

  constructor(value: T | E, has_value: boolean) {
    this.value = value;
    this.has_value = has_value;
  }

  is_ok(): boolean {
    return this.has_value;
  }

  map<U>(f: (value: T) => U): Result<U, E> {
    if (this.has_value) {
      return Ok(f(this.value as T));
    }
    return Err(this.value as E);
  }
}

export function Ok<T, E>(value: T): Result<T, E> {
  return new Result<T, E>(value, true);
}

export function Err<T, E>(error: E): Result<T, E> {
  return new Result<T, E>(error, false);
}

function unwrapResult<T, E>(r: Result<T, E>): T {
  if (!r.is_ok()) throw r.value;
  return r.value as T;
}

export class reqwest_Client {
    delete_(uri: string): any { throw new Error("not implemented"); }
    get<T>(uri: string): any { throw new Error("not implemented"); }
    post<T, R>(uri: string): any { throw new Error("not implemented"); }
    put<T>(uri: string): any { throw new Error("not implemented"); }
    header(key: string, value: string): any { return this; }
    bearer_auth(token: string): any { return this; }
    json<T>(body: T): any { return this; }
    send(): Promise<any> { throw new Error("not implemented"); }
}

export type AgdbApiResult<T> = Result<T, AgdbApiError>;
export type ReqwestClientTypeDef = reqwest_Client;

const DB_ELEMENT_ID_KEY: string = "db_element_id";
const USER_AGENT: string = "User-Agent";

// END OF PREAMBLE

"#
    }

    fn test_preamble(&self, types: &[Type]) -> String {
        format!(
            r#"
// GENERATED TESTS - DO NOT EDIT

// PREAMBLE
declare function readFileSync(path: string, encoding: string): string;
declare function setTimeout(cb: () => void, ms: number): any;
declare const process: {{ env: Record<string, string | undefined> }};
declare const console: {{ log(...args: any[]): void }};

import {{ Result, Ok, Err, Option, Some, None, AgdbApiResult, reqwest_Client, {} }} from "./agdb_api";

class Path {{
    public inner: string;

    constructor(inner: string) {{
        this.inner = inner;
    }}

    static new(inner: string): Path {{
        return new Path(inner);
    }}

    pop(): Path {{
        const parts = this.inner.split("/");
        parts.pop();
        return new Path(parts.join("/"));
    }}

    join(other: string | Path): Path {{
        if (typeof other === "string") {{
            return new Path(`${{this.inner}}/${{other}}`);
        }} else {{
            return new Path(`${{this.inner}}/${{other.inner}}`);
        }}
    }}

    to_string_lossy(): Path {{
        return this;
    }}

    to_string(): string {{
        return this.inner;
    }}
}}

class Duration {{
    public milliseconds: number;

    constructor(milliseconds: number) {{
        this.milliseconds = milliseconds;
    }}

    static from_secs(secs: number): Duration {{
        return new Duration(secs * 1000);
    }}

    static from_millis(ms: number): Duration {{
        return new Duration(ms);
    }}
}}

type PathBuf = Path;

namespace std.env {{
    export function current_exe(): Result<Path, any> {{
        return Ok(new Path("target/release/agdb_server"));
    }}
    export function var_(name: string): Result<string, any> {{
        return Ok(process.env[name] || "");
    }}
}}

namespace std.env.consts {{
    export const EXE_SUFFIX: string = "";
}}

namespace std.fs {{
    export function read_to_string(path: string): string {{
        return readFileSync(path, "utf8");
    }}
}}

namespace std.thread {{
    export async function sleep(duration: Duration): Promise<void> {{
        return new Promise((resolve) => setTimeout(resolve, duration.milliseconds));
    }}
}}

function reqwest_client(): reqwest_Client {{
    return new reqwest_Client();
}}

function unwrapResult<T, E>(r: Result<T, E>): T {{
    if (!r.is_ok()) throw r.value;
    return r.value as T;
}}

function assert_eq(a: any, b: any): void {{
    if (JSON.stringify(a) !== JSON.stringify(b)) throw new Error(`assert_eq failed: ${{JSON.stringify(a)}} !== ${{JSON.stringify(b)}}`);
}}

function assert_ne(a: any, b: any): void {{
    if (JSON.stringify(a) === JSON.stringify(b)) throw new Error(`assert_ne failed: values are equal`);
}}

function assert(condition: any, msg?: string): void {{
    if (!condition) throw new Error(msg || "assertion failed");
}}

function case_(name: string, query: any): [string, any] {{
    return [name, query];
}}

function println(...args: any[]): void {{
    console.log(...args);
}}

function drop(_x: any): void {{}}

class Vec<T> extends Array<T> {{
    static with_capacity<T>(_n: number): Vec<T> {{ return new Vec<T>(); }}
}}

class HashMap<K, V> extends Map<K, V> {{}}

const StatusCode = {{ OK: 200, UNAUTHORIZED: 401, NOT_FOUND: 404, FORBIDDEN: 403, CONFLICT: 409 }};

// END OF PREAMBLE

"#,
            types
                .iter()
                .filter_map(|t| match t {
                    Type::Enum(e) => Some(e.name.as_str()),
                    Type::Struct(s) if !SKIP_LIST.contains(&s.name.as_str()) => {
                        Some(s.name.as_str())
                    }
                    Type::Function(f) => Some(f.name.as_str()),
                    Type::Static(s) => Some(s.name.as_str()),
                    Type::Trait(t) => Some(t.name.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Enum(e) => self.generate_enum(e, &self.emit_type(ty, &e.name)),
            Type::Struct(s) => self.generate_struct(s, &self.emit_type(ty, &s.name)),
            Type::Trait(t) if !SKIP_LIST.contains(&t.name.as_str()) => self.generate_trait(t),
            Type::Trait(_) => String::new(),
            Type::Function(f) => self.generate_function(f),
            Type::Static(s) => self.generate_static(s),
            Type::Test(t) => self.generate_function(t),
            _ => panic!("Unsupported top level type: {:?}", ty),
        }
    }

    fn generate_enum(&self, e: &Enum, e_name: &str) -> String {
        let mut buffer = self.generate_enum_type(e);
        let name = &e.name;
        let full_name = format!("{}{}", e.name, self.generate_generics_decl(&e.generics));

        buffer.push_str(&format!(
            "export class {full_name} {{\n    value: {name}Type;\n\n",
        ));

        buffer.push_str(&format!(
            "    constructor({name}Type: {name}Type) {{\n        this.value = {name}Type;\n    }}\n\n",
        ));

        for variant in &e.variants {
            let variant_name = &variant.name;
            let variant_type =
                self.type_name(variant.ty.expect("expected a type function"), e_name);

            if variant_type == "void" {
                buffer.push_str(&format!(
                    "    static {variant_name}(): {name} {{\n        return new {name}({{ {variant_name}: undefined }});\n    }}\n\n",
                ));
            } else {
                buffer.push_str(&format!(
                    "    static {variant_name}(value: {variant_type}): {name} {{\n        return new {name}({{ {variant_name}: value }});\n    }}\n\n",
                ));
            }
        }

        let e_generics_decl = self.generate_generics_decl(&e.generics);
        for i in &(e.impl_defs)() {
            for f in &i.functions {
                buffer.push_str(&self.generate_member_function(f, i, e_name, &e_generics_decl));
            }
        }

        buffer.push_str("}\n\n");

        let enum_impl_defs = (e.impl_defs)();
        buffer.push_str(&self.generate_input_type_alias(&e.name, &enum_impl_defs));

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
                    self.type_name(v.ty.expect("expected a type function"), &e.name)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        buffer.push_str(&variants);
        buffer.push_str(";\n\n");

        buffer
    }

    fn generate_struct(&self, s: &Struct, s_name: &str) -> String {
        if SKIP_LIST.contains(&s.name.as_str()) {
            return String::new();
        }

        let mut buffer = String::new();
        let full_name = format!("{}{}", s.name, self.generate_generics_decl(&s.generics));

        let impl_defs = (s.impl_defs)();
        let implements: Vec<String> = impl_defs
            .iter()
            .filter_map(|i| {
                i.trait_.map(|t| {
                    let ty = t();
                    match ty {
                        Type::Trait(t) => t.name,
                        _ => String::new(),
                    }
                })
            })
            .filter(|name| !name.is_empty() && !STRIP_BOUNDS.contains(&name.as_str()))
            .collect();

        let implements_clause = if implements.is_empty() {
            String::new()
        } else {
            format!(" implements {}", implements.join(", "))
        };

        buffer.push_str(&format!("export class {full_name}{implements_clause} {{\n"));

        for (i, field) in s.fields.iter().enumerate() {
            if let Some(ty) = &field.ty {
                buffer.push_str(&format!(
                    "    public {}: {};\n",
                    self.field_name(&field.name, i),
                    self.type_name(*ty, &s.name)
                ));
            }
        }

        buffer.push_str(&self.generate_constructor(s));

        let class_generics_decl = self.generate_generics_decl(&s.generics);
        for i in &impl_defs {
            for f in &i.functions {
                buffer.push_str(&self.generate_member_function(f, i, s_name, &class_generics_decl));
            }
        }

        buffer.push_str("}\n\n");
        buffer.push_str(&self.generate_input_type_alias(&s.name, &impl_defs));

        buffer
    }

    fn generate_input_type_alias(&self, name: &str, impl_defs: &[Impl]) -> String {
        let from_types: Vec<String> = impl_defs
            .iter()
            .filter_map(|i| {
                let trait_fn = i.trait_?;
                let ty = trait_fn();
                if let Type::Trait(t) = ty {
                    if t.name == "From" && !t.generics.is_empty() {
                        let source_type = if !t.generics[0].bounds.is_empty() {
                            let source_ty = (t.generics[0].bounds[0])();
                            self.emit_type(&source_ty, name)
                        } else {
                            t.generics[0].name.clone()
                        };
                        if source_type != name
                            && source_type != "void"
                            && !source_type.contains("::")
                        {
                            return Some(source_type);
                        }
                    }
                }
                None
            })
            .collect();

        if from_types.is_empty() {
            return String::new();
        }

        let mut unique_types: Vec<String> = Vec::new();
        for t in from_types {
            if !unique_types.contains(&t) {
                unique_types.push(t);
            }
        }

        let union = unique_types.join(" | ");
        format!("export type {name}Input = {name} | {union};\n\n")
    }

    fn widen_type_if_input(&self, type_name: &str) -> String {
        let base = type_name.trim_end_matches("[]");
        let has_input = self.types.iter().any(|ty| {
            let (name, impl_defs_fn) = match ty {
                Type::Struct(s) => (s.name.as_str(), Some(s.impl_defs)),
                Type::Enum(e) => (e.name.as_str(), Some(e.impl_defs)),
                _ => ("", None),
            };
            if name == base {
                if let Some(impl_defs_fn) = impl_defs_fn {
                    return (impl_defs_fn)().iter().any(|i| {
                        i.trait_
                            .map(|t| matches!(t(), Type::Trait(ref tr) if tr.name == "From"))
                            .unwrap_or(false)
                    });
                }
            }
            false
        });

        if has_input {
            format!("{base}Input{}", if type_name.ends_with("[]") { "[]" } else { "" })
        } else {
            type_name.to_owned()
        }
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
            .filter_map(|b| {
                let name = self.type_name(*b, &g.name);
                if STRIP_BOUNDS.contains(&name.as_str()) {
                    None
                } else if let Some((_, sub)) =
                    SUBSTITUTE_BOUNDS.iter().find(|(from, _)| *from == name)
                {
                    Some(sub.to_string())
                } else {
                    Some(name)
                }
            })
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
                .map(|t| self.type_name(*t, "this"))
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
                let ty_fn = field.ty.expect("expected type function");
                let type_name = self.type_name(ty_fn, &s.name);
                let default = Self::type_default(&ty_fn());
                format!(
                    "{}: {} = {}",
                    self.field_name(&field.name, i),
                    type_name,
                    default
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        buffer.push_str(&format!("\n    constructor({}) {{\n", params));

        for (i, field) in s.fields.iter().enumerate() {
            if field.ty.is_some() {
                buffer.push_str(&format!(
                    "        this.{} = {};\n",
                    self.field_name(&field.name, i),
                    self.field_name(&field.name, i)
                ));
            }
        }

        buffer.push_str("    }\n");

        buffer
    }

    fn type_default(ty: &Type) -> &'static str {
        match ty {
            Type::Literal(lit) => match lit {
                Literal::Bool => "false",
                Literal::F32
                | Literal::F64
                | Literal::I8
                | Literal::I16
                | Literal::I32
                | Literal::I64
                | Literal::I128
                | Literal::U8
                | Literal::U16
                | Literal::U32
                | Literal::U64
                | Literal::U128
                | Literal::Usize
                | Literal::Isize => "0",
                Literal::Str | Literal::String => "\"\"",
                Literal::Unit => "undefined",
            },
            Type::Vec(_) | Type::Slice(_) | Type::Tuple(_) => "[]",
            Type::Option(_) => "null",
            _ => "null as any",
        }
    }

    fn field_name(&self, field: &str, i: usize) -> String {
        if field.is_empty() {
            format!("_{i}")
        } else {
            field.to_owned()
        }
    }

    fn type_name(&self, ty: fn() -> Type, class_name: &str) -> String {
        let mut rewritten = ty();
        let ctx = RewriteContext {
            current_type: Some(class_name.to_owned()),
            ..Default::default()
        };
        self.pipeline.rewrite_type(&mut rewritten, &ctx);
        self.emit_type(&rewritten, class_name)
    }

    fn emit_type(&self, ty: &Type, class_name: &str) -> String {
        match ty {
            Type::Enum(e) => format!(
                "{}{}",
                e.name.to_owned(),
                self.generate_generic_args_from_generics(&e.generics)
            ),
            Type::Struct(s) => {
                let generics = if s.generics.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        s.generics
                            .iter()
                            .map(|g| g.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                format!("{}{}", s.name.replace("::", "_"), generics)
            }
            Type::Literal(l) => self.literal(l).to_string(),
            Type::Vec(inner) => format!("{}[]", self.type_name(*inner, class_name)),
            Type::Function(f) => f.name.to_owned(),
            Type::Test(f) => f.name.to_owned(),
            Type::Generic(g) => self.type_name_generic(g),
            Type::Impl(_) => panic!("impl block does not have a name"),
            Type::Option(inner) => {
                format!("Option<{}>", self.type_name(*inner, class_name))
            }
            Type::Pointer(p) => self.type_name(p.ty, class_name),
            Type::Reference(r) => self.type_name(r.ty, class_name),
            Type::Result { ok, err } => format!(
                "Result<{}, {}>",
                self.type_name(*ok, class_name),
                self.type_name(*err, class_name)
            ),
            Type::SelfType(_) => class_name.to_owned(),
            Type::Slice(s) => format!("{}[]", self.type_name(*s, class_name)),
            Type::Static(s) => self.type_name(s.ty, class_name),
            Type::Trait(t) => {
                let mut associated = Vec::new();
                Self::collect_associated_types(&t.functions, &mut associated);
                if associated.is_empty() && ASSOCIATED_TYPE_TRAITS.contains(&t.name.as_str()) {
                    format!("{}<{}>", t.name, class_name)
                } else if associated.is_empty() {
                    t.name.to_owned()
                } else {
                    let args = vec![class_name; associated.len()].join(", ");
                    format!("{}<{}>", t.name, args)
                }
            }
            Type::Tuple(items) => {
                let types = items
                    .iter()
                    .map(|item| self.type_name(*item, class_name))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", types)
            }
        }
    }

    fn type_name_generic(&self, g: &Generic) -> String {
        let name = g
            .name
            .strip_prefix("Self :: ")
            .unwrap_or(&g.name)
            .to_owned();
        if let GenericKind::Argument = g.kind {
            format!(
                "{}{}",
                name,
                self.generate_generic_args_from_types(&g.bounds)
            )
        } else {
            name
        }
    }

    fn has_field_names(&self, s: &Struct) -> bool {
        s.fields.iter().any(|f| !f.name.is_empty())
    }

    fn literal_value(&self, l: &LiteralValue) -> String {
        match l {
            LiteralValue::Bool(b) => b.to_string(),
            LiteralValue::F32(f) => f.to_string(),
            LiteralValue::F64(f) => f.to_string(),
            LiteralValue::I8(i) => i.to_string(),
            LiteralValue::I16(i) => i.to_string(),
            LiteralValue::I32(i) => i.to_string(),
            LiteralValue::I64(i) => i.to_string(),
            LiteralValue::U8(i) => i.to_string(),
            LiteralValue::U16(i) => i.to_string(),
            LiteralValue::U32(i) => i.to_string(),
            LiteralValue::U64(i) => i.to_string(),
            LiteralValue::Usize(i) => i.to_string(),
            LiteralValue::Str(s) => format!("\"{}\"", s),
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Unit => "undefined".to_owned(),
        }
    }

    fn literal(&self, l: &Literal) -> &str {
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
        let mut associated_types: Vec<String> = Vec::new();
        Self::collect_associated_types(&t.functions, &mut associated_types);

        let mut generics_decl = self.generate_generics_decl(&t.generics);
        if !associated_types.is_empty() {
            let extra = associated_types.join(", ");
            if generics_decl.is_empty() {
                generics_decl = format!("<{extra}>");
            } else {
                generics_decl.pop();
                generics_decl.push_str(&format!(", {extra}>"));
            }
        }

        let extends: Vec<String> = t
            .bounds
            .iter()
            .filter_map(|b| {
                let name = self.type_name(*b, &t.name);
                if STRIP_BOUNDS.contains(&name.as_str()) || name == "TypeDefinition" {
                    None
                } else {
                    Some(name)
                }
            })
            .collect();
        let extends_clause = if extends.is_empty() {
            String::new()
        } else {
            format!(" extends {}", extends.join(", "))
        };

        let has_self = |f: &&Function| f.args.first().is_some_and(|a| a.name == "self");

        let instance_methods: Vec<_> = t.functions.iter().filter(has_self).collect();
        let static_methods: Vec<_> = t.functions.iter().filter(|f| !has_self(f)).collect();

        let mut buffer = format!(
            "export interface {}{}{} {{\n",
            t.name, generics_decl, extends_clause
        );
        for f in &instance_methods {
            buffer.push_str(&self.generate_trait_method(f, &t.name));
        }
        buffer.push_str("}\n\n");

        if !static_methods.is_empty() {
            buffer.push_str(&format!(
                "export interface {}Static{} {{\n",
                t.name, generics_decl
            ));
            for f in &static_methods {
                buffer.push_str(&self.generate_trait_method(f, &t.name));
            }
            buffer.push_str("}\n\n");
        }

        buffer
    }

    fn generate_trait_method(&self, f: &Function, trait_name: &str) -> String {
        let ret = self.type_name(f.ret, trait_name);
        let ret = if f.async_fn {
            format!("Promise<{ret}>")
        } else {
            ret
        };
        let context = Context {
            ret: Some(ret.clone()),
            error_type: None,
            ty: Some(trait_name.to_owned()),
            ..Default::default()
        };
        let args = self.generate_args(&f.args, &context);
        format!(
            "    {}{}({}): {};\n",
            self.ts_name(&f.name),
            self.generate_generics_decl(&f.generics),
            args,
            ret,
        )
    }

    fn collect_associated_types(functions: &[Function], out: &mut Vec<String>) {
        for f in functions {
            Self::collect_associated_from_type((f.ret)(), out);
            for arg in &f.args {
                if let Some(ty_fn) = arg.ty {
                    Self::collect_associated_from_type(ty_fn(), out);
                }
            }
        }
    }

    fn collect_associated_from_type(ty: Type, out: &mut Vec<String>) {
        match ty {
            Type::Generic(g) if g.name.contains("Self :: ") => {
                let name = g.name.strip_prefix("Self :: ").unwrap().to_owned();
                if !out.contains(&name) {
                    out.push(name);
                }
            }
            Type::Result { ok, err } => {
                Self::collect_associated_from_type(ok(), out);
                Self::collect_associated_from_type(err(), out);
            }
            Type::Option(inner) | Type::Vec(inner) | Type::Slice(inner) => {
                Self::collect_associated_from_type(inner(), out);
            }
            Type::Tuple(fields) => {
                for field in fields {
                    Self::collect_associated_from_type(field(), out);
                }
            }
            _ => {}
        }
    }

    fn generate_function(&self, f: &Function) -> String {
        let ret = self.type_name(f.ret, "this");
        let ret = if f.async_fn {
            format!("Promise<{}>", ret)
        } else {
            ret
        };
        let error_type = if let Type::Result { ok: _, err } = (f.ret)() {
            Some(self.type_name(err, "this"))
        } else {
            None
        };
        let async_keyword = if f.async_fn { "async " } else { "" };
        let context = Context {
            ret: Some(ret.clone()),
            error_type,
            ty: None,
            ..Default::default()
        };

        let body_str = if MANUAL_IMPL.contains(&f.name.as_str()) {
            "    throw new Error(\"not implemented\");\n".to_owned()
        } else {
            let body = self.rewrite_body(&f.body, &context);
            self.generate_semicoloned_expressions(&body, "    ", &context)
        };

        format!(
            "{}function {}{}({}): {} {{\n{}}}\n\n",
            async_keyword,
            self.ts_name(&f.name),
            self.generate_generics_decl(&f.generics),
            self.generate_args(&f.args, &context),
            ret,
            body_str,
        )
    }

    fn generate_member_function(&self, f: &Function, _i: &Impl, class_name: &str, class_generics_decl: &str) -> String {
        let ret = self.type_name(f.ret, class_name);
        let ret = if ret == "this" {
            class_name.to_owned()
        } else {
            ret
        };
        let ret = if f.async_fn {
            format!(": Promise<{ret}>")
        } else {
            format!(": {ret}")
        };
        let error_type = if let Type::Result { ok: _, err } = (f.ret)() {
            Some(self.type_name(err, class_name))
        } else {
            None
        };
        let async_keyword = if f.async_fn { "async " } else { "" };
        let static_keyword = if let Some(var) = f.args.first()
            && var.name == "self"
        {
            ""
        } else {
            "static "
        };

        let into_map = Self::resolve_into_generics(f);
        let dbtype_generics = Self::resolve_dbtype_generics(f);

        let context = Context {
            ret: Some(ret.clone()),
            error_type,
            ty: Some(class_name.to_string()),
            into_targets: into_map
                .iter()
                .map(|(param, (target, _))| (param.clone(), target.clone()))
                .collect(),
        };

        let is_manual = MANUAL_IMPL.contains(&f.name.as_str())
            || MANUAL_IMPL_QUALIFIED
                .iter()
                .any(|(cls, method)| class_name.starts_with(cls) && *method == f.name);
        let body_str = if is_manual {
            "        throw new Error(\"not implemented\");\n".to_owned()
        } else {
            let mut body = self.rewrite_body(&f.body, &context);
            Self::rewrite_static_type_calls(&mut body, &dbtype_generics);
            self.generate_semicoloned_expressions(&body, "        ", &context)
        };

        let non_into_generics: Vec<_> = f
            .generics
            .iter()
            .filter(|g| {
                !into_map
                    .values()
                    .any(|(_, _)| Self::generic_is_into_only(g))
            })
            .cloned()
            .collect();

        let generics_decl = if static_keyword == "static " && !class_generics_decl.is_empty() {
            if non_into_generics.is_empty() {
                class_generics_decl.to_owned()
            } else {
                let fn_generics = self.generate_generics_decl(&non_into_generics);
                let inner = &fn_generics[1..fn_generics.len() - 1];
                let class_inner = &class_generics_decl[1..class_generics_decl.len() - 1];
                format!("<{class_inner}, {inner}>")
            }
        } else {
            self.generate_generics_decl(&non_into_generics)
        };

        let companion_params = dbtype_generics
            .iter()
            .map(|(name, _)| format!("{}_type: DbTypeStatic<{name}>", name.to_lowercase()))
            .collect::<Vec<_>>()
            .join(", ");

        let mut args_str = self.generate_args_with_into(&f.args, &context, &into_map);
        if !companion_params.is_empty() {
            if !args_str.is_empty() {
                args_str.push_str(", ");
            }
            args_str.push_str(&companion_params);
        }

        format!(
            "    {}{}{}{}({}){} {{\n{}    }}\n\n",
            static_keyword,
            async_keyword,
            self.ts_name(&f.name),
            generics_decl,
            args_str,
            ret,
            body_str,
        )
    }

    fn resolve_dbtype_generics(f: &Function) -> Vec<(String, String)> {
        let mut result = Vec::new();
        for g in &f.generics {
            for bound_fn in &g.bounds {
                let bound_type = bound_fn();
                if let Type::Trait(t) = &bound_type {
                    if t.name == "DbType" {
                        result.push((g.name.clone(), "DbType".to_owned()));
                    }
                }
            }
        }
        result
    }

    fn rewrite_static_type_calls(body: &mut [Expression], dbtype_generics: &[(String, String)]) {
        if dbtype_generics.is_empty() {
            return;
        }
        for expr in body.iter_mut() {
            Self::rewrite_static_type_call_expr(expr, dbtype_generics);
        }
    }

    fn rewrite_static_type_call_expr(expr: &mut Expression, dbtype_generics: &[(String, String)]) {
        let taken = std::mem::take(expr);
        *expr = Self::transform_static_calls(taken, dbtype_generics);
    }

    fn transform_static_calls(
        expr: Expression,
        dbtype_generics: &[(String, String)],
    ) -> Expression {
        match expr {
            Expression::Call {
                recipient: None,
                function,
                args,
            } if Self::is_type_param_static_call(&function, dbtype_generics).is_some() => {
                let (param_name, method) =
                    Self::is_type_param_static_call(&function, dbtype_generics).unwrap();
                Expression::Call {
                    recipient: Some(Box::new(Expression::Ident(format!(
                        "{}_type",
                        param_name.to_lowercase()
                    )))),
                    function: Box::new(Expression::Ident(method)),
                    args,
                }
            }
            Expression::Block(stmts) => Expression::Block(
                stmts
                    .into_iter()
                    .map(|s| Self::transform_static_calls(s, dbtype_generics))
                    .collect(),
            ),
            Expression::Return(inner) => Expression::Return(
                inner.map(|e| Box::new(Self::transform_static_calls(*e, dbtype_generics))),
            ),
            Expression::Let { name, ty, value } => Expression::Let {
                name,
                ty,
                value: value.map(|v| Box::new(Self::transform_static_calls(*v, dbtype_generics))),
            },
            Expression::Call {
                recipient,
                function,
                args,
            } => Expression::Call {
                recipient: recipient
                    .map(|r| Box::new(Self::transform_static_calls(*r, dbtype_generics))),
                function: Box::new(Self::transform_static_calls(*function, dbtype_generics)),
                args: args
                    .into_iter()
                    .map(|a| Self::transform_static_calls(a, dbtype_generics))
                    .collect(),
            },
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => Expression::If {
                condition: Box::new(Self::transform_static_calls(*condition, dbtype_generics)),
                then_branch: Box::new(Self::transform_static_calls(*then_branch, dbtype_generics)),
                else_branch: else_branch
                    .map(|e| Box::new(Self::transform_static_calls(*e, dbtype_generics))),
            },
            Expression::FieldAccess { base, field } => Expression::FieldAccess {
                base: Box::new(Self::transform_static_calls(*base, dbtype_generics)),
                field,
            },
            Expression::Struct { name, fields } => Expression::Struct {
                name,
                fields: fields
                    .into_iter()
                    .map(|(k, v)| (k, Self::transform_static_calls(v, dbtype_generics)))
                    .collect(),
            },
            Expression::TupleStruct { name, expressions } => Expression::TupleStruct {
                name,
                expressions: expressions
                    .into_iter()
                    .map(|e| Self::transform_static_calls(e, dbtype_generics))
                    .collect(),
            },
            other => other,
        }
    }

    fn is_type_param_static_call(
        function: &Expression,
        dbtype_generics: &[(String, String)],
    ) -> Option<(String, String)> {
        if let Expression::Path {
            ident,
            parent: Some(parent),
            ..
        } = function
        {
            let parent_name = match parent.as_ref() {
                Expression::Ident(name) => Some(name.as_str()),
                Expression::Path {
                    ident,
                    parent: None,
                    ..
                } => Some(ident.as_str()),
                _ => None,
            };
            if let Some(name) = parent_name {
                if dbtype_generics.iter().any(|(g, _)| g == name) {
                    return Some((name.to_owned(), ident.clone()));
                }
            }
        }
        None
    }

    fn resolve_into_generics(f: &Function) -> HashMap<String, (String, String)> {
        let mut generic_to_target: HashMap<String, (String, String)> = HashMap::new();

        for g in &f.generics {
            for bound_fn in &g.bounds {
                let bound_type = bound_fn();
                if let Type::Trait(t) = &bound_type {
                    if t.name == "Into" && !t.generics.is_empty() {
                        let target_name = &t.generics[0].name;
                        let ts_type = if let Some((_, mapped)) =
                            INTO_TYPE_MAP.iter().find(|(name, _)| *name == target_name)
                        {
                            mapped.to_string()
                        } else {
                            format!("{target_name}Input")
                        };
                        generic_to_target.insert(
                            g.name.clone(),
                            (target_name.to_string(), ts_type),
                        );
                    }
                }
            }
        }

        let mut param_map: HashMap<String, (String, String)> = HashMap::new();
        for arg in &f.args {
            if let Some(ty_fn) = arg.ty {
                let ty = ty_fn();
                if let Type::Generic(ref g) = ty {
                    if let Some(entry) = generic_to_target.get(&g.name) {
                        param_map.insert(arg.name.clone(), entry.clone());
                    }
                }
            }
        }

        param_map
    }

    fn generic_is_into_only(g: &Generic) -> bool {
        if g.bounds.is_empty() {
            return false;
        }
        g.bounds.iter().all(|bound_fn| {
            let ty = bound_fn();
            match ty {
                Type::Trait(t) => {
                    t.name == "Into"
                        || t.name == "Send"
                        || t.name == "Sync"
                        || t.name == "Clone"
                        || t.name == "Copy"
                }
                _ => false,
            }
        })
    }

    fn generate_args_with_into(
        &self,
        args: &[Variable],
        context: &Context,
        into_map: &HashMap<String, (String, String)>,
    ) -> String {
        args.iter()
            .filter_map(|arg| {
                if arg.name == "self" {
                    return None;
                }

                if let Some((_target, ts_type)) = into_map.get(&arg.name) {
                    Some(format!("{}: {ts_type}", arg.name))
                } else {
                    let ty = self.type_name(
                        arg.ty.expect("expected type function"),
                        context.ty.as_deref().unwrap_or_default(),
                    );
                    Some(format!("{}: {ty}", arg.name))
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_semicoloned_expressions(
        &self,
        exprs: &[Expression],
        padding: &str,
        context: &Context,
    ) -> String {
        let mut buffer = String::new();
        let mut declared: Vec<String> = Vec::new();
        exprs.iter().for_each(|expr| {
            if matches!(expr, Expression::Block(stmts) if stmts.is_empty()) {
                return;
            }
            buffer.push_str(padding);
            if let Expression::Let { name, ty, value } = expr {
                if let Some(var_name) = Self::extract_let_name(name) {
                    if declared.contains(&var_name) {
                        if let Some(value) = value {
                            buffer.push_str(&format!(
                                "{} = {}",
                                var_name,
                                self.generate_expression(value, context)
                            ));
                        }
                    } else {
                        declared.push(var_name);
                        buffer.push_str(&self.generate_expression(expr, context));
                    }
                } else {
                    buffer.push_str(&self.generate_expression(expr, context));
                }
            } else {
                buffer.push_str(&self.generate_expression(expr, context));
            }
            buffer.push_str(";\n");
        });

        buffer
    }

    fn extract_let_name(name: &Expression) -> Option<String> {
        match name {
            Expression::Ident(n) => Some(n.clone()),
            _ => None,
        }
    }

    fn call_args(&self, exprs: &[Expression], context: &Context) -> String {
        exprs
            .iter()
            .map(|expr| self.generate_expression(expr, context))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_expressions(&self, exprs: &[Expression], context: &Context) -> String {
        let mut buffer = String::new();
        exprs.iter().for_each(|expr| {
            buffer.push_str(&self.generate_expression(expr, context));
        });

        buffer
    }

    fn generate_expression(&self, expr: &Expression, context: &Context) -> String {
        match expr {
            Expression::Array(e) => format!(
                "[{}]",
                e.iter()
                    .map(|expr| self.generate_expression(expr, context))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::Assign { target, value } => format!(
                "{} = {}",
                self.generate_expression(target, context),
                self.generate_expression(value, context)
            ),
            Expression::Await(e) => {
                format!("(await {})", self.generate_expression(e, context))
            }
            Expression::Binary { op, left, right } => self.binary(*op, left, right, context),
            Expression::Block(e) => {
                format!(
                    "{{\n{}\n}}",
                    self.generate_semicoloned_expressions(e, "        ", context)
                )
            }
            Expression::Break => "break".to_owned(),
            Expression::Call {
                recipient,
                function,
                args,
            } => self.call(recipient.as_deref(), function, args, context),
            Expression::Closure(function) => format!(
                "({}) => {{\n{}}}",
                self.generate_closure_args(&function.args, context),
                self.generate_semicoloned_expressions(&function.body, "    ", context)
            ),
            Expression::Continue => "continue".to_owned(),
            Expression::FieldAccess { base, field } => {
                format!("{}.{}", self.generate_expression(base, context), field)
            }
            Expression::For {
                pattern,
                iterable,
                body,
            } => self.for_loop(pattern, iterable, body, context),
            Expression::Format {
                format_string,
                args,
            } => self.generate_format_string(format_string, args, context),
            Expression::Ident(i) => (*i).to_owned(),
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => format!(
                "if ({}) {{\n{}\n}}{}",
                self.generate_expression(condition, context),
                self.generate_expression(then_branch, context),
                if let Some(else_branch) = else_branch {
                    format!(
                        " else {{\n{}\n}}",
                        self.generate_expression(else_branch, context)
                    )
                } else {
                    String::new()
                }
            ),
            Expression::Index { base, index } => format!(
                "{}[{}]",
                self.generate_expression(base, context),
                self.generate_expression(index, context)
            ),
            Expression::Let { name, ty, value } => {
                self.let_expression(name, *ty, value.as_deref(), context)
            }
            Expression::Literal(literal_value) => self.literal_value(literal_value).to_owned(),
            Expression::Path {
                ident,
                parent,
                generics,
            } => self.path(ident, parent.as_deref(), generics, context),
            Expression::Range {
                start,
                end,
                inclusive,
            } => self.range(start.as_deref(), end.as_deref(), *inclusive, context),
            Expression::Reference(expression) => self.generate_expression(expression, context),
            Expression::Return(expression) => {
                if let Some(expression) = expression {
                    format!("return {}", self.generate_expression(expression, context))
                } else {
                    "return".to_owned()
                }
            }
            Expression::Struct { name, fields } => {
                let struct_name = self.generate_expression(name, context);
                let args = fields
                    .iter()
                    .map(|(_field_name, expr)| self.generate_expression(expr, context))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("new {struct_name}({args})")
            }
            Expression::StructPattern { name: _, fields } => format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|pattern| { self.generate_expression(pattern, context) })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::Try(expression) => {
                format!(
                    "unwrapResult({})",
                    self.generate_expression(expression, context)
                )
            }
            Expression::Tuple(expressions) => format!(
                "[{}]",
                expressions
                    .iter()
                    .map(|expr| self.generate_expression(expr, context))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::TupleStruct { name, expressions } => {
                let struct_name = self.generate_expression(name, context);
                let args = expressions
                    .iter()
                    .map(|expr| self.generate_expression(expr, context))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("new {struct_name}({args})")
            }
            Expression::TupleAccess { base, index } => {
                format!("{}[{}]", self.generate_expression(base, context), index)
            }
            Expression::Unary { op, expr } => {
                let inner = self.generate_expression(expr, context);
                let needs_parens = matches!(expr.as_ref(), Expression::Binary { .. });
                if needs_parens {
                    format!("{}({})", self.generate_op(*op), inner)
                } else {
                    format!("{}{}", self.generate_op(*op), inner)
                }
            }
            Expression::While { condition, body } => format!(
                "while ({}) {{\n{}\n}}",
                self.generate_expression(condition, context),
                self.generate_expression(body, context)
            ),
            Expression::Match { scrutinee, arms } => {
                self.generate_match(scrutinee, arms, context)
            }
            Expression::Wild => "_".to_owned(),
        }
    }

    fn generate_match(
        &self,
        scrutinee: &Expression,
        arms: &[agdb::type_def::MatchArm],
        context: &Context,
    ) -> String {
        let scrutinee_str = self.generate_expression(scrutinee, context);
        let mut parts = Vec::new();

        for (i, arm) in arms.iter().enumerate() {
            let is_wild = matches!(arm.pattern, Pattern::Wild);
            let condition = if is_wild {
                None
            } else {
                Some(self.generate_match_condition(
                    &scrutinee_str,
                    &arm.pattern,
                    arm.guard.as_deref(),
                    context,
                ))
            };

            let body_str = match arm.body.as_ref() {
                Expression::Block(stmts) => {
                    self.generate_semicoloned_expressions(stmts, "    ", context)
                }
                other => format!("    return {};\n", self.generate_expression(other, context)),
            };
            let keyword = if i == 0 { "if" } else { "} else if" };

            if let Some(cond) = condition {
                parts.push(format!("{keyword} ({cond}) {{\n{body_str}"));
            } else {
                if i == 0 {
                    parts.push(body_str);
                } else {
                    parts.push(format!("}} else {{\n{body_str}"));
                }
            }
        }

        if !parts.is_empty()
            && !matches!(arms.last(), Some(arm) if matches!(arm.pattern, Pattern::Wild))
        {
            parts.push("}".to_owned());
        } else if arms.len() > 1 {
            parts.push("}".to_owned());
        }

        let body = parts.join("");
        format!("(() => {{\n{body}}})()")
    }

    fn generate_match_condition(
        &self,
        scrutinee: &str,
        pattern: &Pattern,
        guard: Option<&Expression>,
        context: &Context,
    ) -> String {
        let pattern_cond = match pattern {
            Pattern::Literal(lit) => {
                format!("{scrutinee} === {}", self.literal_value(lit))
            }
            Pattern::Ident(_) => "true".to_owned(),
            Pattern::Constructor { name, fields } => {
                if fields.is_empty() {
                    format!("{scrutinee}.value.{name} !== undefined")
                } else {
                    format!("{scrutinee}.value.{name} !== undefined")
                }
            }
            Pattern::Tuple(patterns) => {
                let conditions: Vec<String> = patterns
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| match p {
                        Pattern::Wild | Pattern::Ident(_) => None,
                        Pattern::Constructor { name, .. } => {
                            Some(format!("{scrutinee}[{i}].value.{name} !== undefined"))
                        }
                        Pattern::Literal(lit) => {
                            Some(format!("{scrutinee}[{i}] === {}", self.literal_value(lit)))
                        }
                        _ => None,
                    })
                    .collect();
                if conditions.is_empty() {
                    "true".to_owned()
                } else {
                    conditions.join(" && ")
                }
            }
            Pattern::Or(patterns) => patterns
                .iter()
                .map(|p| self.generate_match_condition(scrutinee, p, None, context))
                .collect::<Vec<_>>()
                .join(" || "),
            Pattern::Struct { .. } => "true".to_owned(),
            Pattern::Wild => "true".to_owned(),
        };

        if let Some(guard) = guard {
            let guard_str = self.generate_expression(guard, context);
            format!("{pattern_cond} && {guard_str}")
        } else {
            pattern_cond
        }
    }

    fn generate_pattern(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Ident(ident) => (*ident).to_owned(),
            Pattern::Literal(literal_value) => self.literal_value(literal_value).to_owned(),
            Pattern::Struct { name: _, fields } => format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|(field_name, pattern)| {
                        format!("{}: {}", field_name, self.generate_pattern(pattern))
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Pattern::Tuple(patterns) => format!(
                "[{}]",
                patterns
                    .iter()
                    .map(|pattern| self.generate_pattern(pattern))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Pattern::Wild => "_".to_owned(),
            Pattern::Constructor { name, fields } => self.constructor(name, fields),
            Pattern::Or(patterns) => format!(
                "({})",
                patterns
                    .iter()
                    .map(|pattern| self.generate_pattern(pattern))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
        }
    }

    fn generate_format_string(
        &self,
        format_string: &str,
        args: &[Expression],
        context: &Context,
    ) -> String {
        format!(
            "`{}`",
            format_string
                .split("{}")
                .zip(
                    args.iter()
                        .map(|arg| self.generate_expression(arg, context))
                )
                .map(|(part, arg)| format!("{}${{{}}}", part, arg))
                .collect::<Vec<_>>()
                .join("")
        )
    }

    fn generate_op(&self, op: Op) -> &str {
        match op {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Rem => "%",
            Op::And => "&&",
            Op::Or => "||",
            Op::BitXor => "^",
            Op::BitAnd => "&",
            Op::BitOr => "|",
            Op::Shl => "<<",
            Op::Shr => ">>",
            Op::Eq => "===",
            Op::Ne => "!==",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
            Op::AddAssign => "+=",
            Op::SubAssign => "-=",
            Op::MulAssign => "*=",
            Op::DivAssign => "/=",
            Op::RemAssign => "%=",
            Op::BitXorAssign => "^=",
            Op::BitAndAssign => "&=",
            Op::BitOrAssign => "|=",
            Op::ShlAssign => "<<=",
            Op::ShrAssign => ">>=",
            Op::Not => "!",
            Op::Neg => "-",
            Op::Deref => "",
        }
    }

    fn generate_args(&self, args: &[Variable], context: &Context) -> String {
        args.iter()
            .filter_map(|arg| {
                if arg.name == "self" {
                    return None;
                }

                let ty = self.type_name(
                    arg.ty.expect("expected type function"),
                    context.ty.as_deref().unwrap_or_default(),
                );
                Some(format!("{}: {ty}", arg.name))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_closure_args(&self, args: &[Variable], context: &Context) -> String {
        args.iter()
            .filter_map(|arg| {
                if arg.name == "self" {
                    return None;
                }

                let name = if arg.name.starts_with('(') {
                    format!("[{}]", &arg.name[1..arg.name.len() - 1])
                } else {
                    arg.name.clone()
                };

                if let Some(ty_fn) = arg.ty {
                    let ty = (ty_fn)();
                    if matches!(ty, Type::Literal(agdb::type_def::Literal::Unit)) {
                        return Some(format!("{name}: any"));
                    }
                    let ty = self.type_name(ty_fn, context.ty.as_deref().unwrap_or_default());
                    Some(format!("{name}: {ty}"))
                } else {
                    Some(format!("{name}: any"))
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_static(&self, s: &Static) -> String {
        let raw_ty = (s.ty)();
        let is_once_lock = matches!(&raw_ty, Type::Pointer(p) if matches!(p.kind, agdb::type_def::PointerKind::OnceLock));
        let is_atomic = matches!(&raw_ty, Type::Struct(st) if st.name.starts_with("Atomic"));
        let mutable = is_once_lock || is_atomic;

        let ts_name = self.ts_name(&s.name);
        let context = Context::default();
        let body = self.rewrite_body(&s.value, &context);

        let declarator = if mutable { "let" } else { "const" };
        let ty = self.type_name(s.ty, "this");
        let initializer = if is_once_lock {
            String::new()
        } else {
            format!(" = {}", self.generate_expressions(&body, &context))
        };

        format!("export {declarator} {ts_name}: {ty}{initializer};\n\n")
    }

    fn ts_name<'a>(&self, name: &'a str) -> &'a str {
        match name {
            "delete" => "delete_",
            "case" => "case_",
            "var" => "var_",
            _ => name,
        }
    }

    fn binary(&self, op: Op, left: &Expression, right: &Expression, context: &Context) -> String {
        format!(
            "{} {} {}",
            self.generate_expression(left, context),
            self.generate_op(op),
            self.generate_expression(right, context)
        )
    }

    fn call(
        &self,
        recipient: Option<&Expression>,
        function: &Expression,
        args: &[Expression],
        context: &Context,
    ) -> String {
        let f = self.generate_expression(function, context);
        let rec = recipient
            .map(|r| self.generate_expression(r, context))
            .unwrap_or_default();

        let args = self.call_args(args, context);

        if rec.is_empty() {
            if let Some(ctor_name) = Self::constructor_name(function) {
                return format!("new {ctor_name}({args})");
            }
            if let Some(name) = Self::new_constructor_name(function) {
                if args.is_empty() {
                    return format!("new {name}()");
                } else {
                    return format!("{name}.new({args})");
                }
            }
        }

        let rec = if !rec.is_empty() {
            format!("{rec}.")
        } else {
            String::new()
        };
        format!("{rec}{f}({args})")
    }

    fn new_constructor_name(function: &Expression) -> Option<String> {
        const FACTORY_TYPES: &[&str] = &["TestServer", "TestServerImpl"];
        if let Expression::Path {
            ident,
            parent: Some(parent),
            ..
        } = function
        {
            if ident == "new" {
                let name = match parent.as_ref() {
                    Expression::Ident(name) => Some(name.clone()),
                    Expression::Path {
                        ident,
                        parent: None,
                        ..
                    } => Some(ident.clone()),
                    _ => None,
                };
                if let Some(ref n) = name {
                    if FACTORY_TYPES.contains(&n.as_str()) {
                        return None;
                    }
                }
                return name;
            }
        }
        None
    }

    fn constructor_name(function: &Expression) -> Option<String> {
        const NOT_CONSTRUCTORS: &[&str] = &["Ok", "Err", "Some", "None"];
        match function {
            Expression::Ident(name)
                if name.chars().next().is_some_and(|c| c.is_uppercase())
                    && !NOT_CONSTRUCTORS.contains(&name.as_str()) =>
            {
                Some(name.clone())
            }
            Expression::Path {
                ident,
                parent: None,
                ..
            } if ident.chars().next().is_some_and(|c| c.is_uppercase())
                && !NOT_CONSTRUCTORS.contains(&ident.as_str()) =>
            {
                Some(ident.clone())
            }
            _ => None,
        }
    }

    fn path(
        &self,
        ident: &str,
        parent: Option<&Expression>,
        generics: &[fn() -> Type],
        context: &Context,
    ) -> String {
        let parent_str = if let Some(parent) = parent {
            format!("{}.", self.generate_expression(parent, context))
        } else {
            String::new()
        };
        let generics = self.generate_generic_args_from_types(generics);
        let suffix = if parent.is_some() && self.is_void_enum_variant(ident, parent) {
            "()"
        } else {
            ""
        };
        format!("{parent_str}{ident}{generics}{suffix}")
    }

    fn is_void_enum_variant(&self, variant: &str, parent: Option<&Expression>) -> bool {
        let parent_name = match parent {
            Some(Expression::Ident(name)) => name.as_str(),
            Some(Expression::Path {
                ident,
                parent: None,
                ..
            }) => ident.as_str(),
            _ => return false,
        };

        self.types.iter().any(|ty| {
            if let Type::Enum(e) = ty {
                if e.name == parent_name {
                    return e.variants.iter().any(|v| {
                        v.name == variant
                            && v.ty.is_some_and(|ty_fn| {
                                matches!(ty_fn(), Type::Literal(Literal::Unit))
                            })
                    });
                }
            }
            false
        })
    }

    fn constructor(&self, name: &str, fields: &[Pattern]) -> String {
        format!(
            "{}({})",
            name,
            fields
                .iter()
                .map(|pattern| self.generate_pattern(pattern))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn for_loop(
        &self,
        pattern: &Expression,
        iterable: &Expression,
        body: &Expression,
        context: &Context,
    ) -> String {
        if let Expression::Range {
            start,
            end,
            inclusive,
        } = iterable
        {
            return format!(
                "{} {}",
                self.range(start.as_deref(), end.as_deref(), *inclusive, context),
                self.generate_expression(body, context)
            );
        }

        format!(
            "for (const {} of {}) {}",
            self.generate_expression(pattern, context),
            self.generate_expression(iterable, context),
            self.generate_expression(body, context)
        )
    }

    fn range(
        &self,
        start: Option<&Expression>,
        end: Option<&Expression>,
        inclusive: bool,
        context: &Context,
    ) -> String {
        format!(
            "for (let i = {}; i {} {}; i++)",
            if let Some(start) = start {
                self.generate_expression(start, context)
            } else {
                "0".to_owned()
            },
            if inclusive { "<=" } else { "<" },
            if let Some(end) = end {
                self.generate_expression(end, context)
            } else {
                "0".to_owned()
            }
        )
    }

    fn let_expression(
        &self,
        name: &Expression,
        ty: Option<fn() -> Type>,
        value: Option<&Expression>,
        context: &Context,
    ) -> String {
        if let Some(Expression::If {
            condition,
            then_branch,
            else_branch: Some(else_branch),
        }) = value
        {
            if let (Some(then_expr), Some(else_expr)) = (
                extract_single_return(then_branch),
                extract_single_return(else_branch),
            ) {
                let ty_str = if let Some(ty) = ty {
                    format!(
                        ": {}",
                        self.type_name(ty, context.ty.as_deref().unwrap_or_default())
                    )
                } else {
                    String::new()
                };
                return format!(
                    "let {}{} = {} ? {} : {}",
                    self.generate_expression(name, context),
                    ty_str,
                    self.generate_expression(condition, context),
                    self.generate_expression(then_expr, context),
                    self.generate_expression(else_expr, context),
                );
            }
        }

        let ty_annotation = if let Some(ty) = ty {
            format!(
                ": {}",
                self.type_name(ty, context.ty.as_deref().unwrap_or_default())
            )
        } else if matches!(value, Some(Expression::Array(elems)) if elems.is_empty()) {
            ": any[]".to_owned()
        } else {
            String::new()
        };

        let value_str = match value {
            Some(Expression::Block(stmts)) if stmts.len() > 1 => {
                let body = self.generate_semicoloned_expressions(stmts, "    ", context);
                format!(" = (() => {{\n{}}})()", body)
            }
            Some(value) => {
                format!(" = {}", self.generate_expression(value, context))
            }
            None => String::new(),
        };

        format!(
            "let {}{}{}",
            self.generate_expression(name, context),
            ty_annotation,
            value_str,
        )
    }
}

fn extract_single_return(expr: &Expression) -> Option<&Expression> {
    match expr {
        Expression::Block(stmts) if stmts.len() == 1 => extract_single_return(&stmts[0]),
        Expression::Return(Some(inner)) => Some(inner),
        _ => None,
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
