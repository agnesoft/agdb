use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;
use std::collections::BTreeSet;

/// Calls of the single-expression constructors (`Type::new`) and private
/// helpers replaced by their bodies. The inlined methods are recorded in
/// [`Module::inlined`]. The [tests](Module::tests) call the API as a client
/// does, so they are left as they are.
pub struct Inline;

/// The inlined bodies can contain calls to inline in turn.
const DEPTH: usize = 4;

impl Pass for Inline {
    fn name(&self) -> &str {
        "inline"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        let snapshot = m.clone();
        let mut inlined = std::mem::take(&mut m.inlined);
        let tests = std::mem::take(&mut m.tests);

        for _ in 0..DEPTH {
            m.for_each_func(&mut |func, _| {
                rewrite_body(&mut func.body, &mut |e| inline(e, &snapshot, &mut inlined));
            });
        }

        m.inlined = inlined;
        m.tests = tests;
    }
}

/// The method if its body is one expression without statements: a
/// constructor (a value without a receiver) or a private helper.
fn inlinable<'a>(m: &'a Module, class: &str, name: &str) -> Option<&'a Func> {
    let f = m.method(class, name)?;
    let [body] = f.body.as_slice() else {
        return None;
    };
    let (value, ctor) = match &body.k {
        K::Return(Some(value)) => (value.as_ref(), f.recv.is_none()),
        _ => (body, false),
    };
    let simple = !value.any(&|e| {
        matches!(
            e.k,
            K::Let { .. }
                | K::While { .. }
                | K::If { .. }
                | K::IfLet { .. }
                | K::Match { .. }
                | K::Return(_)
        )
    });
    (simple && (ctor || f.private)).then_some(f)
}

fn inline(e: &mut Expr, m: &Module, inlined: &mut BTreeSet<(String, String)>) {
    let K::Call {
        callee: Callee::Static { class, name } | Callee::Member { class, name, .. },
        recv,
        args,
    } = &e.k
    else {
        return;
    };
    let Some(f) = inlinable(m, class, name) else {
        return;
    };
    inlined.insert((class.clone(), name.clone()));
    let mut body = f.body[0].clone();
    body.rewrite(&mut |x| {
        let value = match &x.k {
            K::This => recv.as_deref(),
            K::Local(n) => f.params.iter().position(|(p, _)| p == n).map(|i| &args[i]),
            _ => None,
        };

        if let Some(value) = value {
            *x = value.clone();
        }
    });
    let ty = std::mem::replace(&mut e.ty, Ty::Unknown);
    *e = match body.k {
        K::Return(Some(v)) => *v,
        k => Expr::typed(k, Ty::Unit),
    };

    if ty != Ty::Unknown {
        e.ty = ty;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::resolved;

    mod inline {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_inline() {
                    assert_eq!(Inline.name(), "inline");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_inline_the_constructors_and_the_private_helpers() {
                    let mut m = resolved(
                        r#"
                        pub struct Root {
                            pub v: Vec<usize>,
                            pub d: D,
                        }
                        pub struct D {
                            pub n: usize,
                        }
                        impl D {
                            pub fn new(n: usize) -> D {
                                D { n }
                            }
                            pub fn with_let(n: usize) -> D {
                                let d = D { n };
                                d
                            }
                        }
                        impl Root {
                            pub fn api(mut self) -> Self {
                                self.push(D::new(self.twice(1)).n);
                                self.push(D::with_let(2).n);
                                self.public();
                                self
                            }
                            fn push(&mut self, n: usize) {
                                self.v.push(n)
                            }
                            fn twice(&self, n: usize) -> usize {
                                self.inner(n).len() * 2
                            }
                            fn inner(&self, n: usize) -> Vec<usize> {
                                self.v.iter().map(|x| x + n).collect()
                            }
                            fn branches(&self, n: u8) -> u8 {
                                if n > 1 {
                                    1
                                } else {
                                    2
                                }
                            }
                            pub fn public(&self) -> u8 {
                                self.branches(1)
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );

                    Inline.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "inline"),
                        "// ===== after inline\n// Root\nfn api(mut self) -> Root {\n    self.v.Push(D { n: (self.v.Map(|x| {\n    (x + 1);\n}).Len() * 2) }.n);\n    self.v.Push(D::with_let(2).n);\n    self.public();\n    return self;\n}\n\n// Root\nfn push(&mut self, n: usize) -> () {\n    self.v.Push(n);\n}\n\n// Root\nfn twice(&self, n: usize) -> usize {\n    return (self.v.Map(|x| {\n    (x + n);\n}).Len() * 2);\n}\n\n// Root\nfn inner(&self, n: usize) -> Vec<usize> {\n    return self.v.Map(|x| {\n    (x + n);\n});\n}\n\n// Root\nfn branches(&self, n: u8) -> u8 {\n    if (n > 1) {\n        return 1;\n    } else {\n        return 2;\n    };\n}\n\n// Root\nfn public(&self) -> u8 {\n    return self.branches(1);\n}\n\n"
                    );
                    assert_eq!(
                        m.inlined
                            .iter()
                            .map(|(class, name)| format!("{class}::{name}"))
                            .collect::<Vec<_>>(),
                        ["D::new", "Root::inner", "Root::push", "Root::twice"]
                    );
                }

                #[test]
                fn run_should_type_an_untyped_call_by_the_inlined_value() {
                    let mut m = Module::default();
                    let value = Expr::typed(K::Lit(Lit::Int(1)), Ty::Int("u8"));
                    let new = Func::new(
                        "new",
                        Vec::new(),
                        Ty::Int("u8"),
                        vec![Expr::new(K::Return(Some(value.boxed())))],
                    );
                    m.enums.insert(
                        "E".into(),
                        Enum {
                            name: "E".into(),
                            variants: Vec::new(),
                            methods: vec![new],
                        },
                    );
                    let call = Expr::new(K::Call {
                        callee: Callee::Static {
                            class: "E".into(),
                            name: "new".into(),
                        },
                        recv: None,
                        args: Vec::new(),
                    });
                    m.functions
                        .push(Func::new("f", Vec::new(), Ty::Unit, vec![call]));

                    Inline.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        m.functions[0].body,
                        [Expr::typed(K::Lit(Lit::Int(1)), Ty::Int("u8"))]
                    );
                }

                #[test]
                fn run_should_leave_the_calls_of_methods_the_module_does_not_have() {
                    let mut m = Module::default();
                    let call = Expr::new(K::Call {
                        callee: Callee::Static {
                            class: "X".into(),
                            name: "f".into(),
                        },
                        recv: None,
                        args: Vec::new(),
                    });
                    m.functions
                        .push(Func::new("f", Vec::new(), Ty::Unit, vec![call]));

                    Inline.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "inline"),
                        "// ===== after inline\nfn f() -> () {\n    X::f();\n}\n\n"
                    );
                }
            }
        }
    }
}
