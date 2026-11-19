use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;

/// The data structs with a single tuple field are serialized as the value of
/// the field (serde's newtype structs), so they become the type of the
/// field: their values become the values of the field, their fields are
/// cleared and their types recorded in [`Module::newtypes`].
pub struct EraseNewtypes;

impl Pass for EraseNewtypes {
    fn name(&self) -> &str {
        "erase_newtypes"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        let newtypes: Vec<(String, Ty)> = m
            .classes
            .values()
            .filter(|c| !m.is_builder(&c.name))
            .filter_map(|c| match c.fields.as_slice() {
                [(name, ty)] if name == "0" => Some((c.name.clone(), ty.clone())),
                _ => None,
            })
            .collect();
        let inner_of = |name: &str| newtypes.iter().find(|(n, _)| n == name).map(|(_, ty)| ty);
        let is_newtype = |ty: &Ty| ty.strip_ref().name().is_some_and(|n| inner_of(n).is_some());

        m.for_each_func(&mut |f, _| {
            rewrite_body(&mut f.body, &mut |e| match &mut e.k {
                K::Construct { class, fields }
                    if inner_of(class).is_some() && fields.len() == 1 =>
                {
                    *e = fields.remove(0).1;
                }
                K::Field { base, name } if name == "0" && is_newtype(&base.ty) => {
                    *e = std::mem::replace(base.as_mut(), Expr::new(K::This));
                }
                _ => {}
            });
        });

        m.map_types(&|ty| erase(ty, &newtypes));

        m.classes
            .values_mut()
            .filter(|c| inner_of(&c.name).is_some())
            .for_each(|c| c.fields.clear());
        m.newtypes.extend(
            newtypes
                .iter()
                .map(|(n, _)| (n.clone(), erase(&Ty::named(n), &newtypes))),
        );

        m.for_each_func(&mut |f, _| rewrite_body(&mut f.body, &mut identity_map));
    }
}

fn erase(ty: &Ty, newtypes: &[(String, Ty)]) -> Ty {
    match newtypes.iter().find(|(n, _)| ty.name() == Some(n)) {
        Some((_, inner)) => inner.map(&|t| erase(t, newtypes)),
        None => ty.clone(),
    }
}

/// `v.iter().map(DbF64).collect()` became a map with the identity.
fn identity_map(e: &mut Expr) {
    if let K::Call {
        callee: Callee::Intrinsic(Intrinsic::Map),
        recv: Some(r),
        args,
    } = &mut e.k
        && let [
            Expr {
                k: K::Closure { params, body },
                ..
            },
        ] = args.as_slice()
        && let ([param], [value]) = (params.as_slice(), body.as_slice())
        && let K::Local(x) = &returned(value).k
        && x == param
    {
        *e = std::mem::replace(r.as_mut(), Expr::new(K::This));
    }
}

/// The value of the tail of a closure, with or without `return`.
fn returned(e: &Expr) -> &Expr {
    match &e.k {
        K::Return(Some(value)) => value,
        _ => e,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::resolved;

    mod erase_newtypes {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_erase_newtypes() {
                    assert_eq!(EraseNewtypes.name(), "erase_newtypes");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_replace_the_newtypes_with_their_values() {
                    let mut m = resolved(
                        r#"
                        pub struct Root {
                            pub n: N,
                            pub m: M,
                            pub v: Vec<N>,
                        }
                        pub struct N(u64);
                        pub struct M(N);
                        pub struct P(u64, u8);
                        pub struct B(u8);
                        impl B {
                            pub fn get(&self) -> u8 {
                                self.0
                            }
                        }
                        impl Root {
                            pub fn n(&self, n: u64) -> N {
                                N(n)
                            }
                            pub fn m(&self) -> u64 {
                                self.m.0.0
                            }
                            pub fn p(&self) -> P {
                                P(1, 2)
                            }
                            pub fn b(&self) -> B {
                                B(1)
                            }
                            pub fn list(&self, v: Vec<u64>) -> Vec<N> {
                                v.iter().map(|x| N(x)).collect()
                            }
                            pub fn other(&self, v: Vec<u64>) -> Vec<u64> {
                                v.iter().map(|x| x + 1).collect()
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );
                    let closure = Expr::new(K::Closure {
                        params: vec!["x".into()],
                        body: vec![Expr::new(K::Return(Some(
                            Expr::typed(
                                K::Construct {
                                    class: "N".into(),
                                    fields: vec![("0".into(), Expr::local("x", Ty::Int("u64")))],
                                },
                                Ty::named("N"),
                            )
                            .boxed(),
                        )))],
                    });
                    let map = Expr::new(K::Call {
                        callee: Callee::Intrinsic(Intrinsic::Map),
                        recv: Some(Expr::local("v", Ty::List(Box::new(Ty::Int("u64")))).boxed()),
                        args: vec![closure],
                    });
                    m.functions
                        .push(Func::new("made", Vec::new(), Ty::Unit, vec![map]));

                    EraseNewtypes.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "erase_newtypes"),
                        "// ===== after erase_newtypes\n// Root\nfn n(&self, n: u64) -> u64 {\n    return n;\n}\n\n// Root\nfn m(&self) -> u64 {\n    return self.m;\n}\n\n// Root\nfn p(&self) -> P {\n    return P(1, 2);\n}\n\n// Root\nfn b(&self) -> B {\n    return B(1);\n}\n\n// Root\nfn list(&self, v: Vec<u64>) -> Vec<u64> {\n    return v;\n}\n\n// Root\nfn other(&self, v: Vec<u64>) -> Vec<u64> {\n    return v.Map(|x| {\n    (x + 1);\n});\n}\n\n// B\nfn get(&self) -> u8 {\n    return self.0;\n}\n\nfn made() -> () {\n    v;\n}\n\n"
                    );
                    assert_eq!(
                        m.classes
                            .values()
                            .map(|c| format!("{}: {}", c.name, c.fields.len()))
                            .collect::<Vec<_>>(),
                        vec!["B: 1", "M: 0", "N: 0", "P: 2", "Root: 3"]
                    );
                }
            }
        }
    }
}
