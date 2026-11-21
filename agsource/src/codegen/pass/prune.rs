use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Removes the methods nothing calls. First the trait methods no function
/// or conversion calls, from the traits and their impls. Then the methods
/// the builder API does not use: it uses the `pub` methods of the builders,
/// the remaining methods of the trait impls, the functions of the module,
/// the conversions and the tests, and what they call. Reports the
/// [`Func::errors`] of the functions it keeps and a test calling a method
/// that is not `pub`, which a client would not have.
pub struct Prune;

impl Pass for Prune {
    fn name(&self) -> &str {
        "prune"
    }

    fn run(&self, m: &mut Module, errors: &mut Vec<Error>) {
        let called = trait_calls(m);
        let called = |t: &str, f: &Func| called.contains(&(t.to_owned(), f.name.clone()));

        for (t, fs) in m.traits.iter_mut() {
            fs.retain(|f| called(t, f));
        }

        for c in m.classes.values_mut() {
            for (t, fs) in c.trait_impls.iter_mut() {
                fs.retain(|f| called(t, f));
            }
        }

        for test in &m.tests {
            for (class, name) in callees(&test.body) {
                if m.method(&class, &name).is_some_and(|f| f.private) {
                    let (path, line) = test
                        .at
                        .as_ref()
                        .map(|at| (at.path.clone(), at.line))
                        .unwrap_or_default();
                    errors.push(Error::Lowering {
                        path,
                        line,
                        item: test.name.clone(),
                        message: format!("calls `{class}::{name}` a client does not have"),
                    });
                }
            }
        }

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
            .extend(callees(&f.body));
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
    stack.extend(m.tests.iter().flat_map(|f| callees(&f.body)));
    stack.extend(
        m.conversions
            .iter()
            .flat_map(|c| callees(std::slice::from_ref(&c.body))),
    );
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

/// The (trait, method) pairs of the calls of the functions and the
/// conversions: through a dictionary, or of a method of a class's trait impl.
fn trait_calls(m: &mut Module) -> BTreeSet<(String, String)> {
    let mut calls = BTreeSet::new();
    let mut members = BTreeSet::new();
    let mut note = |e: &mut Expr| match &e.k {
        K::Call {
            callee: Callee::Trait { trait_, name, .. },
            ..
        } => {
            calls.insert((trait_.clone(), name.clone()));
        }
        K::Call {
            callee: Callee::Member { class, name } | Callee::Static { class, name },
            ..
        } => {
            members.insert((class.clone(), name.clone()));
        }
        _ => {}
    };
    m.for_each_func(&mut |f, _| rewrite_body(&mut f.body, &mut note));
    m.conversions
        .iter_mut()
        .for_each(|c| c.body.rewrite(&mut note));

    for (class, name) in members {
        let impls = m.classes.get(&class).map(|c| &c.trait_impls);

        for (t, fs) in impls.into_iter().flatten() {
            if fs.iter().any(|f| f.name == name) {
                calls.insert((t.clone(), name.clone()));
            }
        }
    }

    calls
}

fn callees(body: &[Expr]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut body = body.to_vec();

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
    use crate::codegen::test_utilities::tested;

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
                fn run_should_keep_what_the_tests_call_and_report_their_calls_of_private_methods() {
                    let (mut m, _) = tested(
                        r#"pub struct Root { pub d: D }
pub struct D;
impl Root {
    pub fn new() -> Root { Root { d: D {} } }
    fn private(&self) {}
}
impl D {
    pub fn tested(&self) {}
    pub fn unused(&self) {}
}"#,
                        r#"use c::Root;
#[test]
fn t() {
    let r: Root = Root::new();
    r.private();
    r.d.tested();
}"#,
                        &[],
                    );
                    Resolve.run(&mut m, &mut Vec::new());
                    let mut errors = Vec::new();

                    Prune.run(&mut m, &mut errors);

                    assert_eq!(
                        names(&m),
                        ["D::tested", "Root::new", "Root::private", "::t"]
                    );
                    assert_eq!(
                        strings(&errors),
                        ["/c/tests/t.rs:3: `t`: calls `Root::private` a client does not have"]
                    );
                }

                #[test]
                fn run_should_keep_what_the_builder_api_and_the_called_trait_methods_call_and_report_its_errors()
                 {
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
                            fn uncalled(&self);
                        }
                        impl Tr for D {
                            fn t(&self) {
                                self.used_by_trait();
                            }
                            fn uncalled(&self) {
                                self.used_by_uncalled();
                            }
                        }
                        impl Root {
                            pub fn api(&self) {
                                self.d.t();
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
                            pub fn generic<T: Tr>(&self, t: T) {
                                t.t();
                            }
                        }
                        impl From<u8> for D {
                            fn from(v: u8) -> Self {
                                D::conv()
                            }
                        }
                        impl D {
                            pub fn new() -> D {
                                D {}
                            }
                            pub fn used(&self) {}
                            pub fn used_by_trait(&self) {}
                            pub fn used_by_uncalled(&self) {}
                            pub fn unused(&self) {}
                            pub fn conv() -> D {
                                D {}
                            }
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
                            "D::conv",
                            "D::t",
                            "Root::api",
                            "Root::helper",
                            "Root::generic",
                            "::f"
                        ]
                    );
                    assert_eq!(
                        strings(&errors),
                        vec![
                            "/c/src/lib.rs:23: `Root::api`: unknown function `unknown`",
                            "`f`: made"
                        ]
                    );
                }
            }
        }
    }
}
