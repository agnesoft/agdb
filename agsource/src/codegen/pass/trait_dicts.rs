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
/// The values of the callers of a [reflective](Module::is_reflective) trait
/// become [`K::Fields`] of the [one class](Module::fields_of) implementing
/// it, whose dictionary they use.
pub struct TraitDicts;

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

    fn run(&self, m: &mut Module, errors: &mut Vec<Error>) {
        let (reflective, traits): (Vec<String>, Vec<String>) =
            m.traits.keys().cloned().partition(|t| m.is_reflective(t));
        let reflective: Vec<(String, Option<String>)> = reflective
            .into_iter()
            .map(|t| {
                let fields = m.fields_of(&t).map(str::to_owned);
                (t, fields)
            })
            .collect();

        for (t, _) in reflective.iter().filter(|(_, fields)| fields.is_none()) {
            errors.push(Error::Lowering {
                path: Default::default(),
                line: 0,
                item: t.clone(),
                message: "no one class implementing it for the values of the callers".into(),
            });
        }

        let all: Vec<String> = m.traits.keys().cloned().collect();
        let bounded_by = |generics: &[Generic], traits: &[String]| -> Dicts {
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
        let bounded = |generics: &[Generic]| bounded_by(generics, &traits);
        let dict_ty = |g: &str, t: &str| Ty::Dict(t.to_owned(), Box::new(Ty::Param(g.to_owned())));

        let mut classes = Vec::new();

        for (name, c) in m.classes.iter_mut() {
            let dicts = bounded_by(&c.generics, &all);

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
                reflective: &reflective,
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
                        args.insert(0, scope.fields(value, &of, trait_));
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

        for (t, of) in &globals {
            if !m
                .classes
                .get(of)
                .is_some_and(|c| c.trait_impls.contains_key(t))
            {
                errors.push(Error::Lowering {
                    path: Default::default(),
                    line: 0,
                    item: of.clone(),
                    message: format!("no impl of `{t}` in the module for its dictionary"),
                });
            }
        }

        m.dicts = globals;
    }
}

/// The dictionaries a function body can refer to.
struct Scope<'a> {
    reflective: &'a [(String, Option<String>)],
    of_class: &'a Dicts,
    of_func: &'a Dicts,
}

impl Scope<'_> {
    /// The class the value of `of` becomes for a call of `t`: of a
    /// reflective trait unless the dictionary is a field of the class or
    /// the one of a named type.
    fn fields_of(&self, of: &Ty, t: &str) -> Option<&str> {
        match of {
            Ty::Param(p) if has(self.of_class, p, t) => None,
            Ty::Named(..) => None,
            _ => self
                .reflective
                .iter()
                .find(|(r, _)| r == t)
                .and_then(|(_, fields)| fields.as_deref()),
        }
    }

    /// The `value` of a trait call as the class it becomes (see
    /// [`Scope::fields_of`]).
    fn fields(&self, value: Expr, of: &Ty, t: &str) -> Expr {
        match self.fields_of(of, t) {
            Some(fields) => Expr::typed(K::Fields(value.boxed()), Ty::named(fields)),
            None => value,
        }
    }

    fn dict(&self, of: &Ty, t: &str, globals: &mut BTreeSet<(String, String)>) -> Expr {
        let ty = Ty::Dict(t.to_owned(), Box::new(of.clone()));

        match of {
            Ty::Param(p) if has(self.of_class, p, t) => {
                Expr::typed(K::This, Ty::Unknown).field(&dict_field(p, t), ty)
            }
            _ if let Some(fields) = self.fields_of(of, t) => {
                globals.insert((t.to_owned(), fields.to_owned()));
                Expr::typed(K::Path(vec![dict_global(fields, t)]), ty)
            }
            Ty::Param(p) if has(self.of_func, p, t) => Expr::local(&dict_field(p, t), ty),
            Ty::Named(n, _) => {
                globals.insert((t.to_owned(), n.clone()));
                Expr::typed(K::Path(vec![dict_global(n, t)]), ty)
            }
            _ => Expr::typed(K::Path(vec![t.to_owned()]), ty),
        }
    }
}

/// Whether the dictionary of `t` for the generic `p` is in `dicts`.
fn has(dicts: &Dicts, p: &str, t: &str) -> bool {
    dicts.iter().any(|(g, x)| g == p && x == t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::resolved;
    use crate::codegen::test_utilities::strings;

    mod trait_dicts {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_trait_dicts() {
                    assert_eq!(TraitDicts.name(), "trait_dicts");
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
                        pub enum En {
                            A,
                        }
                        impl Tr for En {
                            fn t(&self) -> u8 {
                                3
                            }
                            fn s() -> u8 {
                                4
                            }
                        }
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
                            pub fn en(self) -> Q<En> {
                                Q { t: En::A }
                            }
                            fn func<G: Tr>(self, g: G) -> u8 {
                                g.t()
                            }
                            fn stat<G: Tr>(self) -> u8 {
                                G::s()
                            }
                            pub fn refl<R: Refl>(self, r: R) -> u8 {
                                r.r()
                            }
                            pub fn lone<L: Lone>(self, l: L) -> u8 {
                                l.l()
                            }
                        }
                        pub trait Lone {
                            fn l(&self) -> u8;
                        }
                        pub struct F(Vec<(u8, u8)>);
                        impl Refl for F {
                            fn r(&self) -> u8 {
                                5
                            }
                        }
                        "#,
                        &["Tr", "Refl", "Lone"],
                        &[],
                    );
                    let unknown = Expr::new(K::Call {
                        callee: Callee::Trait {
                            trait_: "Tr".into(),
                            name: "t".into(),
                            types: Vec::new(),
                        },
                        recv: Some(Expr::local("x", Ty::Unknown).boxed()),
                        args: Vec::new(),
                    });
                    let named = Expr::new(K::Call {
                        callee: Callee::Trait {
                            trait_: "Tr".into(),
                            name: "t".into(),
                            types: Vec::new(),
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
                    let mut errors = Vec::new();

                    TraitDicts.run(&mut m, &mut errors);

                    assert_eq!(
                        print::module(&m, "trait_dicts"),
                        "// ===== after trait_dicts\n// Root\nfn q(self) -> Q<V> {\n    return Q { t: V {  }, T_Tr: V_Tr };\n}\n\n// Root\nfn en(self) -> Q<En> {\n    return Q { t: En::A, T_Tr: En_Tr };\n}\n\n// Root\nfn func<G: Tr>(self, g: G, G_Tr: dyn Tr<G>) -> u8 {\n    return <Tr>::t(G_Tr, g);\n}\n\n// Root\nfn stat<G: Tr>(self, G_Tr: dyn Tr<G>) -> u8 {\n    return <Tr>::s(G_Tr, );\n}\n\n// Root\nfn refl<R: Refl>(self, r: R) -> u8 {\n    return <Refl>::r(F_Refl, fields::<F>(r));\n}\n\n// Root\nfn lone<L: Lone>(self, l: L) -> u8 {\n    return <Lone>::l(Lone, l);\n}\n\n// Q\nfn get(&self) -> u8 {\n    return <Tr>::t(self.T_Tr, self.t);\n}\n\n// impl Refl for F\nfn r(&self) -> u8 {\n    return 5;\n}\n\n// impl Tr for V\nfn t(&self) -> u8 {\n    return 1;\n}\n\n// impl Tr for V\nfn s() -> u8 {\n    return 2;\n}\n\nfn made() -> () {\n    <Tr>::t(Tr, x);\n    <Tr>::t(V_Tr, v);\n    Q { T_Tr: Tr };\n    V {  };\n}\n\n"
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
                        m.dicts
                            .iter()
                            .map(|(t, of)| format!("{t} {of}"))
                            .collect::<Vec<_>>(),
                        ["Refl F", "Tr En", "Tr V"]
                    );
                    assert_eq!(
                        strings(&errors),
                        [
                            "`Lone`: no one class implementing it for the values of the callers",
                            "`En`: no impl of `Tr` in the module for its dictionary"
                        ]
                    );
                }
            }
        }
    }
}
