use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;
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

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        m.for_each_func(&mut |f, _| lower_generics(f));

        let facts = std::mem::take(&mut m.conversions);
        let mut needed = BTreeSet::new();
        m.for_each_func(&mut |f, _| {
            rewrite_body(&mut f.body, &mut |e| dispatch(e, &facts, &mut needed));
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

            for c in facts.iter().filter(|c| c.to == to) {
                let mut body = c.body.clone();
                body.rewrite(&mut |e| dispatch(e, &facts, &mut needed));
                arms.push((c.from.clone(), Pat::Bind("v".into()), body));
            }

            let value = Expr::local("v", Ty::Into(to.clone())).boxed();
            let case = Expr::typed(K::TypeCase { value, arms }, ty.clone());
            m.functions.push(Func::new(
                &normalizer(&to),
                vec![("v".into(), Ty::Into(to.clone()))],
                ty,
                vec![Expr::new(K::Return(Some(case.boxed())))],
            ));
        }

        m.conversions = facts;
    }
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
/// type or a call of the normalizer, recorded in `needed`.
fn dispatch(e: &mut Expr, facts: &[Conversion], needed: &mut BTreeSet<String>) {
    let K::Convert { value, to } = &mut e.k else {
        return;
    };
    let src = value.ty.strip_ref();

    if src.name() == Some(to) {
        *e = std::mem::replace(value.as_mut(), Expr::new(K::This));
        return;
    }

    let Some(c) = facts.iter().find(|c| c.to == *to && c.from == *src) else {
        needed.insert(to.clone());
        return;
    };

    let to = to.clone();
    let v = std::mem::replace(value.as_mut(), Expr::new(K::This));
    let mut body = c.body.clone();
    body.rewrite(&mut |x| {
        if matches!(&x.k, K::Local(n) if n == "v") {
            *x = v.clone();
        }
    });
    body.rewrite(&mut |x| dispatch(x, facts, needed));
    *e = body;
    e.ty = Ty::named(&to);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
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
                        Conversion {
                            to: "V".into(),
                            from: Ty::Int("u64"),
                            body: Expr::typed(
                                K::Construct {
                                    class: "V".into(),
                                    fields: vec![("0".into(), Expr::local("v", Ty::Int("u64")))],
                                },
                                Ty::named("V"),
                            ),
                        },
                        Conversion {
                            to: "V".into(),
                            from: Ty::Str,
                            body: Expr::typed(
                                K::Convert {
                                    value: Expr::local("v", Ty::Str).boxed(),
                                    to: "W".into(),
                                },
                                Ty::named("W"),
                            ),
                        },
                    ];

                    LowerConversions.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "lower_conversions"),
                        "// ===== after lower_conversions\n// Root\nfn into(mut self, v: impl Into<V>) -> Root {\n    self.v.Push(v.into::<V>());\n    return self;\n}\n\n// Root\nfn list<R: Tr>(self, vs: Vec<impl Into<V>>, r: impl Into<V>, t: R, n: u8) -> Root {\n    return self;\n}\n\n// Root\nfn same(mut self, v: V) -> Root {\n    self.v.Push(v);\n    return self;\n}\n\n// Root\nfn known(mut self, n: u64) -> Root {\n    self.v.Push(V(n));\n    return self;\n}\n\nfn into_V(v: impl Into<V>) -> V {\n    return typecase v { V@v => v, u64@v => V(v), str@v => v.into::<W>() };\n}\n\nfn into_W(v: impl Into<W>) -> W {\n    return typecase v { W@v => v };\n}\n\n"
                    );
                    assert_eq!(m.conversions.len(), 2);
                }
            }
        }
    }
}
