use crate::Error;
use crate::codegen::lowered::*;
use crate::codegen::pass::Pass;

/// Every number is a `number` and every string a `string` in TS: the casts
/// between the primitives are the values themselves.
pub struct EraseCasts;

impl Pass for EraseCasts {
    fn name(&self) -> &str {
        "erase_casts"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        m.for_each_func(&mut |f, _| {
            rewrite_body(&mut f.body, &mut |e| {
                if let K::Cast(value) = &mut e.k {
                    let ty = std::mem::replace(&mut e.ty, Ty::Unknown);
                    *e = std::mem::replace(value.as_mut(), Expr::new(K::This));
                    e.ty = ty;
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

    mod erase_casts {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_erase_casts() {
                    assert_eq!(EraseCasts.name(), "erase_casts");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_replace_the_casts_with_their_values_of_the_target_type() {
                    let mut m = resolved(
                        r#"
                        pub struct Root;
                        impl Root {
                            pub fn f(&self, a: u8) -> u64 {
                                a as u64
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );

                    EraseCasts.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "erase_casts"),
                        "// ===== after erase_casts\n// Root\nfn f(&self, a: u8) -> u64 {\n    return a;\n}\n\n"
                    );
                    assert_eq!(
                        m.classes["Root"].methods[0].body[0].k,
                        K::Return(Some(
                            Expr::typed(K::Local("a".into()), Ty::Int("u64")).boxed()
                        ))
                    );
                }
            }
        }
    }
}
