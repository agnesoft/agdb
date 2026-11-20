use crate::Error;
use crate::codegen::lowered::*;
use crate::codegen::pass::Pass;
use crate::codegen::pass::dict_field;
use crate::codegen::pass::snake_case;
use crate::codegen::print;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Erases the traits whose methods take only the receiver from the classes
/// whose generics they bound. A value of such a generic becomes a
/// [record](Module::records) named after the trait, holding the results of
/// its methods, and a trait call reads its field. The record is made where a
/// builder gets a concrete type, by evaluating the impl of the type there.
///
/// The result made there must be the one of every later call, which holds
/// where values share their objects, as in TS, and each impl method is one
/// of:
/// - for the one method that may take a reference, a part of `self` that is
///   an object (a struct that is no newtype): the generic code changes it in
///   place, as the trait call could not replace it;
/// - for the methods taking `self`, `self` wrapped in variants, whose parts
///   change along.
///
/// The method either returns that result, or returns it from an `if let` on
/// a place of `self` outside of the part it returns. The variant of the
/// place must be known where the builder is made, from the value's literal
/// or from the assignment of the place before the statement making the
/// builder, with nothing in between mentioning the place's root. The payload
/// the pattern binds becomes a local if it isn't a place, and the branches
/// of the variants the value cannot have (a `panic!`) go away. A value used
/// by more than one result, and not a place, becomes a local too. A use of a
/// type's dictionary the pass cannot evaluate is an error.
///
/// A trait stays a dictionary in any of these cases:
/// - it bounds a generic of a function (a reflective one does);
/// - a method takes more than the receiver or returns a generic;
/// - more than one method takes a reference;
/// - an impl lacks a method or has one of another shape;
/// - the module assigns the result of a call of the trait as a whole;
/// - the module calls a method of an impl directly, on a named type;
/// - a generic it bounds has another bound, whose dictionary is a field;
/// - a field of the class has the generic within another type;
/// - a method that a client or a function outside of the classes can call
///   takes or returns the generic.
///
/// It runs after [`TagResults`](crate::codegen::pass::TagResults), on the
/// dictionaries.
pub struct EraseTraits;

/// The local `self` of an impl method while it is evaluated, a name no
/// local of the source can have.
const SELF: &str = "self";

/// The generics of the classes that a trait bounds, as (position, name)
/// pairs, by class.
type Bounded = BTreeMap<String, Vec<(usize, String)>>;

/// The impl methods of a trait, in the order of its signatures, by class.
type Shapes = BTreeMap<String, Vec<Shape>>;

/// An impl method with `self` as the local [`SELF`]: the result it returns,
/// after the `if let` test of the pattern on the place of `self` at the
/// path, if any.
struct Shape {
    test: Option<(Pat, Vec<String>)>,
    result: Expr,
}

impl Pass for EraseTraits {
    fn name(&self) -> &str {
        "erase_traits"
    }

    fn run(&self, m: &mut Module, errors: &mut Vec<Error>) {
        let erasable: Vec<(String, Bounded, Shapes)> = m
            .traits
            .keys()
            .filter_map(|t| erasable(m, t).map(|(b, s)| (t.clone(), b, s)))
            .collect();

        for (t, bounded, shapes) in erasable {
            erase(m, &t, &bounded, &shapes, errors);
        }
    }
}

/// The generics that `t` bounds and the shapes of its impls, if the trait
/// can be erased.
fn erasable(m: &Module, t: &str) -> Option<(Bounded, Shapes)> {
    let sigs = &m.traits[t];
    let bounds = |g: &Generic| g.bounds.iter().any(|(b, _)| b == t);
    let receiver_only = sigs.iter().all(|f| {
        f.recv.is_some() && f.params.is_empty() && f.generics.is_empty() && is_concrete(&f.ret)
    });
    let refs = sigs.iter().filter(|f| takes_ref(f)).count();
    let assigns = |e: &Expr| {
        matches!(&e.k, K::Assign { place, .. } if matches!(&place.k, K::Call {
            callee: Callee::Trait { trait_, .. },
            ..
        } if trait_ == t))
    };
    let implements = |class: &str, name: &str| {
        m.classes
            .get(class)
            .and_then(|c| c.trait_impls.get(t))
            .is_some_and(|fs| fs.iter().any(|f| f.name == name))
    };
    let direct = |e: &Expr| {
        matches!(&e.k, K::Call {
            callee: Callee::Member { class, name },
            ..
        } if implements(class, name))
    };
    let mut bodies = m.funcs().chain(&m.tests).flat_map(|f| &f.body);

    if !receiver_only
        || refs > 1
        || m.funcs().any(|f| f.generics.iter().any(bounds))
        || bodies.any(|s| s.any(&assigns) || s.any(&direct))
    {
        return None;
    }

    let shapes = m
        .classes
        .iter()
        .filter_map(|(name, c)| c.trait_impls.get(t).map(|fs| (name, fs)))
        .map(|(name, fs)| {
            let shape = |sig: &Func| {
                let f = fs.iter().find(|f| f.name == sig.name)?;
                Shape::of(f).filter(|s| s.shares(m, sig))
            };
            let shapes = sigs.iter().map(shape).collect::<Option<Vec<_>>>()?;
            Some((name.clone(), shapes))
        })
        .collect::<Option<Shapes>>()?;
    let bounded: Bounded = m
        .classes
        .iter()
        .map(|(name, c)| {
            let generics = c.generics.iter().enumerate();
            let erased = generics
                .filter(|(_, g)| bounds(g))
                .map(|(i, g)| (i, g.name.clone()));
            (name.clone(), erased.collect::<Vec<_>>())
        })
        .filter(|(_, generics)| !generics.is_empty())
        .collect();
    let kept = |class: &str, (_, g): &(usize, String)| {
        let c = &m.classes[class];
        let leaks = |ty: &Ty| strip(ty, &bounded).mentions(g);
        let holds = |ty: &Ty| match ty {
            Ty::Param(_) => true,
            Ty::Dict(d, of) => d == t || **of != Ty::Param(g.clone()),
            ty => !leaks(ty),
        };

        c.fields.iter().all(|(_, ty)| holds(ty))
            && c.methods
                .iter()
                .chain(c.trait_impls.values().flatten())
                .filter(|f| !f.private || called_outside(m, &bounded, class, &f.name))
                .all(|f| !leaks(&f.ret) && f.params.iter().all(|(_, ty)| !leaks(ty)))
    };
    let erasable = !bounded.is_empty()
        && bounded
            .iter()
            .all(|(class, generics)| generics.iter().all(|g| kept(class, g)));

    erasable.then_some((bounded, shapes))
}

fn takes_ref(f: &Func) -> bool {
    matches!(f.recv, Some(Recv::Ref | Recv::RefMut))
}

impl Shape {
    /// The shape of the impl method `f`: a `return`, or an `if let` on a
    /// place of `self` that returns from its match.
    fn of(f: &Func) -> Option<Shape> {
        let mut body = f.body.clone();
        rewrite_body(&mut body, &mut |e| {
            if matches!(e.k, K::This) {
                e.k = K::Local(SELF.into());
            }
        });

        match body.as_slice() {
            [
                Expr {
                    k: K::Return(Some(result)),
                    ..
                },
            ] => Some(Shape {
                test: None,
                result: (**result).clone(),
            }),
            [
                Expr {
                    k: K::IfLet {
                        pat, value, then, ..
                    },
                    ..
                },
            ] => match (then.as_slice(), place(value)) {
                (
                    [
                        Expr {
                            k: K::Return(Some(result)),
                            ..
                        },
                    ],
                    Some((_, path)),
                ) => Some(Shape {
                    test: Some((pat.clone(), path)),
                    result: (**result).clone(),
                }),
                _ => None,
            },
            _ => None,
        }
    }

    /// Whether the result made where the builder is made is the one of
    /// every call of `sig`.
    fn shares(&self, m: &Module, sig: &Func) -> bool {
        if !takes_ref(sig) {
            return wraps(&self.result);
        }

        let object = self
            .result
            .ty
            .name()
            .and_then(|n| m.classes.get(n))
            .is_some_and(|c| !matches!(c.fields.as_slice(), [(f, _)] if f == "0"));
        // A root other than `self` is a binding of the pattern, within the
        // payload of the tested place.
        let part = match place(&self.result) {
            Some((Some(root), path)) if root == SELF => self
                .test
                .as_ref()
                .is_none_or(|(_, tested)| !tested.starts_with(&path)),
            Some(_) => true,
            None => false,
        };

        object && part
    }
}

/// Whether `e` is `self` wrapped in variants.
fn wraps(e: &Expr) -> bool {
    match &e.k {
        K::Variant {
            payload: Some(p), ..
        } => wraps(p),
        _ => is_self(e),
    }
}

/// Whether a function outside of the `bounded` classes calls the method
/// `name` of `class`, with a concrete type for its generic.
fn called_outside(m: &Module, bounded: &Bounded, class: &str, name: &str) -> bool {
    let calls = |e: &Expr| match &e.k {
        K::Call {
            callee: Callee::Member { class: c, name: n } | Callee::Static { class: c, name: n },
            ..
        } => c == class && n == name,
        _ => false,
    };
    let outside = m
        .classes
        .iter()
        .filter(|(n, _)| !bounded.contains_key(*n))
        .flat_map(|(_, c)| c.methods.iter().chain(c.trait_impls.values().flatten()))
        .chain(m.enums.values().flat_map(|e| &e.methods))
        .chain(&m.functions)
        .chain(&m.tests);

    outside.flat_map(|f| &f.body).any(|s| s.any(&calls))
}

/// Whether the type has no generic and no `Self`.
fn is_concrete(ty: &Ty) -> bool {
    let concrete = std::cell::Cell::new(true);
    ty.map(&|t| {
        if matches!(t, Ty::Param(_) | Ty::SelfTy) {
            concrete.set(false);
        }

        t.clone()
    });
    concrete.get()
}

/// The type without the arguments of the generics in `bounded`.
fn strip(ty: &Ty, bounded: &Bounded) -> Ty {
    ty.map(&|t| match t {
        Ty::Named(name, args) if let Some(generics) = bounded.get(name) => {
            let kept = args
                .iter()
                .enumerate()
                .filter(|(i, _)| !generics.iter().any(|(p, _)| p == i))
                .map(|(_, a)| a.clone());
            Ty::Named(name.clone(), kept.collect())
        }
        t => t.clone(),
    })
}

fn erase(m: &mut Module, t: &str, bounded: &Bounded, shapes: &Shapes, errors: &mut Vec<Error>) {
    let sigs = m.traits.remove(t).unwrap_or_default();
    m.classes.values_mut().for_each(|c| {
        c.trait_impls.remove(t);
    });
    let values = bounded
        .iter()
        .map(|(class, generics)| {
            let fields = &m.classes[class].fields;
            let of = |g: &String| {
                let values = fields.iter().filter(|(_, ty)| *ty == Ty::Param(g.clone()));
                (
                    dict_field(g, t),
                    values.map(|(n, _)| n.clone()).collect::<Vec<_>>(),
                )
            };
            (class.clone(), generics.iter().map(|(_, g)| of(g)).collect())
        })
        .collect();
    let sites = Sites {
        t,
        sigs: &sigs,
        shapes,
        values,
    };

    m.for_each_func(&mut |f, class| {
        if let Err(message) = sites.func(f) {
            errors.push(error(f, class, message));
        }
    });
    m.for_each_func(&mut |f, class| {
        let mut stray = BTreeSet::new();
        rewrite_body(&mut f.body, &mut |e| sites.generic(e, &mut stray));
        errors.extend(stray.into_iter().map(|of| {
            let message = format!("uses the `{t}` of `{of}` where it cannot be evaluated");
            error(f, class, message)
        }));
    });

    for (name, generics) in bounded {
        let erased = |p: &String| generics.iter().any(|(_, g)| g == p);
        let record = |ty: &Ty| match ty {
            Ty::Param(p) if erased(p) => Ty::named(t),
            ty => ty.clone(),
        };
        let c = m
            .classes
            .get_mut(name)
            .expect("bounded classes are classes");
        c.fields
            .retain(|(_, ty)| !matches!(ty, Ty::Dict(d, _) if d == t));
        c.fields
            .iter_mut()
            .for_each(|(_, ty)| *ty = ty.map(&record));
        c.methods
            .iter_mut()
            .chain(c.trait_impls.values_mut().flatten())
            .for_each(|f| f.map_types(&record));
        c.generics.retain(|g| !erased(&g.name));
    }

    m.map_types(&|ty| strip(ty, bounded));
    m.dicts.retain(|(d, _)| d != t);
    m.classes.insert(
        t.to_owned(),
        Class {
            name: t.to_owned(),
            generics: Vec::new(),
            fields: sigs
                .iter()
                .map(|s| (s.name.clone(), s.ret.clone()))
                .collect(),
            methods: Vec::new(),
            trait_impls: BTreeMap::new(),
        },
    );
    m.records.insert(t.to_owned());
}

fn error(f: &Func, class: &str, message: String) -> Error {
    let item = if class.is_empty() {
        f.name.clone()
    } else {
        format!("{class}::{}", f.name)
    };
    let (path, line) =
        f.at.as_ref()
            .map(|at| (at.path.clone(), at.line))
            .unwrap_or_default();
    Error::Lowering {
        path,
        line,
        item,
        message,
    }
}

/// What the sites making the builders of an erased trait evaluate.
struct Sites<'a> {
    t: &'a str,
    sigs: &'a [Func],
    shapes: &'a Shapes,
    /// The dictionary field and the fields holding the values of its
    /// generic, by class.
    values: BTreeMap<String, Vec<(String, Vec<String>)>>,
}

/// The locals a function has and the ones the evaluation adds, in the
/// order of the top-level statements each goes before.
struct Locals {
    taken: BTreeSet<String>,
    added: Vec<(usize, Expr)>,
}

impl Locals {
    /// `value` bound to a new local named after `name` before the
    /// statement `at`, and replaced by it.
    fn bind(&mut self, value: &mut Expr, name: &str, at: usize) -> Expr {
        let mut fresh = name.to_owned();
        let mut i = 0;

        while self.taken.contains(&fresh) {
            fresh = format!("{name}{i}");
            i += 1;
        }

        self.taken.insert(fresh.clone());
        let local = Expr::local(&fresh, value.ty.clone());
        let value = std::mem::replace(value, local.clone());
        self.added.push((
            at,
            Expr::typed(
                K::Let {
                    name: fresh,
                    value: Some(value.boxed()),
                },
                Ty::Unit,
            ),
        ));
        local
    }
}

impl Sites<'_> {
    /// Makes the records of the builders the top-level statements of `f`
    /// make, outside of their blocks.
    fn func(&self, f: &mut Func) -> Result<(), String> {
        let mut taken: BTreeSet<String> = f.params.iter().map(|(n, _)| n.clone()).collect();
        let mut body = f.body.clone();
        rewrite_body(&mut body, &mut |e| {
            if let K::Local(name) | K::Let { name, .. } = &e.k {
                taken.insert(name.clone());
            }
        });
        let mut locals = Locals {
            taken,
            added: Vec::new(),
        };

        for at in 0..f.body.len() {
            let (before, rest) = f.body.split_at_mut(at);
            self.visit(&mut rest[0], before, at, &mut locals)?;
        }

        for (at, s) in locals.added.into_iter().rev() {
            f.body.insert(at, s);
        }

        Ok(())
    }

    fn visit(
        &self,
        e: &mut Expr,
        before: &mut [Expr],
        at: usize,
        locals: &mut Locals,
    ) -> Result<(), String> {
        if let K::Construct { class, fields } = &mut e.k
            && let Some(values) = self.values.get(class.as_str())
        {
            for (dict, values) in values {
                let impl_ = fields
                    .iter()
                    .find(|(n, _)| n == dict)
                    .and_then(|(_, d)| self.concrete(d))
                    .and_then(|of| self.shapes.get_key_value(of));

                if let Some((of, shapes)) = impl_ {
                    fields.retain(|(n, _)| n != dict);

                    for (_, v) in fields.iter_mut().filter(|(n, _)| values.contains(n)) {
                        let value = std::mem::replace(v, Expr::new(K::This));
                        *v = self.record(of, shapes, value, before, at, locals)?;
                    }
                }
            }
        }

        let children = match &mut e.k {
            K::If { cond, .. } => vec![cond.as_mut()],
            K::IfLet { value, .. } | K::Match { value, .. } => vec![value.as_mut()],
            K::While { .. } | K::Closure { .. } => Vec::new(),
            _ => e.children_mut(),
        };

        children
            .into_iter()
            .try_for_each(|c| self.visit(c, before, at, locals))
    }

    /// The type of the dictionary `d` of the trait, if it is a named one.
    fn concrete<'d>(&self, d: &'d Expr) -> Option<&'d str> {
        match &d.ty {
            Ty::Dict(t, of) if t == self.t => of.name(),
            _ => None,
        }
    }

    /// The record of the methods of the impl of `of` evaluated on `v`.
    fn record(
        &self,
        of: &str,
        shapes: &[Shape],
        mut v: Expr,
        before: &mut [Expr],
        at: usize,
        locals: &mut Locals,
    ) -> Result<Expr, String> {
        let mut results = Vec::new();

        for (sig, shape) in self.sigs.iter().zip(shapes) {
            let result = evaluate(shape, &mut v, before, at, locals)
                .map_err(|why| format!("cannot evaluate `{of}::{}`: {why}", sig.name))?;
            results.push((sig.name.clone(), result));
        }

        let uses: usize = results
            .iter()
            .map(|(_, r)| {
                let mut n = 0;
                r.clone().rewrite(&mut |e| n += usize::from(is_self(e)));
                n
            })
            .sum();

        if !v.is_place() && uses != 1 {
            let name = snake_case(v.ty.name().unwrap_or("value"));
            locals.bind(&mut v, &name, at);
        }

        for (_, r) in &mut results {
            r.rewrite(&mut |e| {
                if is_self(e) {
                    *e = v.clone();
                }
            });
        }

        Ok(Expr::typed(
            K::Construct {
                class: self.t.to_owned(),
                fields: results,
            },
            Ty::named(self.t),
        ))
    }

    /// The trait call and the dictionary fields of `e` erased: a call
    /// reads the field of the record and a dictionary goes away. A named
    /// type's dictionary left is noted in `stray`.
    fn generic(&self, e: &mut Expr, stray: &mut BTreeSet<String>) {
        let mut note = |d: &Expr| {
            if let Some(of) = self.concrete(d) {
                stray.insert(of.to_owned());
            }
        };

        match &mut e.k {
            K::Construct { fields, .. } => fields.retain(|(_, v)| {
                note(v);
                !matches!(&v.ty, Ty::Dict(t, _) if t == self.t)
            }),
            K::Call {
                callee: Callee::Trait { trait_, name },
                recv: Some(dict),
                args,
            } if trait_ == self.t && args.len() == 1 => {
                note(dict);
                let base = args.remove(0).boxed();
                e.k = K::Field {
                    base,
                    name: name.clone(),
                };
            }
            _ => {}
        }
    }
}

fn is_self(e: &Expr) -> bool {
    matches!(&e.k, K::Local(n) if n == SELF)
}

/// The result of the impl method of the shape for `v`.
fn evaluate(
    shape: &Shape,
    v: &mut Expr,
    before: &mut [Expr],
    at: usize,
    locals: &mut Locals,
) -> Result<Expr, String> {
    let mut result = shape.result.clone();
    let Some((pat, path)) = &shape.test else {
        return Ok(result);
    };
    let (known, at) = known(v, path, before, at).ok_or_else(|| {
        let place: Vec<&str> = std::iter::once(SELF)
            .chain(path.iter().map(String::as_str))
            .collect();
        format!("the value of `{}` is not known", place.join("."))
    })?;
    let mut binds = Vec::new();
    bind(pat, known, at, &mut binds, locals)?;
    result.rewrite(&mut |e| {
        if let K::Local(n) = &e.k
            && let Some((_, bound)) = binds.iter().find(|(b, _)| b == n)
        {
            *e = bound.clone();
        }
    });
    Ok(result)
}

/// The root (`None` for `self`) and the fields of a place.
fn place(e: &Expr) -> Option<(Option<String>, Vec<String>)> {
    match &e.k {
        K::This => Some((None, Vec::new())),
        K::Local(name) => Some((Some(name.clone()), Vec::new())),
        K::Field { base, name } => place(base).map(|(root, mut fields)| {
            fields.push(name.clone());
            (root, fields)
        }),
        _ => None,
    }
}

/// The expression giving the value at `path` within `v`, and the
/// top-level statement holding it: the literal `v` at `at`, or the last
/// statement in `before` mentioning the root of the place `v`, if that
/// statement assigns the place.
fn known<'a>(
    v: &'a mut Expr,
    path: &[String],
    before: &'a mut [Expr],
    at: usize,
) -> Option<(&'a mut Expr, usize)> {
    if let Some((root, mut fields)) = place(v) {
        fields.extend(path.iter().cloned());
        let mentions = |s: &Expr| {
            s.any(&|e| match (&e.k, &root) {
                (K::This, None) => true,
                (K::Local(n), Some(r)) => n == r,
                _ => false,
            })
        };
        let j = before.iter().rposition(mentions)?;

        return match &mut before[j].k {
            K::Assign { place: p, value }
                if place(p).is_some_and(|(r, f)| r == root && f == fields) =>
            {
                Some((value, j))
            }
            _ => None,
        };
    }

    let value = path.iter().try_fold(v, |e, name| match &mut e.k {
        K::Construct { fields, .. } => fields.iter_mut().find(|(n, _)| n == name).map(|(_, e)| e),
        _ => None,
    })?;
    Some((value, at))
}

/// Matches the pattern against the known value `e`, binding the payloads it
/// binds, made locals before the statement `at` unless they are places.
fn bind(
    pat: &Pat,
    e: &mut Expr,
    at: usize,
    binds: &mut Vec<(String, Expr)>,
    locals: &mut Locals,
) -> Result<(), String> {
    let pattern = print::pat(pat);

    match (pat, &mut e.k) {
        (Pat::Wild, _) => Ok(()),
        (Pat::Bind(name), _) => {
            let value = if e.is_place() {
                e.clone()
            } else {
                locals.bind(e, name, at)
            };
            binds.push((name.clone(), value));
            Ok(())
        }
        (
            Pat::Variant { en, var, inner },
            K::Variant {
                en: e_en,
                var: e_var,
                payload,
            },
        ) if en == e_en && var == e_var => match (inner.as_slice(), payload) {
            ([], None) => Ok(()),
            ([p], Some(payload)) => bind(p, payload, at, binds, locals),
            _ => Err(format!("`{pattern}` binds more than one payload")),
        },
        (Pat::Variant { .. }, K::Variant { .. }) => {
            Err(format!("the value does not match `{pattern}`"))
        }
        _ => Err(format!(
            "`{pattern}` is not matched against a known variant"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::pass::EraseOwnership;
    use crate::codegen::pass::TraitDicts;
    use crate::codegen::test_utilities::resolved;
    use crate::codegen::test_utilities::strings;

    /// The module of `source` with the dictionaries of `traits`, before
    /// the erasure.
    fn dicts(source: &str, traits: &[&str]) -> Module {
        let mut m = resolved(source, traits, &[]);
        TraitDicts.run(&mut m, &mut Vec::new());
        EraseOwnership.run(&mut m, &mut Vec::new());
        m
    }

    mod erase_traits {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_erase_traits() {
                    assert_eq!(EraseTraits.name(), "erase_traits");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_turn_the_values_of_a_trait_into_records_evaluated_where_the_builders_are_made()
                 {
                    let mut m = dicts(
                        r#"
                        pub struct Root {
                            pub q: Qa,
                        }
                        pub struct Qa {
                            pub e: E,
                            pub n: u8,
                        }
                        pub struct Qb {
                            pub o: Option<u8>,
                            pub i: Inner,
                        }
                        pub enum E {
                            A(Inner),
                            B,
                        }
                        pub struct Inner {
                            pub n: u8,
                        }
                        pub struct Qc {
                            pub e: E,
                            pub i: Inner,
                        }
                        pub enum Wire {
                            Qa(Qa),
                            Qb(Qb),
                            Qc(Qc),
                            Inner(Inner),
                        }
                        pub trait Tr {
                            fn inner(&mut self) -> &mut Inner;
                            fn into_wire(self) -> Wire;
                        }
                        pub struct G<T: Tr>(pub T);
                        pub struct H<T: Tr, U> {
                            pub g: G<T>,
                            pub n: U,
                        }
                        impl Tr for Qa {
                            fn inner(&mut self) -> &mut Inner {
                                if let E::A(inner) = &mut self.e {
                                    inner
                                } else {
                                    panic!("not A")
                                }
                            }
                            fn into_wire(self) -> Wire {
                                Wire::Qa(self)
                            }
                        }
                        impl Tr for Qb {
                            fn inner(&mut self) -> &mut Inner {
                                if let Some(_) = &self.o {
                                    &mut self.i
                                } else {
                                    panic!("none")
                                }
                            }
                            fn into_wire(self) -> Wire {
                                Wire::Qb(self)
                            }
                        }
                        impl Tr for Qc {
                            fn inner(&mut self) -> &mut Inner {
                                if let E::B = &self.e {
                                    &mut self.i
                                } else {
                                    panic!("not B")
                                }
                            }
                            fn into_wire(self) -> Wire {
                                Wire::Qc(self)
                            }
                        }
                        impl Qc {
                            pub fn own(mut self) -> G<Qc> {
                                self.e = E::B;
                                G(self)
                            }
                        }
                        impl Tr for Inner {
                            fn inner(&mut self) -> &mut Inner {
                                self
                            }
                            fn into_wire(self) -> Wire {
                                Wire::Inner(self)
                            }
                        }
                        impl Root {
                            pub fn literal(&self) -> G<Qa> {
                                G(Qa { e: E::A(Inner { n: 1 }), n: 0 })
                            }
                            pub fn assigned(mut self) -> G<Qa> {
                                self.q.n = 1;
                                self.q.e = E::A(Inner { n: 2 });
                                G(self.q)
                            }
                            pub fn bound(&self, inner: Inner) -> G<Qa> {
                                let g = G(Qa { e: E::A(inner), n: 0 });
                                g
                            }
                            pub fn own(&self) -> G<Qc> {
                                Qc { e: E::A(Inner { n: 6 }), i: Inner { n: 7 } }.own()
                            }
                            pub fn local(&self) -> G<Qa> {
                                let mut q = Qa { e: E::B, n: 0 };
                                q.e = E::A(Inner { n: 8 });
                                G(q)
                            }
                            pub fn two(&self) -> G<Inner> {
                                let a = G(Qa { e: E::A(Inner { n: 9 }), n: 0 });
                                G(Inner { n: 10 })
                            }
                            pub fn taken(&self) -> G<Qa> {
                                let inner = 1;
                                G(Qa { e: E::A(Inner { n: 5 }), n: inner })
                            }
                            pub fn identity(&self) -> G<Inner> {
                                G(Inner { n: 3 })
                            }
                            pub fn some(&self) -> G<Qb> {
                                G(Qb { o: Some(1), i: Inner { n: 4 } })
                            }
                            pub fn blocks(&self, b: bool) -> u8 {
                                let mut n = 0;
                                if b {
                                    n = 1;
                                }
                                while n > 5 {
                                    n = n - 1;
                                }
                                match self.q.n {
                                    0 => n = 2,
                                    _ => n = 3,
                                }
                                let f = |x: u8| x;
                                n
                            }
                        }
                        impl<T: Tr> G<T> {
                            pub fn n(mut self, n: u8) -> G<T> {
                                self.0.inner().n = n;
                                self
                            }
                            pub fn h(self) -> H<T, u8> {
                                H { n: 0, g: self }
                            }
                            pub fn wire(self) -> Wire {
                                self.0.into_wire()
                            }
                            pub fn again(self) -> G<T> {
                                G::wrap(self.0)
                            }
                            fn wrap(t: T) -> G<T> {
                                G(t)
                            }
                        }
                        "#,
                        &["Tr"],
                    );
                    let mut errors = Vec::new();

                    EraseTraits.run(&mut m, &mut errors);

                    assert_eq!(strings(&errors), Vec::<String>::new());
                    assert_eq!(
                        print::module(&m, "erase_traits"),
                        "// ===== after erase_traits\n// Qc\nfn own(self) -> G {\n    self.e = E::B;\n    return G(Tr { inner: self.i, into_wire: Wire::Qc(self) });\n}\n\n// G\nfn n(self, n: u8) -> G {\n    self.0.inner.n = n;\n    return self;\n}\n\n// G\nfn h(self) -> H<u8> {\n    return H { n: 0, g: self };\n}\n\n// G\nfn wire(self) -> Wire {\n    return self.0.into_wire;\n}\n\n// G\nfn again(self) -> G {\n    return G::wrap(self.0);\n}\n\n// G\nfn wrap(t: Tr) -> G {\n    return G(t);\n}\n\n// Root\nfn literal(self) -> G {\n    let inner: Inner = Inner { n: 1 };\n    return G(Tr { inner: inner, into_wire: Wire::Qa(Qa { e: E::A(inner), n: 0 }) });\n}\n\n// Root\nfn assigned(self) -> G {\n    self.q.n = 1;\n    let inner: Inner = Inner { n: 2 };\n    self.q.e = E::A(inner);\n    return G(Tr { inner: inner, into_wire: Wire::Qa(self.q) });\n}\n\n// Root\nfn bound(self, inner: Inner) -> G {\n    let g: G = G(Tr { inner: inner, into_wire: Wire::Qa(Qa { e: E::A(inner), n: 0 }) });\n    return g;\n}\n\n// Root\nfn own(self) -> G {\n    return Qc { e: E::A(Inner { n: 6 }), i: Inner { n: 7 } }.own();\n}\n\n// Root\nfn local(self) -> G {\n    let q: Qa = Qa { e: E::B, n: 0 };\n    let inner: Inner = Inner { n: 8 };\n    q.e = E::A(inner);\n    return G(Tr { inner: inner, into_wire: Wire::Qa(q) });\n}\n\n// Root\nfn two(self) -> G {\n    let inner: Inner = Inner { n: 9 };\n    let a: G = G(Tr { inner: inner, into_wire: Wire::Qa(Qa { e: E::A(inner), n: 0 }) });\n    let inner0: Inner = Inner { n: 10 };\n    return G(Tr { inner: inner0, into_wire: Wire::Inner(inner0) });\n}\n\n// Root\nfn taken(self) -> G {\n    let inner: i32 = 1;\n    let inner0: Inner = Inner { n: 5 };\n    return G(Tr { inner: inner0, into_wire: Wire::Qa(Qa { e: E::A(inner0), n: inner }) });\n}\n\n// Root\nfn identity(self) -> G {\n    let inner: Inner = Inner { n: 3 };\n    return G(Tr { inner: inner, into_wire: Wire::Inner(inner) });\n}\n\n// Root\nfn some(self) -> G {\n    let qb: Qb = Qb { o: Option::Some(1), i: Inner { n: 4 } };\n    return G(Tr { inner: qb.i, into_wire: Wire::Qb(qb) });\n}\n\n// Root\nfn blocks(self, b: bool) -> u8 {\n    let n: i32 = 0;\n    if b {\n        n = 1;\n    };\n    while (n > 5) {\n        n = (n - 1);\n    };\n    match self.q.n {\n        0 => {\n            n = 2;\n        },\n        _ => {\n            n = 3;\n        },\n    };\n    let f: ? = |x| {\n    x;\n};\n    return n;\n}\n\n"
                    );
                    assert_eq!(
                        m.classes["Tr"].fields,
                        [
                            ("inner".to_owned(), Ty::named("Inner")),
                            ("into_wire".to_owned(), Ty::named("Wire"))
                        ]
                    );
                    assert_eq!(
                        m.classes["H"].fields,
                        [
                            ("g".to_owned(), Ty::named("G")),
                            ("n".to_owned(), Ty::Param("U".into()))
                        ]
                    );
                    assert_eq!(m.classes["G"].generics, []);
                    assert_eq!(m.records, BTreeSet::from(["Tr".to_owned()]));
                    assert_eq!(m.traits, BTreeMap::new());
                    assert_eq!(m.dicts, BTreeSet::new());
                }

                #[test]
                fn run_should_keep_the_dictionaries_of_the_traits_it_cannot_erase() {
                    let mut m = dicts(
                        r#"
                        pub struct Root {
                            pub q: Qa,
                            pub p: P<Qa>,
                            pub m: M<Qa>,
                            pub two: Two<Qa>,
                            pub l: L<Qa>,
                            pub r: R<Qa>,
                            pub s: S<Qa>,
                            pub rf: Rf<Qa>,
                            pub pr: Pr<Qa>,
                            pub nt: Nt<Qa>,
                            pub wi: Wi<Qa>,
                            pub te: Te<Qa>,
                            pub un: Un<Qa>,
                            pub de: De<Qa>,
                            pub st: St<Qa>,
                            pub th: Th<Qa>,
                            pub a: A<Qa>,
                            pub ca: Ca<Qa>,
                            pub rl: Rl<Qa>,
                        }
                        pub struct Qa {
                            pub n: u8,
                            pub i: Inner,
                            pub w: Wrap,
                            pub c: Qc,
                        }
                        pub struct Qc {
                            pub o: Option<u8>,
                        }
                        pub struct Inner {
                            pub n: u8,
                        }
                        pub struct Wrap(pub Inner);
                        pub trait Refl {
                            fn refl(&self) -> u8;
                        }
                        pub trait WithParam {
                            fn add(&self, n: u8) -> u8;
                        }
                        pub trait Me {
                            fn me(&self) -> Self;
                        }
                        pub trait First {
                            fn first(&self) -> u8;
                        }
                        pub trait Second {
                            fn second(&self) -> u8;
                        }
                        pub trait InList {
                            fn in_list(&self) -> u8;
                        }
                        pub trait Returned {
                            fn returned(&self) -> u8;
                        }
                        pub trait Made {
                            fn made(&self) -> u8;
                        }
                        pub trait Refs {
                            fn a(&self) -> &Inner;
                            fn b(&mut self) -> &mut Inner;
                        }
                        pub trait Primitive {
                            fn prim(&self) -> &u8;
                        }
                        pub trait Newtype {
                            fn wrap(&mut self) -> &mut Wrap;
                        }
                        pub trait Within {
                            fn within(&mut self) -> &mut Qc;
                        }
                        pub trait Temporary {
                            fn temp(&self) -> &Inner;
                        }
                        pub trait Unwrapped {
                            fn unwrapped(self) -> u8;
                        }
                        pub trait Defaulted {
                            fn defaulted(&mut self) -> &mut Inner;
                            fn by_default(self) -> u8 {
                                0
                            }
                        }
                        pub trait Statements {
                            fn statements(&mut self) -> &mut Inner;
                        }
                        pub trait Then {
                            fn then(&mut self) -> &mut Inner;
                        }
                        pub trait Assigned {
                            fn assigned(&mut self) -> &mut Inner;
                        }
                        pub trait Called {
                            fn called(&mut self) -> &mut Inner;
                        }
                        pub struct P<T: WithParam>(pub T);
                        pub struct M<T: Me>(pub T);
                        pub struct Two<T: First + Second>(pub T);
                        pub struct L<T: InList> {
                            pub items: Vec<T>,
                        }
                        pub struct R<T: Returned>(pub T);
                        pub struct S<T: Made>(pub T);
                        pub struct Rf<T: Refs>(pub T);
                        pub struct Pr<T: Primitive>(pub T);
                        pub struct Nt<T: Newtype>(pub T);
                        pub struct Wi<T: Within>(pub T);
                        pub struct Te<T: Temporary>(pub T);
                        pub struct Un<T: Unwrapped>(pub T);
                        pub struct De<T: Defaulted>(pub T);
                        pub struct St<T: Statements>(pub T);
                        pub struct Th<T: Then>(pub T);
                        pub struct A<T: Assigned>(pub T);
                        pub struct Ca<T: Called>(pub T);
                        pub struct Rl<T: Refl>(pub T);
                        impl Root {
                            pub fn refl<T: Refl>(&self, t: &T) -> u8 {
                                t.refl()
                            }
                            pub fn called(mut self) -> u8 {
                                self.q.called().n
                            }
                        }
                        impl<T: Returned> R<T> {
                            pub fn get(self) -> T {
                                self.0
                            }
                        }
                        impl<T: Made> S<T> {
                            fn make(t: T) -> S<T> {
                                S(t)
                            }
                        }
                        impl<T: Assigned> A<T> {
                            pub fn reset(mut self) -> A<T> {
                                *self.0.assigned() = Inner { n: 0 };
                                self
                            }
                        }
                        impl Primitive for Qa {
                            fn prim(&self) -> &u8 {
                                &self.n
                            }
                        }
                        impl Newtype for Qa {
                            fn wrap(&mut self) -> &mut Wrap {
                                &mut self.w
                            }
                        }
                        impl Within for Qa {
                            fn within(&mut self) -> &mut Qc {
                                if let Some(_) = &self.c.o {
                                    &mut self.c
                                } else {
                                    panic!("none")
                                }
                            }
                        }
                        impl Temporary for Qa {
                            fn temp(&self) -> &Inner {
                                &Inner { n: 1 }
                            }
                        }
                        impl Unwrapped for Qa {
                            fn unwrapped(self) -> u8 {
                                self.n
                            }
                        }
                        impl Defaulted for Qa {
                            fn defaulted(&mut self) -> &mut Inner {
                                &mut self.i
                            }
                        }
                        impl Statements for Qa {
                            fn statements(&mut self) -> &mut Inner {
                                let i = &mut self.i;
                                i
                            }
                        }
                        impl Then for Qa {
                            fn then(&mut self) -> &mut Inner {
                                if let Some(_) = &self.c.o {
                                    self.n = 1;
                                    &mut self.i
                                } else {
                                    panic!("none")
                                }
                            }
                        }
                        impl Assigned for Qa {
                            fn assigned(&mut self) -> &mut Inner {
                                &mut self.i
                            }
                        }
                        impl Called for Qa {
                            fn called(&mut self) -> &mut Inner {
                                &mut self.i
                            }
                        }
                        "#,
                        &[
                            "Refl",
                            "WithParam",
                            "Me",
                            "First",
                            "Second",
                            "InList",
                            "Returned",
                            "Made",
                            "Refs",
                            "Primitive",
                            "Newtype",
                            "Within",
                            "Temporary",
                            "Unwrapped",
                            "Defaulted",
                            "Statements",
                            "Then",
                            "Assigned",
                            "Called",
                        ],
                    );
                    let mut f = Func::new("outside", Vec::new(), Ty::Unit, Vec::new());
                    f.body.push(Expr::new(K::Call {
                        callee: Callee::Static {
                            class: "S".into(),
                            name: "make".into(),
                        },
                        recv: None,
                        args: vec![Expr::typed(
                            K::Construct {
                                class: "Qa".into(),
                                fields: Vec::new(),
                            },
                            Ty::named("Qa"),
                        )],
                    }));
                    m.functions.push(f);
                    let before = m.clone();

                    EraseTraits.run(&mut m, &mut Vec::new());

                    assert_eq!(m, before);
                }

                #[test]
                fn run_should_report_the_uses_of_the_dictionaries_it_cannot_evaluate() {
                    let mut m = dicts(
                        r#"
                        pub struct Root {
                            pub q: Qa,
                        }
                        pub struct Qa {
                            pub e: E,
                            pub n: u8,
                        }
                        pub struct Qb {
                            pub p: Pair,
                        }
                        pub enum E {
                            A(Inner),
                            B,
                        }
                        pub enum Pair {
                            P(Inner, u8),
                        }
                        pub struct Inner {
                            pub n: u8,
                        }
                        pub enum Wire {
                            Qa(Qa),
                            Qb(Qb),
                        }
                        pub trait Tr {
                            fn inner(&mut self) -> &mut Inner;
                            fn into_wire(self) -> Wire;
                        }
                        pub struct G<T: Tr>(pub T);
                        impl Tr for Qa {
                            fn inner(&mut self) -> &mut Inner {
                                if let E::A(inner) = &mut self.e {
                                    inner
                                } else {
                                    panic!("not A")
                                }
                            }
                            fn into_wire(self) -> Wire {
                                Wire::Qa(self)
                            }
                        }
                        impl Tr for Qb {
                            fn inner(&mut self) -> &mut Inner {
                                if let Pair::P(a, _) = &mut self.p {
                                    a
                                } else {
                                    panic!("not P")
                                }
                            }
                            fn into_wire(self) -> Wire {
                                Wire::Qb(self)
                            }
                        }
                        impl Root {
                            pub fn param(&self, q: Qa) -> G<Qa> {
                                G(q)
                            }
                            pub fn read(mut self) -> G<Qa> {
                                let n = self.q.n;
                                G(self.q)
                            }
                            pub fn call(&self) -> G<Qa> {
                                G(Qa::make())
                            }
                            pub fn other(&self) -> G<Qa> {
                                G(Qa { e: E::B, n: 0 })
                            }
                            pub fn unknown(mut self) -> G<Qa> {
                                self.q.e = E::make();
                                G(self.q)
                            }
                            pub fn pair(&self) -> G<Qb> {
                                G(Qb { p: Pair::P(Inner { n: 1 }, 2) })
                            }
                            pub fn nested(&self, b: bool) -> G<Qa> {
                                if b {
                                    return G(Qa { e: E::A(Inner { n: 1 }), n: 0 });
                                }
                                G(Qa { e: E::A(Inner { n: 2 }), n: 0 })
                            }
                        }
                        impl Qa {
                            pub fn make() -> Qa {
                                Qa { e: E::B, n: 0 }
                            }
                        }
                        impl E {
                            pub fn make() -> E {
                                E::B
                            }
                        }
                        impl<T: Tr> G<T> {
                            pub fn wire(self) -> Wire {
                                self.0.into_wire()
                            }
                        }
                        "#,
                        &["Tr"],
                    );
                    let other = m.classes["Root"].methods.iter().find(|f| f.name == "other");
                    let free = other.cloned().map(|f| Func {
                        name: "free".into(),
                        ..f
                    });
                    m.functions.extend(free);
                    let mut errors = Vec::new();

                    EraseTraits.run(&mut m, &mut errors);

                    assert_eq!(
                        strings(&errors),
                        [
                            "/c/src/lib.rs:56: `Root::param`: cannot evaluate `Qa::inner`: the value of `self.e` is not known",
                            "/c/src/lib.rs:59: `Root::read`: cannot evaluate `Qa::inner`: the value of `self.e` is not known",
                            "/c/src/lib.rs:63: `Root::call`: cannot evaluate `Qa::inner`: the value of `self.e` is not known",
                            "/c/src/lib.rs:66: `Root::other`: cannot evaluate `Qa::inner`: the value does not match `E::A(inner)`",
                            "/c/src/lib.rs:69: `Root::unknown`: cannot evaluate `Qa::inner`: `E::A(inner)` is not matched against a known variant",
                            "/c/src/lib.rs:73: `Root::pair`: cannot evaluate `Qb::inner`: `Pair::P(a, _)` binds more than one payload",
                            "/c/src/lib.rs:66: `free`: cannot evaluate `Qa::inner`: the value does not match `E::A(inner)`",
                            "/c/src/lib.rs:76: `Root::nested`: uses the `Tr` of `Qa` where it cannot be evaluated"
                        ]
                    );
                }
            }
        }
    }
}
