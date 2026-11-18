use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;
use std::collections::BTreeSet;

/// Trait calls dispatched through dictionaries, the receivers of
/// [`Callee::Trait`]:
///
/// - a class with a generic bounded by a trait stores its dictionary as a
///   field (see [`dict_field`]),
/// - a function with one takes it as a parameter,
/// - a named type refers to its global dictionary (see [`dict_global`]),
///   recorded in [`Module::dicts`] as (trait, type).
///
/// The `reflective` traits are implemented by types the module does not see
/// and get the one dictionary named as the trait that the target runtime
/// provides.
pub struct TraitDicts {
    pub reflective: Vec<String>,
}

/// The field or parameter holding the dictionary of `trait_` for the generic
/// `param`.
pub fn dict_field(param: &str, trait_: &str) -> String {
    format!("{param}_{trait_}")
}

/// The dictionary of `trait_` for the type `of`.
pub fn dict_global(of: &str, trait_: &str) -> String {
    format!("{of}_{trait_}")
}

/// The (generic, trait) pairs of the generics bounded by the traits.
type Dicts = Vec<(String, String)>;

impl Pass for TraitDicts {
    fn name(&self) -> &str {
        "trait_dicts"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        let traits: BTreeSet<String> = m
            .traits
            .keys()
            .filter(|t| !self.reflective.contains(t))
            .cloned()
            .collect();
        let bounded = |generics: &[Generic]| -> Dicts {
            generics
                .iter()
                .flat_map(|g| {
                    g.bounds
                        .iter()
                        .filter(|(t, _)| traits.contains(t))
                        .map(|(t, _)| (g.name.clone(), t.clone()))
                })
                .collect()
        };
        let dict_ty = |g: &str, t: &str| Ty::Dict(t.to_owned(), Box::new(Ty::Param(g.to_owned())));

        let mut classes = Vec::new();

        for (name, c) in m.classes.iter_mut() {
            let dicts = bounded(&c.generics);

            for (g, t) in &dicts {
                c.fields.push((dict_field(g, t), dict_ty(g, t)));
            }

            if !dicts.is_empty() {
                classes.push((name.clone(), dicts));
            }
        }

        let generics = m.clone();
        let mut globals = std::mem::take(&mut m.dicts);

        m.for_each_func(&mut |func, class| {
            let of_class = classes
                .iter()
                .find(|(n, _)| n == class)
                .map(|(_, d)| d.clone())
                .unwrap_or_default();
            let of_func = bounded(&func.generics);

            for (g, t) in &of_func {
                func.params.push((dict_field(g, t), dict_ty(g, t)));
            }

            let scope = Scope {
                reflective: &self.reflective,
                of_class: &of_class,
                of_func: &of_func,
            };

            rewrite_body(&mut func.body, &mut |e| match &mut e.k {
                K::Call {
                    callee: Callee::Trait { trait_, .. },
                    recv: Some(recv),
                    args,
                } => {
                    let of = recv.ty.strip_ref().clone();
                    let dict = scope.dict(&of, trait_, &mut globals);
                    let value = std::mem::replace(recv.as_mut(), dict);

                    if !matches!(value.k, K::Path(_)) {
                        args.insert(0, value);
                    }
                }
                K::Construct { class, fields } => {
                    let Some((_, dicts)) = classes.iter().find(|(n, _)| n == class) else {
                        return;
                    };
                    let args = match &e.ty {
                        Ty::Named(_, args) => args.as_slice(),
                        _ => &[],
                    };
                    let params = generics.generics_of(class);

                    for (g, t) in dicts {
                        let of = params
                            .iter()
                            .position(|p| &p.name == g)
                            .and_then(|i| args.get(i))
                            .cloned()
                            .unwrap_or_else(|| Ty::Param(g.clone()));
                        fields.push((dict_field(g, t), scope.dict(&of, t, &mut globals)));
                    }
                }
                _ => {}
            });
        });

        m.dicts = globals;
    }
}

/// The dictionaries a function body can refer to.
struct Scope<'a> {
    reflective: &'a [String],
    of_class: &'a Dicts,
    of_func: &'a Dicts,
}

impl Scope<'_> {
    fn dict(&self, of: &Ty, t: &str, globals: &mut BTreeSet<(String, String)>) -> Expr {
        let ty = Ty::Dict(t.to_owned(), Box::new(of.clone()));
        let has = |dicts: &Dicts, p: &str| dicts.iter().any(|(g, x)| g == p && x == t);

        match of {
            _ if self.reflective.iter().any(|r| r == t) => {
                Expr::typed(K::Path(vec![t.to_owned()]), ty)
            }
            Ty::Param(p) if has(self.of_class, p) => {
                Expr::typed(K::This, Ty::Unknown).field(&dict_field(p, t), ty)
            }
            Ty::Param(p) if has(self.of_func, p) => Expr::local(&dict_field(p, t), ty),
            Ty::Named(n, _) => {
                globals.insert((t.to_owned(), n.clone()));
                Expr::typed(K::Path(vec![dict_global(n, t)]), ty)
            }
            _ => Expr::typed(K::Path(vec![t.to_owned()]), ty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::resolved;

    mod trait_dicts {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_trait_dicts() {
                    let pass = TraitDicts {
                        reflective: Vec::new(),
                    };

                    assert_eq!(pass.name(), "trait_dicts");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_pass_the_dictionaries_of_the_trait_calls() {
                    let mut m = resolved(
                        r#"
                        pub trait Tr {
                            fn t(&self) -> u8;
                            fn s() -> u8;
                        }
                        pub trait Refl {
                            fn r(&self) -> u8;
                        }
                        pub struct Root {
                            pub v: V,
                        }
                        pub struct V;
                        impl Tr for V {
                            fn t(&self) -> u8 {
                                1
                            }
                            fn s() -> u8 {
                                2
                            }
                        }
                        pub struct Q<T: Tr> {
                            pub t: T,
                        }
                        impl<T: Tr> Q<T> {
                            pub fn get(&self) -> u8 {
                                self.t.t()
                            }
                        }
                        impl Root {
                            pub fn q(self) -> Q<V> {
                                Q { t: V {} }
                            }
                            pub fn func<G: Tr>(self, g: G) -> u8 {
                                g.t()
                            }
                            pub fn stat<G: Tr>(self) -> u8 {
                                G::s()
                            }
                            pub fn refl<R: Refl>(self, r: R) -> u8 {
                                r.r()
                            }
                        }
                        "#,
                        &["Tr", "Refl"],
                        &[],
                    );
                    let unknown = Expr::new(K::Call {
                        callee: Callee::Trait {
                            trait_: "Tr".into(),
                            name: "t".into(),
                        },
                        recv: Some(Expr::local("x", Ty::Unknown).boxed()),
                        args: Vec::new(),
                    });
                    let named = Expr::new(K::Call {
                        callee: Callee::Trait {
                            trait_: "Tr".into(),
                            name: "t".into(),
                        },
                        recv: Some(Expr::local("v", Ty::named("V")).boxed()),
                        args: Vec::new(),
                    });
                    let untyped = Expr::new(K::Construct {
                        class: "Q".into(),
                        fields: Vec::new(),
                    });
                    let other = Expr::new(K::Construct {
                        class: "V".into(),
                        fields: Vec::new(),
                    });
                    m.functions.push(Func::new(
                        "made",
                        Vec::new(),
                        Ty::Unit,
                        vec![unknown, named, untyped, other],
                    ));
                    let pass = TraitDicts {
                        reflective: vec!["Refl".into()],
                    };

                    pass.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "trait_dicts"),
                        "// ===== after trait_dicts\n// Root\nfn q(self) -> Q<V> {\n    return Q { t: V {  }, T_Tr: V_Tr };\n}\n\n// Root\nfn func<G: Tr>(self, g: G, G_Tr: dyn Tr<G>) -> u8 {\n    return <Tr>::t(G_Tr, g);\n}\n\n// Root\nfn stat<G: Tr>(self, G_Tr: dyn Tr<G>) -> u8 {\n    return <Tr>::s(G_Tr, );\n}\n\n// Root\nfn refl<R: Refl>(self, r: R) -> u8 {\n    return <Refl>::r(Refl, r);\n}\n\n// Q\nfn get(&self) -> u8 {\n    return <Tr>::t(self.T_Tr, self.t);\n}\n\n// impl Tr for V\nfn t(&self) -> u8 {\n    return 1;\n}\n\n// impl Tr for V\nfn s() -> u8 {\n    return 2;\n}\n\nfn made() -> () {\n    <Tr>::t(Tr, x);\n    <Tr>::t(V_Tr, v);\n    Q { T_Tr: Tr };\n    V {  };\n}\n\n"
                    );
                    assert_eq!(
                        m.classes["Q"]
                            .fields
                            .iter()
                            .map(|(name, ty)| format!("{name}: {}", print::ty(ty)))
                            .collect::<Vec<_>>(),
                        ["t: T", "T_Tr: dyn Tr<T>"]
                    );
                    assert_eq!(
                        m.dicts.iter().cloned().collect::<Vec<_>>(),
                        [("Tr".to_owned(), "V".to_owned())]
                    );
                }
            }
        }
    }
}
