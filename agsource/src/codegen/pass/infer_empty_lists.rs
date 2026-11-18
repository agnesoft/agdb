use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;

/// `let mut v = vec![]` gets the element type of the first `v.push(..)`.
pub struct InferEmptyLists;

impl Pass for InferEmptyLists {
    fn name(&self) -> &str {
        "infer_empty_lists"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        m.for_each_func(&mut |f, _| {
            let mut pushed = Vec::new();

            rewrite_body(&mut f.body.clone(), &mut |e| {
                if let K::Call {
                    callee: Callee::Intrinsic(Intrinsic::Push),
                    recv: Some(r),
                    args,
                } = &e.k
                    && let K::Local(n) = &r.k
                    && let [arg] = args.as_slice()
                    && arg.ty != Ty::Unknown
                {
                    pushed.push((n.clone(), arg.ty.clone()));
                }
            });

            rewrite_body(&mut f.body, &mut |e| {
                if let K::Let {
                    name,
                    value: Some(v),
                } = &mut e.k
                    && matches!(&v.k, K::Array(items) if items.is_empty())
                    && (v.ty == Ty::Unknown || v.ty == Ty::List(Box::new(Ty::Unknown)))
                    && let Some((_, ty)) = pushed.iter().find(|(n, _)| n == name)
                {
                    v.ty = Ty::List(Box::new(ty.clone()));
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::resolved;

    mod infer_empty_lists {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_infer_empty_lists() {
                    assert_eq!(InferEmptyLists.name(), "infer_empty_lists");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_type_the_empty_lists_by_their_pushes() {
                    let mut m = resolved(
                        r#"
                        pub struct Root {
                            pub v: Vec<u8>,
                        }
                        impl Root {
                            pub fn lists(&self, n: u8) {
                                let mut pushed = vec![];
                                pushed.push(n);
                                let mut typed: Vec<u8> = vec![];
                                typed.push(n);
                                let mut full = vec![n];
                                full.push(n);
                                let mut never = vec![];
                                let mut unknown = vec![];
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );
                    let push = Expr::new(K::Call {
                        callee: Callee::Intrinsic(Intrinsic::Push),
                        recv: Some(Expr::local("unknown", Ty::Unknown).boxed()),
                        args: vec![Expr::new(K::Ident("x".into()))],
                    });
                    m.classes.get_mut("Root").unwrap().methods[0]
                        .body
                        .push(push);

                    InferEmptyLists.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "infer_empty_lists"),
                        "// ===== after infer_empty_lists\n// Root\nfn lists(&self, n: u8) -> () {\n    let pushed: Vec<u8> = vec![];\n    pushed.Push(n);\n    let typed: Vec<u8> = vec![];\n    typed.Push(n);\n    let full: Vec<u8> = vec![n];\n    full.Push(n);\n    let never: Vec<?> = vec![];\n    let unknown: Vec<?> = vec![];\n    unknown.Push(x/*unresolved*/);\n}\n\n"
                    );
                }
            }
        }
    }
}
