use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;

/// A `Result` as the value it holds, for targets that raise errors: `Ok(x)`
/// is `x`, `Err(e)` throws `e` and `?` passes the value on, the error
/// propagating by itself.
pub struct ThrowErrors;

impl Pass for ThrowErrors {
    fn name(&self) -> &str {
        "throw_errors"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        m.for_each_func(&mut |f, _| rewrite_body(&mut f.body, &mut throw));
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
        K::Try(value) => {
            let ty = std::mem::replace(&mut e.ty, Ty::Unknown);
            *e = std::mem::replace(value.as_mut(), Expr::new(K::This));
            e.ty = ty;
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
