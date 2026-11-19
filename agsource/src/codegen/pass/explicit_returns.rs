use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;

/// The tail expressions of the functions that return a value as `return`,
/// through `if` branches and `match` arms. An `if` chain without `else` gets
/// one that panics: Rust accepted it, so the chain is exhaustive.
pub struct ExplicitReturns;

impl Pass for ExplicitReturns {
    fn name(&self) -> &str {
        "explicit_returns"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        m.for_each_func(&mut |f, _| {
            if f.ret != Ty::Unit {
                return_tail(&mut f.body);
            }
        });
    }
}

/// The tail of `body` as `return`.
pub fn return_tail(body: &mut [Expr]) {
    let Some(last) = body.last_mut() else {
        return;
    };

    match &mut last.k {
        K::Return(_) | K::Let { .. } | K::Assign { .. } | K::While { .. } | K::Panic(_) => {}
        K::If { then, els, .. } | K::IfLet { then, els, .. } => {
            return_tail(then);

            match els {
                Some(e) if !e.is_empty() => return_tail(e),
                _ => *els = Some(vec![Expr::new(K::Panic("unreachable".into()))]),
            }
        }
        K::Match { arms, .. } => arms.iter_mut().for_each(|(_, body)| return_tail(body)),
        _ => {
            let value = std::mem::replace(last, Expr::new(K::This));
            *last = Expr::typed(K::Return(Some(value.boxed())), Ty::Unit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::lower;

    mod explicit_returns {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_explicit_returns() {
                    assert_eq!(ExplicitReturns.name(), "explicit_returns");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_return_the_tails_of_the_functions_with_a_value() {
                    let (mut m, _) = lower(
                        r#"
                        pub struct Root;
                        impl Root {
                            pub fn value(&self) -> u8 {
                                let a = 1;
                                a
                            }
                            pub fn branches(&self, c: bool, o: Option<u8>) -> u8 {
                                if c {
                                    1
                                } else if let Some(v) = o {
                                    v
                                } else {
                                    2
                                }
                            }
                            pub fn without_else(&self, c: bool) -> u8 {
                                if c {
                                    return 1;
                                }
                            }
                            pub fn empty_else(&self, c: bool) -> u8 {
                                if c {
                                    1
                                } else {
                                }
                            }
                            pub fn statements(&self) -> u8 {
                                panic!("p")
                            }
                            pub fn arms(&self, o: Option<u8>) -> u8 {
                                match o {
                                    Some(v) => v,
                                    None => {
                                        let a = 1;
                                        a
                                    }
                                }
                            }
                            pub fn empty(&self) -> u8 {}
                            pub fn unit(&self) {
                                self.value()
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );

                    ExplicitReturns.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "explicit_returns"),
                        "// ===== after explicit_returns\n// Root\nfn value(&self) -> u8 {\n    let a: ? = 1;\n    return a;\n}\n\n// Root\nfn branches(&self, c: bool, o: Option<u8>) -> u8 {\n    if c {\n        return 1;\n    } else {\n        if let Some(v) = o {\n            return v;\n        } else {\n            return 2;\n        };\n    };\n}\n\n// Root\nfn without_else(&self, c: bool) -> u8 {\n    if c {\n        return 1;\n    } else {\n        panic!(\"unreachable\");\n    };\n}\n\n// Root\nfn empty_else(&self, c: bool) -> u8 {\n    if c {\n        return 1;\n    } else {\n        panic!(\"unreachable\");\n    };\n}\n\n// Root\nfn statements(&self) -> u8 {\n    panic!(\"p\");\n}\n\n// Root\nfn arms(&self, o: Option<u8>) -> u8 {\n    match o {\n        Some(v) => {\n            return v;\n        },\n        None => {\n            let a: ? = 1;\n            return a;\n        },\n    };\n}\n\n// Root\nfn empty(&self) -> u8 {\n}\n\n// Root\nfn unit(&self) -> () {\n    self.value()/*unresolved*/;\n}\n\n"
                    );
                }
            }
        }
    }
}
