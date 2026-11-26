use crate::Error;
use crate::codegen::lowered::*;
use crate::codegen::pass::Pass;
use crate::codegen::print;

/// The `skip_serializing_if` fields holding their default left out of the
/// values, as serde leaves them out of the JSON (their schema type has them
/// optional).
pub struct OmitSkippedDefaults;

impl Pass for OmitSkippedDefaults {
    fn name(&self) -> &str {
        "omit_skipped_defaults"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        let skipped: Vec<(String, String, String)> = m
            .skipped_defaults
            .iter()
            .map(|(class, field, default)| (class.clone(), field.clone(), print::expr(default)))
            .collect();

        m.for_each_func(&mut |f, _| {
            rewrite_body(&mut f.body, &mut |e| {
                if let K::Construct { class, fields } = &mut e.k {
                    fields.retain(|(name, value)| {
                        !skipped
                            .iter()
                            .any(|(c, f, d)| c == class && f == name && *d == print::expr(value))
                    });
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::test_utilities::resolved;

    mod omit_skipped_defaults {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_omit_skipped_defaults() {
                    assert_eq!(OmitSkippedDefaults.name(), "omit_skipped_defaults");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_leave_out_the_skipped_fields_holding_their_default() {
                    let mut m = resolved(
                        r#"
                        pub struct Root {
                            #[serde(skip_serializing_if = "Option::is_none")]
                            pub o: Option<u8>,
                            #[serde(skip_serializing_if = "Vec::is_empty")]
                            pub v: Vec<u8>,
                            pub n: Option<u8>,
                        }
                        impl Root {
                            pub fn none(&self) -> Root {
                                Root { o: None, v: vec![], n: None }
                            }
                            pub fn some(&self) -> Root {
                                Root { o: Some(1), v: vec![1], n: None }
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );

                    OmitSkippedDefaults.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "omit_skipped_defaults"),
                        "// ===== after omit_skipped_defaults\n// Root\nfn none(&self) -> Root {\n    return Root { n: Option::None };\n}\n\n// Root\nfn some(&self) -> Root {\n    return Root { o: Option::Some(1), v: vec![1], n: Option::None };\n}\n\n"
                    );
                }
            }
        }
    }
}
