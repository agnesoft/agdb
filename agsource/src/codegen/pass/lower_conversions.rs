use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;
use crate::codegen::print;
use std::collections::BTreeSet;

/// `G: Into<X>` generics become `Into(X)` parameters. A `.into()` is
/// dispatched statically where the type of the value is known and calls the
/// normalizer of the target (see [`normalizer`]) otherwise. The normalizers
/// are generated from [`Module::conversions`] as functions of the module.
pub struct LowerConversions;

/// The name of the function converting an `Into(to)` into `to`.
pub fn normalizer(to: &str) -> String {
    format!("into_{to}")
}

impl Pass for LowerConversions {
    fn name(&self) -> &str {
        "lower_conversions"
    }

    fn run(&self, m: &mut Module, errors: &mut Vec<Error>) {
        m.for_each_func(&mut |f, _| lower_generics(f));

        let facts = select(std::mem::take(&mut m.conversions));
        let mut needed = BTreeSet::new();
        let mut used = BTreeSet::new();
        m.for_each_func(&mut |f, _| {
            rewrite_body(&mut f.body, &mut |e| {
                dispatch(e, &facts, &mut needed, &mut used)
            });
        });

        let mut done = BTreeSet::new();

        while let Some(to) = needed.difference(&done).next().cloned() {
            done.insert(to.clone());
            let ty = Ty::named(&to);
            let mut arms = vec![(
                ty.clone(),
                Pat::Bind("v".into()),
                Expr::local("v", ty.clone()),
            )];
            let mut generics: Vec<Generic> = Vec::new();

            for (i, c) in facts.iter().enumerate().filter(|(_, c)| c.to == to) {
                used.insert(i);
                let mut body = c.body.clone();
                body.rewrite(&mut |e| dispatch(e, &facts, &mut needed, &mut used));
                arms.push((c.from.clone(), Pat::Bind("v".into()), body));
                generics.extend(
                    c.generics
                        .iter()
                        .filter(|g| !generics.iter().any(|x| x.name == g.name))
                        .cloned()
                        .collect::<Vec<_>>(),
                );
            }

            let value = Expr::local("v", Ty::Into(to.clone())).boxed();
            let case = Expr::typed(K::TypeCase { value, arms }, ty.clone());
            m.functions.push(Func {
                generics,
                ..Func::new(
                    &normalizer(&to),
                    vec![("v".into(), Ty::Into(to.clone()))],
                    ty,
                    vec![Expr::new(K::Return(Some(case.boxed())))],
                )
            });
        }

        for c in used.into_iter().map(|i| &facts[i]) {
            let at = c.at.clone().unwrap_or_default();
            errors.extend(c.errors.iter().map(|message| Error::Lowering {
                path: at.path.clone(),
                line: at.line,
                item: format!("From<{}> for {}", print::ty(&c.from), c.to),
                message: message.clone(),
            }));
        }

        m.conversions = facts;
    }
}

/// The conversions by the source types of the target: the generics with an
/// `Into` bound become the types they convert into, the references are
/// left out and of the impls with the same source the first one that does
/// not convert into the target again (`Self::from(&value)`) is kept.
fn select(conversions: Vec<Conversion>) -> Vec<Conversion> {
    let mut out: Vec<Conversion> = Vec::new();

    for mut c in conversions {
        lower_conversion_generics(&mut c);

        match out.iter().position(|o| o.to == c.to && o.from == c.from) {
            None => out.push(c),
            Some(i) if delegates(&out[i]) && !delegates(&c) => out[i] = c,
            Some(_) => {}
        }
    }

    out
}

fn lower_conversion_generics(c: &mut Conversion) {
    let mut lowered: Vec<(String, Ty)> = Vec::new();
    c.generics.retain(|g| {
        let into = g
            .bounds
            .iter()
            .find_map(|(t, arg)| if t == "Into" { arg.clone() } else { None });

        match into {
            Some(Ty::Named(x, _)) => lowered.push((g.name.clone(), Ty::Into(x))),
            Some(p) if p.is_primitive() => lowered.push((g.name.clone(), p)),
            _ => return true,
        }

        false
    });
    let lower = |t: &Ty| match t {
        Ty::Param(p) => lowered
            .iter()
            .find(|(g, _)| g == p)
            .map_or_else(|| t.clone(), |(_, l)| l.clone()),
        t => t.clone(),
    };
    c.from = c.from.map(&lower).strip_refs();
    c.body.rewrite(&mut |e| e.ty = e.ty.map(&lower));
}

/// Whether the body converts the value into the target again.
fn delegates(c: &Conversion) -> bool {
    matches!(&c.body.k, K::Convert { value, to } if *to == c.to && value.ty.strip_refs() == c.from)
}

fn lower_generics(f: &mut Func) {
    let into: Vec<(String, String)> = f
        .generics
        .iter()
        .filter_map(|g| {
            g.bounds.iter().find_map(|(t, arg)| match arg {
                Some(Ty::Named(x, _)) if t == "Into" => Some((g.name.clone(), x.clone())),
                _ => None,
            })
        })
        .collect();

    if into.is_empty() {
        return;
    }

    let into_of = |ty: &Ty| match ty.strip_ref() {
        Ty::Param(p) => into
            .iter()
            .find(|(g, _)| g == p)
            .map(|(_, x)| Ty::Into(x.clone())),
        _ => None,
    };
    let lower = |ty: &mut Ty| {
        let lowered = match ty.strip_ref() {
            Ty::List(inner) => into_of(inner).map(|i| Ty::List(Box::new(i))),
            t => into_of(t),
        };

        if let Some(lowered) = lowered {
            *ty = lowered;
        }
    };

    f.params.iter_mut().for_each(|(_, ty)| lower(ty));
    f.generics
        .retain(|g| !into.iter().any(|(n, _)| n == &g.name));
    rewrite_body(&mut f.body, &mut |e| lower(&mut e.ty));
}

/// A `Convert` as the value itself, the body of the `From` conversion of its
/// type or a call of the normalizer, recorded in `needed`, if the body is a
/// statement; the conversions it uses in `used`.
fn dispatch(
    e: &mut Expr,
    facts: &[Conversion],
    needed: &mut BTreeSet<String>,
    used: &mut BTreeSet<usize>,
) {
    let K::Convert { value, to } = &mut e.k else {
        return;
    };
    let src = value.ty.strip_refs();

    if src.name() == Some(to) {
        *e = std::mem::replace(value.as_mut(), Expr::new(K::This));
        return;
    }

    let Some(i) = facts
        .iter()
        .position(|c| c.to == *to && c.from == src && !is_statement(&c.body))
    else {
        needed.insert(to.clone());
        return;
    };

    used.insert(i);
    let to = to.clone();
    let v = std::mem::replace(value.as_mut(), Expr::new(K::This));
    let mut body = facts[i].body.clone();
    body.rewrite(&mut |x| {
        if matches!(&x.k, K::Local(n) if n == "v") {
            *x = v.clone();
        }
    });
    body.rewrite(&mut |x| dispatch(x, facts, needed, used));
    *e = body;
    e.ty = Ty::named(&to);
}

/// Whether the body is a statement rather than a value outside of Rust.
pub fn is_statement(body: &Expr) -> bool {
    matches!(body.k, K::Match { .. } | K::IfLet { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::conversion;
    use crate::codegen::test_utilities::resolved;

    mod lower_conversions {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_lower_conversions() {
                    assert_eq!(LowerConversions.name(), "lower_conversions");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_dispatch_the_conversions_and_make_the_normalizers() {
                    let mut m = resolved(
                        r#"
                        pub struct Root {
                            pub v: Vec<V>,
                        }
                        pub struct V(u64);
                        pub struct W(V);
                        pub trait Tr {}
                        impl Root {
                            pub fn into<T: Into<V>>(mut self, v: T) -> Self {
                                self.v.push(v.into());
                                self
                            }
                            pub fn list<T: Into<V>, R: Tr>(self, vs: Vec<T>, r: &T, t: R, n: u8) -> Self {
                                self
                            }
                            pub fn same(mut self, v: V) -> Self {
                                self.v.push(v.into());
                                self
                            }
                            pub fn known(mut self, n: u64) -> Self {
                                self.v.push(n.into());
                                self
                            }
                        }
                        "#,
                        &["Tr"],
                        &[],
                    );
                    m.conversions = vec![
                        conversion(
                            "V",
                            Ty::Int("u64"),
                            Expr::typed(
                                K::Construct {
                                    class: "V".into(),
                                    fields: vec![("0".into(), Expr::local("v", Ty::Int("u64")))],
                                },
                                Ty::named("V"),
                            ),
                        ),
                        conversion(
                            "V",
                            Ty::Str,
                            Expr::typed(
                                K::Convert {
                                    value: Expr::local("v", Ty::Str).boxed(),
                                    to: "W".into(),
                                },
                                Ty::named("W"),
                            ),
                        ),
                    ];

                    LowerConversions.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "lower_conversions"),
                        "// ===== after lower_conversions\n// Root\nfn into(mut self, v: impl Into<V>) -> Root {\n    self.v.Push(v.into::<V>());\n    return self;\n}\n\n// Root\nfn list<R: Tr>(self, vs: Vec<impl Into<V>>, r: impl Into<V>, t: R, n: u8) -> Root {\n    return self;\n}\n\n// Root\nfn same(mut self, v: V) -> Root {\n    self.v.Push(v);\n    return self;\n}\n\n// Root\nfn known(mut self, n: u64) -> Root {\n    self.v.Push(V(n));\n    return self;\n}\n\nfn into_V(v: impl Into<V>) -> V {\n    return typecase v { V@v => v, u64@v => V(v), str@v => v.into::<W>() };\n}\n\nfn into_W(v: impl Into<W>) -> W {\n    return typecase v { W@v => v };\n}\n\n"
                    );
                    assert_eq!(m.conversions.len(), 2);
                }

                #[test]
                fn run_should_lower_the_generics_keep_the_first_non_delegating_duplicate_and_report_the_errors_of_the_used_conversions()
                 {
                    let mut m = resolved(
                        r#"
                        pub struct Root {
                            pub v: Vec<V>,
                        }
                        pub struct V(u64);
                        pub struct W;
                        impl Root {
                            pub fn into<T: Into<V>>(mut self, v: T) -> Self {
                                self.v.push(v.into());
                                self
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );
                    let generic = |name: &str, bound: &str, arg: Option<Ty>| Generic {
                        name: name.into(),
                        bounds: vec![(bound.into(), arg)],
                    };
                    let v = |t: Ty| Expr::local("v", t);
                    let back = |t: Ty| {
                        Expr::typed(
                            K::Convert {
                                value: v(t).boxed(),
                                to: "V".into(),
                            },
                            Ty::named("V"),
                        )
                    };
                    m.conversions = vec![
                        Conversion {
                            generics: vec![
                                generic("G", "Into", Some(Ty::named("W"))),
                                generic("Q", "Tr", None),
                            ],
                            ..conversion(
                                "V",
                                Ty::Param("G".into()),
                                v(Ty::Tuple(vec![
                                    Ty::Param("G".into()),
                                    Ty::Param("Q".into()),
                                ])),
                            )
                        },
                        Conversion {
                            generics: vec![
                                generic("P", "Into", Some(Ty::Int("u64"))),
                                generic("Q", "Tr", None),
                            ],
                            ..conversion("V", Ty::Param("P".into()), v(Ty::Param("P".into())))
                        },
                        conversion("V", Ty::Int("u8"), back(Ty::Int("u8"))),
                        Conversion {
                            at: Some(Location {
                                path: "/c/src/lib.rs".into(),
                                line: 3,
                            }),
                            errors: vec!["unsupported x".into()],
                            ..conversion("V", Ty::Int("u8"), v(Ty::Int("u8")))
                        },
                        conversion("V", Ty::Int("u8"), Expr::new(K::Unsupported("y".into()))),
                    ];
                    let mut errors = Vec::new();

                    LowerConversions.run(&mut m, &mut errors);

                    assert_eq!(
                        print::module(&m, "lower_conversions"),
                        "// ===== after lower_conversions\n// Root\nfn into(mut self, v: impl Into<V>) -> Root {\n    self.v.Push(v.into::<V>());\n    return self;\n}\n\nfn into_V<Q: Tr>(v: impl Into<V>) -> V {\n    return typecase v { V@v => v, impl Into<W>@v => v, u64@v => v, u8@v => v };\n}\n\n"
                    );
                    assert_eq!(
                        errors.iter().map(ToString::to_string).collect::<Vec<_>>(),
                        ["/c/src/lib.rs:3: `From<u8> for V`: unsupported x"]
                    );
                }
            }
        }
    }
}
