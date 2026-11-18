//! The lowered IR: classes, enums and functions with typed expressions. It
//! starts out "syntactic" from [`SourceCrate::lower`](crate::SourceCrate::lower)
//! (method calls by name, paths, unknown types) and the passes of
//! [`pass`](super::pass) resolve it and strip what only Rust needs.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Ty {
    Unit,
    Bool,
    Int(&'static str),
    Float,
    Str,
    List(Box<Ty>),
    Option(Box<Ty>),
    Tuple(Vec<Ty>),
    Named(String, Vec<Ty>),
    Param(String),
    SelfTy,
    Ref(Box<Ty>),
    /// A value convertible into the named type: a lowered `G: Into<X>`.
    Into(String),
    /// The implementation of the trait for the type, a trait dictionary.
    Dict(String, Box<Ty>),
    Fn,
    Unknown,
}

impl Ty {
    pub fn named(name: &str) -> Ty {
        Ty::Named(name.to_owned(), Vec::new())
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Ty::Named(name, _) => Some(name),
            _ => None,
        }
    }

    pub fn strip_ref(&self) -> &Ty {
        match self {
            Ty::Ref(ty) => ty.strip_ref(),
            ty => ty,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Lit {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Unit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Pat {
    Wild,
    Bind(String),
    Lit(Lit),
    /// `Enum::Variant(inner..)`, `Some(inner)`; `en` is empty until resolved.
    Variant {
        en: String,
        var: String,
        inner: Vec<Pat>,
    },
    Struct {
        name: String,
        fields: Vec<(String, Pat)>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Callee {
    /// An unresolved call of `a::b::c(..)` with the type arguments of the
    /// path.
    Path(Vec<String>, Vec<Ty>),
    /// An unresolved method call by name.
    Method(String),
    /// A method of a class or enum of the module.
    Member {
        class: String,
        name: String,
    },
    /// An associated function, `Type::name(..)`.
    Static {
        class: String,
        name: String,
    },
    /// A trait method, dispatched through the dictionary that is the
    /// receiver.
    Trait {
        trait_: String,
        name: String,
    },
    Intrinsic(Intrinsic),
    /// A function the target runtime provides.
    Extern(String),
}

/// An operation on std types (`Vec`, `Option`, `mem`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intrinsic {
    Push,
    Pop,
    LastMut,
    Len,
    IsEmpty,
    Extend,
    Reserve,
    UnwrapOrDefault,
    Expect,
    MemTake,
    MemSwap,
    IterForEach,
    Map,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub k: K,
    pub ty: Ty,
}

#[derive(Clone, Debug, PartialEq)]
pub enum K {
    /// An unresolved name.
    Ident(String),
    Local(String),
    This,
    Lit(Lit),
    Array(Vec<Expr>),
    Tuple(Vec<Expr>),
    /// A struct or class value; the fields of a tuple struct are "0", "1",
    /// ...
    Construct {
        class: String,
        fields: Vec<(String, Expr)>,
    },
    Variant {
        en: String,
        var: String,
        payload: Option<Box<Expr>>,
    },
    /// An unresolved `Type::Variant` or constant.
    Path(Vec<String>),
    Field {
        base: Box<Expr>,
        name: String,
    },
    /// The payload of `en::var(..)` once a pattern matched it.
    Payload {
        base: Box<Expr>,
        en: String,
        var: String,
    },
    /// The value in a `Some(..)` once a pattern matched it.
    Unwrap(Box<Expr>),
    Match {
        value: Box<Expr>,
        arms: Vec<(Pat, Vec<Expr>)>,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        callee: Callee,
        recv: Option<Box<Expr>>,
        args: Vec<Expr>,
    },
    /// `value.into()` with the type it converts into.
    Convert {
        value: Box<Expr>,
        to: String,
    },
    /// The arm for the runtime representation of `value`.
    TypeCase {
        value: Box<Expr>,
        arms: Vec<(Ty, Pat, Expr)>,
    },
    Assign {
        place: Box<Expr>,
        value: Box<Expr>,
    },
    Let {
        name: String,
        value: Option<Box<Expr>>,
    },
    If {
        cond: Box<Expr>,
        then: Vec<Expr>,
        els: Option<Vec<Expr>>,
    },
    IfLet {
        pat: Pat,
        value: Box<Expr>,
        then: Vec<Expr>,
        els: Option<Vec<Expr>>,
    },
    While {
        cond: Box<Expr>,
        body: Vec<Expr>,
    },
    Return(Option<Box<Expr>>),
    Binary {
        op: &'static str,
        l: Box<Expr>,
        r: Box<Expr>,
    },
    Not(Box<Expr>),
    Ref(Box<Expr>),
    Deref(Box<Expr>),
    Closure {
        params: Vec<String>,
        body: Vec<Expr>,
    },
    Panic(String),
    /// A construct the lowered IR does not have.
    Unsupported(String),
}

impl Expr {
    pub fn new(k: K) -> Expr {
        Expr { k, ty: Ty::Unknown }
    }

    pub fn typed(k: K, ty: Ty) -> Expr {
        Expr { k, ty }
    }

    pub fn boxed(self) -> Box<Expr> {
        Box::new(self)
    }

    pub fn local(name: &str, ty: Ty) -> Expr {
        Expr::typed(K::Local(name.to_owned()), ty)
    }

    pub fn field(self, name: &str, ty: Ty) -> Expr {
        Expr::typed(
            K::Field {
                base: self.boxed(),
                name: name.to_owned(),
            },
            ty,
        )
    }

    pub fn variant(en: &str, var: &str, payload: Option<Expr>) -> Expr {
        Expr::typed(
            K::Variant {
                en: en.to_owned(),
                var: var.to_owned(),
                payload: payload.map(Box::new),
            },
            Ty::named(en),
        )
    }

    pub fn children_mut(&mut self) -> Vec<&mut Expr> {
        let mut out: Vec<&mut Expr> = Vec::new();

        match &mut self.k {
            K::Array(items) | K::Tuple(items) => out.extend(items.iter_mut()),
            K::Construct { fields, .. } => out.extend(fields.iter_mut().map(|(_, e)| e)),
            K::Variant { payload, .. } => out.extend(payload.as_deref_mut()),
            K::Field { base, .. } | K::Payload { base, .. } | K::Unwrap(base) => out.push(base),
            K::Match { value, arms } => {
                out.push(value);
                out.extend(arms.iter_mut().flat_map(|(_, body)| body.iter_mut()));
            }
            K::Index { base, index } => {
                out.push(base);
                out.push(index);
            }
            K::Call { recv, args, .. } => {
                out.extend(recv.as_deref_mut());
                out.extend(args.iter_mut());
            }
            K::Convert { value, .. } => out.push(value),
            K::TypeCase { value, arms } => {
                out.push(value);
                out.extend(arms.iter_mut().map(|(_, _, e)| e));
            }
            K::Assign { place, value } => {
                out.push(place);
                out.push(value);
            }
            K::Let { value, .. } => out.extend(value.as_deref_mut()),
            K::If { cond, then, els } => {
                out.push(cond);
                out.extend(then.iter_mut());
                out.extend(els.iter_mut().flatten());
            }
            K::IfLet {
                value, then, els, ..
            } => {
                out.push(value);
                out.extend(then.iter_mut());
                out.extend(els.iter_mut().flatten());
            }
            K::While { cond, body } => {
                out.push(cond);
                out.extend(body.iter_mut());
            }
            K::Return(value) => out.extend(value.as_deref_mut()),
            K::Binary { l, r, .. } => {
                out.push(l);
                out.push(r);
            }
            K::Not(e) | K::Ref(e) | K::Deref(e) => out.push(e),
            K::Closure { body, .. } => out.extend(body.iter_mut()),
            K::Ident(_)
            | K::Local(_)
            | K::This
            | K::Lit(_)
            | K::Path(_)
            | K::Panic(_)
            | K::Unsupported(_) => {}
        }

        out
    }

    /// Calls `f` on every expression, children first.
    pub fn rewrite(&mut self, f: &mut dyn FnMut(&mut Expr)) {
        for child in self.children_mut() {
            child.rewrite(f);
        }

        f(self);
    }

    /// Whether `f` holds for the expression or one within it.
    pub fn any(&self, f: &dyn Fn(&Expr) -> bool) -> bool {
        fn any_mut(e: &mut Expr, f: &dyn Fn(&Expr) -> bool) -> bool {
            f(e) || e.children_mut().into_iter().any(|c| any_mut(c, f))
        }

        any_mut(&mut self.clone(), f)
    }
}

pub fn rewrite_body(body: &mut [Expr], f: &mut dyn FnMut(&mut Expr)) {
    for e in body {
        e.rewrite(f);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Generic {
    pub name: String,
    /// The traits by name, with the first type argument (`Into<X>`).
    pub bounds: Vec<(String, Option<Ty>)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Recv {
    Value,
    ValueMut,
    Ref,
    RefMut,
}

/// Where a function is in the source.
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub path: PathBuf,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Func {
    pub name: String,
    pub generics: Vec<Generic>,
    pub recv: Option<Recv>,
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
    pub body: Vec<Expr>,
    /// Not `pub`.
    pub private: bool,
    /// None for the functions the passes make.
    pub at: Option<Location>,
    /// The constructs of the function the passes could not lower, reported
    /// once it is known to be used (see [`Prune`](super::pass::Prune)).
    pub errors: Vec<String>,
}

impl Func {
    /// A function the passes make.
    pub fn new(name: &str, params: Vec<(String, Ty)>, ret: Ty, body: Vec<Expr>) -> Func {
        Func {
            name: name.to_owned(),
            generics: Vec::new(),
            recv: None,
            params,
            ret,
            body,
            private: false,
            at: None,
            errors: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub name: String,
    pub generics: Vec<Generic>,
    /// The fields of a tuple struct are "0", "1", ...
    pub fields: Vec<(String, Ty)>,
    pub methods: Vec<Func>,
    /// The methods of the impls of the traits, by trait.
    pub trait_impls: BTreeMap<String, Vec<Func>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Variant {
    Unit(String),
    Tuple(String, Ty),
    Struct(String, Vec<(String, Ty)>),
}

impl Variant {
    pub fn name(&self) -> &str {
        match self {
            Variant::Unit(name) | Variant::Tuple(name, _) | Variant::Struct(name, _) => name,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<Variant>,
    pub methods: Vec<Func>,
}

/// A `From` conversion into `to`, with the value converted as local `v` of
/// the body.
#[derive(Clone, Debug, PartialEq)]
pub struct Conversion {
    pub to: String,
    pub from: Ty,
    pub body: Expr,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Module {
    /// The builders and the data types.
    pub classes: BTreeMap<String, Class>,
    pub enums: BTreeMap<String, Enum>,
    pub consts: BTreeMap<String, Expr>,
    pub conversions: Vec<Conversion>,
    /// The classes of the builder API in the order they were found.
    pub builders: Vec<String>,
    /// The method signatures of the traits.
    pub traits: BTreeMap<String, Vec<Func>>,
    /// The functions the passes make, such as conversions.
    pub functions: Vec<Func>,
    /// The `Default` values of types.
    pub defaults: BTreeMap<String, Expr>,
    /// The fields left out of the serialized value when they hold the value:
    /// (struct, field, value).
    pub skipped_defaults: Vec<(String, String, Expr)>,
    /// The methods (class, name) the `inline` pass inlined.
    pub inlined: BTreeSet<(String, String)>,
    /// The (trait, type) dictionaries the `trait_dicts` pass refers to.
    pub dicts: BTreeSet<(String, String)>,
}

impl Module {
    pub fn is_builder(&self, name: &str) -> bool {
        self.builders.iter().any(|b| b == name)
    }

    pub fn field_ty(&self, class: &str, field: &str) -> Ty {
        self.classes
            .get(class)
            .and_then(|c| c.fields.iter().find(|(name, _)| name == field))
            .map(|(_, ty)| ty.clone())
            .unwrap_or(Ty::Unknown)
    }

    pub fn method(&self, class: &str, name: &str) -> Option<&Func> {
        if let Some(e) = self.enums.get(class) {
            return e.methods.iter().find(|m| m.name == name);
        }

        let c = self.classes.get(class)?;
        c.methods
            .iter()
            .chain(c.trait_impls.values().flatten())
            .find(|m| m.name == name)
    }

    pub fn generics_of(&self, class: &str) -> &[Generic] {
        self.classes
            .get(class)
            .map(|c| c.generics.as_slice())
            .unwrap_or(&[])
    }

    pub fn variant(&self, en: &str, var: &str) -> Option<&Variant> {
        self.enums
            .get(en)
            .and_then(|e| e.variants.iter().find(|v| v.name() == var))
    }

    /// Every function: the methods of classes and their trait impls, of
    /// enums, and the functions of the module, with the name of their type
    /// ("" for the functions of the module).
    pub fn for_each_func(&mut self, f: &mut dyn FnMut(&mut Func, &str)) {
        for (name, c) in self.classes.iter_mut() {
            for func in c
                .methods
                .iter_mut()
                .chain(c.trait_impls.values_mut().flatten())
            {
                f(func, name);
            }
        }

        for (name, e) in self.enums.iter_mut() {
            for func in e.methods.iter_mut() {
                f(func, name);
            }
        }

        for func in self.functions.iter_mut() {
            f(func, "");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;

    fn lit(i: i64) -> Expr {
        Expr::new(K::Lit(Lit::Int(i)))
    }

    fn every_kind() -> Vec<Expr> {
        let b = |i| lit(i).boxed();

        [
            K::Ident("i".into()),
            K::Local("l".into()),
            K::This,
            K::Lit(Lit::Unit),
            K::Array(vec![lit(1), lit(2)]),
            K::Tuple(vec![lit(1)]),
            K::Construct {
                class: "C".into(),
                fields: vec![("a".into(), lit(1))],
            },
            K::Variant {
                en: "E".into(),
                var: "A".into(),
                payload: Some(b(1)),
            },
            K::Path(vec!["p".into()]),
            K::Field {
                base: b(1),
                name: "f".into(),
            },
            K::Payload {
                base: b(1),
                en: "E".into(),
                var: "A".into(),
            },
            K::Unwrap(b(1)),
            K::Match {
                value: b(1),
                arms: vec![(Pat::Wild, vec![lit(2), lit(3)])],
            },
            K::Index {
                base: b(1),
                index: b(2),
            },
            K::Call {
                callee: Callee::Method("m".into()),
                recv: Some(b(1)),
                args: vec![lit(2)],
            },
            K::Convert {
                value: b(1),
                to: "T".into(),
            },
            K::TypeCase {
                value: b(1),
                arms: vec![(Ty::Bool, Pat::Wild, lit(2))],
            },
            K::Assign {
                place: b(1),
                value: b(2),
            },
            K::Let {
                name: "l".into(),
                value: Some(b(1)),
            },
            K::If {
                cond: b(1),
                then: vec![lit(2)],
                els: Some(vec![lit(3)]),
            },
            K::IfLet {
                pat: Pat::Wild,
                value: b(1),
                then: vec![lit(2)],
                els: Some(vec![lit(3)]),
            },
            K::While {
                cond: b(1),
                body: vec![lit(2)],
            },
            K::Return(Some(b(1))),
            K::Binary {
                op: "+",
                l: b(1),
                r: b(2),
            },
            K::Not(b(1)),
            K::Ref(b(1)),
            K::Deref(b(1)),
            K::Closure {
                params: Vec::new(),
                body: vec![lit(1)],
            },
            K::Panic("p".into()),
            K::Unsupported("u".into()),
        ]
        .into_iter()
        .map(Expr::new)
        .collect()
    }

    mod ty {
        use super::*;

        mod name {
            use super::*;

            #[test]
            fn name_should_be_the_name_of_a_named_type_only() {
                assert_eq!((Ty::named("A").name(), Ty::Bool.name()), (Some("A"), None));
            }
        }
    }

    mod expr {
        use super::*;

        mod local {
            use super::*;

            #[test]
            fn local_should_be_a_typed_local() {
                assert_eq!(
                    Expr::local("a", Ty::Bool),
                    Expr::typed(K::Local("a".into()), Ty::Bool)
                );
            }
        }

        mod field {
            use super::*;

            #[test]
            fn field_should_be_a_typed_field_of_the_expression() {
                assert_eq!(
                    Expr::new(K::This).field("a", Ty::Bool),
                    Expr::typed(
                        K::Field {
                            base: Expr::new(K::This).boxed(),
                            name: "a".into()
                        },
                        Ty::Bool
                    )
                );
            }
        }

        mod children_mut {
            use super::*;

            #[test]
            fn children_mut_should_be_the_expressions_within_in_order() {
                let children = every_kind()
                    .iter_mut()
                    .map(|e| {
                        e.children_mut()
                            .iter()
                            .map(|c| print::expr(c))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect::<Vec<_>>();

                assert_eq!(
                    children,
                    vec![
                        "", "", "", "", "1 2", "1", "1", "1", "", "1", "1", "1", "1 2 3", "1 2",
                        "1 2", "1", "1 2", "1 2", "1", "1 2 3", "1 2 3", "1 2", "1", "1 2", "1",
                        "1", "1", "1", "", ""
                    ]
                );
            }
        }

        mod rewrite {
            use super::*;

            #[test]
            fn rewrite_should_visit_the_children_first() {
                let mut e = Expr::new(K::Binary {
                    op: "+",
                    l: lit(1).boxed(),
                    r: lit(2).boxed(),
                });
                let mut order = Vec::new();

                e.rewrite(&mut |e| order.push(e.k.clone()));

                assert_eq!(order, [lit(1).k, lit(2).k, e.k]);
            }
        }

        mod any {
            use super::*;

            #[test]
            fn any_should_find_an_expression_within() {
                let e = Expr::new(K::Not(Expr::new(K::Return(Some(lit(1).boxed()))).boxed()));
                let is = |i| move |e: &Expr| e.k == K::Lit(Lit::Int(i));

                assert_eq!((e.any(&is(1)), e.any(&is(2))), (true, false));
            }
        }
    }
}
