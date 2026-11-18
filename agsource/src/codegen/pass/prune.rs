use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Removes the methods nothing calls: the functions the builder API uses
/// are the `pub` methods of the builders, the methods of the trait impls
/// and the functions of the module, and what they call. Reports the
/// [`Func::errors`] of the functions it keeps.
pub struct Prune;

impl Pass for Prune {
    fn name(&self) -> &str {
        "prune"
    }

    fn run(&self, m: &mut Module, errors: &mut Vec<Error>) {
        let used = used(m);
        let keep = |class: &str, f: &Func| used.contains(&(class.to_owned(), f.name.clone()));

        for (name, c) in m.classes.iter_mut() {
            c.methods.retain(|f| keep(name, f));
        }

        for (name, e) in m.enums.iter_mut() {
            e.methods.retain(|f| keep(name, f));
        }

        m.for_each_func(&mut |f, class| {
            let item = if class.is_empty() {
                f.name.clone()
            } else {
                format!("{class}::{}", f.name)
            };
            let (path, line) =
                f.at.as_ref()
                    .map(|at| (at.path.clone(), at.line))
                    .unwrap_or_default();

            errors.extend(f.errors.iter().map(|message| Error::Lowering {
                path: path.clone(),
                line,
                item: item.clone(),
                message: message.clone(),
            }));
        });
    }
}

/// The (class, method) pairs reachable from the builder API. The trait
/// impls are kept whole, so a trait call needs no edge.
fn used(m: &mut Module) -> BTreeSet<(String, String)> {
    let mut calls: BTreeMap<(String, String), Vec<(String, String)>> = BTreeMap::new();
    m.for_each_func(&mut |f, class| {
        calls
            .entry((class.to_owned(), f.name.clone()))
            .or_default()
            .extend(callees(f));
    });

    let mut stack: Vec<(String, String)> = Vec::new();

    for (name, c) in &m.classes {
        let public = c
            .methods
            .iter()
            .filter(|f| m.is_builder(name) && !f.private);

        for f in public.chain(c.trait_impls.values().flatten()) {
            stack.push((name.clone(), f.name.clone()));
        }
    }

    stack.extend(m.functions.iter().map(|f| (String::new(), f.name.clone())));
    let mut used = BTreeSet::new();

    while let Some(id) = stack.pop() {
        if let Some(callees) = calls.get(&id)
            && used.insert(id)
        {
            stack.extend(callees.iter().cloned());
        }
    }

    used
}

fn callees(f: &Func) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut body = f.body.clone();

    rewrite_body(&mut body, &mut |e| {
        if let K::Call {
            callee: Callee::Member { class, name } | Callee::Static { class, name },
            ..
        } = &e.k
        {
            out.push((class.clone(), name.clone()));
        }
    });

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::pass::Resolve;
    use crate::codegen::test_utilities::lower;
    use crate::codegen::test_utilities::strings;

    fn names(m: &Module) -> Vec<String> {
        let mut out = Vec::new();
        m.clone()
            .for_each_func(&mut |f, class| out.push(format!("{class}::{}", f.name)));
        out
    }

    mod prune {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_prune() {
                    assert_eq!(Prune.name(), "prune");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_keep_what_the_builder_api_calls_and_report_its_errors() {
                    let (mut m, _) = lower(
                        r#"
                        pub struct Root {
                            pub d: D,
                            pub e: E,
                        }
                        pub struct D;
                        pub enum E {
                            A,
                        }
                        pub trait Tr {
                            fn t(&self);
                        }
                        impl Tr for D {
                            fn t(&self) {
                                self.used_by_trait();
                            }
                        }
                        impl Root {
                            pub fn api(&self) {
                                self.helper();
                                D::new();
                                unknown();
                            }
                            fn helper(&self) {
                                self.d.used();
                                self.helper();
                            }
                            fn unused(&self) {
                                unknown();
                            }
                        }
                        impl D {
                            pub fn new() -> D {
                                D {}
                            }
                            pub fn used(&self) {}
                            pub fn used_by_trait(&self) {}
                            pub fn unused(&self) {}
                        }
                        impl E {
                            pub fn unused(&self) {}
                        }
                        "#,
                        &["Tr"],
                        &[],
                    );
                    let mut f = Func::new("f", Vec::new(), Ty::Unit, Vec::new());
                    f.errors.push("made".into());
                    m.functions.push(f);
                    Resolve.run(&mut m, &mut Vec::new());
                    let mut errors = Vec::new();

                    Prune.run(&mut m, &mut errors);

                    assert_eq!(
                        names(&m),
                        vec![
                            "D::new",
                            "D::used",
                            "D::used_by_trait",
                            "D::t",
                            "Root::api",
                            "Root::helper",
                            "::f"
                        ]
                    );
                    assert_eq!(
                        strings(&errors),
                        vec![
                            "/c/src/lib.rs:19: `Root::api`: unknown function `unknown`",
                            "`f`: made"
                        ]
                    );
                }
            }
        }
    }
}
