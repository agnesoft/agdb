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

const SKIP_LIST: &[&str] = &["reqwest_Client", "PathBuf", "Duration", "AtomicU16"];

#[derive(Default)]
struct Context {
    ret: Option<String>,
    error_type: Option<String>,
    ty: Option<String>,
}

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
  has_value: boolean;

  constructor(value: T | E, has_value: boolean) {
    this.value = value;
    this.has_value = has_value;
  }

  is_ok(): boolean {
    return this.has_value;
  }
}

export function Ok<T, E>(value: T): Result<T, E> {
  return new Result<T, E>(value, true);
}

export function Err<T, E>(error: E): Result<T, E> {
  return new Result<T, E>(error, false);
}

export class reqwest_Client {
    // This is a placeholder for the actual reqwest.Client type
}

export type AgdbApiResult<T> = Result<T, AgdbApiError>;

// END OF PREAMBLE

"#
    }

    fn test_preamble(&self, types: &[Type]) -> String {
        format!(
            r#"
// GENERATED TESTS - DO NOT EDIT

// PREAMBLE
import {{ readFileSync }} from "fs";
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
    export function current_exe(): Path {{
        return new Path("target/release/agdb_server");
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

// END OF PREAMBLE

"#,
            types
                .iter()
                .filter_map(|t| match t {
                    Type::Enum(e) => Some(e.name),
                    Type::Struct(s) if !SKIP_LIST.contains(&s.name) => Some(s.name),
                    Type::Function(f) => Some(f.name),
                    Type::Static(s) => Some(s.name),
                    Type::Trait(t) => Some(t.name),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Enum(e) => self.generate_enum(e, &self.type_name(ty, e.name)),
            Type::Struct(s) => self.generate_struct(s, &self.type_name(ty, s.name)),
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
            let variant_type =
                self.type_name(&(variant.ty.expect("expected a type function"))(), e_name);

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
                    self.type_name(&(v.ty.expect("expected a type function"))(), e.name)
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
                    self.type_name(&(ty)(), s.name)
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
            .map(|b| self.type_name(&(b)(), g.name))
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
                .map(|t| self.type_name(&t(), "this"))
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
                    self.type_name(&(field.ty.expect("expected type function"))(), s.name)
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

    fn type_name(&self, ty: &Type, class_name: &str) -> String {
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
            Type::Literal(l) => self.literal(l).to_string(),
            Type::Vec(inner) => format!("{}[]", self.type_name(&(inner)(), class_name)),
            Type::Function(f) => f.name.to_owned(),
            Type::Test(f) => f.name.to_owned(),
            Type::Generic(g) => self.type_name_generic(g),
            Type::Impl(_) => panic!("impl block does not have a name"),
            Type::Option(inner) => format!("Option<{}>", self.type_name(&(inner)(), class_name)),
            Type::Pointer(p) => self.type_name(&(p.ty)(), class_name),
            Type::Reference(r) => self.type_name(&(r.ty)(), class_name),
            Type::Result { ok, err } => format!(
                "Result<{}, {}>",
                self.type_name(&(ok)(), class_name),
                self.type_name(&(err)(), class_name)
            ),
            Type::SelfType(_) => class_name.to_owned(),
            Type::Slice(s) => format!("{}[]", self.type_name(&(s)(), class_name)),
            Type::Static(s) => self.type_name(&(s.ty)(), class_name),
            Type::Trait(t) => t.name.to_owned(),
            Type::Tuple(items) => {
                let types = items
                    .iter()
                    .map(|item| self.type_name(&(item)(), class_name))
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
        format!(
            "export interface {}{} {{\n}}\n\n",
            t.name,
            self.generate_generics_decl(t.generics)
        )
    }

    fn generate_function(&self, f: &Function) -> String {
        let ret = self.type_name(&(f.ret)(), "this");
        let ret = if f.async_fn {
            format!("Promise<{}>", ret)
        } else {
            ret
        };
        let error_type = if let Type::Result { ok: _, err } = &(f.ret)() {
            Some(self.type_name(&(err)(), "this"))
        } else {
            None
        };
        let async_keyword = if f.async_fn { "async " } else { "" };

        format!(
            "{}function {}{}({}): {} {{\n{}}}\n\n",
            async_keyword,
            self.ts_name(f.name),
            self.generate_generics_decl(f.generics),
            self.generate_args(
                f.args,
                &Context {
                    ret: Some(ret.clone()),
                    error_type: error_type.clone(),
                    ty: None,
                }
            ),
            ret,
            self.generate_semicoloned_expressions(
                f.body,
                "    ",
                &Context {
                    ret: Some(ret.clone()),
                    error_type,
                    ty: None,
                }
            ),
        )
    }

    fn generate_member_function(&self, f: &Function, _i: &Impl, class_name: &str) -> String {
        let ret = self.type_name(&(f.ret)(), class_name);
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
        let error_type = if let Type::Result { ok: _, err } = &(f.ret)() {
            Some(self.type_name(&(err)(), class_name))
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

        format!(
            "    {}{}{}{}({}){} {{\n{}    }}\n\n",
            static_keyword,
            async_keyword,
            self.ts_name(f.name),
            self.generate_generics_decl(f.generics),
            self.generate_args(
                f.args,
                &Context {
                    ret: Some(ret.clone()),
                    error_type: error_type.clone(),
                    ty: Some(class_name.to_string()),
                }
            ),
            ret,
            self.generate_semicoloned_expressions(
                f.body,
                "        ",
                &Context {
                    ret: Some(ret.clone()),
                    error_type,
                    ty: Some(class_name.to_string()),
                }
            ),
        )
    }

    fn generate_semicoloned_expressions(
        &self,
        exprs: &[Expression],
        padding: &str,
        context: &Context,
    ) -> String {
        let mut buffer = String::new();
        exprs.iter().for_each(|expr| {
            buffer.push_str(padding);
            buffer.push_str(&self.generate_expression(expr, context));
            buffer.push_str(";\n");
        });

        buffer
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
            Expression::Array(e) => format!("[{}]", self.generate_expressions(e, context)),
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
            } => self.call(*recipient, function, args, context),
            Expression::Closure(function) => format!(
                "({}) => {{{}}}",
                self.generate_args(function.args, context),
                self.generate_expressions(function.body, context)
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
            Expression::Ident(i) => {
                if *i == "self" {
                    "this".to_owned()
                } else {
                    (*i).to_owned()
                }
            }
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
            Expression::Let { name, ty, value } => self.let_expression(name, *ty, *value, context),
            Expression::Literal(literal_value) => self.literal_value(literal_value).to_owned(),
            Expression::Path {
                ident,
                parent,
                generics,
            } => self.path(ident, *parent, generics, context),
            Expression::Range {
                start,
                end,
                inclusive,
            } => self.range(*start, *end, *inclusive, context),
            Expression::Reference(expression) => self.generate_expression(expression, context),
            Expression::Return(expression) => {
                if let Some(expression) = expression {
                    format!("return {}", self.generate_expression(expression, context))
                } else {
                    "return".to_owned()
                }
            }
            Expression::Struct { name: _, fields } => format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|(field_name, expr)| {
                        format!(
                            "{}: {}",
                            field_name,
                            self.generate_expression(expr, context)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::StructPattern { name: _, fields } => format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|pattern| { self.generate_expression(pattern, context) })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::Try(expression) => self.generate_expression(expression, context),
            Expression::Tuple(expressions) => format!(
                "[{}]",
                expressions
                    .iter()
                    .map(|expr| self.generate_expression(expr, context))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::TupleStruct {
                name: _,
                expressions,
            } => format!(
                "[{}]",
                expressions
                    .iter()
                    .map(|expr| self.generate_expression(expr, context))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::TupleAccess { base, index } => {
                format!("{}[{}]", self.generate_expression(base, context), index)
            }
            Expression::Unary { op, expr } => format!(
                "{}{}",
                self.generate_op(*op),
                self.generate_expression(expr, context)
            ),
            Expression::While { condition, body } => format!(
                "while ({}) {{\n{}\n}}",
                self.generate_expression(condition, context),
                self.generate_expression(body, context)
            ),
            Expression::Match { scrutinee, arms } => format!(
                "switch ({}) {{\n{}\n}}",
                self.generate_expression(scrutinee, context),
                arms.iter()
                    .map(|arm| {
                        format!(
                            "case {}:\n{}\nbreak;",
                            self.generate_pattern(&arm.pattern),
                            if let Some(guard) = &arm.guard {
                                format!(
                                    "if ({}) {{\n{}\n}}",
                                    self.generate_expression(guard, context),
                                    self.generate_expression(arm.body, context)
                                )
                            } else {
                                self.generate_expression(arm.body, context)
                            }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            Expression::Wild => "_".to_owned(),
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
        format_string: &&'static str,
        args: &&'static [Expression],
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
                    &(arg.ty.expect("expected type function"))(),
                    context.ty.as_deref().unwrap_or_default(),
                );
                Some(format!("{}: {ty}", arg.name))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn generate_static(&self, s: &Static) -> String {
        let ty_name = self.type_name(&(s.ty)(), "this");
        let ts_name = self.ts_name(s.name);
        let expr = self.generate_expressions(s.value, &Context::default());
        let declarator = if expr.contains("OnceLock") || ty_name == "AtomicU16" {
            "let"
        } else {
            "const"
        };
        let ty = if ty_name == "AtomicU16" {
            "number".to_owned()
        } else {
            ty_name
        };
        let e = if expr.contains("OnceLock") {
            String::new()
        } else {
            format!(" = {expr}")
        };

        format!("export {declarator} {ts_name}: {ty}{e};\n\n")
    }

    fn ts_name<'a>(&self, name: &'a str) -> &'a str {
        match name {
            "delete" => "delete_",
            _ => name,
        }
    }

    fn binary(
        &self,
        op: Op,
        left: &'static Expression,
        right: &'static Expression,
        context: &Context,
    ) -> String {
        format!(
            "{} {} {}",
            self.generate_expression(left, context),
            self.generate_op(op),
            self.generate_expression(right, context)
        )
    }

    fn call(
        &self,
        recipient: Option<&'static Expression>,
        function: &'static Expression,
        args: &'static [Expression],
        context: &Context,
    ) -> String {
        let f = self.generate_expression(function, context);
        let rec = recipient
            .map(|r| self.generate_expression(r, context))
            .unwrap_or_default();

        match f.as_str() {
            "AtomicU16.new" => {
                return self.generate_expression(&args[0], context);
            }
            "Arc.downgrade" => {
                return self.generate_expression(&args[0], context);
            }
            "Arc.new" => {
                return self.generate_expression(&args[0], context);
            }
            "bail" if let Some(error_type) = &context.error_type => {
                return format!(
                    "Err(new {}({}))",
                    error_type,
                    self.generate_expression(&args[0], context)
                );
            }
            "fetch_add" => {
                return self.binary(
                    Op::AddAssign,
                    recipient.expect("fetch_add must have a recipient"),
                    &args[0],
                    context,
                );
            }
            "get_or_init" => {
                return rec;
            }
            "is_empty" => {
                return format!("{rec}.length === 0");
            }
            "len" => {
                return format!("{rec}.length");
            }
            "map_err" => {
                return rec;
            }
            "serde_json.from_str" => {
                return format!(
                    "JSON.parse({})",
                    self.generate_expression(&args[0], context)
                );
            }
            "std.thread.sleep" => {
                return format!(
                    "await std.thread.sleep({})",
                    self.generate_expression(&args[0], context)
                );
            }
            "upgrade" => {
                return rec;
            }
            "write" => {
                return rec;
            }
            _ => {}
        }

        let rec = if !rec.is_empty() {
            format!("{}.", rec)
        } else {
            String::new()
        };

        let args = self.call_args(args, context);

        format!("{rec}{f}({args})")
    }

    fn path(
        &self,
        ident: &'static str,
        parent: Option<&'static Expression>,
        generics: &'static [fn() -> Type],
        context: &Context,
    ) -> String {
        let parent = if let Some(parent) = parent {
            format!("{}.", self.generate_expression(parent, context))
        } else {
            String::new()
        };
        let generics = self.generate_generic_args_from_types(generics);
        let ident = if ident == "Self" {
            context.ty.as_deref().unwrap_or(ident)
        } else {
            ident
        };
        format!("{parent}{ident}{generics}")
    }

    fn constructor(&self, name: &str, fields: &'static [Pattern]) -> String {
        if name == "AtomicU16" {
            return self.generate_pattern(&fields[0]);
        }

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
        pattern: &'static Expression,
        iterable: &'static Expression,
        body: &'static Expression,
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
                self.range(*start, *end, *inclusive, context),
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
        start: Option<&'static Expression>,
        end: Option<&'static Expression>,
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
        name: &'static Expression,
        ty: Option<fn() -> Type>,
        value: Option<&'static Expression>,
        context: &Context,
    ) -> String {
        if let Some(value) = value
            && matches!(
                value,
                Expression::If {
                    condition: _,
                    then_branch: _,
                    else_branch: _
                }
            )
        {}

        format!(
            "let {}{}{}",
            self.generate_expression(name, context),
            if let Some(ty) = ty {
                format!(
                    ": {}",
                    self.type_name(&ty(), context.ty.as_deref().unwrap_or_default())
                )
            } else {
                String::new()
            },
            if let Some(value) = value {
                format!(" = {}", self.generate_expression(value, context))
            } else {
                String::new()
            }
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
