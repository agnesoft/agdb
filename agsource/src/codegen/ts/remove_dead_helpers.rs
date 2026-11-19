use crate::Error;
use crate::codegen::lowered::*;
use crate::codegen::pass::Pass;
use std::collections::BTreeSet;

/// The [inlined](crate::codegen::pass::Inline) methods without callers
/// left removed: TS has no private methods to keep them out of the API.
pub struct RemoveDeadHelpers;

impl Pass for RemoveDeadHelpers {
    fn name(&self) -> &str {
        "remove_dead_helpers"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        let mut called = BTreeSet::new();
        let mut snapshot = m.clone();
        snapshot.for_each_func(&mut |f, _| {
            rewrite_body(&mut f.body, &mut |e| {
                if let K::Call {
                    callee: Callee::Member { class, name } | Callee::Static { class, name },
                    ..
                } = &e.k
                {
                    called.insert((class.clone(), name.clone()));
                }
            });
        });
        let dead: BTreeSet<_> = m.inlined.difference(&called).cloned().collect();

        for (name, class) in &mut m.classes {
            class
                .methods
                .retain(|f| !dead.contains(&(name.clone(), f.name.clone())));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::test_utilities::resolved;

    mod remove_dead_helpers {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_remove_dead_helpers() {
                    assert_eq!(RemoveDeadHelpers.name(), "remove_dead_helpers");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_remove_the_inlined_methods_nothing_calls() {
                    let mut m = resolved(
                        r#"
                        pub struct Root;
                        impl Root {
                            pub fn api(&self) -> u8 {
                                self.called()
                            }
                            fn called(&self) -> u8 {
                                1
                            }
                            fn dead(&self) -> u8 {
                                2
                            }
                            fn kept(&self) -> u8 {
                                3
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );
                    m.inlined.insert(("Root".into(), "called".into()));
                    m.inlined.insert(("Root".into(), "dead".into()));

                    RemoveDeadHelpers.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        m.classes["Root"]
                            .methods
                            .iter()
                            .map(|f| f.name.as_str())
                            .collect::<Vec<_>>(),
                        ["api", "called", "kept"]
                    );
                }
            }
        }
    }
}
