use crate::Error;
use crate::codegen::lowered::*;
use crate::codegen::pass::Pass;

/// The Rust keywords TS allows as member names.
const KEYWORDS: &[&str] = &[
    "fn", "impl", "loop", "match", "mod", "move", "ref", "type", "use", "where",
];

/// The methods named as a Rust keyword with the `_` escape (`where_`) named
/// as the keyword.
pub struct UnescapeIdentifiers;

impl Pass for UnescapeIdentifiers {
    fn name(&self) -> &str {
        "unescape_identifiers"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        m.for_each_func(&mut |f, _| {
            unescape(&mut f.name);
            rewrite_body(&mut f.body, &mut |e| {
                if let K::Call {
                    callee: Callee::Member { name, .. },
                    ..
                } = &mut e.k
                {
                    unescape(name);
                }
            });
        });
    }
}

fn unescape(name: &mut String) {
    if let Some(keyword) = name.strip_suffix('_')
        && KEYWORDS.contains(&keyword)
    {
        *name = keyword.to_owned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::resolved;

    mod unescape_identifiers {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_unescape_identifiers() {
                    assert_eq!(UnescapeIdentifiers.name(), "unescape_identifiers");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_name_the_escaped_keyword_methods_and_their_calls_as_the_keyword() {
                    let mut m = resolved(
                        r#"
                        pub struct Root;
                        impl Root {
                            pub fn where_(&self) -> Root {
                                self.end_()
                            }
                            pub fn end_(&self) -> Root {
                                self.where_()
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );

                    UnescapeIdentifiers.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "unescape_identifiers"),
                        "// ===== after unescape_identifiers\n// Root\nfn where(&self) -> Root {\n    return self.end_();\n}\n\n// Root\nfn end_(&self) -> Root {\n    return self.where();\n}\n\n"
                    );
                }
            }
        }
    }
}
