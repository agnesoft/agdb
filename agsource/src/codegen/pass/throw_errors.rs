use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;

/// A `Result` as the value it holds, for targets that raise errors: `Ok(x)`
/// is `x`, `Err(e)` throws `e`, and `?` and `unwrap` pass the value on, the
/// error propagating by itself. A function ending with `Ok(())` just ends.
pub struct ThrowErrors;

impl Pass for ThrowErrors {
    fn name(&self) -> &str {
        "throw_errors"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        m.for_each_func(&mut |f, _| {
            rewrite_body(&mut f.body, &mut throw);

            if f.body
                .last()
                .is_some_and(|e| matches!(e.k, K::Return(None)))
            {
                f.body.pop();
            }
        });
        m.map_types(&|t| match t {
            Ty::Result(ok, _) => (**ok).clone(),
            t => t.clone(),
        });
    }
}

fn throw(e: &mut Expr) {
    match &mut e.k {
        K::Variant { en, var, payload } if en == "Result" && var == "Ok" => {
            *e = payload
                .take()
                .map_or_else(|| Expr::typed(K::Lit(Lit::Unit), Ty::Unit), |p| *p);
        }
        K::Variant {
            en,
            var,
            payload: Some(p),
        } if en == "Result" && var == "Err" => {
            let error = std::mem::replace(p.as_mut(), Expr::new(K::This));
            *e = Expr::typed(K::Throw(error.boxed()), Ty::Unit);
        }
        K::Call {
            callee: Callee::Intrinsic(Intrinsic::Unwrap),
            recv: Some(value),
            ..
        } if matches!(value.ty, Ty::Result(..)) => {
            let ty = std::mem::replace(&mut e.ty, Ty::Unknown);
            *e = std::mem::replace(value.as_mut(), Expr::new(K::This));
            e.ty = ty;
        }
        K::Try(value) => {
            let ty = std::mem::replace(&mut e.ty, Ty::Unknown);
            *e = std::mem::replace(value.as_mut(), Expr::new(K::This));
            e.ty = ty;
        }
        K::Return(Some(value)) if matches!(value.k, K::Lit(Lit::Unit)) => {
            *e = Expr::new(K::Return(None))
        }
        K::Return(Some(value)) if matches!(value.k, K::Throw(_)) => {
            *e = std::mem::replace(value.as_mut(), Expr::new(K::This));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::pass::ExplicitReturns;
    use crate::codegen::pass::Resolve;
    use crate::codegen::print;
    use crate::codegen::test_utilities::CLIENT;
    use crate::codegen::test_utilities::client;
    use crate::codegen::test_utilities::lower;

    mod throw_errors {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_throw_errors() {
                    assert_eq!(ThrowErrors.name(), "throw_errors");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_return_the_ok_values_and_throw_the_errors() {
                    let (mut m, _) = client(CLIENT);
                    Resolve.run(&mut m, &mut Vec::new());
                    ExplicitReturns.run(&mut m, &mut Vec::new());

                    ThrowErrors.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        [
                            print::func(m.method("Api", "login").unwrap()),
                            print::func(m.method("Api", "fail").unwrap()),
                            print::func(&m.traits["Http"][0]),
                            print::func(&m.traits["Http"][1]),
                        ],
                        [
                            "async fn login(&mut self) -> u16 {\n    let tuple1: (u16, str) = <Http>::get(self.client, &self.url(\"/login\")).await;\n    let status: u16 = tuple1.0;\n    let token: str = tuple1.1;\n    self.token = Option::Some(token);\n    return status;\n}",
                            "fn fail(&self) -> u16 {\n    throw ApiError { status: 1, description: \"fail\" };\n}",
                            "async fn get<T>(&self, uri: &str) -> (u16, T) {\n}",
                            "async fn status(&self, uri: &str) -> u16 {\n    return <Http>::get::<()>(self, uri).await.0;\n}",
                        ]
                    );
                }

                #[test]
                fn run_should_pass_the_unwrapped_results_on_and_end_on_the_unit() {
                    let (mut m, _) = lower(
                        r#"
                        pub struct Root;
                        impl Root {
                            pub fn done(&self, r: Result<u8, bool>) -> Result<(), bool> {
                                let v = r.unwrap();
                                if v > 1 {
                                    return Ok(());
                                }
                                Ok(())
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );
                    Resolve.run(&mut m, &mut Vec::new());
                    ExplicitReturns.run(&mut m, &mut Vec::new());

                    ThrowErrors.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::func(m.method("Root", "done").unwrap()),
                        "fn done(&self, r: u8) -> () {\n    let v: u8 = r;\n    if (v > 1) {\n        return ;\n    };\n}"
                    );
                }
            }
        }
    }

    mod throw {
        use super::*;

        #[test]
        fn throw_should_make_an_ok_without_a_payload_the_unit() {
            let mut e = Expr::variant("Result", "Ok", None);

            throw(&mut e);

            assert_eq!(e, Expr::typed(K::Lit(Lit::Unit), Ty::Unit));
        }
    }
}
