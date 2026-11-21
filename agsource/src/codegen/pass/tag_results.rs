use super::Pass;
use super::dict_field;
use crate::Error;
use crate::codegen::lowered::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// The values the builders make tagged with the variant of the data type
/// they convert into, which Rust callers get from `.into()`: a target
/// without static dispatch cannot pick the `From` impl. A tag is a
/// [conversion](Module::conversions) into one of the [`Module::data`] types
/// whose body is the variant holding the value.
///
/// The `pub` builder methods returning a tagged type return the data type.
/// So do those returning a generic bounded by a trait whose implementors are
/// all tagged into it, through the method `into_<data>` added to the trait
/// and its dictionaries. The [`TypeCase`](K::TypeCase) arms of a tagged type
/// match its variant. The calls of the methods returning the data type
/// instead are typed with it, and the callers' conversions of their values
/// into it left out. It runs after [`TraitDicts`](super::TraitDicts) and
/// before [`EraseNewtypes`](super::EraseNewtypes), which can make tagged
/// types the same.
pub struct TagResults;

/// The data type and variant of a tagged type.
type Tags = BTreeMap<String, (String, String)>;

impl Pass for TagResults {
    fn name(&self) -> &str {
        "tag_results"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        let tags = tags(m);
        let into = tagged_traits(m, &tags);

        for (tr, data) in &into {
            let dicts = &m.dicts;
            let implementors = m
                .classes
                .iter_mut()
                .filter(|(of, _)| dicts.contains(&(tr.clone(), (*of).clone())));

            for (of, class) in implementors {
                let (_, var) = &tags[of];
                let value = Expr::variant(data, var, Some(Expr::typed(K::This, Ty::named(of))));
                let f = method(data, vec![Expr::new(K::Return(Some(value.boxed())))]);
                class.trait_impls.entry(tr.clone()).or_default().push(f);
            }

            m.traits
                .entry(tr.clone())
                .or_default()
                .push(method(data, Vec::new()));
        }

        let builders = &m.builders;
        let mut retagged = Retagged::new();

        for (name, class) in m.classes.iter_mut().filter(|(n, _)| builders.contains(n)) {
            let dicts: Vec<String> = class.fields.iter().map(|(n, _)| n.clone()).collect();

            for f in class.methods.iter_mut().filter(|f| !f.private) {
                if let Some((data, var)) = f.ret.name().and_then(|n| tags.get(n)) {
                    retagged.insert((name.clone(), f.name.clone()), data.clone());
                    f.ret = Ty::named(data);
                    rewrite_returns(&mut f.body, &|e| Expr::variant(data, var, Some(e)));
                } else if let Ty::Param(p) = &f.ret
                    && let Some((tr, data)) = into
                        .iter()
                        .find(|(tr, _)| dicts.contains(&dict_field(p, tr)))
                {
                    let dict = dict_field(p, tr);
                    retagged.insert((name.clone(), f.name.clone()), data.clone());
                    f.ret = Ty::named(data);
                    rewrite_returns(&mut f.body, &|e| {
                        Expr::typed(
                            K::Call {
                                callee: Callee::Trait {
                                    trait_: tr.clone(),
                                    name: method_name(data),
                                    types: Vec::new(),
                                },
                                recv: Some(
                                    Expr::typed(K::This, Ty::Unknown)
                                        .field(&dict, Ty::Unknown)
                                        .boxed(),
                                ),
                                args: vec![e],
                            },
                            Ty::named(data),
                        )
                    });
                }
            }
        }

        m.for_each_func(&mut |f, _| rewrite_body(&mut f.body, &mut |e| retype(e, &retagged)));

        for f in &mut m.functions {
            rewrite_body(&mut f.body, &mut |e| {
                if let K::TypeCase { arms, .. } = &mut e.k {
                    for (ty, pat, _) in arms.iter_mut() {
                        if let Some((data, var)) = ty.name().and_then(|n| tags.get(n)) {
                            *ty = Ty::named(data);
                            *pat = Pat::Variant {
                                en: data.clone(),
                                var: var.clone(),
                                inner: vec![pat.clone()],
                            };
                        }
                    }
                }
            });
        }
    }
}

/// The data type the builder methods (class, name) return instead.
type Retagged = BTreeMap<(String, String), String>;

/// A call of a retagged method typed with its data type, and wrapped into the
/// data type by the variant the caller converted it with (`QueryType::from`)
/// as it is.
fn retype(e: &mut Expr, retagged: &Retagged) {
    match &mut e.k {
        K::Call {
            callee: Callee::Member { class, name },
            ..
        } => {
            if let Some(data) = retagged.get(&(class.clone(), name.clone())) {
                e.ty = Ty::named(data);
            }
        }
        K::Variant {
            en,
            payload: Some(value),
            ..
        } if matches!(&value.k, K::Call { callee: Callee::Member { class, name }, .. }
                if retagged.get(&(class.clone(), name.clone())) == Some(en)) =>
        {
            *e = std::mem::replace(value.as_mut(), Expr::new(K::This));
        }
        _ => {}
    }
}

/// The types converted into a data type by its variant holding them.
fn tags(m: &Module) -> Tags {
    let mut out = Tags::new();

    for c in m.conversions.iter().filter(|c| m.data.contains(&c.to)) {
        if let Ty::Named(from, _) = &c.from
            && let K::Variant {
                en,
                var,
                payload: Some(value),
            } = &c.body.k
            && *en == c.to
            && value.k == K::Local("v".into())
        {
            out.insert(from.clone(), (c.to.clone(), var.clone()));
        }
    }

    out
}

/// The data type of the traits whose dictionaries are all of types tagged
/// into it.
fn tagged_traits(m: &Module, tags: &Tags) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();

    for tr in m.traits.keys() {
        let data: Option<BTreeSet<&String>> = m
            .dicts
            .iter()
            .filter(|(t, _)| t == tr)
            .map(|(_, of)| tags.get(of).map(|(d, _)| d))
            .collect();

        if let Some(data) = data
            && let [data] = data.into_iter().collect::<Vec<_>>()[..]
        {
            out.insert(tr.clone(), data.clone());
        }
    }

    out
}

/// `into_<data>` in snake case.
fn method_name(data: &str) -> String {
    format!("into_{}", snake_case(data))
}

/// The type name `name` in snake case.
pub fn snake_case(name: &str) -> String {
    let mut out = String::new();

    for c in name.chars() {
        if c.is_uppercase() && !out.is_empty() {
            out.push('_');
        }

        out.extend(c.to_lowercase());
    }

    out
}

fn method(data: &str, body: Vec<Expr>) -> Func {
    let mut f = Func::new(&method_name(data), Vec::new(), Ty::named(data), body);
    f.recv = Some(Recv::Value);
    f
}

fn rewrite_returns(body: &mut [Expr], wrap: &dyn Fn(Expr) -> Expr) {
    rewrite_body(body, &mut |e| {
        if let K::Return(Some(value)) = &mut e.k {
            let inner = std::mem::replace(value.as_mut(), Expr::new(K::This));
            **value = wrap(inner);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::pass::LowerConversions;
    use crate::codegen::pass::TraitDicts;
    use crate::codegen::print;
    use crate::codegen::test_utilities::resolved;

    mod tag_results {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_tag_results() {
                    assert_eq!(TagResults.name(), "tag_results");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_return_the_data_type_the_results_convert_into_by_their_variant() {
                    let mut m = resolved(
                        r#"
                        pub enum Wire {
                            A(Qa),
                            B(Qb),
                        }
                        pub struct Qa {
                            pub n: u8,
                        }
                        pub struct Qb;
                        pub struct Untagged;
                        impl From<Qa> for Wire {
                            fn from(value: Qa) -> Self {
                                Wire::A(value)
                            }
                        }
                        impl From<Qb> for Wire {
                            fn from(value: Qb) -> Self {
                                Wire::B(value)
                            }
                        }
                        impl From<Untagged> for Wire {
                            fn from(value: Untagged) -> Self {
                                Wire::B(Qb {})
                            }
                        }
                        pub trait Tr {
                            fn get(&self) -> u8;
                        }
                        impl Tr for Qa {
                            fn get(&self) -> u8 {
                                self.n
                            }
                        }
                        impl Tr for Qb {
                            fn get(&self) -> u8 {
                                0
                            }
                        }
                        pub trait Other {
                            fn other(&self) -> u8;
                        }
                        impl Other for Untagged {
                            fn other(&self) -> u8 {
                                1
                            }
                        }
                        pub struct G<T: Tr>(T);
                        impl<T: Tr> G<T> {
                            pub fn query(self) -> T {
                                self.0
                            }
                            pub fn get(&self) -> u8 {
                                self.0.get()
                            }
                        }
                        pub struct H<T: Other>(T);
                        impl<T: Other> H<T> {
                            pub fn query(self) -> T {
                                self.0
                            }
                            pub fn other(&self) -> u8 {
                                self.0.other()
                            }
                        }
                        pub struct Root {
                            pub w: Wire,
                        }
                        impl Root {
                            pub fn a(&self) -> Qa {
                                Qa { n: 1 }
                            }
                            pub fn wrapped(&self) -> Wire {
                                Wire::A(self.a())
                            }
                            pub fn g(&self) -> G<Qb> {
                                G(Qb {})
                            }
                            pub fn h(&self) -> H<Untagged> {
                                H(Untagged {})
                            }
                            pub fn take<W: Into<Wire>>(mut self, w: W) -> Root {
                                self.w = w.into();
                                self
                            }
                            pub fn n(&self) -> u8 {
                                self.private().n
                            }
                            fn private(&self) -> Qa {
                                Qa { n: 2 }
                            }
                        }
                        "#,
                        &["Tr", "Other"],
                        &["Wire"],
                    );
                    LowerConversions.run(&mut m, &mut Vec::new());
                    TraitDicts.run(&mut m, &mut Vec::new());

                    TagResults.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "tag_results"),
                        "// ===== after tag_results\n// Root\nfn a(&self) -> Wire {\n    return Wire::A(Qa { n: 1 });\n}\n\n// Root\nfn wrapped(&self) -> Wire {\n    return self.a();\n}\n\n// Root\nfn g(&self) -> G<Qb> {\n    return G { 0: Qb {  }, T_Tr: Qb_Tr };\n}\n\n// Root\nfn h(&self) -> H<Untagged> {\n    return H { 0: Untagged {  }, T_Other: Untagged_Other };\n}\n\n// Root\nfn take(mut self, w: impl Into<Wire>) -> Root {\n    self.w = w.into::<Wire>();\n    return self;\n}\n\n// Root\nfn n(&self) -> u8 {\n    return self.private().n;\n}\n\n// Root\nfn private(&self) -> Qa {\n    return Qa { n: 2 };\n}\n\n// H\nfn query(self) -> T {\n    return self.0;\n}\n\n// H\nfn other(&self) -> u8 {\n    return <Other>::other(self.T_Other, self.0);\n}\n\n// G\nfn query(self) -> Wire {\n    return <Tr>::into_wire(self.T_Tr, self.0);\n}\n\n// G\nfn get(&self) -> u8 {\n    return <Tr>::get(self.T_Tr, self.0);\n}\n\n// impl Tr for Qa\nfn get(&self) -> u8 {\n    return self.n;\n}\n\n// impl Tr for Qb\nfn get(&self) -> u8 {\n    return 0;\n}\n\n// impl Tr for Qb\nfn into_wire(self) -> Wire {\n    return Wire::B(self);\n}\n\n// impl Other for Untagged\nfn other(&self) -> u8 {\n    return 1;\n}\n\nfn into_Wire(v: impl Into<Wire>) -> Wire {\n    return typecase v { Wire@v => v, Wire@Wire::A(v) => Wire::A(v), Wire@Wire::B(v) => Wire::B(v), Untagged@v => Wire::B(Qb {  }) };\n}\n\n"
                    );
                    assert_eq!(
                        m.traits
                            .iter()
                            .map(|(t, fs)| format!(
                                "{t}: {}",
                                fs.iter()
                                    .map(|f| f.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            ))
                            .collect::<Vec<_>>(),
                        ["Other: other", "Tr: get into_wire"]
                    );
                }
            }
        }
    }

    mod snake_case {
        use super::*;

        #[test]
        fn snake_case_should_separate_the_words_of_a_type_name() {
            assert_eq!(snake_case("QueryType"), "query_type");
        }
    }
}
