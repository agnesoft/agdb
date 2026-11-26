use crate::Error;
use crate::codegen::lowered::*;
use crate::codegen::pass::Pass;
use std::collections::BTreeSet;

/// Rust callers convert list elements themselves (`[("k", 1).into()]`), TS
/// callers pass the sources (`[["k", 1]]`): the list elements of the
/// conversions into a conversion target accept its sources too, unless the
/// lists of every source are accepted already. It runs before
/// [`LowerConversions`](crate::codegen::pass::LowerConversions).
pub struct AcceptElementSources;

impl Pass for AcceptElementSources {
    fn name(&self) -> &str {
        "accept_element_sources"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        let targets: BTreeSet<String> = m.conversions.iter().map(|c| c.to.clone()).collect();
        let sources = |to: &str| -> Vec<Ty> {
            m.conversions
                .iter()
                .filter(|c| c.to == to)
                .map(|c| c.from.strip_refs())
                .collect()
        };
        let mut widened = Vec::new();

        for (i, c) in m.conversions.iter().enumerate() {
            let Some((from, mapped)) =
                widen(&c.from, Expr::local("v", c.from.clone()), &targets, 0)
            else {
                continue;
            };
            let element = innermost(&c.from).expect("a widened list has a named element");
            let accepted = sources(&c.to);

            if sources(element)
                .iter()
                .all(|s| accepted.contains(&with_innermost(&c.from.strip_refs(), s)))
            {
                continue;
            }

            widened.push((i, from, mapped));
        }

        for (i, from, mapped) in widened {
            let c = &mut m.conversions[i];
            c.from = from;
            c.body.rewrite(&mut |e| {
                if matches!(&e.k, K::Local(n) if n == "v") {
                    *e = mapped.clone();
                }
            });
        }
    }
}

/// The name of the type the lists of `ty` hold.
fn innermost(ty: &Ty) -> Option<&str> {
    match ty.strip_ref() {
        Ty::List(inner) => innermost(inner),
        t => t.name(),
    }
}

/// The lists of `ty` holding `element`.
fn with_innermost(ty: &Ty, element: &Ty) -> Ty {
    match ty {
        Ty::List(inner) => Ty::List(Box::new(with_innermost(inner, element))),
        _ => element.clone(),
    }
}

/// The type accepting the sources of the targets in `ty` and `v` of it
/// mapped to `ty`.
fn widen(ty: &Ty, v: Expr, targets: &BTreeSet<String>, depth: usize) -> Option<(Ty, Expr)> {
    let Ty::List(inner) = ty else {
        return None;
    };
    let x = format!("x{depth}");
    let (accepted, body) = match inner.as_ref() {
        Ty::Named(n, _) if targets.contains(n) => (
            Ty::Into(n.clone()),
            Expr::typed(
                K::Convert {
                    value: Expr::local(&x, Ty::Into(n.clone())).boxed(),
                    to: n.clone(),
                },
                Ty::named(n),
            ),
        ),
        inner => widen(inner, Expr::local(&x, inner.clone()), targets, depth + 1)?,
    };
    let map = Expr::typed(
        K::Call {
            callee: Callee::Intrinsic(Intrinsic::Map),
            recv: Some(v.boxed()),
            args: vec![Expr::new(K::Closure {
                params: vec![x],
                body: vec![Expr::new(K::Return(Some(body.boxed())))],
            })],
        },
        ty.clone(),
    );

    Some((Ty::List(Box::new(accepted)), map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::conversion;

    mod accept_element_sources {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_accept_element_sources() {
                    assert_eq!(AcceptElementSources.name(), "accept_element_sources");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_widen_the_lists_of_targets_whose_sources_lists_are_not_accepted() {
                    let list = |t: Ty| Ty::List(Box::new(t));
                    let v = |t: Ty| Expr::local("v", t);
                    let mut m = Module {
                        conversions: vec![
                            conversion("V", Ty::Int("u64"), v(Ty::Int("u64"))),
                            conversion(
                                "W",
                                list(Ty::named("V")),
                                Expr::typed(
                                    K::Construct {
                                        class: "W".into(),
                                        fields: vec![("0".into(), v(list(Ty::named("V"))))],
                                    },
                                    Ty::named("W"),
                                ),
                            ),
                            conversion(
                                "X",
                                list(list(Ty::named("V"))),
                                v(list(list(Ty::named("V")))),
                            ),
                            conversion("Y", list(Ty::named("V")), v(list(Ty::named("V")))),
                            conversion(
                                "Y",
                                Ty::Ref(Box::new(list(Ty::Int("u64")))),
                                v(list(Ty::Int("u64"))),
                            ),
                            conversion("Z", list(Ty::Int("u8")), v(list(Ty::Int("u8")))),
                        ],
                        ..Module::default()
                    };

                    AcceptElementSources.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        m.conversions
                            .iter()
                            .map(|c| format!(
                                "{} from {}: {}",
                                c.to,
                                print::ty(&c.from),
                                print::expr(&c.body)
                            ))
                            .collect::<Vec<_>>(),
                        [
                            "V from u64: v",
                            "W from Vec<impl Into<V>>: W(v.Map(|x0| {\n    return x0.into::<V>();\n}))",
                            "X from Vec<Vec<impl Into<V>>>: v.Map(|x0| {\n    return x0.Map(|x1| {\n    return x1.into::<V>();\n});\n})",
                            "Y from Vec<V>: v",
                            "Y from &Vec<u64>: v",
                            "Z from Vec<u8>: v"
                        ]
                    );
                }
            }
        }
    }
}
