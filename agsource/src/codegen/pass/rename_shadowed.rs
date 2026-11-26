use crate::Error;
use crate::codegen::lowered::*;
use crate::codegen::pass::Pass;
use std::collections::BTreeMap;

/// A `let` rebinding a name the function already bound renamed `name_N`,
/// the `N`th binding: TS does not declare a name twice in a scope.
pub struct RenameShadowed;

impl Pass for RenameShadowed {
    fn name(&self) -> &str {
        "rename_shadowed"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        m.for_each_func(&mut |f, _| {
            let mut scope = Scope::default();

            for (name, _) in &f.params {
                scope.bind(name);
            }

            scope.block(&mut f.body);
        });
    }
}

#[derive(Default)]
struct Scope {
    /// The names in scope with what they are renamed to, innermost last.
    names: Vec<(String, String)>,
    /// The bindings of each name in the function.
    bound: BTreeMap<String, usize>,
}

impl Scope {
    /// The name bound once more, renamed from its second binding on.
    fn bind(&mut self, name: &str) -> String {
        let n = self.bound.entry(name.to_owned()).or_default();
        *n += 1;
        let renamed = if *n == 1 {
            name.to_owned()
        } else {
            format!("{name}_{n}")
        };
        self.names.push((name.to_owned(), renamed.clone()));
        renamed
    }

    /// Names in scope that no `let` binds, e.g. those of a pattern.
    fn keep(&mut self, names: impl IntoIterator<Item = String>) {
        self.names.extend(names.into_iter().map(|n| (n.clone(), n)));
    }

    fn block(&mut self, body: &mut [Expr]) {
        let len = self.names.len();

        for s in body {
            if let K::Let { name, value } = &mut s.k {
                if let Some(v) = value {
                    self.expr(v);
                }

                *name = self.bind(name);
            } else {
                self.expr(s);
            }
        }

        self.names.truncate(len);
    }

    fn nested(&mut self, names: Vec<String>, body: &mut [Expr]) {
        let len = self.names.len();
        self.keep(names);
        self.block(body);
        self.names.truncate(len);
    }

    fn expr(&mut self, e: &mut Expr) {
        match &mut e.k {
            K::Local(n) => {
                if let Some((_, renamed)) = self.names.iter().rev().find(|(name, _)| name == n) {
                    *n = renamed.clone();
                }
            }
            K::If { cond, then, els } => {
                self.expr(cond);
                self.block(then);
                els.iter_mut().for_each(|e| self.block(e));
            }
            K::IfLet {
                pat,
                value,
                then,
                els,
            } => {
                self.expr(value);
                self.nested(bindings(pat, value), then);
                els.iter_mut().for_each(|e| self.block(e));
            }
            K::Match { value, arms } => {
                self.expr(value);

                for (pat, body) in arms.iter_mut() {
                    self.nested(bindings(pat, value), body);
                }
            }
            K::While { cond, body } => {
                self.expr(cond);
                self.block(body);
            }
            K::Closure { params, body } => self.nested(params.clone(), body),
            _ => e.children_mut().into_iter().for_each(|c| self.expr(c)),
        }
    }
}

fn bindings(pat: &Pat, value: &Expr) -> Vec<String> {
    pat.projections(value)
        .1
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::lower;

    mod rename_shadowed {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_rename_shadowed() {
                    assert_eq!(RenameShadowed.name(), "rename_shadowed");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_rename_the_bindings_of_a_name_after_the_first() {
                    let (mut m, _) = lower(
                        r#"
                        pub struct Root;
                        impl Root {
                            pub fn shadow(&self, a: u8, o: Option<u8>) -> u8 {
                                let a = a + 1;
                                let b = a;
                                let b = b + a;
                                if a > 1 {
                                    let b = b;
                                    b;
                                } else {
                                    b;
                                }
                                while a > b {
                                    let a = 1;
                                }
                                if let Some(b) = o {
                                    b;
                                } else {
                                    let a = b;
                                }
                                match o {
                                    Some(a) => a,
                                    None => b,
                                };
                                let f = |b| b + a;
                                let s;
                                b
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );

                    m.classes.get_mut("Root").unwrap().methods[0]
                        .body
                        .push(Expr::local("unbound", Ty::Unknown));

                    RenameShadowed.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::func(m.method("Root", "shadow").unwrap()),
                        "fn shadow(&self, a: u8, o: Option<u8>) -> u8 {\n    let a_2: ? = (a + 1);\n    let b: ? = a_2;\n    let b_2: ? = (b + a_2);\n    if (a_2 > 1) {\n        let b_3: ? = b_2;\n        b_3;\n    } else {\n        b_2;\n    };\n    while (a_2 > b_2) {\n        let a_3: ? = 1;\n    };\n    if let Some(b) = o {\n        b;\n    } else {\n        let a_4: ? = b_2;\n    };\n    match o {\n        Some(a) => {\n            a;\n        },\n        None => {\n            b_2;\n        },\n    };\n    let f: ? = |b| {\n    (b + a_2);\n};\n    let s;\n    b_2;\n    unbound;\n}"
                    );
                }
            }
        }
    }
}
