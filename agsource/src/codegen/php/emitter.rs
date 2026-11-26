//! The PHP emitter: lowers the IR into PHP 8.1+ code. Builder classes use
//! method chaining, data types are plain classes with typed properties, and
//! the conversion functions are normalizer functions. PHP has no generics at
//! runtime so type parameters are erased; `int` and `float` stay distinct.

use crate::codegen::lowered::*;
use crate::codegen::pass::dict_global;
use crate::codegen::pass::is_statement;
use crate::codegen::pass::normalizer;
use crate::codegen::pass::return_tail;
use crate::codegen::print;
use std::collections::BTreeSet;
use std::fmt::Write;

/// `Result::unwrap_err` of a call that throws its error.
const UNWRAP_ERR: &str = "function unwrap_err(callable $fn): mixed {
    try { $fn(); }
    catch (\\Throwable $e) { return $e; }
    throw new \\RuntimeException(\"called `Result::unwrap_err()` on an `Ok` value\");
}
";

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
    /// The helpers the code calls, such as [`UNWRAP_ERR`].
    helpers: BTreeSet<&'static str>,
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
            helpers: BTreeSet::new(),
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

        let head = self.php_header();

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
        self.line(
            0,
            &format!("final class {name} extends \\PHPUnit\\Framework\\TestCase {{"),
        );
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

            // PHPUnit has no nested grouping like TS describe blocks, so
            // the suite prefix is included in the method name to avoid
            // collisions (e.g. admin_dashboard::add → testAdminDashboardAdd).
            let full = match inner {
                Some(group) => format!("{group}::{test}"),
                None => test.to_owned(),
            };
            let test_name = to_camel_case(&full);
            self.current = f.name.clone();
            self.reassigned = reassigned(&f.body);
            self.line(1, &format!("public function test{test_name}(): void {{"));
            self.block(&f.body, 2);
            self.line(1, "}");
        }

        self.line(0, "}");

        let mut head = self.php_header();
        for h in &self.helpers {
            head.push_str(h);
            head.push('\n');
        }

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
            Ty::Option(inner) => {
                let inner_ty = self.ty(inner);
                if inner_ty == "mixed" {
                    "mixed".into()
                } else {
                    format!("{inner_ty}|null")
                }
            }
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
                // PHP is synchronous: unwrap_err wraps the call in a closure
                // that a try/catch helper catches.
                self.helpers.insert(UNWRAP_ERR);
                let inner = recv
                    .as_deref()
                    .map(|r| match &r.k {
                        // Peel K::Await — PHP is sync, the call itself throws.
                        K::Await(value) => self.expr(value),
                        _ => self.expr(r),
                    })
                    .unwrap_or_default();
                format!("unwrap_err(fn() => {inner})")
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
                    let params: Vec<String> = params.iter().map(|p| format!("${p}")).collect();
                    format!("fn({}) => {}", params.join(", "), self.expr(returned(one)))
                }
                _ => {
                    let params: Vec<String> = params.iter().map(|p| format!("${p}")).collect();
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
            Intrinsic::Push => {
                // If the receiver is a guarded expression (e.g. `($x ?? throw ...)`),
                // PHP does not allow it in write context.  Unwrap the parenthesised
                // null-coalescing guard and push to the left operand directly — the
                // null case is unreachable at runtime because the builder always has
                // at least one conditions layer.
                if r.starts_with('(') && r.contains("?? throw") {
                    let inner = r
                        .strip_prefix('(')
                        .and_then(|s| s.rsplit_once(" ?? throw"))
                        .map_or(r.as_ref(), |(lhs, _)| lhs);
                    format!("{inner}[] = {a}")
                } else {
                    format!("{r}[] = {a}")
                }
            }
            Intrinsic::Pop => format!("array_pop({r})"),
            Intrinsic::LastMut => format!("{r}[count({r}) - 1]"),
            Intrinsic::Len => format!("count({r})"),
            Intrinsic::IsEmpty => format!("count({r}) === 0"),
            Intrinsic::Extend => format!("array_push({r}, ...{a})"),
            Intrinsic::Reserve => "null".into(),
            Intrinsic::UnwrapOrDefault => format!("({r} ?? {a})"),
            Intrinsic::Expect => {
                format!("({r} ?? throw new \\RuntimeException({a}))",)
            }
            Intrinsic::Unwrap => {
                format!(
                    "({r} ?? throw new \\RuntimeException('called `Option::unwrap()` on a `None` value'))",
                )
            }
            Intrinsic::IsSome => format!("({r} !== null)"),
            Intrinsic::IsNone => format!("({r} === null)"),
            Intrinsic::MapSome => {
                format!("({r} === null ? null : ({a})({r}))")
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

    /// The standard PHP file header: `<?php`, the generated comment, and the
    /// namespace when configured.
    pub(crate) fn php_header(&self) -> String {
        let mut head = String::from("<?php\n");
        head.push_str(&self.config.header);
        head.push('\n');
        if !self.config.namespace.is_empty() {
            head.push('\n');
            head.push_str(&format!("namespace {};\n", self.config.namespace));
        }
        if !self.config.types_require.is_empty() {
            head.push('\n');
            head.push_str(&format!(
                "require_once __DIR__ . '/{}.php';\n",
                self.config.types_require
            ));
        }
        head.push('\n');
        head
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
        self.line(0, &format!("function {}($v): {ret} {{", f.name));

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
                &format!(
                    "public function {}({}){ret_annotation};",
                    f.name,
                    params.join(", ")
                ),
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
            self.line(0, &format!("${} = [", dict_global(of, tr)));
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
            self.line(
                1,
                "private function __construct(private string $tag, private mixed $value) {}",
            );

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
                self.func(&f, 0, "");
                self.this = "$this";
                self.out.push('\n');
            }
        }
    }

    fn class(&mut self, c: &Class, _export: bool) {
        // A class whose name ends with `Error` is a Rust error type (it
        // implements `std::error::Error`) and must extend `\Exception` in
        // PHP so it can be thrown.
        let is_error = c.name.ends_with("Error");
        let extends = if is_error { " extends \\Exception" } else { "" };
        self.line(0, &format!("class {}{extends} {{", c.name));
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
            if is_error {
                // Error classes extend \Exception and must call the parent
                // constructor so the standard message is set.
                self.line(
                    1,
                    &format!("public function __construct({}) {{ parent::__construct($this->to_string()); }}\n", fields.join(", ")),
                );
            } else {
                self.line(
                    1,
                    &format!("public function __construct({}) {{}}\n", fields.join(", ")),
                );
            }
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
    s.split(['_', ':'])
        .filter_map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().to_string() + chars.as_str())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::pass::*;
    use crate::codegen::test_utilities::SUITE;
    use crate::codegen::test_utilities::conversion;
    use crate::codegen::test_utilities::lower;
    use crate::codegen::test_utilities::suite;
    use crate::codegen::test_utilities::tested;
    use crate::codegen::ts::AcceptElementSources;
    use crate::codegen::ts::EraseCasts;
    use crate::codegen::ts::EraseTraits;
    use crate::codegen::ts::OmitSkippedDefaults;
    use crate::codegen::ts::RemoveDeadHelpers;
    use crate::codegen::ts::RenameShadowed;
    use crate::codegen::ts::UnescapeIdentifiers;
    use std::collections::BTreeMap;

    fn config() -> PhpConfig {
        PhpConfig {
            header: "// header".into(),
            namespace: "Test".into(),
            types_require: "".into(),
        }
    }

    fn emitted(source: &str, traits: &[&str], data: &[&str]) -> Php {
        let (mut m, errors) = lower(source, traits, data);
        assert_eq!(errors, Vec::<String>::new());
        passes(&mut m);
        Emitter::new(&m, &config()).module()
    }

    fn emitted_tests(source: &str, tests: &str, data: &[&str]) -> Php {
        let (mut m, errors) = tested(source, &[], tests, data);
        assert_eq!(errors, Vec::<String>::new());
        passes(&mut m);
        Emitter::new(&m, &config()).tests("Suite", "", &[])
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
            .with(EraseTraits)
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

    /// The passes of the route tests: those of the builder, [`ThrowErrors`]
    /// and [`RenameShadowed`].
    fn route_passes(m: &mut Module) {
        Passes::new()
            .with(Resolve)
            .with(Prune)
            .with(ExplicitReturns)
            .with(Inline)
            .with(AcceptElementSources)
            .with(LowerConversions)
            .with(TraitDicts)
            .with(EraseOwnership)
            .with(ThrowErrors)
            .with(InferEmptyLists)
            .with(TagResults)
            .with(EraseTraits)
            .with(EraseCasts)
            .with(EraseNewtypes)
            .with(UnescapeIdentifiers)
            .with(OmitSkippedDefaults)
            .with(RemoveDeadHelpers)
            .with(RenameShadowed)
            .run(m, &mut |_, _| {});
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
            fn tests_should_be_a_phpunit_class_with_assertions_and_variables() {
                let php = emitted_tests(
                    "pub struct Root;",
                    "#[test]\nfn t() { assert_eq!(1, 1); }",
                    &[],
                );

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nfinal class Suite extends \\PHPUnit\\Framework\\TestCase {\n    public function testT(): void {\n        $this->assertEquals(1, 1);\n    }\n}\n"
                );
            }

            #[test]
            fn tests_should_emit_the_builder_calls_and_data_types() {
                let php = emitted_tests(
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
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nfinal class Suite extends \\PHPUnit\\Framework\\TestCase {\n    public function testFirst(): void {\n        $r = Root::new();\n        $this->assertEquals($r->q(), new Q(n: 1));\n    }\n\n    public function testSecond(): void {\n        $a = into_A([1, 2]);\n        $n = E_get('X');\n        throw new \\RuntimeException(\"p\");\n    }\n}\n"
                );
                assert_eq!(php.unsupported, Vec::<String>::new());
            }

            #[test]
            fn tests_should_build_the_variants_of_the_normalized_enums_through_their_constructors()
            {
                let php = emitted_tests(
                    r#"pub struct Root { pub v: V, pub w: W }
pub enum V { Id(u64), Name(String), None }
pub enum W { P(u8) }
impl From<u64> for V {
    fn from(id: u64) -> Self { V::Id(id) }
}
impl Root {
    pub fn new() -> Root { Root { v: V::None, w: W::P(0) } }
    pub fn id(&self, v: impl Into<V>) -> V { v.into() }
}"#,
                    r#"use c::Root;
use c::V;
#[test]
fn t() {
    let r = Root::new();
    assert_eq!(r.id(V::Id(1)), V::Id(1));
    assert_ne!(r.id(V::Name("a".to_string())), V::None);
    let w = c::W::P(1);
}"#,
                    &["V", "W"],
                );

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nfinal class Suite extends \\PHPUnit\\Framework\\TestCase {\n    public function testT(): void {\n        $r = Root::new();\n        $this->assertEquals($r->id(V::Id(1)), ['Id' => 1]);\n        $this->assertNotEquals($r->id(V::Name('a')), 'None');\n        $w = ['P' => 1];\n    }\n}\n"
                );
                assert!(php.unsupported.is_empty());
            }

            #[test]
            fn tests_should_emit_the_methods_of_the_options_lists_and_strings() {
                let php = emitted_tests(
                    "pub struct Root;",
                    r#"#[test]
fn checks() {
    let o: Option<u64> = Some(1);
    let mut v: Vec<u64> = vec![2, 1];
    let a = o.unwrap();
    assert!(o.is_some());
    assert!(!o.is_none());
    let d = o.map(|x| x + 1);
    let f = v.iter().find(|x| **x == 1);
    assert!(v.iter().any(|x| *x == 1));
    assert!(v.iter().all(|x| *x > 0));
    v.sort_by_key(|x| *x);
    let r = v.remove(0);
    assert_ne!(1, 2);
}"#,
                    &[],
                );

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nfinal class Suite extends \\PHPUnit\\Framework\\TestCase {\n    public function testChecks(): void {\n        $o = 1;\n        $v = [2, 1];\n        $a = ($o ?? throw new \\RuntimeException('called `Option::unwrap()` on a `None` value'));\n        $this->assertTrue(($o !== null));\n        $this->assertTrue(!($o === null));\n        $d = ($o === null ? null : (fn($x) => $x + 1)($o));\n        $f = array_values(array_filter($v, fn($x) => $x === 1))[0] ?? null;\n        $this->assertTrue(count(array_filter($v, fn($x) => $x === 1)) > 0);\n        $this->assertTrue(count(array_filter($v, fn($x) => $x > 0)) === count($v));\n        usort($v, fn($x) => $x);\n        $r = array_splice($v, 0, 1)[0];\n        $this->assertNotEquals(1, 2);\n    }\n}\n"
                );
                assert_eq!(php.unsupported, Vec::<String>::new());
            }

            #[test]
            fn tests_should_escape_single_quotes_and_backslashes_in_php_strings() {
                assert_eq!(escape_php("hello"), "hello");
                assert_eq!(escape_php("it's"), "it\\'s");
                assert_eq!(escape_php("a\\b"), "a\\\\b");
                assert_eq!(escape_php("a\\'b"), "a\\\\\\'b");
            }

            #[test]
            fn tests_should_emit_the_suites_as_flat_camel_case_methods() {
                let (mut m, _) = suite(
                    "pub struct Root;\nimpl Root { pub fn new() -> Root { Root } }",
                    SUITE,
                );
                route_passes(&mut m);
                m.tests.rotate_left(1);

                let php =
                    Emitter::new(&m, &config()).tests("Routes", "", &[("api", ""), ("glue", "")]);

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nfunction unwrap_err(callable $fn): mixed {\n    try { $fn(); }\n    catch (\\Throwable $e) { return $e; }\n    throw new \\RuntimeException(\"called `Result::unwrap_err()` on an `Ok` value\");\n}\n\nfinal class Routes extends \\PHPUnit\\Framework\\TestCase {\n    public function testLoginLogsIn(): void {\n        $server = Server::new();\n        $status = $server->api->login();\n        $name = next_name('user');\n        $agent = Helpers::agent();\n        $pair = unsupported();\n        $port = unsupported();\n        $this->assertNotEquals($status, 0);\n    }\n\n    public function testLoginFails(): void {\n        $server = Server::new();\n        $status = unwrap_err(fn() => $server->api->fail())->status;\n        $code = unwrap_err(fn() => $server->api->ping())->status;\n        $root = Root::new();\n        $this->assertTrue(count($server->address()) === 0);\n    }\n\n    public function testLogoutLogsOut(): void {\n        $server = Server::new();\n        $this->assertNotEquals($server->dir, 'admin');\n    }\n\n    public function testStatus(): void {\n        $server = Server::new();\n        $this->assertEquals($server->dir, 'admin');\n    }\n}\n"
                );
                assert_eq!(
                    php.unsupported,
                    [
                        "login::logs_in: `/* unsupported: unknown function `Pair` */`",
                        "login::logs_in: `/* unsupported: unknown path `PORT` */`",
                    ]
                );
            }

            #[test]
            fn tests_should_skip_the_ignored_tests() {
                let php = emitted_tests(
                    "pub struct Root;",
                    "#[test]\n#[ignore]\nfn t() {}\n#[test]\n#[ignore = \"slow\"]\nfn u() {}",
                    &[],
                );

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nfinal class Suite extends \\PHPUnit\\Framework\\TestCase {\n    public function testT(): void {\n    }\n\n    public function testU(): void {\n    }\n}\n"
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
                            types: Vec::new(),
                        },
                        Some(e.clone()),
                        vec![int(1)],
                    ),
                    call(
                        Callee::Member {
                            class: "E".into(),
                            name: "g".into(),
                            types: Vec::new(),
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
                    call(Callee::Method("m".into(), Vec::new()), None, Vec::new()),
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
                        vec![
                            Func {
                                recv: Some(Recv::Ref),
                                ..Func::new(
                                    "keys",
                                    vec![("n".into(), Ty::Int("u64"))],
                                    Ty::SelfTy,
                                    Vec::new(),
                                )
                            },
                            Func::new(
                                "make",
                                vec![("n".into(), Ty::Int("u64"))],
                                Ty::SelfTy,
                                Vec::new(),
                            ),
                        ],
                    )]
                    .into_iter()
                    .collect(),
                    ..Module::default()
                };

                let php = Emitter::new(&m, &config()).module();

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\ninterface Refl {\n    public function make(int $n);\n}\n\nclass Root {\n    public function exprs(bool $b, $l, E $e, $t, $s, $o, $r, D $n, $dict, $x, $pp): void {\n        1.5;\n        true;\n        'A';\n        $t[0];\n        $e[0];\n        $root->_0;\n        $e['B'];\n        $l[0];\n        !($b !== true);\n        !(count($l) === 0);\n        !$b;\n        $y = ($b ? 1 : 2);\n        function($x) {\n            $z = $x;\n            return $x;\n        };\n        fn() => new D(a: 1);\n        unsupported();\n        E_f($e, 1);\n        E_g($e);\n        Root::new(1);\n        ext(1);\n        unsupported();\n        unsupported();\n        array_map(fn($x) => $x, $l);\n        array_pop($l);\n        $l[count($l) - 1];\n        count($l);\n        array_push($l, ...$l);\n        ($l ?? 0);\n        ($l ?? throw new \\RuntimeException('e'));\n        unsupported();\n        unsupported();\n        unsupported();\n        unsupported();\n        $r = 1;\n        $r = 2;\n        $a = [];\n        $u = [];\n        $c = new D(a: 1);\n        $cu = new D(a: 1);\n        $v = ['B' => 1];\n        $n = null;\n        3;\n        if ($b) {\n            4;\n        }\n        if ($b) {\n            5;\n        } else {\n            6;\n        }\n        if ($b) {\n            7;\n        } elseif ($b !== true) {\n            8;\n        }\n        if (is_array($e) && array_key_exists('A', $e)) {\n            9;\n        }\n        if (is_array($e) && array_key_exists('B', $e)) {\n            $p = $e['B'];\n            return $p;\n        }\n        if (true) {\n            10;\n        }\n        while ($b);\n        while ($b) {\n            11;\n        }\n        if (is_array($e) && array_key_exists('A', $e)) {\n            12;\n        } elseif ($e === 1) {\n            13;\n        } else {\n            $w = $e;\n            return $w;\n        }\n        if (is_array($e) && array_key_exists('A', $e)) {\n            14;\n        } else {\n            15;\n        }\n        return null;\n    }\n\n    public static function new(): Root {\n    }\n}\n\n"
                );
                assert_eq!(php.notes, Vec::<String>::new());
                assert_eq!(
                    php.unsupported,
                    [
                        "exprs: `unresolved/*unresolved*/`",
                        "exprs: `Path([\"p\"], [])`",
                        "exprs: `Method(\"m\", [])`",
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
                        (list(Ty::named("U")), bind(), v(Ty::Unknown)),
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

                let php = Emitter::new(&m, &config()).module();

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\ninterface Refl {\n}\n\n$C_Tr = [\n    'add' => function(int $n): int {\n        return $n;\n    },\n];\n\nclass X {\n    private function __construct(private string $tag, private mixed $value) {}\n    public static function P(int $v): self { return new self('P', $v); }\n    public static function B(bool $v): self { return new self('B', $v); }\n    public function jsonSerialize(): mixed { return [$this->tag => $this->value]; }\n}\n\nclass Y {\n    private function __construct(private string $tag, private mixed $value) {}\n    public static function P(int $v): self { return new self('P', $v); }\n    public static function Q(int $v): self { return new self('Q', $v); }\n    public static function R(int $v): self { return new self('R', $v); }\n    public function jsonSerialize(): mixed { return [$this->tag => $this->value]; }\n}\n\nclass Z {\n    private function __construct(private string $tag, private mixed $value) {}\n    public static function Z1($v): self { return new self('Z1', into_X($v)); }\n    public function jsonSerialize(): mixed { return [$this->tag => $this->value]; }\n}\n\nfunction into_X($v): X {\n    if (is_bool($v)) {\n        return ['B' => $v];\n    }\n    if (is_int($v)) {\n        return ['P' => $v];\n    }\n    if (is_float($v)) {\n        return $v;\n    }\n    if (is_string($v)) {\n        return $v;\n    }\n    if (is_array($v)) {\n        return $v;\n    }\n    if ($v instanceof D) {\n        return $v;\n    }\n    if ($v instanceof Nope) {\n        return $v;\n    }\n    if (is_array($v) && array_key_exists('Q', $v)) {\n        return ['P' => $v['Q']];\n    }\n    if (is_array($v) && array_key_exists('R', $v)) {\n        return ['P' => 0];\n    }\n    if (is_into_Z_value($v)) {\n        return $v;\n    }\n    if (is_object($v)) {\n        return $v;\n    }\n    return $v;\n}\n\nfunction into_Z($v): Z {\n    if (is_array($v)) {\n        return $v;\n    }\n    return $v;\n}\n\nfunction D_get(D $self): int {\n    return $self->a;\n}\n\nfunction D_make(): D {\n    return 1;\n}\n\n"
                );
                assert_eq!(
                    php.notes,
                    [
                        "X: `i64` has the PHP representation of an earlier arm, dropped",
                        "X: `u8` has the PHP representation of an earlier arm, dropped",
                        "X: `str` has the PHP representation of an earlier arm, dropped",
                        "X: `(str, u64)` has the PHP representation of an earlier arm, dropped",
                        "X: `Vec<U>` has the PHP representation of an earlier arm, dropped",
                        "X: `Vec<u64>` has the PHP representation of an earlier arm, dropped",
                        "X: `X` has the PHP representation of an earlier arm, dropped",
                        "X: `U` has the PHP representation of an earlier arm, dropped",
                        "X: `Y::P(p)` arm is the identity in PHP, dropped",
                        "Z: `(impl Into<X>)` has the PHP representation of an earlier arm, dropped",
                        "Z: `Z` has the PHP representation of an earlier arm, dropped",
                    ]
                );
                assert_eq!(
                    php.unsupported,
                    [
                        "empty: `a function other than a conversion`",
                        "returns: `a function other than a conversion`"
                    ]
                );
            }

            #[test]
            fn module_should_emit_the_records_casts_wire_names_and_untyped_signatures() {
                let list = |t: Ty| Ty::List(Box::new(t));
                let t = || Ty::Param("T".into());
                let b = Expr::local("b", Ty::Bool);
                let l = Expr::local("l", list(Ty::Int("u64")));
                let o = Expr::local("o", Ty::Option(Box::new(Ty::Int("u64"))));
                let d = Expr::local("d", Ty::named("D"));
                let str_ = |s: &str| Expr::new(K::Lit(Lit::Str(s.into())));
                let intrinsic = |i: Intrinsic, recv: Expr, args: Vec<Expr>| {
                    call(Callee::Intrinsic(i), Some(recv), args)
                };
                let cast = |ty: Ty| Expr::typed(K::Cast(int(1).boxed()), ty);
                let if_ = |then: Vec<Expr>, els: Option<Vec<Expr>>| {
                    Expr::new(K::If {
                        cond: b.clone().boxed(),
                        then,
                        els,
                    })
                };
                let closure = |body: Vec<Expr>| {
                    Expr::new(K::Closure {
                        params: Vec::new(),
                        body,
                    })
                };
                let if_let = |var: &str, inner: Vec<Pat>| {
                    Expr::new(K::IfLet {
                        pat: variant("Option", var, inner),
                        value: o.clone().boxed(),
                        then: vec![ret(Expr::local("y", Ty::Int("u64")))],
                        els: None,
                    })
                };
                let body = vec![
                    Expr::new(K::Path(vec!["A".into(), "B".into()])),
                    Expr::local("x", Ty::Unknown).field("y", Ty::Unknown),
                    d.clone().field("a", Ty::Int("u64")),
                    Expr::typed(
                        K::Construct {
                            class: "D".into(),
                            fields: vec![("a".into(), int(1))],
                        },
                        Ty::named("D"),
                    ),
                    Expr::new(K::Ref(d.clone().boxed())),
                    Expr::new(K::Deref(d.clone().boxed())),
                    cast(Ty::Int("u8")),
                    cast(Ty::Float),
                    cast(Ty::Bool),
                    cast(Ty::Str),
                    cast(Ty::Unknown),
                    Expr::new(K::Format(Vec::new())),
                    Expr::new(K::Format(vec![
                        str_("a'b"),
                        str_(""),
                        Expr::new(K::Format(vec![Expr::local("x", Ty::Unknown)])),
                        int(1),
                    ])),
                    call(
                        Callee::Static {
                            class: "D".into(),
                            name: "make".into(),
                        },
                        None,
                        vec![int(1)],
                    ),
                    intrinsic(Intrinsic::Push, l.clone(), vec![int(1)]),
                    intrinsic(
                        Intrinsic::Push,
                        intrinsic(Intrinsic::Expect, l.clone(), vec![str_("e")]),
                        vec![int(2)],
                    ),
                    intrinsic(Intrinsic::Repeat, Expr::local("s", Ty::Str), vec![int(2)]),
                    intrinsic(
                        Intrinsic::IterForEach,
                        l.clone(),
                        vec![closure(vec![int(1)])],
                    ),
                    Expr::typed(Expr::variant("E", "B", Some(int(1))).k, Ty::named("E")),
                    Expr::variant("I", "A", None),
                    Expr::typed(
                        K::Fields(d.clone().boxed()),
                        list(Ty::Tuple(vec![Ty::Str, Ty::Unknown])),
                    ),
                    closure(vec![let_("z", int(1))]),
                    closure(vec![if_(vec![int(1)], Some(vec![int(2)]))]),
                    closure(vec![if_(vec![int(1)], None)]),
                    if_(vec![int(3)], Some(vec![int(4)])),
                    if_let("Some", vec![Pat::Bind("y".into())]),
                    if_let("None", Vec::new()),
                ];
                let method =
                    |name: &str, params: Vec<(String, Ty)>, ret: Ty, body: Vec<Expr>| Func {
                        recv: Some(Recv::Ref),
                        ..Func::new(name, params, ret, body)
                    };
                let into_k = Func::new(
                    "into_K",
                    vec![("v".into(), Ty::Into("K".into()))],
                    Ty::named("K"),
                    vec![ret(Expr::new(K::TypeCase {
                        value: Expr::local("v", Ty::Into("K".into())).boxed(),
                        arms: vec![
                            (
                                Ty::Int("u64"),
                                Pat::Bind("v".into()),
                                Expr::variant("K", "Kt", Some(Expr::local("v", Ty::Int("u64")))),
                            ),
                            (
                                Ty::named("E"),
                                variant("E", "A", Vec::new()),
                                Expr::new(K::Match {
                                    value: Expr::local("v", Ty::named("E")).boxed(),
                                    arms: vec![(Pat::Wild, vec![int(1)])],
                                }),
                            ),
                        ],
                    }))],
                );
                let wire = |tagging: Tagging, members: &[(&str, &str)]| Wire {
                    tagging,
                    members: members
                        .iter()
                        .map(|(name, wire)| {
                            (
                                (*name).to_owned(),
                                Member {
                                    name: (*wire).into(),
                                    ..Member::default()
                                },
                            )
                        })
                        .collect(),
                    ..Wire::default()
                };
                let m = Module {
                    root: "Root".into(),
                    builders: vec!["Root".into()],
                    records: ["R".to_owned()].into_iter().collect(),
                    classes: [
                        class(
                            "Root",
                            Vec::new(),
                            vec![
                                method(
                                    "rest",
                                    vec![
                                        ("b".into(), Ty::Bool),
                                        ("into".into(), Ty::Into("K".into())),
                                    ],
                                    Ty::Unit,
                                    body,
                                ),
                                method("empty", Vec::new(), Ty::Unit, Vec::new()),
                                method(
                                    "param",
                                    vec![("p".into(), t())],
                                    t(),
                                    vec![ret(Expr::local("p", t()))],
                                ),
                            ],
                        ),
                        (
                            "D".to_owned(),
                            Class {
                                trait_impls: [(
                                    "Tr".to_owned(),
                                    vec![method(
                                        "id",
                                        vec![("p".into(), t())],
                                        t(),
                                        vec![ret(Expr::local("p", t()))],
                                    )],
                                )]
                                .into_iter()
                                .collect(),
                                ..class("D", vec![("a", Ty::Int("u64"))], Vec::new()).1
                            },
                        ),
                        class("R", vec![("a", Ty::Float), ("b", t())], Vec::new()),
                    ]
                    .into_iter()
                    .collect(),
                    enums: [
                        en(
                            "E",
                            vec![
                                Variant::Unit("A".into()),
                                Variant::Tuple("B".into(), Ty::Int("u64")),
                            ],
                            Vec::new(),
                        ),
                        en("I", vec![Variant::Unit("A".into())], Vec::new()),
                        en(
                            "K",
                            vec![
                                Variant::Unit("Ku".into()),
                                Variant::Tuple("Kt".into(), Ty::Int("u64")),
                            ],
                            Vec::new(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    wire: [
                        ("D".to_owned(), wire(Tagging::External, &[("a", "x")])),
                        ("E".to_owned(), wire(Tagging::External, &[("B", "b")])),
                        ("I".to_owned(), wire(Tagging::Internal("t".into()), &[])),
                    ]
                    .into_iter()
                    .collect(),
                    conversions: vec![conversion(
                        "K",
                        Ty::Int("u64"),
                        Expr::local("v", Ty::Int("u64")),
                    )],
                    functions: vec![into_k],
                    dicts: [("Tr".to_owned(), "D".to_owned())].into_iter().collect(),
                    traits: [(
                        "Tr".to_owned(),
                        vec![Func::new(
                            "f",
                            vec![("p".into(), t())],
                            Ty::Int("u64"),
                            Vec::new(),
                        )],
                    )]
                    .into_iter()
                    .collect(),
                    ..Module::default()
                };
                let config = PhpConfig {
                    header: "// header".into(),
                    namespace: String::new(),
                    types_require: "types".into(),
                };

                let php = Emitter::new(&m, &config).module();

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nrequire_once __DIR__ . '/types.php';\n\ninterface Tr {\n    public function f($p): int;\n}\n\nclass R {\n    public float $a;\n    public $b;\n}\n\n$D_Tr = [\n    'id' => function(D $self, $p) {\n        return $p;\n    },\n];\n\nclass K {\n    private function __construct(private string $tag, private mixed $value) {}\n    public static function Ku(): self { return new self('Ku', null); }\n    public static function Kt(int $v): self { return new self('Kt', $v); }\n    public function jsonSerialize(): mixed { return [$this->tag => $this->value]; }\n}\n\nfunction into_K($v): K {\n    if (is_int($v)) {\n        return ['Kt' => $v];\n    }\n    return 1;\n}\n\nclass Root {\n    public function rest(bool $b, IntoK $into): void {\n        A::B;\n        $x->y;\n        $d->x;\n        new D(x: 1);\n        $d;\n        $d;\n        (int)1;\n        (float)1;\n        (bool)1;\n        (string)1;\n        1;\n        '';\n        'a\\'b' . $x . 1;\n        D_make(1);\n        $l[] = 1;\n        $l[] = 2;\n        str_repeat($s, 2);\n        array_map(fn() => 1, $l);\n        ['b' => 1];\n        'unsupported()';\n        array_map(fn($k, $v) => [$k, $v], array_keys((array)$d), array_values((array)$d));\n        function() {\n            $z = 1;\n        };\n        fn() => ($b ? 1 : 2);\n        function() {\n            if ($b) {\n                1;\n            }\n        };\n        if ($b) {\n            3;\n        } else {\n            4;\n        }\n        if ($o !== null) {\n            $y = $o;\n            return $y;\n        }\n        if ($o === null) {\n            return $y;\n        }\n    }\n\n    public function empty(): void {\n    }\n\n    public function param($p) {\n        return $p;\n    }\n}\n\n"
                );
                assert_eq!(php.notes, Vec::<String>::new());
                assert_eq!(php.unsupported, ["rest: `I is not externally tagged`"]);
            }

            #[test]
            fn module_should_emit_a_simple_builder_without_panic_helper() {
                let php = emitted(
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
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nclass Root {\n    public function n(): int {\n        return 1;\n    }\n}\n\n"
                );
            }

            #[test]
            fn module_should_emit_the_serialized_names_of_the_wire_types() {
                let php = emitted(
                    r#"
                    pub struct Root;
                    #[derive(Serialize)]
                    #[serde(rename_all = "lowercase")]
                    pub enum W { Ab(Q), Cd }
                    #[derive(Serialize)]
                    pub struct Q { #[serde(rename = "x")] pub a: u8 }
                    "#,
                    &[],
                    &["W"],
                );

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nclass Root {\n}\n\n"
                );
            }
        }
    }
}
