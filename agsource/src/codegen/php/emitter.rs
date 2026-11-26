//! The PHP emitter: lowers the IR into PHP 8.1+ code. Builder classes use
//! method chaining, data types are plain classes with typed properties, and
//! the conversion functions are normalizer functions. PHP has no generics at
//! runtime so type parameters are erased; `int` and `float` stay distinct.

use crate::codegen::lowered::*;
use crate::codegen::pass::dict_global;
use crate::codegen::pass::is_statement;
use crate::codegen::pass::normalizer;
use crate::codegen::pass::return_tail;
use crate::codegen::pass::subst;
use crate::codegen::print;
use std::collections::BTreeSet;
use std::fmt::Write;

mod client;
mod types;

/// Configuration for the PHP emitter.
#[derive(Clone, Debug, Default)]
pub struct PhpConfig {
    /// The comment the file starts with (after `<?php`).
    pub header: String,
    /// The PHP namespace, e.g. `Agnesoft\\AgdbApi`.
    pub namespace: String,
    /// The `require_once` for the types file, relative to the emitted file.
    pub types_require: String,
}

/// An emitted PHP module.
#[derive(Debug)]
pub struct Php {
    pub code: String,
    /// What PHP represents differently from Rust.
    pub notes: Vec<String>,
    /// The expressions without a PHP form, `function: expression`, emitted
    /// as `unsupported()`.
    pub unsupported: Vec<String>,
    /// The exported values named as a schema type.
    pub values: Vec<String>,
}

/// The PHP emitter of a lowered module.
#[allow(dead_code)]
pub struct Emitter<'a> {
    m: &'a Module,
    config: &'a PhpConfig,
    out: String,
    this: &'static str,
    notes: Vec<String>,
    unsupported: Vec<String>,
    /// The function being emitted, for [`Php::unsupported`].
    current: String,
    /// The locals of the current function assigned after their `let`.
    reassigned: BTreeSet<String>,
    /// The conversion targets other normalizers accept.
    guarded: BTreeSet<String>,
    /// The generics of the current function.
    generics: Vec<Generic>,
    /// The indent of the current statement, for the bodies of its closures.
    indent: usize,
    /// The names of the module the code uses.
    imports: BTreeSet<String>,
    /// The exported values named as a schema type.
    values: BTreeSet<String>,
    /// The methods emitted are of a client: they take their parameters by
    /// name.
    by_name: bool,
    /// The traits are types bounding the generics, as in a
    /// [client](Emitter::client), not dictionaries.
    pub(crate) bounded: bool,
    /// The enums whose variants the tests build through the exported
    /// constructors.
    constructors: BTreeSet<String>,
}

impl<'a> Emitter<'a> {
    pub fn new(m: &'a Module, config: &'a PhpConfig) -> Self {
        Self {
            m,
            config,
            out: String::new(),
            this: "$this",
            notes: Vec::new(),
            unsupported: Vec::new(),
            current: String::new(),
            reassigned: BTreeSet::new(),
            guarded: BTreeSet::new(),
            generics: Vec::new(),
            indent: 0,
            imports: BTreeSet::new(),
            values: BTreeSet::new(),
            by_name: false,
            bounded: false,
            constructors: BTreeSet::new(),
        }
    }

    /// Emit the query builder module.
    pub fn module(mut self) -> Php {
        let m = self.m;

        for name in m.traits.keys() {
            self.trait_interface(name);
        }

        for name in &m.records {
            self.record(&m.classes[name]);
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
            self.class(&m.classes[b], *b == m.root);
        }

        let head = format!("{}\n\n", self.config.header);

        Php {
            code: head + &self.out,
            notes: self.notes,
            unsupported: self.unsupported,
            values: self.values.into_iter().collect(),
        }
    }

    /// Emit the tests as a PHPUnit test class.
    pub fn tests(mut self, name: &str, _module: &str, _glue: &[(&str, &str)]) -> Php {
        let m = self.m;
        self.constructors = constructed(m).into_iter().map(str::to_owned).collect();
        self.line(0, &format!("final class {name} extends \\PHPUnit\\Framework\\TestCase {{"));
        let mut suite: Option<&str> = None;

        for (i, f) in m.tests.iter().enumerate() {
            let (inner, test) = f
                .name
                .rsplit_once("::")
                .map_or((None, f.name.as_str()), |(s, t)| (Some(s), t));

            if inner != suite {
                if suite.is_some() {
                    self.line(0, "");
                }
                suite = inner;
            } else if i > 0 {
                self.line(0, "");
            }

            let test_name = to_camel_case(test);
            self.current = f.name.clone();
            self.reassigned = reassigned(&f.body);
            self.line(1, &format!("public function test{test_name}(): void {{"));
            self.block(&f.body, 2);
            self.line(1, "}");
        }

        self.line(0, "}");

        let head = format!("{}\n\n", self.config.header);

        Php {
            code: head + &self.out,
            notes: self.notes,
            unsupported: self.unsupported,
            values: Vec::new(),
        }
    }

    // ── Types ────────────────────────────────────────────────────────────

    pub(crate) fn ty(&self, ty: &Ty) -> String {
        match ty {
            Ty::Unit => "void".into(),
            Ty::Bool => "bool".into(),
            Ty::Int(_) => "int".into(),
            Ty::Float => "float".into(),
            Ty::Str => "string".into(),
            Ty::List(_) => "array".into(),
            Ty::Option(inner) => format!("{}|null", self.ty(inner)),
            Ty::Tuple(_) => "array".into(),
            Ty::Named(name, _) if self.is_class(name) => name.clone(),
            Ty::Named(name, _) => name.clone(),
            Ty::Param(_) => "mixed".into(),
            Ty::Into(to) => format!("Into{to}"),
            Ty::Dict(..) => "mixed".into(),
            Ty::SelfTy | Ty::Ref(_) | Ty::Result(..) | Ty::Fn | Ty::Unknown => "mixed".into(),
        }
    }

    /// The return type annotation for PHP, using `void` for unit and `mixed`
    /// for unknown.
    pub(crate) fn return_ty(&self, ty: &Ty) -> String {
        self.ty(ty)
    }

    /// Whether the type is a builder or opaque class.
    fn is_class(&self, name: &str) -> bool {
        self.m.is_builder(name) || self.m.opaque.contains_key(name)
    }

    fn is_builder_value(&self, e: &Expr) -> bool {
        matches!(e.k, K::This) || e.ty.name().is_some_and(|n| self.is_class(n))
    }

    /// The signatures of the interface of a trait.
    fn interface_sigs(&self, name: &str) -> impl Iterator<Item = &'a Func> + use<'a> {
        let m = self.m;
        let reflective = m.is_reflective(name);
        m.traits[name]
            .iter()
            .filter(move |f| !reflective || f.recv.is_none())
    }

    #[allow(dead_code)]
    fn has_self(&self, name: &str) -> bool {
        self.interface_sigs(name).any(|f| {
            f.recv.is_some()
                || f.params
                    .iter()
                    .map(|(_, t)| t)
                    .chain([&f.ret])
                    .any(|t| subst(t, &Ty::Param("Self".into()), &[]).mentions("Self"))
        })
    }

    #[allow(dead_code)]
    fn is_reflective(&self, param: &str) -> bool {
        self.generics
            .iter()
            .any(|g| g.name == param && g.bounds.iter().any(|(b, _)| self.m.is_reflective(b)))
    }

    fn self_ty(&self, class: &str) -> Ty {
        self.m
            .newtypes
            .get(class)
            .cloned()
            .unwrap_or_else(|| Ty::named(class))
    }

    // ── Literals ─────────────────────────────────────────────────────────

    fn lit(lit: &Lit) -> String {
        match lit {
            Lit::Bool(b) => b.to_string(),
            Lit::Int(i) => i.to_string(),
            Lit::Float(f) => format!("{f:?}"),
            Lit::Str(s) => format!("'{}'", escape_php(s)),
            Lit::Unit => "null".into(),
        }
    }

    fn list(&mut self, items: &[Expr]) -> String {
        items
            .iter()
            .map(|e| self.expr(e))
            .collect::<Vec<_>>()
            .join(", ")
    }

    // ── Expressions ──────────────────────────────────────────────────────

    pub(crate) fn expr(&mut self, e: &Expr) -> String {
        match &e.k {
            K::This => self.this.into(),
            K::Local(name) => format!("${name}"),
            K::Lit(lit) => Self::lit(lit),
            K::Array(items) | K::Tuple(items) => format!("[{}]", self.list(items)),
            K::Construct { class, fields } if self.is_class(class) => {
                let args: Vec<String> = self.m.classes[class]
                    .fields
                    .iter()
                    .map(|(name, _)| {
                        fields
                            .iter()
                            .find(|(f, _)| f == name)
                            .map_or("null".into(), |(_, value)| self.expr(value))
                    })
                    .collect();
                format!("new {class}({})", args.join(", "))
            }
            K::Construct { class, fields } => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|(name, value)| {
                        let n = self.field_name(class, name);
                        format!("{n}: {}", self.expr(value))
                    })
                    .collect();
                format!("new {class}({})", fields.join(", "))
            }
            K::Variant { en, payload, .. } if en == "Option" => match payload {
                Some(p) => self.expr(p),
                None => "null".into(),
            },
            K::Variant {
                en,
                var,
                payload: None,
            } => {
                let wire = self.variant_name(en, var);
                format!("'{wire}'")
            }
            K::Variant {
                en,
                var,
                payload: Some(p),
            } if self.constructors.contains(en)
                && matches!(self.m.variant(en, var), Some(Variant::Tuple(..))) =>
            {
                format!("{en}::{var}({})", self.expr(p))
            }
            K::Variant {
                en,
                var,
                payload: Some(p),
            } => {
                let wire = self.variant_name(en, var);
                format!("['{wire}' => {}]", self.expr(p))
            }
            K::Path(path) => path.join("::"),
            K::Field { base, name }
                if name.parse::<usize>().is_ok() && !self.is_builder_value(base) =>
            {
                format!("{}[{name}]", self.expr(base))
            }
            K::Field { base, name } => {
                let name = match &base.ty {
                    Ty::Named(class, _) => self.field_name(class, name),
                    _ => php_member(name),
                };
                format!("{}->${{name}}", self.expr(base)).replace("${name}", &name)
            }
            K::Payload { base, en, var } => {
                let var = self.variant_name(en, var);
                format!("{}['{var}']", self.expr(base))
            }
            K::Unwrap(value) | K::Ref(value) | K::Deref(value) => self.expr(value),
            K::Index { base, index } => format!("{}[{}]", self.expr(base), self.expr(index)),
            K::Call {
                callee: Callee::Intrinsic(Intrinsic::UnwrapErr),
                recv,
                ..
            } => {
                // PHP is synchronous, so unwrap_err is a try/catch.
                let r = recv
                    .as_deref()
                    .map(|r| self.expr(r))
                    .unwrap_or_default();
                self.unsupported(&format!("unwrap_err({r})"))
            }
            K::Call { callee, recv, args } => self.call(callee, recv.as_deref(), args),
            K::Convert { value, to } => {
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
                [one] if !is_block_statement(one) => {
                    let params: Vec<String> =
                        params.iter().map(|p| format!("${p}")).collect();
                    format!("fn({}) => {}", params.join(", "), self.expr(returned(one)))
                }
                _ => {
                    let params: Vec<String> =
                        params.iter().map(|p| format!("${p}")).collect();
                    let saved = std::mem::take(&mut self.out);
                    self.block(body, self.indent + 1);
                    let body_str = std::mem::replace(&mut self.out, saved);
                    // Collect captured variables for `use`.
                    format!(
                        "function({}) {{\n{body_str}{}}}",
                        params.join(", "),
                        "    ".repeat(self.indent)
                    )
                }
            },
            // PHP is synchronous: strip await.
            K::Await(value) => self.expr(value),
            K::Format(pieces) => self.concat(pieces),
            K::Cast(value) => {
                let cast = match &e.ty {
                    Ty::Int(_) => "(int)",
                    Ty::Float => "(float)",
                    Ty::Bool => "(bool)",
                    Ty::Str => "(string)",
                    _ => "",
                };
                if cast.is_empty() {
                    self.expr(value)
                } else {
                    format!("{cast}{}", self.expr(value))
                }
            }
            K::Panic(message) => format!("throw new \\RuntimeException({message:?})"),
            _ => self.unsupported(&print::expr(e)),
        }
    }

    /// String concatenation for `format!` pieces.
    fn concat(&mut self, pieces: &[Expr]) -> String {
        let mut parts = Vec::new();
        self.concat_pieces(pieces, &mut parts);

        if parts.is_empty() {
            return "''".into();
        }

        parts.join(" . ")
    }

    fn concat_pieces(&mut self, pieces: &[Expr], out: &mut Vec<String>) {
        for p in pieces {
            match &p.k {
                K::Lit(Lit::Str(s)) => {
                    if !s.is_empty() {
                        out.push(format!("'{}'", escape_php(s)));
                    }
                }
                K::Format(nested) => self.concat_pieces(nested, out),
                _ => out.push(self.expr(p)),
            }
        }
    }

    fn call(&mut self, callee: &Callee, recv: Option<&Expr>, args: &[Expr]) -> String {
        let a = if matches!(
            callee,
            Callee::Intrinsic(Intrinsic::AssertEq | Intrinsic::AssertNe)
        ) {
            String::new()
        } else {
            self.list(args)
        };
        let r = recv.map(|r| self.expr(r)).unwrap_or_default();

        match callee {
            Callee::Member { class, name, .. } if !self.is_class(class) => {
                let a = if a.is_empty() {
                    r.clone()
                } else {
                    format!("{r}, {a}")
                };
                format!("{class}_{name}({a})")
            }
            Callee::Member { name, .. } | Callee::Trait { name, .. } => {
                format!("{r}->{name}({a})")
            }
            Callee::Static { class, name } if !self.is_class(class) => {
                format!("{class}_{name}({a})")
            }
            Callee::Static { class, name } => format!("{class}::{name}({a})"),
            Callee::Extern(name) => format!("{name}({a})"),
            Callee::Intrinsic(i) => self.intrinsic(i, &r, &a, args),
            Callee::Path(..) | Callee::Method(..) => self.unsupported(&format!("{callee:?}")),
        }
    }

    fn intrinsic(&mut self, i: &Intrinsic, r: &str, a: &str, args: &[Expr]) -> String {
        match i {
            Intrinsic::Push => format!("{r}[] = {a}"),
            Intrinsic::Pop => format!("array_pop({r})"),
            Intrinsic::LastMut => format!("{r}[count({r}) - 1]"),
            Intrinsic::Len => format!("count({r})"),
            Intrinsic::IsEmpty => format!("count({r}) === 0"),
            Intrinsic::Extend => format!("array_push({r}, ...{a})"),
            Intrinsic::Reserve => "null".into(),
            Intrinsic::UnwrapOrDefault => format!("({r} ?? {a})"),
            Intrinsic::Expect => {
                format!(
                    "({r} ?? throw new \\RuntimeException({a}))",
                )
            }
            Intrinsic::Unwrap => {
                format!(
                    "({r} ?? throw new \\RuntimeException('called `Option::unwrap()` on a `None` value'))",
                )
            }
            Intrinsic::IsSome => format!("({r} !== null)"),
            Intrinsic::IsNone => format!("({r} === null)"),
            Intrinsic::MapSome => {
                format!(
                    "({r} === null ? null : ({a})({r}))"
                )
            }
            Intrinsic::Find => {
                format!("array_values(array_filter({r}, {a}))[0] ?? null")
            }
            Intrinsic::Any => format!("count(array_filter({r}, {a})) > 0"),
            Intrinsic::All => format!("count(array_filter({r}, {a})) === count({r})"),
            Intrinsic::SortByKey => format!("usort({r}, {a})"),
            Intrinsic::Remove => format!("array_splice({r}, {a}, 1)[0]"),
            Intrinsic::Repeat => format!("str_repeat({r}, {a})"),
            Intrinsic::Assert => format!("$this->assertTrue({a})"),
            Intrinsic::Map => format!("array_map({a}, {r})"),
            Intrinsic::MapOk => format!("({a})({r})"),
            Intrinsic::StartsWith => format!("str_starts_with({r}, {a})"),
            Intrinsic::IterForEach => format!("array_map({a}, {r})"),
            Intrinsic::AssertNe => self.assert_ne(args),
            Intrinsic::AssertEq => self.assert_eq(args),
            Intrinsic::MemTake | Intrinsic::MemSwap | Intrinsic::UnwrapErr => {
                self.unsupported(&format!("{i:?}"))
            }
        }
    }

    fn assert_eq(&mut self, args: &[Expr]) -> String {
        let left = self.expr(&args[0]);
        let constructors = std::mem::take(&mut self.constructors);
        let right = self.expr(&args[1]);
        self.constructors = constructors;
        format!("$this->assertEquals({left}, {right})")
    }

    fn assert_ne(&mut self, args: &[Expr]) -> String {
        let left = self.expr(&args[0]);
        let constructors = std::mem::take(&mut self.constructors);
        let right = self.expr(&args[1]);
        self.constructors = constructors;
        format!("$this->assertNotEquals({left}, {right})")
    }

    // ── Variant names ────────────────────────────────────────────────────

    fn variant_name(&mut self, en: &str, var: &str) -> String {
        let Some(wire) = self.m.wire.get(en) else {
            return var.to_owned();
        };

        if wire.tagging != Tagging::External {
            return self.unsupported(&format!("{en} is not externally tagged"));
        }

        wire.members.get(var).map_or(var, |m| &m.name).to_owned()
    }

    fn field_name(&self, class: &str, name: &str) -> String {
        match self.m.wire.get(class).and_then(|w| w.members.get(name)) {
            Some(m) => m.name.clone(),
            None => php_member(name),
        }
    }

    // ── Statements ───────────────────────────────────────────────────────

    pub(crate) fn unsupported(&mut self, what: &str) -> String {
        self.unsupported.push(format!("{}: `{what}`", self.current));
        "unsupported()".into()
    }

    fn test(&mut self, test: &Test) -> String {
        match test {
            Test::IsSome(base) => format!("{} !== null", self.expr(base)),
            Test::IsNone(base) => format!("{} === null", self.expr(base)),
            Test::IsVariant(base, _en, var) => {
                let base = self.expr(base);
                format!("is_array({base}) && array_key_exists('{var}', {base})")
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

    pub(crate) fn line(&mut self, indent: usize, line: &str) {
        let _ = writeln!(self.out, "{}{line}", "    ".repeat(indent));
    }

    pub(crate) fn block(&mut self, body: &[Expr], indent: usize) {
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
            K::Throw(value) => {
                let value = self.expr(value);
                self.line(indent, &format!("throw {value};"));
            }
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

                if e != "null" {
                    self.line(indent, &format!("{e};"));
                }
            }
        }
    }

    /// PHP bindings are just `$name = value;`.
    fn binding(&mut self, name: &str, value: Option<&Expr>, indent: usize) {
        let value = value.map_or("null".into(), |v| self.expr(v));
        self.line(indent, &format!("${name} = {value};"));
    }

    fn arm(&mut self, binds: &[(String, Expr)], body: &[Expr], indent: usize) {
        for (name, place) in binds {
            if body
                .iter()
                .any(|s| s.any(&|e| matches!(&e.k, K::Local(x) if x == name)))
            {
                let place = self.expr(place);
                self.line(indent, &format!("${name} = {place};"));
            }
        }

        self.block(body, indent);
    }

    fn arms(&mut self, value: &Expr, arms: &[(Pat, Vec<Expr>)], indent: usize) {
        let mut open = false;

        for (i, (pat, body)) in arms.iter().enumerate() {
            let (tests, binds) = pat.projections(value);

            if tests.is_empty() || i + 1 == arms.len() {
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
            let keyword = if open { "} elseif" } else { "if" };
            self.line(indent, &format!("{keyword} ({cond}) {{"));
            self.arm(&binds, body, indent + 1);
            open = true;
        }
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
                        "{}}} else{}",
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

    // ── Normalizers ──────────────────────────────────────────────────────

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
        arms.sort_by_key(|(ty, _, _)| (shape_rank(ty), literal_rank(ty)));
        let mut guards: Vec<String> = Vec::new();
        let mut kept = Vec::new();

        for (ty, pat, body) in arms {
            if self.subsumed(&pat, f.ret.name().unwrap_or(&to)) {
                self.notes.push(format!(
                    "{to}: `{}` arm is the identity in PHP, dropped",
                    print::pat(&pat)
                ));
                continue;
            }

            let (tests, binds) = pat.projections(&v);
            let guard = match &pat {
                Pat::Variant { .. } => self.conditions(&tests),
                _ => self.guard(&ty, "$v"),
            };

            if guard.is_empty() || guards.contains(&guard) {
                self.notes.push(format!(
                    "{to}: `{}` has the PHP representation of an earlier arm, dropped",
                    print::ty(&ty)
                ));
                continue;
            }

            guards.push(guard.clone());
            kept.push((ty, binds, guard, body));
        }

        let ret = self.ty(&f.ret);
        self.line(
            0,
            &format!("function {}($v): {ret} {{", f.name),
        );

        let last = kept.len().saturating_sub(1);

        for (i, (_ty, binds, guard, mut body)) in kept.into_iter().enumerate() {
            body.rewrite(&mut |e| {
                if let K::Local(n) = &e.k
                    && let Some((_, place)) = binds.iter().find(|(b, _)| b == n)
                    && !matches!(&place.k, K::Local(x) if x == n)
                {
                    *e = place.clone();
                }
            });
            let tail = i == last;
            let indent = if tail { 1 } else { 2 };

            if !tail {
                self.line(1, &format!("if ({guard}) {{"));
            }

            if is_statement(&body) {
                let mut block = vec![body];
                return_tail(&mut block);
                self.block(&block, indent);
            } else {
                let body = self.expr(&body);
                self.line(indent, &format!("return {body};"));
            }

            if !tail {
                self.line(1, "}");
            }
        }

        self.line(0, "}\n");
    }

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

    fn guard(&self, ty: &Ty, v: &str) -> String {
        match ty {
            Ty::Bool => format!("is_bool({v})"),
            Ty::Int(_) => format!("is_int({v})"),
            Ty::Float => format!("is_float({v})"),
            Ty::Str => format!("is_string({v})"),
            Ty::List(_) => format!("is_array({v})"),
            Ty::Tuple(_) => format!("is_array({v})"),
            Ty::Option(inner) => format!("({v} === null || {})", self.guard(inner, v)),
            Ty::Named(name, _) => {
                if self.m.enums.contains_key(name) {
                    format!("is_array({v})")
                } else {
                    format!("{v} instanceof {name}")
                }
            }
            Ty::Into(to) => format!("is_{}_value({v})", normalizer(to)),
            Ty::Param(_) => format!("is_object({v})"),
            _ => "true".into(),
        }
    }

    // ── Class / function emission ────────────────────────────────────────

    fn func(&mut self, f: &Func, indent: usize, prefix: &str) {
        let signature = self.signature(f, prefix);
        self.reassigned = reassigned(&f.body);
        self.line(indent, &format!("{signature} {{"));
        self.block(&f.body, indent + 1);
        self.line(indent, "}");
    }

    fn signature(&mut self, f: &Func, prefix: &str) -> String {
        self.current = f.name.clone();
        self.generics = f.generics.clone();
        let params: Vec<String> = f
            .params
            .iter()
            .map(|(n, t)| {
                let ty = self.ty(t);
                if ty == "mixed" || ty == "void" || ty == "array" {
                    format!("${n}")
                } else {
                    format!("{ty} ${n}")
                }
            })
            .collect();
        let params = params.join(", ");
        let ret = self.return_ty(&f.ret);
        let ret_annotation = if ret == "void" && f.body.is_empty() {
            ": void".to_owned()
        } else if ret == "mixed" {
            String::new()
        } else {
            format!(": {ret}")
        };
        // PHP is synchronous, strip async.
        format!("{prefix}function {}({params}){ret_annotation}", f.name)
    }

    fn trait_interface(&mut self, name: &str) {
        self.line(0, &format!("interface {name} {{"));

        for f in self.interface_sigs(name) {
            let params: Vec<String> = f
                .params
                .iter()
                .map(|(n, t)| {
                    let ty = self.ty(t);
                    if ty == "mixed" {
                        format!("${n}")
                    } else {
                        format!("{ty} ${n}")
                    }
                })
                .collect();
            let ret = self.return_ty(&f.ret);
            let ret_annotation = if ret == "mixed" {
                String::new()
            } else {
                format!(": {ret}")
            };
            self.line(
                1,
                &format!("public function {}({}){ret_annotation};", f.name, params.join(", ")),
            );
        }

        self.line(0, "}\n");
    }

    fn record(&mut self, c: &Class) {
        self.line(0, &format!("class {} {{", c.name));

        for (name, ty) in &c.fields {
            let ty = self.ty(ty);
            let ty_annotation = if ty == "mixed" {
                String::new()
            } else {
                format!("{ty} ")
            };
            self.line(1, &format!("public {ty_annotation}${name};"));
        }

        self.line(0, "}\n");
    }

    fn dicts(&mut self) {
        let m = self.m;

        for (tr, of) in &m.dicts {
            let Some(impls) = m.classes.get(of).and_then(|c| c.trait_impls.get(tr)) else {
                continue;
            };
            let self_ty = self.ty(&self.self_ty(of));
            self.line(
                0,
                &format!("${} = [", dict_global(of, tr)),
            );
            self.this = "$self";

            for f in impls {
                let params: Vec<String> = f
                    .recv
                    .as_ref()
                    .map(|_| format!("{self_ty} $self"))
                    .into_iter()
                    .chain(f.params.iter().map(|(n, t)| {
                        let ty = self.ty(t);
                        if ty == "mixed" {
                            format!("${n}")
                        } else {
                            format!("{ty} ${n}")
                        }
                    }))
                    .collect();
                let ret = self.return_ty(&f.ret);
                let ret_annotation = if ret == "mixed" {
                    String::new()
                } else {
                    format!(": {ret}")
                };
                self.line(
                    1,
                    &format!(
                        "'{name}' => function({params}){ret_annotation} {{",
                        name = f.name,
                        params = params.join(", ")
                    ),
                );
                self.reassigned = reassigned(&f.body);
                self.block(&f.body, 2);
                self.line(1, "},");
            }

            self.this = "$this";
            self.line(0, "];\n");
        }
    }

    fn constructors(&mut self) {
        let m = self.m;
        let normalized = normalized(m);

        for name in constructed(m) {
            self.values.insert(name.to_owned());
            self.line(0, &format!("class {name} {{"));
            self.line(1, "private function __construct(private string $tag, private mixed $value) {}");

            for variant in &m.enums[name].variants {
                let wire = self.variant_name(name, variant.name());

                match variant {
                    Variant::Unit(var) => {
                        self.line(
                            1,
                            &format!(
                                "public static function {var}(): self {{ return new self('{wire}', null); }}"
                            ),
                        );
                    }
                    Variant::Tuple(var, payload) => {
                        let (param, value) = match payload.name() {
                            Some(x) if normalized.contains(x) => {
                                ("mixed".to_owned(), format!("{}($v)", normalizer(x)))
                            }
                            _ => (self.ty(payload), "$v".into()),
                        };
                        let param_decl = if param == "mixed" {
                            "$v".to_owned()
                        } else {
                            format!("{param} $v")
                        };
                        self.line(
                            1,
                            &format!(
                                "public static function {var}({param_decl}): self {{ return new self('{wire}', {value}); }}"
                            ),
                        );
                    }
                    Variant::Struct(..) => {}
                }
            }

            self.line(
                1,
                "public function jsonSerialize(): mixed { return [$this->tag => $this->value]; }",
            );
            self.line(0, "}\n");
        }
    }

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
                self.this = "$self";
                self.func(&f, 0, "function ");
                self.this = "$this";
                self.out.push('\n');
            }
        }
    }

    fn class(&mut self, c: &Class, _export: bool) {
        self.line(0, &format!("class {} {{", c.name));

        if !c.fields.is_empty() {
            let fields: Vec<String> = c
                .fields
                .iter()
                .map(|(n, t)| {
                    let ty = self.ty(t);
                    let fname = self.field_name(&c.name, n);
                    if ty == "mixed" {
                        format!("public ${fname}")
                    } else {
                        format!("public {ty} ${fname}")
                    }
                })
                .collect();
            self.line(
                1,
                &format!("public function __construct({}) {{}}\n", fields.join(", ")),
            );
        }

        self.by_name = self.m.clients.contains(&c.name);

        for f in &c.methods {
            if f.recv.is_some() {
                self.func(f, 1, "public ");
            } else {
                let mut f = f.clone();
                f.generics.splice(0..0, c.generics.iter().cloned());
                self.func(&f, 1, "public static ");
            }

            self.out.push('\n');
        }

        self.by_name = false;
        self.out.truncate(self.out.trim_end().len());
        self.out.push('\n');
        self.line(0, "}\n");
    }

    /// The entries of the object `value` as the (key, value) list `ty`.
    fn fields(&mut self, value: &Expr, ty: &Ty) -> String {
        let Ty::List(entry) = ty else {
            return self.unsupported(&print::ty(ty));
        };
        let Ty::Tuple(kv) = entry.as_ref() else {
            return self.unsupported(&print::ty(ty));
        };
        let [_k, _v] = kv.as_slice() else {
            return self.unsupported(&print::ty(ty));
        };
        let object = self.expr(value);
        // PHP: convert object properties to [key, value] pairs.
        format!(
            "array_map(fn($k, $v) => [$k, $v], array_keys((array){object}), array_values((array){object}))"
        )
    }
}

// ── Free helpers ─────────────────────────────────────────────────────────

/// Whether the client method `f` takes its parameters as named: a method
/// with parameters.
#[allow(dead_code)]
fn by_name(f: &Func) -> bool {
    f.recv.is_some() && !f.params.is_empty()
}

/// The targets of the normalizers of `m`.
fn normalized(m: &Module) -> BTreeSet<&str> {
    m.functions
        .iter()
        .map(|f| f.name.trim_start_matches("into_"))
        .collect()
}

/// The enums of `m` the users build the values of.
fn constructed(m: &Module) -> BTreeSet<&str> {
    let normalized = normalized(m);
    m.conversions
        .iter()
        .filter(|c| normalized.contains(c.to.as_str()))
        .flat_map(|c| [Some(c.to.as_str()), c.from.name()])
        .flatten()
        .filter(|n| m.enums.contains_key(*n))
        .collect()
}

fn php_member(name: &str) -> String {
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

fn literal_rank(ty: &Ty) -> usize {
    match ty {
        Ty::Int("i64") => 1,
        Ty::Int("i32") => 0,
        Ty::Int(_) => 2,
        Ty::List(inner) => literal_rank(inner),
        _ => 0,
    }
}

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

/// Escape a string for PHP single-quoted strings.
fn escape_php(s: &str) -> String {
    let mut out = String::new();

    for c in s.chars() {
        match c {
            '\'' => out += "\\'",
            '\\' => out += "\\\\",
            c => out.push(c),
        }
    }

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

/// Convert snake_case to CamelCase for PHPUnit test method names.
fn to_camel_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

