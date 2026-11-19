use crate::codegen::lowered::*;
use crate::codegen::pass::dict_global;
use crate::codegen::pass::is_statement;
use crate::codegen::pass::normalizer;
use crate::codegen::pass::return_tail;
use crate::codegen::pass::subst;
use crate::codegen::print;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt::Write;

/// What the TS module of a generator has beyond the module it is emitted
/// from.
#[derive(Clone, Debug, Default)]
pub struct TsConfig {
    /// The comment the file starts with.
    pub header: String,
    /// The import of the namespace of the schema types.
    pub import: String,
    /// The namespace of the schema types, e.g. `Components.Schemas`.
    pub schemas: String,
}

/// An emitted module.
#[derive(Debug)]
pub struct Ts {
    pub code: String,
    /// What TS represents differently from Rust, e.g. the arms of a
    /// conversion that TS does not tell apart.
    pub notes: Vec<String>,
    /// The expressions without a TS form, `function: expression`, emitted
    /// as `unsupported()`.
    pub unsupported: Vec<String>,
}

/// The TS module of a lowered module. The data types are their schema
/// types, the methods of data types are free functions, the builders are
/// classes and the conversion functions are normalizers over the runtime
/// representation of their arguments.
pub struct Emitter<'a> {
    m: &'a Module,
    config: &'a TsConfig,
    out: String,
    this: &'static str,
    notes: Vec<String>,
    unsupported: Vec<String>,
    /// The function being emitted, for [`Ts::unsupported`].
    current: String,
    schemas: RefCell<BTreeSet<String>>,
    /// The locals of the current function assigned after their `let`.
    reassigned: BTreeSet<String>,
    /// The conversion targets other normalizers accept, with a type guard.
    guarded: BTreeSet<String>,
    /// The generics of the current function.
    generics: Vec<Generic>,
    /// The indent of the current statement, for the bodies of its closures.
    indent: usize,
    /// Whether the code calls [`PANIC`].
    panics: bool,
    /// The names of the module the code uses, for [`Emitter::tests`].
    imports: BTreeSet<String>,
}

/// `panic!`: TS has no throw expression.
const PANIC: &str = "function panic(msg: string): never {\n    throw new Error(msg);\n}\n";

impl<'a> Emitter<'a> {
    pub fn new(m: &'a Module, config: &'a TsConfig) -> Self {
        Self {
            m,
            config,
            out: String::new(),
            this: "this",
            notes: Vec::new(),
            unsupported: Vec::new(),
            current: String::new(),
            schemas: RefCell::default(),
            reassigned: BTreeSet::new(),
            guarded: BTreeSet::new(),
            generics: Vec::new(),
            indent: 0,
            panics: false,
            imports: BTreeSet::new(),
        }
    }

    pub fn module(mut self) -> Ts {
        let m = self.m;

        for (name, sigs) in &m.traits {
            self.trait_interface(name, sigs);
        }

        self.dicts();
        self.constructors();

        for f in &m.functions {
            if let Some(arms) = type_case(f) {
                arms.iter()
                    .for_each(|(ty, _, _)| into_targets(ty, &mut self.guarded));
            }
        }

        for f in &m.functions {
            self.normalizer(f);
        }

        self.data_methods();

        for b in &m.builders {
            self.class(&m.classes[b]);
        }

        let mut head = format!("{}\n\n{}\n\n", self.config.header, self.config.import);

        for name in self.schemas.borrow().iter() {
            let _ = writeln!(head, "type {name} = {}.{name};", self.config.schemas);
        }

        if self.panics {
            head += PANIC;
        }

        Ts {
            code: head + &self.out,
            notes: self.notes,
            unsupported: self.unsupported,
        }
    }

    /// The [tests](Module::tests) of the module emitted as `module` as a
    /// vitest suite `name`. An `assert_eq!` expects its left value to equal
    /// the right one as its schema type, so the compiler checks the expected
    /// values against the schema.
    pub fn tests(mut self, name: &str, module: &str) -> Ts {
        let m = self.m;
        self.line(0, &format!("describe({name:?}, () => {{"));

        for (i, f) in m.tests.iter().enumerate() {
            if i > 0 {
                self.line(0, "");
            }

            self.current = f.name.clone();
            self.reassigned = reassigned(&f.body);
            self.line(1, &format!("it({:?}, () => {{", f.name));
            self.block(&f.body, 2);
            self.line(1, "});");
        }

        self.line(0, "});");
        let exported = |n: &String| *n == m.root || m.functions.iter().any(|f| f.name == *n);

        for n in self.imports.clone().iter().filter(|n| !exported(n)) {
            self.unsupported(&format!("{n} is not exported"));
        }

        let imports: Vec<&String> = self.imports.iter().filter(|n| exported(n)).collect();
        let mut head = format!(
            "{}

{}
import {{ describe, expect, it }} from \"vitest\";
",
            self.config.header, self.config.import
        );

        if !imports.is_empty() {
            let imports: Vec<&str> = imports.iter().map(|n| n.as_str()).collect();
            let _ = writeln!(head, "import {{ {} }} from {module:?};", imports.join(", "));
        }

        head += "\n";
        let mut types = String::new();

        for name in self.schemas.borrow().iter() {
            let _ = writeln!(types, "type {name} = {}.{name};", self.config.schemas);
        }

        if self.panics {
            types += PANIC;
        }

        if !types.is_empty() {
            head += &(types + "\n");
        }

        Ts {
            code: head + &self.out,
            notes: self.notes,
            unsupported: self.unsupported,
        }
    }

    fn ty(&self, ty: &Ty) -> String {
        match ty {
            Ty::Unit => "void".into(),
            Ty::Bool => "boolean".into(),
            Ty::Int(_) | Ty::Float => "number".into(),
            Ty::Str => "string".into(),
            Ty::List(inner) => {
                let inner = self.ty(inner);

                if inner.contains(' ') {
                    format!("({inner})[]")
                } else {
                    format!("{inner}[]")
                }
            }
            Ty::Option(inner) => format!("{} | undefined", self.ty(inner)),
            Ty::Tuple(items) => format!("[{}]", self.tys(items)),
            Ty::Named(name, args) if self.m.is_builder(name) => {
                let generics = self.m.generics_of(name);

                if generics.is_empty() {
                    return name.clone();
                }

                let args: Vec<String> = generics
                    .iter()
                    .enumerate()
                    .map(|(i, g)| args.get(i).map_or(g.name.clone(), |t| self.ty(t)))
                    .collect();
                format!("{name}<{}>", args.join(", "))
            }
            Ty::Named(name, _) => self.schema(name),
            Ty::Param(p) if self.is_reflective(p) => "object".into(),
            Ty::Param(p) => p.clone(),
            Ty::Into(to) => format!("Into{to}"),
            Ty::Dict(tr, of) => format!("{tr}<{}>", self.ty(of)),
            Ty::SelfTy | Ty::Ref(_) | Ty::Fn | Ty::Unknown => "unknown".into(),
        }
    }

    /// Whether the generic of the current function is bounded by a
    /// reflective trait: its values are the plain objects of the caller.
    fn is_reflective(&self, param: &str) -> bool {
        self.generics
            .iter()
            .any(|g| g.name == param && g.bounds.iter().any(|(b, _)| self.m.is_reflective(b)))
    }

    /// The type of `self` of the methods of a class, the erased one of a
    /// newtype.
    fn self_ty(&self, class: &str) -> Ty {
        self.m
            .newtypes
            .get(class)
            .cloned()
            .unwrap_or_else(|| Ty::named(class))
    }

    /// The entries of the object `value` as the (key, value) list `ty`.
    fn fields(&mut self, value: &Expr, ty: &Ty) -> String {
        let Ty::List(entry) = ty else {
            return self.unsupported(&print::ty(ty));
        };
        let Ty::Tuple(kv) = entry.as_ref() else {
            return self.unsupported(&print::ty(ty));
        };
        let [k, v] = kv.as_slice() else {
            return self.unsupported(&print::ty(ty));
        };
        let object = self.expr(value);
        format!(
            "Object.entries({object}).map(([k, v]): {} => [{}, {}])",
            self.ty(entry),
            self.caller_value(k, "k"),
            self.caller_value(v, "v")
        )
    }

    /// The value `v` of a caller's object as `ty`: converted by its
    /// normalizer, undefined or null as `None`.
    fn caller_value(&self, ty: &Ty, v: &str) -> String {
        match ty {
            Ty::Option(inner) => format!(
                "{v} === undefined || {v} === null ? undefined : {}",
                self.caller_value(inner, v)
            ),
            Ty::Named(n, _) if self.m.functions.iter().any(|f| f.name == normalizer(n)) => {
                format!("{}({v} as Into{n})", normalizer(n))
            }
            ty => format!("{v} as {}", self.ty(ty)),
        }
    }

    fn tys(&self, tys: &[Ty]) -> String {
        tys.iter()
            .map(|t| self.ty(t))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn schema(&self, name: &str) -> String {
        self.schemas.borrow_mut().insert(name.to_owned());
        name.to_owned()
    }

    fn lit(lit: &Lit) -> String {
        match lit {
            Lit::Bool(b) => b.to_string(),
            Lit::Int(i) => i.to_string(),
            Lit::Float(f) => format!("{f:?}"),
            Lit::Str(s) => format!("{s:?}"),
            Lit::Unit => "undefined".into(),
        }
    }

    fn list(&mut self, items: &[Expr]) -> String {
        items
            .iter()
            .map(|e| self.expr(e))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn is_builder_value(&self, e: &Expr) -> bool {
        matches!(e.k, K::This) || e.ty.name().is_some_and(|n| self.m.is_builder(n))
    }

    fn expr(&mut self, e: &Expr) -> String {
        match &e.k {
            K::This => self.this.into(),
            K::Local(name) => name.clone(),
            K::Lit(lit) => Self::lit(lit),
            K::Array(items) | K::Tuple(items) => format!("[{}]", self.list(items)),
            K::Construct { class, fields } if self.m.is_builder(class) => {
                let args: Vec<String> = self.m.classes[class]
                    .fields
                    .iter()
                    .map(|(name, _)| {
                        fields
                            .iter()
                            .find(|(f, _)| f == name)
                            .map_or("undefined".into(), |(_, value)| self.expr(value))
                    })
                    .collect();
                format!("new {class}({})", args.join(", "))
            }
            K::Construct { fields, .. } => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|(name, value)| format!("{}: {}", member(name), self.expr(value)))
                    .collect();
                format!("{{ {} }}", fields.join(", "))
            }
            K::Variant { en, payload, .. } if en == "Option" => match payload {
                Some(p) => self.expr(p),
                None => "undefined".into(),
            },
            K::Variant {
                var, payload: None, ..
            } => format!("{var:?}"),
            K::Variant {
                var,
                payload: Some(p),
                ..
            } => format!("{{ {var}: {} }}", self.expr(p)),
            K::Path(path) => path.join("."),
            K::Field { base, name }
                if name.parse::<usize>().is_ok() && !self.is_builder_value(base) =>
            {
                format!("{}[{name}]", self.expr(base))
            }
            K::Field { base, name } => format!("{}.{}", self.expr(base), member(name)),
            K::Payload { base, var, .. } => format!("{}.{var}", self.expr(base)),
            K::Unwrap(value) | K::Ref(value) | K::Deref(value) => self.expr(value),
            K::Index { base, index } => format!("{}[{}]", self.expr(base), self.expr(index)),
            K::Call { callee, recv, args } => self.call(callee, recv.as_deref(), args),
            K::Convert { value, to } => {
                self.imports.insert(normalizer(to));
                format!("{}({})", normalizer(to), self.expr(value))
            }
            K::Fields(value) => self.fields(value, &e.ty),
            K::Binary { op, l, r } => {
                let op = match *op {
                    "==" => "===",
                    "!=" => "!==",
                    op => op,
                };
                format!("{} {op} {}", self.expr(l), self.expr(r))
            }
            K::Not(value)
                if matches!(
                    value.k,
                    K::Binary { .. }
                        | K::Call {
                            callee: Callee::Intrinsic(Intrinsic::IsEmpty),
                            ..
                        }
                ) =>
            {
                format!("!({})", self.expr(value))
            }
            K::Not(value) => format!("!{}", self.expr(value)),
            K::If {
                cond,
                then,
                els: Some(els),
            } if then.len() == 1 && els.len() == 1 => format!(
                "({} ? {} : {})",
                self.expr(cond),
                self.expr(returned(&then[0])),
                self.expr(returned(&els[0]))
            ),
            K::Closure { params, body } => match body.as_slice() {
                [one] if !is_block_statement(one) => arrow(params, self.expr(returned(one))),
                _ => {
                    let saved = std::mem::take(&mut self.out);
                    self.block(body, self.indent + 1);
                    let body = std::mem::replace(&mut self.out, saved);
                    format!(
                        "({}) => {{\n{body}{}}}",
                        params.join(", "),
                        "    ".repeat(self.indent)
                    )
                }
            },
            K::Panic(message) => self.panic(&format!("{message:?}")),
            _ => self.unsupported(&print::expr(e)),
        }
    }

    fn call(&mut self, callee: &Callee, recv: Option<&Expr>, args: &[Expr]) -> String {
        let a = self.list(args);
        let r = recv.map(|r| self.expr(r)).unwrap_or_default();

        match callee {
            Callee::Member { class, name } if !self.m.is_builder(class) => {
                let a = if a.is_empty() { r } else { format!("{r}, {a}") };
                self.imports.insert(format!("{class}_{name}"));
                format!("{class}_{name}({a})")
            }
            Callee::Member { name, .. } | Callee::Trait { name, .. } => format!("{r}.{name}({a})"),
            Callee::Static { class, name } if !self.m.is_builder(class) => {
                self.imports.insert(format!("{class}_{name}"));
                format!("{class}_{name}({a})")
            }
            Callee::Static { class, name } => {
                self.imports.insert(class.clone());
                format!("{class}.{name}({a})")
            }
            Callee::Extern(name) => format!("{name}({a})"),
            Callee::Intrinsic(i) => match i {
                Intrinsic::Push => format!("{r}.push({a})"),
                Intrinsic::Pop => format!("{r}.pop()"),
                Intrinsic::LastMut => format!("{r}.at(-1)"),
                Intrinsic::Len => format!("{r}.length"),
                Intrinsic::IsEmpty => format!("{r}.length === 0"),
                Intrinsic::Extend => format!("{r}.push(...{a})"),
                Intrinsic::Reserve => "void 0".into(),
                Intrinsic::UnwrapOrDefault => format!("({r} ?? {a})"),
                Intrinsic::Expect => format!("({r} ?? {})", self.panic(&a)),
                Intrinsic::Map => format!("{r}.map({a})"),
                Intrinsic::IterForEach => format!("{r}.forEach({a})"),
                Intrinsic::AssertEq => {
                    let left = self.expr(&args[0]);
                    let right = self.expr(&args[1]);
                    let ty = self.ty(&args[1].ty);
                    format!("expect({left}).toEqual<{ty}>({right})")
                }
                Intrinsic::MemTake | Intrinsic::MemSwap => self.unsupported(&format!("{i:?}")),
            },
            Callee::Path(..) | Callee::Method(_) => self.unsupported(&format!("{callee:?}")),
        }
    }

    fn unsupported(&mut self, what: &str) -> String {
        self.unsupported.push(format!("{}: `{what}`", self.current));
        "unsupported()".into()
    }

    fn test(&mut self, test: &Test) -> String {
        match test {
            Test::IsSome(base) => format!("{} !== undefined", self.expr(base)),
            Test::IsNone(base) => format!("{} === undefined", self.expr(base)),
            Test::IsVariant(base, en, var)
                if matches!(self.m.variant(en, var), Some(Variant::Unit(_))) =>
            {
                format!("{} === {var:?}", self.expr(base))
            }
            Test::IsVariant(base, _, var) => {
                let base = self.expr(base);
                format!("typeof {base} === \"object\" && {base} !== null && {var:?} in {base}")
            }
            Test::Eq(base, lit) => format!("{} === {}", self.expr(base), Self::lit(lit)),
        }
    }

    fn conditions(&mut self, tests: &[Test]) -> String {
        if tests.is_empty() {
            return "true".into();
        }

        tests
            .iter()
            .map(|t| self.test(t))
            .collect::<Vec<_>>()
            .join(" && ")
    }

    fn panic(&mut self, message: &str) -> String {
        self.panics = true;
        format!("panic({message})")
    }

    fn line(&mut self, indent: usize, line: &str) {
        let _ = writeln!(self.out, "{}{line}", "    ".repeat(indent));
    }

    fn block(&mut self, body: &[Expr], indent: usize) {
        for s in body {
            self.stmt(s, indent);
        }
    }

    fn stmt(&mut self, s: &Expr, indent: usize) {
        let outer = std::mem::replace(&mut self.indent, indent);
        self.statement(s, indent);
        self.indent = outer;
    }

    fn statement(&mut self, s: &Expr, indent: usize) {
        match &s.k {
            K::Let { name, value } => self.binding(name, value.as_deref(), indent),
            K::Assign { place, value } => {
                let (place, value) = (self.expr(place), self.expr(value));
                self.line(indent, &format!("{place} = {value};"));
            }
            K::Return(value) => {
                let value = value
                    .as_ref()
                    .map(|v| format!(" {}", self.expr(v)))
                    .unwrap_or_default();
                self.line(indent, &format!("return{value};"));
            }
            K::If { cond, then, els } => {
                let cond = self.expr(cond);

                if cond == "true" && els.is_none() {
                    return self.block(then, indent);
                }

                self.line(indent, &format!("if ({cond}) {{"));
                self.block(then, indent + 1);
                self.els(els.as_deref(), indent);
            }
            K::IfLet {
                pat,
                value,
                then,
                els,
            } => {
                let (tests, binds) = pat.projections(value);
                let cond = self.conditions(&tests);
                self.line(indent, &format!("if ({cond}) {{"));
                self.arm(&binds, then, indent + 1);
                self.els(els.as_deref(), indent);
            }
            K::Match { value, arms } => self.arms(value, arms, indent),
            K::While { cond, body } => {
                let cond = self.expr(cond);

                if body.is_empty() {
                    self.line(indent, &format!("while ({cond});"));
                } else {
                    self.line(indent, &format!("while ({cond}) {{"));
                    self.block(body, indent + 1);
                    self.line(indent, "}");
                }
            }
            _ => {
                let e = self.expr(s);

                if e != "void 0" {
                    self.line(indent, &format!("{e};"));
                }
            }
        }
    }

    /// TS widens the literals of a `let` (a unit variant `"A"` to `string`,
    /// `[]` to `never[]`), so those have their type.
    fn binding(&mut self, name: &str, value: Option<&Expr>, indent: usize) {
        let literal = |v: &Expr| match &v.k {
            K::Array(items) => items.is_empty(),
            K::Construct { class, .. } => !self.m.is_builder(class),
            K::Variant { en, .. } => en != "Option",
            _ => false,
        };
        let typed = value.filter(|v| {
            (literal(v) || self.reassigned.contains(name))
                && match &v.ty {
                    Ty::Unknown => false,
                    Ty::List(inner) => **inner != Ty::Unknown,
                    _ => true,
                }
        });
        let ty = typed
            .map(|v| format!(": {}", self.ty(&v.ty)))
            .unwrap_or_default();
        let value = value.map_or("undefined".into(), |v| self.expr(v));
        let keyword = if self.reassigned.contains(name) {
            "let"
        } else {
            "const"
        };
        self.line(indent, &format!("{keyword} {name}{ty} = {value};"));
    }

    /// The body of a pattern arm. The bindings the body only assigns
    /// through were lowered to their places.
    fn arm(&mut self, binds: &[(String, Expr)], body: &[Expr], indent: usize) {
        for (name, place) in binds {
            if body
                .iter()
                .any(|s| s.any(&|e| matches!(&e.k, K::Local(x) if x == name)))
            {
                let place = self.expr(place);
                self.line(indent, &format!("const {name} = {place};"));
            }
        }

        self.block(body, indent);
    }

    /// A `match` as an `if` chain, exhaustive as Rust checked it.
    fn arms(&mut self, value: &Expr, arms: &[(Pat, Vec<Expr>)], indent: usize) {
        let mut open = false;

        for (pat, body) in arms {
            let (tests, binds) = pat.projections(value);

            if tests.is_empty() {
                if open {
                    self.line(indent, "} else {");
                    self.arm(&binds, body, indent + 1);
                    self.line(indent, "}");
                } else {
                    self.arm(&binds, body, indent);
                }

                return;
            }

            let cond = self.conditions(&tests);
            let keyword = if open { "} else if" } else { "if" };
            self.line(indent, &format!("{keyword} ({cond}) {{"));
            self.arm(&binds, body, indent + 1);
            open = true;
        }

        self.line(indent, "} else {");
        let panic = self.panic("\"unreachable\"");
        self.line(indent + 1, &format!("{panic};"));
        self.line(indent, "}");
    }

    fn els(&mut self, els: Option<&[Expr]>, indent: usize) {
        match els {
            Some(
                els @ [
                    Expr {
                        k: K::If { .. } | K::IfLet { .. } | K::Match { .. },
                        ..
                    },
                ],
            ) => {
                let saved = std::mem::take(&mut self.out);
                self.block(els, indent);
                let nested = std::mem::replace(&mut self.out, saved);

                if nested.trim_start().starts_with("if (") {
                    let _ = write!(
                        self.out,
                        "{}}} else {}",
                        "    ".repeat(indent),
                        nested.trim_start()
                    );
                } else {
                    self.line(indent, "} else {");
                    self.block(els, indent + 1);
                    self.line(indent, "}");
                }
            }
            Some(els) if !els.is_empty() => {
                self.line(indent, "} else {");
                self.block(els, indent + 1);
                self.line(indent, "}");
            }
            _ => self.line(indent, "}"),
        }
    }

    /// The test that a value is of the runtime representation of `ty`.
    fn guard(&self, ty: &Ty, v: &str) -> String {
        let object = |v: &str| format!("typeof {v} === \"object\" && {v} !== null");

        match ty {
            Ty::Bool => format!("typeof {v} === \"boolean\""),
            Ty::Int(_) => format!("Number.isInteger({v})"),
            Ty::Float => format!("typeof {v} === \"number\""),
            Ty::Str => format!("typeof {v} === \"string\""),
            Ty::List(inner) => format!(
                "Array.isArray({v}) && {v}.every((x) => {})",
                self.guard(inner, "x")
            ),
            Ty::Tuple(items) => {
                let mut guards = vec![
                    format!("Array.isArray({v})"),
                    format!("{v}.length === {}", items.len()),
                ];
                guards.extend(
                    items
                        .iter()
                        .enumerate()
                        .map(|(i, t)| self.guard(t, &format!("{v}[{i}]"))),
                );
                guards.join(" && ")
            }
            Ty::Option(inner) => format!("({v} === undefined || {})", self.guard(inner, v)),
            Ty::Into(to) => format!("is_{}({v})", normalizer(to)),
            Ty::Param(_) => format!("{} && !Array.isArray({v})", object(v)),
            Ty::Named(name, _) => {
                if let Some(e) = self.m.enums.get(name) {
                    let (units, tagged): (Vec<&Variant>, Vec<&Variant>) = e
                        .variants
                        .iter()
                        .partition(|x| matches!(x, Variant::Unit(_)));
                    let mut alternatives: Vec<String> = units
                        .iter()
                        .map(|x| format!("{v} === {:?}", x.name()))
                        .collect();

                    if !tagged.is_empty() {
                        let tags: Vec<String> = tagged
                            .iter()
                            .map(|x| format!("{:?} in {v}", x.name()))
                            .collect();
                        alternatives.push(format!("({} && ({}))", object(v), tags.join(" || ")));
                    }

                    alternatives.join(" || ")
                } else if let Some(c) = self.m.classes.get(name) {
                    std::iter::once(object(v))
                        .chain(c.fields.iter().map(|(f, _)| format!("{f:?} in {v}")))
                        .collect::<Vec<_>>()
                        .join(" && ")
                } else {
                    "true".into()
                }
            }
            _ => "true".into(),
        }
    }

    /// The order of the arms of a normalizer: the types of the Rust
    /// literals without a suffix (`i32`, `f64`) first, the way they select
    /// the `From` impl in Rust.
    fn literal_rank(ty: &Ty) -> usize {
        match ty {
            Ty::Int("i64") => 1,
            Ty::Int("i32") => 0,
            Ty::Int(_) => 2,
            Ty::List(inner) => Self::literal_rank(inner),
            _ => 0,
        }
    }

    /// The order of the arms by how specific their guards are.
    fn shape_rank(ty: &Ty) -> usize {
        match ty {
            Ty::Bool | Ty::Int(_) => 0,
            Ty::Float | Ty::Str => 1,
            Ty::List(_) | Ty::Tuple(_) => 2,
            Ty::Named(..) => 3,
            Ty::Into(_) => 4,
            _ => 5,
        }
    }

    fn normalizer(&mut self, f: &Func) {
        self.current = f.name.clone();
        self.generics = f.generics.clone();
        let Some(arms) = type_case(f) else {
            self.unsupported("a function other than a conversion");
            return;
        };
        let to = f.name.trim_start_matches("into_").to_owned();
        let v = Expr::local("v", Ty::Unknown);
        let mut arms = arms.to_vec();
        arms.sort_by_key(|(ty, _, _)| (Self::shape_rank(ty), Self::literal_rank(ty)));
        let mut accepted: Vec<String> = Vec::new();
        let mut guards: Vec<String> = Vec::new();
        let mut kept = Vec::new();

        for (ty, pat, body) in arms {
            let union = self.ty(&ty);

            if !accepted.contains(&union) {
                accepted.push(union);
            }

            if self.subsumed(&pat, f.ret.name().unwrap_or(&to)) {
                self.notes.push(format!(
                    "{to}: `{}` arm is the identity in TS, dropped",
                    print::pat(&pat)
                ));
                continue;
            }

            let (tests, binds) = pat.projections(&v);
            let guard = if matches!(pat, Pat::Variant { .. }) {
                self.conditions(&tests)
            } else {
                self.guard(&ty, "v")
            };

            if guards.contains(&guard) {
                self.notes.push(format!(
                    "{to}: `{}` has the TS representation of an earlier arm, dropped",
                    print::ty(&ty)
                ));
                continue;
            }

            guards.push(guard.clone());
            let cast = match &pat {
                Pat::Variant { en, var, .. } => {
                    format!("Extract<{}, {{ {var}: unknown }}>", self.schema(en))
                }
                _ => self.ty(&ty),
            };
            kept.push((ty, binds, guard, body, cast));
        }

        self.line(
            0,
            &format!("export type Into{to} = {};\n", accepted.join(" | ")),
        );

        if self.guarded.contains(&to) {
            self.line(
                0,
                &format!("function is_{}(v: unknown): v is Into{to} {{", f.name),
            );
            let all: Vec<String> = kept
                .iter()
                .map(|(_, _, g, _, _)| format!("({g})"))
                .collect();
            self.line(1, &format!("return {};", all.join(" || ")));
            self.line(0, "}\n");
        }

        let ret = self.ty(&f.ret);
        self.line(
            0,
            &format!("export function {}(v: Into{to}): {ret} {{", f.name),
        );

        for (ty, binds, guard, mut body, cast) in kept {
            body.rewrite(&mut |e| {
                if let K::Local(n) = &e.k
                    && let Some((_, place)) = binds.iter().find(|(b, _)| b == n)
                    && !matches!(&place.k, K::Local(x) if x == n)
                {
                    *e = place.clone();
                }
            });
            let narrowed = !matches!(ty, Ty::Bool | Ty::Str | Ty::Float);
            self.line(1, &format!("if ({guard}) {{"));

            if is_statement(&body) {
                let mut block = vec![body];

                // A cast is not narrowed by the tests of a `match`.
                if narrowed {
                    let value = fresh(&block, "value");
                    rewrite_body(&mut block, &mut |e| {
                        if matches!(&e.k, K::Local(n) if n == "v") {
                            e.k = K::Local(value.clone());
                        }
                    });
                    self.line(2, &format!("const {value} = v as {cast};"));
                }

                return_tail(&mut block);
                self.block(&block, 2);
            } else {
                // `Number.isInteger`, `every` and `in` do not narrow the type.
                if narrowed {
                    body.rewrite(&mut |e| {
                        if matches!(&e.k, K::Local(n) if n == "v") {
                            *e = Expr::new(K::Path(vec![format!("(v as {cast})")]));
                        }
                    });
                }

                let body = self.expr(&body);
                self.line(2, &format!("return {body};"));
            }

            self.line(1, "}");
        }

        let panic = self.panic(&format!(
            "`cannot convert ${{JSON.stringify(v)}} into {to}`"
        ));
        self.line(1, &format!("return {panic};"));
        self.line(0, "}\n");
    }

    /// `From::Var(v) => To::Var(v)` with the same payload type serializes
    /// as the value, so the identity arm covers it.
    fn subsumed(&self, pat: &Pat, to: &str) -> bool {
        let Pat::Variant { en, var, inner } = pat else {
            return false;
        };

        if !matches!(inner.as_slice(), [Pat::Bind(_)]) {
            return false;
        }

        match (self.m.variant(en, var), self.m.variant(to, var)) {
            (Some(Variant::Tuple(_, a)), Some(Variant::Tuple(_, b))) => a == b,
            _ => false,
        }
    }

    fn func(&mut self, f: &Func, indent: usize, prefix: &str) {
        self.current = f.name.clone();
        self.generics.clear();
        self.reassigned = reassigned(&f.body);
        // The type parameters only Rust's static dispatch used are erased.
        let generics: Vec<String> = f
            .generics
            .iter()
            .filter(|g| {
                f.params
                    .iter()
                    .map(|(_, t)| t)
                    .chain([&f.ret])
                    .any(|t| mentions(t, &g.name))
            })
            .map(|g| {
                if g.bounds.iter().any(|(b, _)| self.m.is_reflective(b)) {
                    format!("{} extends object", g.name)
                } else {
                    g.name.clone()
                }
            })
            .collect();
        let generics = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };
        let params: Vec<String> = f
            .params
            .iter()
            .map(|(n, t)| format!("{n}: {}", self.ty(t)))
            .collect();
        let ret = self.ty(&f.ret);
        self.line(
            indent,
            &format!(
                "{prefix}{}{generics}({}): {ret} {{",
                f.name,
                params.join(", ")
            ),
        );
        self.block(&f.body, indent + 1);
        self.line(indent, "}");
    }

    fn trait_interface(&mut self, name: &str, sigs: &[Func]) {
        self.line(0, &format!("export interface {name}<Self> {{"));

        for f in sigs {
            let params: Vec<String> = f
                .recv
                .iter()
                .map(|_| "self: Self".to_owned())
                .chain(f.params.iter().map(|(n, t)| format!("{n}: {}", self.ty(t))))
                .collect();
            let ret = self.ty(&subst(&f.ret, &Ty::Param("Self".into()), &[]));
            self.line(1, &format!("{}({}): {ret};", f.name, params.join(", ")));
        }

        self.line(0, "}\n");
    }

    /// The dictionaries of the trait implementations of the types.
    fn dicts(&mut self) {
        let m = self.m;

        for (tr, of) in &m.dicts {
            let Some(impls) = m.classes.get(of).and_then(|c| c.trait_impls.get(tr)) else {
                continue;
            };
            let self_ty = self.ty(&self.self_ty(of));
            self.line(
                0,
                &format!("const {}: {tr}<{self_ty}> = {{", dict_global(of, tr)),
            );
            self.this = "self";

            for f in impls {
                let receiver = f.recv.as_ref().map(|_| format!("self: {self_ty}"));
                let params: Vec<String> = receiver
                    .into_iter()
                    .chain(f.params.iter().map(|(n, t)| format!("{n}: {}", self.ty(t))))
                    .collect();
                let ret = self.ty(&f.ret);
                self.line(1, &format!("{}({}): {ret} {{", f.name, params.join(", ")));
                self.reassigned = reassigned(&f.body);
                self.block(&f.body, 2);
                self.line(1, "},");
            }

            self.this = "this";
            self.line(0, "};\n");
        }
    }

    /// The objects of variant constructors of the enums the users build the
    /// values of: the ones the normalizers convert into and from.
    fn constructors(&mut self) {
        let m = self.m;
        let normalized: BTreeSet<&str> = m
            .functions
            .iter()
            .map(|f| f.name.trim_start_matches("into_"))
            .collect();
        let enums: BTreeSet<&str> = m
            .conversions
            .iter()
            .filter(|c| normalized.contains(c.to.as_str()))
            .flat_map(|c| [Some(c.to.as_str()), c.from.name()])
            .flatten()
            .filter(|n| m.enums.contains_key(*n))
            .collect();

        for name in enums {
            self.line(0, &format!("export const {name} = {{"));
            let ty = self.schema(name);

            for variant in &m.enums[name].variants {
                match variant {
                    Variant::Unit(var) => self.line(1, &format!("{var}: {var:?} as {ty},")),
                    Variant::Tuple(var, payload) => {
                        let (param, value) = match payload.name() {
                            Some(x) if normalized.contains(x) => {
                                (format!("Into{x}"), format!("{}(v)", normalizer(x)))
                            }
                            _ => (self.ty(payload), "v".into()),
                        };
                        self.line(
                            1,
                            &format!("{var}: (v: {param}): {ty} => ({{ {var}: {value} }}),"),
                        );
                    }
                    Variant::Struct(..) => {}
                }
            }

            self.line(0, "};\n");
        }
    }

    /// The methods of the data types as functions: the data are plain
    /// objects.
    fn data_methods(&mut self) {
        let m = self.m;
        let methods = m.enums.iter().map(|(n, e)| (n, &e.methods)).chain(
            m.classes
                .iter()
                .filter(|(n, _)| !m.is_builder(n))
                .map(|(n, c)| (n, &c.methods)),
        );

        for (name, methods) in methods {
            for f in methods {
                let mut f = f.clone();

                if f.recv.is_some() {
                    f.params.insert(0, ("self".into(), self.self_ty(name)));
                }

                f.name = format!("{name}_{}", f.name);
                self.this = "self";
                self.func(&f, 0, "function ");
                self.this = "this";
                self.out.push('\n');
            }
        }
    }

    fn class(&mut self, c: &Class) {
        let generics: Vec<&str> = c.generics.iter().map(|g| g.name.as_str()).collect();
        let generics = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };
        let export = if self.m.root == c.name { "export " } else { "" };
        self.line(0, &format!("{export}class {}{generics} {{", c.name));

        if !c.fields.is_empty() {
            let fields: Vec<String> = c
                .fields
                .iter()
                .map(|(n, t)| format!("public {}: {}", member(n), self.ty(t)))
                .collect();
            self.line(1, &format!("constructor({}) {{}}\n", fields.join(", ")));
        }

        for f in &c.methods {
            let prefix = if f.recv.is_none() { "static " } else { "" };
            self.func(f, 1, prefix);
            self.out.push('\n');
        }

        self.out.truncate(self.out.trim_end().len());
        self.out.push('\n');
        self.line(0, "}\n");
    }
}

/// A member name: the tuple fields `0`, `1`, ... are `_0`, `_1`, ...
fn member(name: &str) -> String {
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{name}")
    } else {
        name.to_owned()
    }
}

/// The value of a `return` or the expression.
fn returned(e: &Expr) -> &Expr {
    match &e.k {
        K::Return(Some(value)) => value,
        _ => e,
    }
}

fn arrow(params: &[String], body: String) -> String {
    let body = if body.starts_with('{') {
        format!("({body})")
    } else {
        body
    };
    format!("({}) => {body}", params.join(", "))
}

/// The arms of a conversion function, `return typecase v {..}`.
fn type_case(f: &Func) -> Option<&[(Ty, Pat, Expr)]> {
    let [
        Expr {
            k: K::Return(Some(value)),
            ..
        },
    ] = f.body.as_slice()
    else {
        return None;
    };
    let K::TypeCase { arms, .. } = &value.k else {
        return None;
    };
    Some(arms)
}

fn into_targets(ty: &Ty, out: &mut BTreeSet<String>) {
    match ty {
        Ty::Into(to) => {
            out.insert(to.clone());
        }
        Ty::List(inner) | Ty::Option(inner) => into_targets(inner, out),
        Ty::Tuple(items) => items.iter().for_each(|t| into_targets(t, out)),
        _ => {}
    }
}

fn mentions(ty: &Ty, generic: &str) -> bool {
    match ty {
        Ty::Param(p) => p == generic,
        Ty::List(inner) | Ty::Option(inner) | Ty::Ref(inner) | Ty::Dict(_, inner) => {
            mentions(inner, generic)
        }
        Ty::Tuple(items) | Ty::Named(_, items) => items.iter().any(|t| mentions(t, generic)),
        _ => false,
    }
}

/// `name`, or with a number if the body uses it.
fn fresh(body: &[Expr], name: &str) -> String {
    let used = |n: &str| {
        body.iter()
            .any(|s| s.any(&|e| matches!(&e.k, K::Local(x) if x == n)))
    };
    let mut out = name.to_owned();
    let mut i = 0;

    while used(&out) {
        out = format!("{name}{i}");
        i += 1;
    }

    out
}

/// The locals the body assigns.
fn reassigned(body: &[Expr]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut body = body.to_vec();
    rewrite_body(&mut body, &mut |e| {
        if let K::Assign { place, .. } = &e.k
            && let K::Local(name) = &place.k
        {
            out.insert(name.clone());
        }
    });
    out
}

/// Whether the statement has no expression form.
fn is_block_statement(e: &Expr) -> bool {
    match &e.k {
        K::Let { .. } | K::Assign { .. } | K::IfLet { .. } | K::Match { .. } | K::While { .. } => {
            true
        }
        K::If { then, els, .. } => then.len() != 1 || els.as_ref().is_none_or(|e| e.len() != 1),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::pass::*;
    use crate::codegen::test_utilities::conversion;
    use crate::codegen::test_utilities::lower;
    use crate::codegen::test_utilities::tested;
    use crate::codegen::ts::*;
    use std::collections::BTreeMap;

    fn config() -> TsConfig {
        TsConfig {
            header: "// header".into(),
            import: "import type { S } from \"s\";".into(),
            schemas: "S".into(),
        }
    }

    fn emitted(source: &str, traits: &[&str], data: &[&str]) -> Ts {
        let (mut m, errors) = lower(source, traits, data);
        assert_eq!(errors, Vec::<String>::new());
        passes(&mut m);
        Emitter::new(&m, &config()).module()
    }

    fn emitted_tests(source: &str, tests: &str, data: &[&str]) -> Ts {
        let (mut m, errors) = tested(source, tests, data);
        assert_eq!(errors, Vec::<String>::new());
        passes(&mut m);
        Emitter::new(&m, &config()).tests("suite", "./m.ts")
    }

    fn passes(m: &mut Module) {
        let errors = Passes::new()
            .with(Resolve)
            .with(Prune)
            .with(ExplicitReturns)
            .with(Inline)
            .with(AcceptElementSources)
            .with(LowerConversions)
            .with(TraitDicts)
            .with(EraseOwnership)
            .with(InferEmptyLists)
            .with(TagResults)
            .with(EraseCasts)
            .with(EraseNewtypes)
            .with(UnescapeIdentifiers)
            .with(OmitSkippedDefaults)
            .with(RemoveDeadHelpers)
            .run(m, &mut |_, _| {});
        assert_eq!(
            errors.iter().map(ToString::to_string).collect::<Vec<_>>(),
            Vec::<String>::new()
        );
    }

    fn class(name: &str, fields: Vec<(&str, Ty)>, methods: Vec<Func>) -> (String, Class) {
        (
            name.into(),
            Class {
                name: name.into(),
                generics: Vec::new(),
                fields: fields.into_iter().map(|(n, t)| (n.into(), t)).collect(),
                methods,
                trait_impls: BTreeMap::new(),
            },
        )
    }

    fn en(name: &str, variants: Vec<Variant>, methods: Vec<Func>) -> (String, Enum) {
        (
            name.into(),
            Enum {
                name: name.into(),
                variants,
                methods,
            },
        )
    }

    fn int(i: i64) -> Expr {
        Expr::typed(K::Lit(Lit::Int(i)), Ty::Int("u64"))
    }

    fn call(callee: Callee, recv: Option<Expr>, args: Vec<Expr>) -> Expr {
        Expr::new(K::Call {
            callee,
            recv: recv.map(Expr::boxed),
            args,
        })
    }

    fn ret(e: Expr) -> Expr {
        Expr::new(K::Return(Some(e.boxed())))
    }

    fn let_(name: &str, value: Expr) -> Expr {
        Expr::new(K::Let {
            name: name.into(),
            value: Some(value.boxed()),
        })
    }

    fn variant(en: &str, var: &str, inner: Vec<Pat>) -> Pat {
        Pat::Variant {
            en: en.into(),
            var: var.into(),
            inner,
        }
    }

    mod emitter {
        use super::*;

        mod tests {
            use super::*;

            #[test]
            fn tests_should_be_a_vitest_suite_expecting_the_schema_types_and_importing_what_the_module_exports()
             {
                let ts = emitted_tests(
                    r#"pub struct Root { pub q: Q, pub a: A, pub v: V, pub e: E }
pub struct Q { pub n: u8 }
pub enum E { X }
pub struct A;
pub struct V;
impl<T: Into<V>> From<(T, T)> for A {
    fn from(v: (T, T)) -> Self { A {} }
}
impl From<u8> for V {
    fn from(v: u8) -> Self { V {} }
}
impl E {
    pub fn get(&self) -> u8 { 1 }
}
impl Root {
    pub fn new() -> Root { Root { q: Q { n: 1 }, a: A {}, v: V {}, e: E::X } }
    pub fn q(&self) -> Q { Q { n: self.q.n } }
}"#,
                    r#"use c::Root;
#[test]
fn first() {
    let r = Root::new();
    assert_eq!(r.q(), c::Q { n: 1 });
}
#[test]
fn second() {
    let a: c::A = (1, 2).into();
    let n = c::E::X.get();
    panic!("p");
}"#,
                    &["Q", "E"],
                );

                assert_eq!(
                    ts.code,
                    "// header\n\nimport type { S } from \"s\";\nimport { describe, expect, it } from \"vitest\";\nimport { Root, into_A } from \"./m.ts\";\n\ntype Q = S.Q;\nfunction panic(msg: string): never {\n    throw new Error(msg);\n}\n\ndescribe(\"suite\", () => {\n    it(\"first\", () => {\n        const r = Root.new();\n        expect(r.q()).toEqual<Q>({ n: 1 });\n    });\n\n    it(\"second\", () => {\n        const a = into_A([1, 2]);\n        const n = E_get(\"X\");\n        panic(\"p\");\n    });\n});\n"
                );
                assert_eq!(ts.unsupported, ["second: `E_get is not exported`"]);
            }

            #[test]
            fn tests_should_import_nothing_of_the_module_the_tests_do_not_use() {
                let ts = emitted_tests(
                    "pub struct Root;",
                    "#[test]\nfn t() { assert_eq!(1, 1); }",
                    &[],
                );

                assert_eq!(
                    ts.code,
                    "// header\n\nimport type { S } from \"s\";\nimport { describe, expect, it } from \"vitest\";\n\ndescribe(\"suite\", () => {\n    it(\"t\", () => {\n        expect(1).toEqual<number>(1);\n    });\n});\n"
                );
            }
        }

        mod module {
            use super::*;

            #[test]
            fn module_should_emit_the_expressions_and_statements_of_the_builder_methods() {
                let list = |t: Ty| Ty::List(Box::new(t));
                let b = Expr::local("b", Ty::Bool);
                let l = Expr::local("l", list(Ty::Option(Box::new(Ty::Int("u64")))));
                let e = Expr::local("e", Ty::named("E"));
                let t = Expr::local("t", Ty::Tuple(vec![Ty::Int("u64"), Ty::Str]));
                let intrinsic = |i: Intrinsic, args: Vec<Expr>| {
                    call(Callee::Intrinsic(i), Some(l.clone()), args)
                };
                let d = |ty: Ty| {
                    Expr::typed(
                        K::Construct {
                            class: "D".into(),
                            fields: vec![("a".into(), int(1))],
                        },
                        ty,
                    )
                };
                let if_ = |cond: Expr, then: Vec<Expr>, els: Option<Vec<Expr>>| {
                    Expr::new(K::If {
                        cond: cond.boxed(),
                        then,
                        els,
                    })
                };
                let match_ = |arms: Vec<(Pat, Vec<Expr>)>| {
                    Expr::new(K::Match {
                        value: e.clone().boxed(),
                        arms,
                    })
                };
                let if_let = |pat: Pat, then: Vec<Expr>| {
                    Expr::new(K::IfLet {
                        pat,
                        value: e.clone().boxed(),
                        then,
                        els: None,
                    })
                };
                let while_ = |body: Vec<Expr>| {
                    Expr::new(K::While {
                        cond: b.clone().boxed(),
                        body,
                    })
                };
                let not = |v: Expr| Expr::new(K::Not(v.boxed()));
                let neq = Expr::new(K::Binary {
                    op: "!=",
                    l: b.clone().boxed(),
                    r: Expr::new(K::Lit(Lit::Bool(true))).boxed(),
                });
                let x = Expr::local("x", Ty::Unknown);
                let body = vec![
                    Expr::new(K::Lit(Lit::Float(1.5))),
                    Expr::new(K::Lit(Lit::Unit)),
                    Expr::new(K::Lit(Lit::Bool(true))),
                    Expr::variant("E", "A", None),
                    t.clone().field("0", Ty::Int("u64")),
                    e.clone().field("0", Ty::Int("u64")),
                    Expr::local("root", Ty::named("Root")).field("0", Ty::Int("u64")),
                    Expr::new(K::Payload {
                        base: e.clone().boxed(),
                        en: "E".into(),
                        var: "B".into(),
                    }),
                    Expr::new(K::Index {
                        base: l.clone().boxed(),
                        index: int(0).boxed(),
                    }),
                    not(neq.clone()),
                    not(intrinsic(Intrinsic::IsEmpty, Vec::new())),
                    not(b.clone()),
                    let_("y", if_(b.clone(), vec![ret(int(1))], Some(vec![int(2)]))),
                    Expr::new(K::Closure {
                        params: vec!["x".into()],
                        body: vec![let_("z", x.clone()), ret(x.clone())],
                    }),
                    Expr::new(K::Closure {
                        params: Vec::new(),
                        body: vec![d(Ty::named("D"))],
                    }),
                    Expr::new(K::Ident("unresolved".into())),
                    call(
                        Callee::Member {
                            class: "E".into(),
                            name: "f".into(),
                        },
                        Some(e.clone()),
                        vec![int(1)],
                    ),
                    call(
                        Callee::Member {
                            class: "E".into(),
                            name: "g".into(),
                        },
                        Some(e.clone()),
                        Vec::new(),
                    ),
                    call(
                        Callee::Static {
                            class: "Root".into(),
                            name: "new".into(),
                        },
                        None,
                        vec![int(1)],
                    ),
                    call(Callee::Extern("ext".into()), None, vec![int(1)]),
                    call(Callee::Path(vec!["p".into()], Vec::new()), None, Vec::new()),
                    call(Callee::Method("m".into()), None, Vec::new()),
                    intrinsic(
                        Intrinsic::Map,
                        vec![Expr::new(K::Closure {
                            params: vec!["x".into()],
                            body: vec![x.clone()],
                        })],
                    ),
                    intrinsic(Intrinsic::Pop, Vec::new()),
                    intrinsic(Intrinsic::LastMut, Vec::new()),
                    intrinsic(Intrinsic::Len, Vec::new()),
                    intrinsic(Intrinsic::Extend, vec![l.clone()]),
                    intrinsic(Intrinsic::Reserve, vec![int(1)]),
                    intrinsic(Intrinsic::UnwrapOrDefault, vec![int(0)]),
                    intrinsic(
                        Intrinsic::Expect,
                        vec![Expr::new(K::Lit(Lit::Str("e".into())))],
                    ),
                    intrinsic(Intrinsic::MemTake, Vec::new()),
                    Expr::typed(K::Fields(l.clone().boxed()), Ty::Int("u8")),
                    Expr::typed(K::Fields(l.clone().boxed()), list(Ty::Int("u8"))),
                    Expr::typed(K::Fields(l.clone().boxed()), list(Ty::Tuple(vec![Ty::Str]))),
                    let_("r", int(1)),
                    Expr::new(K::Assign {
                        place: Expr::local("r", Ty::Int("u64")).boxed(),
                        value: int(2).boxed(),
                    }),
                    let_("a", Expr::typed(K::Array(Vec::new()), list(Ty::Int("u64")))),
                    let_("u", Expr::typed(K::Array(Vec::new()), list(Ty::Unknown))),
                    let_("c", d(Ty::named("D"))),
                    let_("cu", d(Ty::Unknown)),
                    let_(
                        "v",
                        Expr::typed(Expr::variant("E", "B", Some(int(1))).k, Ty::named("E")),
                    ),
                    Expr::new(K::Let {
                        name: "n".into(),
                        value: None,
                    }),
                    if_(Expr::new(K::Lit(Lit::Bool(true))), vec![int(3)], None),
                    if_(b.clone(), vec![int(4)], Some(Vec::new())),
                    if_(
                        b.clone(),
                        vec![int(5)],
                        Some(vec![match_(vec![(Pat::Wild, vec![int(6)])])]),
                    ),
                    if_(
                        b.clone(),
                        vec![int(7)],
                        Some(vec![if_(neq.clone(), vec![int(8)], None)]),
                    ),
                    if_let(variant("E", "A", Vec::new()), vec![int(9)]),
                    if_let(
                        variant("E", "B", vec![Pat::Bind("p".into())]),
                        vec![ret(Expr::local("p", Ty::Int("u64")))],
                    ),
                    if_let(Pat::Bind("q".into()), vec![int(10)]),
                    while_(Vec::new()),
                    while_(vec![int(11)]),
                    match_(vec![
                        (variant("E", "A", Vec::new()), vec![int(12)]),
                        (Pat::Lit(Lit::Int(1)), vec![int(13)]),
                        (
                            Pat::Bind("w".into()),
                            vec![ret(Expr::local("w", Ty::named("E")))],
                        ),
                    ]),
                    match_(vec![
                        (variant("E", "A", Vec::new()), vec![int(14)]),
                        (variant("E", "B", vec![Pat::Wild]), vec![int(15)]),
                    ]),
                    ret(Expr::new(K::Lit(Lit::Unit))),
                ];
                let refl = |name: &str| Generic {
                    name: name.into(),
                    bounds: vec![("Refl".into(), None)],
                };
                let exprs = Func {
                    generics: vec![
                        refl("T"),
                        refl("Erased"),
                        Generic {
                            name: "P".into(),
                            bounds: vec![("Other".into(), None)],
                        },
                    ],
                    recv: Some(Recv::Value),
                    ..Func::new(
                        "exprs",
                        vec![
                            ("b".into(), Ty::Bool),
                            ("l".into(), l.ty.clone()),
                            ("e".into(), Ty::named("E")),
                            ("t".into(), t.ty.clone()),
                            ("s".into(), Ty::SelfTy),
                            ("o".into(), Ty::Option(Box::new(Ty::Param("T".into())))),
                            ("r".into(), Ty::Ref(Box::new(Ty::Int("u8")))),
                            ("n".into(), Ty::Named("D".into(), vec![Ty::Int("u8")])),
                            (
                                "dict".into(),
                                Ty::Dict("Refl".into(), Box::new(Ty::Param("U".into()))),
                            ),
                            ("x".into(), Ty::Param("T".into())),
                            ("pp".into(), Ty::Param("P".into())),
                        ],
                        Ty::Unit,
                        body,
                    )
                };
                let m = Module {
                    root: "Root".into(),
                    builders: vec!["Root".into()],
                    classes: [
                        class(
                            "Root",
                            Vec::new(),
                            vec![
                                exprs,
                                Func::new("new", Vec::new(), Ty::named("Root"), Vec::new()),
                            ],
                        ),
                        class("D", vec![("a", Ty::Int("u64"))], Vec::new()),
                    ]
                    .into_iter()
                    .collect(),
                    enums: [en(
                        "E",
                        vec![
                            Variant::Unit("A".into()),
                            Variant::Tuple("B".into(), Ty::Int("u64")),
                        ],
                        Vec::new(),
                    )]
                    .into_iter()
                    .collect(),
                    traits: [(
                        "Refl".to_owned(),
                        vec![Func {
                            recv: Some(Recv::Ref),
                            ..Func::new(
                                "keys",
                                vec![("n".into(), Ty::Int("u64"))],
                                Ty::SelfTy,
                                Vec::new(),
                            )
                        }],
                    )]
                    .into_iter()
                    .collect(),
                    ..Module::default()
                };

                let ts = Emitter::new(&m, &config()).module();

                assert_eq!(
                    ts.code,
                    "// header\n\nimport type { S } from \"s\";\n\ntype D = S.D;\ntype E = S.E;\nfunction panic(msg: string): never {\n    throw new Error(msg);\n}\nexport interface Refl<Self> {\n    keys(self: Self, n: number): Self;\n}\n\nexport class Root {\n    exprs<T extends object, P>(b: boolean, l: (number | undefined)[], e: E, t: [number, string], s: unknown, o: T | undefined, r: unknown, n: D, dict: Refl<U>, x: T, pp: P): void {\n        1.5;\n        undefined;\n        true;\n        \"A\";\n        t[0];\n        e[0];\n        root._0;\n        e.B;\n        l[0];\n        !(b !== true);\n        !(l.length === 0);\n        !b;\n        const y = (b ? 1 : 2);\n        (x) => {\n            const z = x;\n            return x;\n        };\n        () => ({ a: 1 });\n        unsupported();\n        E_f(e, 1);\n        E_g(e);\n        Root.new(1);\n        ext(1);\n        unsupported();\n        unsupported();\n        l.map((x) => x);\n        l.pop();\n        l.at(-1);\n        l.length;\n        l.push(...l);\n        (l ?? 0);\n        (l ?? panic(\"e\"));\n        unsupported();\n        unsupported();\n        unsupported();\n        unsupported();\n        let r: number = 1;\n        r = 2;\n        const a: number[] = [];\n        const u = [];\n        const c: D = { a: 1 };\n        const cu = { a: 1 };\n        const v: E = { B: 1 };\n        const n = undefined;\n        3;\n        if (b) {\n            4;\n        }\n        if (b) {\n            5;\n        } else {\n            6;\n        }\n        if (b) {\n            7;\n        } else if (b !== true) {\n            8;\n        }\n        if (e === \"A\") {\n            9;\n        }\n        if (typeof e === \"object\" && e !== null && \"B\" in e) {\n            const p = e.B;\n            return p;\n        }\n        if (true) {\n            10;\n        }\n        while (b);\n        while (b) {\n            11;\n        }\n        if (e === \"A\") {\n            12;\n        } else if (e === 1) {\n            13;\n        } else {\n            const w = e;\n            return w;\n        }\n        if (e === \"A\") {\n            14;\n        } else if (typeof e === \"object\" && e !== null && \"B\" in e) {\n            15;\n        } else {\n            panic(\"unreachable\");\n        }\n        return undefined;\n    }\n\n    static new(): Root {\n    }\n}\n\n"
                );
                assert_eq!(ts.notes, Vec::<String>::new());
                assert_eq!(
                    ts.unsupported,
                    [
                        "exprs: `unresolved/*unresolved*/`",
                        "exprs: `Path([\"p\"], [])`",
                        "exprs: `Method(\"m\")`",
                        "exprs: `MemTake`",
                        "exprs: `u8`",
                        "exprs: `Vec<u8>`",
                        "exprs: `Vec<(str)>`"
                    ]
                );
            }

            #[test]
            fn module_should_emit_a_guard_per_runtime_representation_and_the_constructors_and_data_methods()
             {
                let v = |t: Ty| Expr::local("v", t);
                let bind = || Pat::Bind("v".into());
                let p = |e: Expr| Expr::typed(Expr::variant("X", "P", Some(e)).k, Ty::named("X"));
                let list = |t: Ty| Ty::List(Box::new(t));
                let case = |name: &str, to: &str, arms: Vec<(Ty, Pat, Expr)>| {
                    Func::new(
                        name,
                        vec![("v".into(), Ty::Into(to.into()))],
                        Ty::named(to),
                        vec![ret(Expr::new(K::TypeCase {
                            value: v(Ty::Into(to.into())).boxed(),
                            arms,
                        }))],
                    )
                };
                let mut into_x = case(
                    "into_X",
                    "X",
                    vec![
                        (Ty::named("X"), bind(), v(Ty::named("X"))),
                        (Ty::Bool, bind(), Expr::variant("X", "B", Some(v(Ty::Bool)))),
                        (Ty::Float, bind(), v(Ty::Float)),
                        (Ty::Int("u8"), bind(), p(v(Ty::Int("u8")))),
                        (Ty::Int("i64"), bind(), p(v(Ty::Int("i64")))),
                        (Ty::Int("i32"), bind(), p(v(Ty::Int("i32")))),
                        (Ty::Str, bind(), v(Ty::Str)),
                        (Ty::Str, bind(), v(Ty::Str)),
                        (list(Ty::Int("u64")), bind(), v(Ty::Unknown)),
                        (list(Ty::Int("i32")), bind(), v(Ty::Unknown)),
                        (
                            Ty::Tuple(vec![Ty::Str, Ty::Int("u64")]),
                            bind(),
                            v(Ty::Unknown),
                        ),
                        (Ty::Into("Z".into()), bind(), v(Ty::Unknown)),
                        (Ty::Param("T".into()), bind(), v(Ty::Unknown)),
                        (Ty::named("U"), bind(), v(Ty::Unknown)),
                        (Ty::named("D"), bind(), v(Ty::Unknown)),
                        (Ty::named("Nope"), bind(), v(Ty::Unknown)),
                        (Ty::Unit, bind(), v(Ty::Unknown)),
                        (
                            Ty::named("Y"),
                            variant("Y", "P", vec![Pat::Bind("p".into())]),
                            p(Expr::local("p", Ty::Int("u64"))),
                        ),
                        (
                            Ty::named("Y"),
                            variant("Y", "Q", vec![Pat::Bind("q".into())]),
                            p(Expr::local("q", Ty::Int("u64"))),
                        ),
                        (
                            Ty::named("Y"),
                            variant("Y", "R", vec![Pat::Wild]),
                            p(int(0)),
                        ),
                    ],
                );
                into_x.generics = vec![
                    Generic {
                        name: "Other".into(),
                        bounds: Vec::new(),
                    },
                    Generic {
                        name: "T".into(),
                        bounds: vec![("Refl".into(), None)],
                    },
                ];
                let into_z = case(
                    "into_Z",
                    "Z",
                    vec![
                        (Ty::named("Z"), bind(), v(Ty::named("Z"))),
                        (list(Ty::Into("X".into())), bind(), v(Ty::Unknown)),
                        (
                            Ty::Option(Box::new(Ty::Into("X".into()))),
                            bind(),
                            v(Ty::Unknown),
                        ),
                        (
                            Ty::Tuple(vec![Ty::Into("X".into())]),
                            bind(),
                            v(Ty::Unknown),
                        ),
                    ],
                );
                let empty = Func::new("empty", Vec::new(), Ty::Unit, Vec::new());
                let returns = Func::new("returns", Vec::new(), Ty::Unit, vec![ret(int(1))]);
                let get = Func {
                    recv: Some(Recv::Ref),
                    ..Func::new(
                        "get",
                        Vec::new(),
                        Ty::Int("u64"),
                        vec![ret(
                            Expr::typed(K::This, Ty::named("D")).field("a", Ty::Int("u64"))
                        )],
                    )
                };
                let make = Func::new("make", Vec::new(), Ty::named("D"), vec![ret(int(1))]);
                let tuple = |n: &str, t: Ty| Variant::Tuple(n.into(), t);
                let m = Module {
                    classes: [
                        class("D", vec![("a", Ty::Int("u64"))], vec![get, make]),
                        (
                            "C".to_owned(),
                            Class {
                                trait_impls: [(
                                    "Tr".to_owned(),
                                    vec![Func::new(
                                        "add",
                                        vec![("n".into(), Ty::Int("u64"))],
                                        Ty::Int("u64"),
                                        vec![ret(Expr::local("n", Ty::Int("u64")))],
                                    )],
                                )]
                                .into_iter()
                                .collect(),
                                ..class("C", Vec::new(), Vec::new()).1
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    enums: [
                        en(
                            "X",
                            vec![tuple("P", Ty::Int("u64")), tuple("B", Ty::Bool)],
                            Vec::new(),
                        ),
                        en(
                            "Y",
                            vec![
                                tuple("P", Ty::Int("u64")),
                                tuple("Q", Ty::Int("u64")),
                                tuple("R", Ty::Int("u64")),
                            ],
                            Vec::new(),
                        ),
                        en(
                            "U",
                            vec![Variant::Unit("A".into()), Variant::Unit("B".into())],
                            Vec::new(),
                        ),
                        en(
                            "Z",
                            vec![
                                tuple("Z1", Ty::named("X")),
                                Variant::Struct("S".into(), vec![("s".into(), Ty::Str)]),
                            ],
                            Vec::new(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    conversions: vec![
                        conversion("X", Ty::named("Y"), v(Ty::named("Y"))),
                        conversion("Z", Ty::Unit, v(Ty::Unit)),
                        conversion("W", Ty::named("U"), v(Ty::named("U"))),
                    ],
                    functions: vec![into_x, into_z, empty, returns],
                    dicts: [
                        ("Tr".to_owned(), "C".to_owned()),
                        ("Tr".to_owned(), "D".to_owned()),
                    ]
                    .into_iter()
                    .collect(),
                    traits: [("Refl".to_owned(), Vec::new())].into_iter().collect(),
                    ..Module::default()
                };

                let ts = Emitter::new(&m, &config()).module();

                assert_eq!(
                    ts.code,
                    "// header\n\nimport type { S } from \"s\";\n\ntype C = S.C;\ntype D = S.D;\ntype Nope = S.Nope;\ntype U = S.U;\ntype X = S.X;\ntype Y = S.Y;\ntype Z = S.Z;\nfunction panic(msg: string): never {\n    throw new Error(msg);\n}\nexport interface Refl<Self> {\n}\n\nconst C_Tr: Tr<C> = {\n    add(n: number): number {\n        return n;\n    },\n};\n\nexport const X = {\n    P: (v: number): X => ({ P: v }),\n    B: (v: boolean): X => ({ B: v }),\n};\n\nexport const Y = {\n    P: (v: number): Y => ({ P: v }),\n    Q: (v: number): Y => ({ Q: v }),\n    R: (v: number): Y => ({ R: v }),\n};\n\nexport const Z = {\n    Z1: (v: IntoX): Z => ({ Z1: into_X(v) }),\n};\n\nexport type IntoX = boolean | number | string | number[] | [string, number] | X | U | D | Nope | Y | IntoZ | object | void;\n\nfunction is_into_X(v: unknown): v is IntoX {\n    return (typeof v === \"boolean\") || (Number.isInteger(v)) || (typeof v === \"number\") || (typeof v === \"string\") || (Array.isArray(v) && v.every((x) => Number.isInteger(x))) || (Array.isArray(v) && v.length === 2 && typeof v[0] === \"string\" && Number.isInteger(v[1])) || ((typeof v === \"object\" && v !== null && (\"P\" in v || \"B\" in v))) || (v === \"A\" || v === \"B\") || (typeof v === \"object\" && v !== null && \"a\" in v) || (true) || (typeof v === \"object\" && v !== null && \"Q\" in v) || (typeof v === \"object\" && v !== null && \"R\" in v) || (is_into_Z(v)) || (typeof v === \"object\" && v !== null && !Array.isArray(v));\n}\n\nexport function into_X(v: IntoX): X {\n    if (typeof v === \"boolean\") {\n        return { B: v };\n    }\n    if (Number.isInteger(v)) {\n        return { P: (v as number) };\n    }\n    if (typeof v === \"number\") {\n        return v;\n    }\n    if (typeof v === \"string\") {\n        return v;\n    }\n    if (Array.isArray(v) && v.every((x) => Number.isInteger(x))) {\n        return (v as number[]);\n    }\n    if (Array.isArray(v) && v.length === 2 && typeof v[0] === \"string\" && Number.isInteger(v[1])) {\n        return (v as [string, number]);\n    }\n    if ((typeof v === \"object\" && v !== null && (\"P\" in v || \"B\" in v))) {\n        return (v as X);\n    }\n    if (v === \"A\" || v === \"B\") {\n        return (v as U);\n    }\n    if (typeof v === \"object\" && v !== null && \"a\" in v) {\n        return (v as D);\n    }\n    if (true) {\n        return (v as Nope);\n    }\n    if (typeof v === \"object\" && v !== null && \"Q\" in v) {\n        return { P: (v as Extract<Y, { Q: unknown }>).Q };\n    }\n    if (typeof v === \"object\" && v !== null && \"R\" in v) {\n        return { P: 0 };\n    }\n    if (is_into_Z(v)) {\n        return (v as IntoZ);\n    }\n    if (typeof v === \"object\" && v !== null && !Array.isArray(v)) {\n        return (v as object);\n    }\n    return panic(`cannot convert ${JSON.stringify(v)} into X`);\n}\n\nexport type IntoZ = IntoX[] | [IntoX] | Z | IntoX | undefined;\n\nfunction is_into_Z(v: unknown): v is IntoZ {\n    return (Array.isArray(v) && v.every((x) => is_into_X(x))) || (Array.isArray(v) && v.length === 1 && is_into_X(v[0])) || ((typeof v === \"object\" && v !== null && (\"Z1\" in v || \"S\" in v))) || ((v === undefined || is_into_X(v)));\n}\n\nexport function into_Z(v: IntoZ): Z {\n    if (Array.isArray(v) && v.every((x) => is_into_X(x))) {\n        return (v as IntoX[]);\n    }\n    if (Array.isArray(v) && v.length === 1 && is_into_X(v[0])) {\n        return (v as [IntoX]);\n    }\n    if ((typeof v === \"object\" && v !== null && (\"Z1\" in v || \"S\" in v))) {\n        return (v as Z);\n    }\n    if ((v === undefined || is_into_X(v))) {\n        return (v as IntoX | undefined);\n    }\n    return panic(`cannot convert ${JSON.stringify(v)} into Z`);\n}\n\nfunction D_get(self: D): number {\n    return self.a;\n}\n\nfunction D_make(): D {\n    return 1;\n}\n\n"
                );
                assert_eq!(
                    ts.notes,
                    [
                        "X: `i64` has the TS representation of an earlier arm, dropped",
                        "X: `u8` has the TS representation of an earlier arm, dropped",
                        "X: `str` has the TS representation of an earlier arm, dropped",
                        "X: `Vec<u64>` has the TS representation of an earlier arm, dropped",
                        "X: `Y::P(p)` arm is the identity in TS, dropped",
                        "X: `()` has the TS representation of an earlier arm, dropped"
                    ]
                );
                assert_eq!(
                    ts.unsupported,
                    [
                        "empty: `a function other than a conversion`",
                        "returns: `a function other than a conversion`"
                    ]
                );
            }

            #[test]
            fn module_should_emit_the_panic_helper_only_when_it_is_called() {
                let ts = emitted(
                    r#"
                    pub struct Root;
                    impl Root {
                        pub fn n(&self) -> u64 {
                            1
                        }
                    }
                    "#,
                    &[],
                    &[],
                );

                assert_eq!(
                    ts.code,
                    "// header\n\nimport type { S } from \"s\";\n\nexport class Root {\n    n(): number {\n        return 1;\n    }\n}\n\n"
                );
            }

            #[test]
            fn module_should_emit_the_builders_data_and_normalizers_of_a_crate() {
                let ts = emitted(
                    r#"
                    pub enum Wire {
                        A(Qa),
                        B(Qb),
                    }
                    pub struct Qa {
                        pub ids: Vec<V>,
                        pub n: Option<u64>,
                        pub keys: Vec<String>,
                    }
                    pub struct Qb(pub u64);
                    pub enum V {
                        Id(u64),
                        Name(String),
                        Pair(u64, u64),
                        None,
                    }
                    impl From<u64> for V {
                        fn from(v: u64) -> Self {
                            V::Id(v)
                        }
                    }
                    impl From<&str> for V {
                        fn from(v: &str) -> Self {
                            V::Name(v.to_string())
                        }
                    }
                    impl From<Qa> for Wire {
                        fn from(v: Qa) -> Self {
                            Wire::A(v)
                        }
                    }
                    impl From<Qb> for Wire {
                        fn from(v: Qb) -> Self {
                            Wire::B(v)
                        }
                    }
                    pub trait Keys {
                        fn keys(&self) -> Vec<String>;
                        fn none() -> Vec<String>;
                    }
                    pub struct Fields(pub Vec<(String, Option<V>)>);
                    impl Keys for Fields {
                        fn keys(&self) -> Vec<String> {
                            let mut keys = Vec::new();
                            self.0.iter().for_each(|f| {
                                if let Some(v) = &f.1
                                    && v.is_id()
                                {
                                    keys.push(f.0.clone());
                                }
                            });
                            keys
                        }
                        fn none() -> Vec<String> {
                            Vec::new()
                        }
                    }
                    impl From<bool> for V {
                        fn from(b: bool) -> Self {
                            match b {
                                true => V::Id(1),
                                false => V::None,
                            }
                        }
                    }
                    impl From<Wire> for V {
                        fn from(w: Wire) -> Self {
                            match w {
                                Wire::A(_) => V::None,
                                Wire::B(value) => V::Id(value.0),
                            }
                        }
                    }
                    pub trait Ided {
                        fn id(&self) -> u64;
                    }
                    impl Ided for Qb {
                        fn id(&self) -> u64 {
                            self.0
                        }
                    }
                    pub struct Root;
                    impl Root {
                        pub fn a(self) -> A {
                            A(Qa { ids: vec![], n: None, keys: vec![] })
                        }
                        pub fn b(self, n: u64) -> Qb {
                            Qb(n)
                        }
                        pub fn g(self) -> G<Qb> {
                            G(Qb(1))
                        }
                    }
                    pub struct A(pub Qa);
                    impl A {
                        pub fn ids<T: Into<V>>(mut self, ids: Vec<T>) -> Self {
                            ids.iter().for_each(|x| self.0.ids.push(x.into()));
                            self
                        }
                        pub fn id<T: Into<V>>(mut self, id: T) -> Self {
                            self.0.ids.push(id.into());
                            self
                        }
                        pub fn n(mut self, n: u64) -> Self {
                            if n > 0 {
                                self.0.n = Some(n);
                            } else if n == 0 {
                                self.0.n = None;
                            } else {
                                panic!("negative");
                            }
                            self
                        }
                        pub fn values<T: Keys>(mut self, t: &T) -> Self {
                            self.0.keys = t.keys();
                            if self.0.keys.is_empty() {
                                self.0.keys = T::none();
                            }
                            self
                        }
                        pub fn count(&self) -> u64 {
                            let mut n: u64 = 0;
                            self.0.ids.iter().for_each(|x| n = n + 1);
                            self.0.ids.iter().for_each(|x| if x.is_id() && n > 1 { n = n + 1; });
                            n
                        }
                        pub fn flags(&self) -> Vec<u64> {
                            self.0.ids.iter().map(|x| if x.is_id() { 1 } else { 0 }).collect()
                        }
                        pub fn wire(mut self, w: Wire) -> Self {
                            self.0.ids.push(w.into());
                            self.0.ids.push(V::zero());
                            self
                        }
                        pub fn first(&self) -> u64 {
                            match self.0.n {
                                Some(id) => id,
                                None => 0,
                            }
                        }
                        pub fn query(self) -> Qa {
                            self.0
                        }
                    }
                    pub struct G<T: Ided>(T);
                    impl<T: Ided> G<T> {
                        pub fn id(&self) -> u64 {
                            self.0.id()
                        }
                        pub fn query(self) -> T {
                            self.0
                        }
                    }
                    impl V {
                        pub fn zero() -> V {
                            let zero = V::Id(0);
                            zero
                        }
                        pub fn is_id(&self) -> bool {
                            match self {
                                V::Id(_) => true,
                                V::Pair(a, b) => *a == *b,
                                _ => false,
                            }
                        }
                    }
                    "#,
                    &["Keys", "Ided"],
                    &["Wire"],
                );

                assert_eq!(
                    ts.code,
                    "// header\n\nimport type { S } from \"s\";\n\ntype Qa = S.Qa;\ntype V = S.V;\ntype Wire = S.Wire;\nfunction panic(msg: string): never {\n    throw new Error(msg);\n}\nexport interface Ided<Self> {\n    id(self: Self): number;\n    into_wire(self: Self): Wire;\n}\n\nexport interface Keys<Self> {\n    keys(self: Self): string[];\n    none(): string[];\n}\n\nconst Qb_Ided: Ided<number> = {\n    id(self: number): number {\n        return self;\n    },\n    into_wire(self: number): Wire {\n        return { B: self };\n    },\n};\n\nconst Fields_Keys: Keys<([string, V | undefined])[]> = {\n    keys(self: ([string, V | undefined])[]): string[] {\n        const keys: string[] = [];\n        self.forEach((f) => {\n            if (f[1] !== undefined) {\n                const v = f[1];\n                if (V_is_id(v)) {\n                    keys.push(f[0]);\n                }\n            }\n        });\n        return keys;\n    },\n    none(): string[] {\n        return [];\n    },\n};\n\nexport const V = {\n    Id: (v: number): V => ({ Id: v }),\n    Name: (v: string): V => ({ Name: v }),\n    Pair: (v: [number, number]): V => ({ Pair: v }),\n    None: \"None\" as V,\n};\n\nexport const Wire = {\n    A: (v: Qa): Wire => ({ A: v }),\n    B: (v: number): Wire => ({ B: v }),\n};\n\nexport type IntoV = boolean | number | string | V | Wire;\n\nexport function into_V(v: IntoV): V {\n    if (typeof v === \"boolean\") {\n        if (v === true) {\n            return { Id: 1 };\n        } else if (v === false) {\n            return \"None\";\n        } else {\n            panic(\"unreachable\");\n        }\n    }\n    if (Number.isInteger(v)) {\n        return { Id: (v as number) };\n    }\n    if (typeof v === \"string\") {\n        return { Name: v };\n    }\n    if (v === \"None\" || (typeof v === \"object\" && v !== null && (\"Id\" in v || \"Name\" in v || \"Pair\" in v))) {\n        return (v as V);\n    }\n    if ((typeof v === \"object\" && v !== null && (\"A\" in v || \"B\" in v))) {\n        const value0 = v as Wire;\n        if (typeof value0 === \"object\" && value0 !== null && \"A\" in value0) {\n            return \"None\";\n        } else if (typeof value0 === \"object\" && value0 !== null && \"B\" in value0) {\n            const value = value0.B;\n            return { Id: value };\n        } else {\n            panic(\"unreachable\");\n        }\n    }\n    return panic(`cannot convert ${JSON.stringify(v)} into V`);\n}\n\nfunction V_zero(): V {\n    const zero: V = { Id: 0 };\n    return zero;\n}\n\nfunction V_is_id(self: V): boolean {\n    if (typeof self === \"object\" && self !== null && \"Id\" in self) {\n        return true;\n    } else if (typeof self === \"object\" && self !== null && \"Pair\" in self) {\n        const a = self.Pair[0];\n        const b = self.Pair[1];\n        return a === b;\n    } else {\n        return false;\n    }\n}\n\nexport class Root {\n    a(): A {\n        return new A({ ids: [], n: undefined, keys: [] });\n    }\n\n    b(n: number): Wire {\n        return { B: n };\n    }\n\n    g(): G<number> {\n        return new G(1, Qb_Ided);\n    }\n}\n\nclass G<T> {\n    constructor(public _0: T, public T_Ided: Ided<T>) {}\n\n    id(): number {\n        return this.T_Ided.id(this._0);\n    }\n\n    query(): Wire {\n        return this.T_Ided.into_wire(this._0);\n    }\n}\n\nclass A {\n    constructor(public _0: Qa) {}\n\n    ids(ids: IntoV[]): A {\n        ids.forEach((x) => this._0.ids.push(into_V(x)));\n        return this;\n    }\n\n    id(id: IntoV): A {\n        this._0.ids.push(into_V(id));\n        return this;\n    }\n\n    n(n: number): A {\n        if (n > 0) {\n            this._0.n = n;\n        } else if (n === 0) {\n            this._0.n = undefined;\n        } else {\n            panic(\"negative\");\n        }\n        return this;\n    }\n\n    values<T extends object>(t: T): A {\n        this._0.keys = Fields_Keys.keys(Object.entries(t).map(([k, v]): [string, V | undefined] => [k as string, v === undefined || v === null ? undefined : into_V(v as IntoV)]));\n        if (this._0.keys.length === 0) {\n            this._0.keys = Fields_Keys.none();\n        }\n        return this;\n    }\n\n    count(): number {\n        let n: number = 0;\n        this._0.ids.forEach((x) => {\n            n = n + 1;\n        });\n        this._0.ids.forEach((x) => {\n            if (V_is_id(x) && n > 1) {\n                n = n + 1;\n            }\n        });\n        return n;\n    }\n\n    flags(): number[] {\n        return this._0.ids.map((x) => (V_is_id(x) ? 1 : 0));\n    }\n\n    wire(w: Wire): A {\n        this._0.ids.push(into_V(w));\n        this._0.ids.push(V_zero());\n        return this;\n    }\n\n    first(): number {\n        if (this._0.n !== undefined) {\n            const id = this._0.n;\n            return id;\n        } else if (this._0.n === undefined) {\n            return 0;\n        } else {\n            panic(\"unreachable\");\n        }\n    }\n\n    query(): Wire {\n        return { A: this._0 };\n    }\n}\n\n"
                );
                assert_eq!(ts.notes, Vec::<String>::new());
                assert_eq!(ts.unsupported, Vec::<String>::new());
            }
        }
    }
}
