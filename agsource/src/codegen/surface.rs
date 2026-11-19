//! [`SourceCrate::external_api`]: the types of other crates the public API
//! of a crate carries.

use super::ir;
use super::ir::Target;
use crate::Error;
use crate::ItemId;
use crate::Namespace;
use crate::Resolved;
use crate::SourceCrate;
use std::collections::BTreeSet;
use std::collections::HashSet;

impl SourceCrate {
    /// The paths into other crates the types of the public API name: of the
    /// `pub` fields and the variants of the exported types, and of the
    /// signatures of their `pub` inherent methods, of the exported functions
    /// and of the methods of the exported traits. Bounds only constrain the
    /// callers' types and are left out, apart from the arguments of
    /// `impl Trait` and `dyn Trait`.
    pub fn external_api(&self) -> (BTreeSet<Vec<String>>, Vec<Error>) {
        let exported: HashSet<ItemId> = self
            .exports()
            .iter()
            .filter_map(|e| match e.target() {
                Resolved::Item(id) => Some(*id),
                _ => None,
            })
            .collect();
        let mut ids: Vec<ItemId> = exported.iter().copied().collect();

        for id in self.item_ids() {
            if let syn::Item::Impl(i) = self.item(id)
                && i.trait_.is_none()
                && let syn::Type::Path(ty) = &*i.self_ty
                && let Resolved::Item(ty) = self.resolve(id.module, &ty.path, Namespace::Type)
                && exported.contains(&ty)
            {
                ids.push(id);
            }
        }

        ids.sort();
        let (items, errors) = self.ir(ids);
        let mut out = BTreeSet::new();

        for (_, item) in &items {
            match &item.kind {
                ir::ItemKind::Struct(s) => {
                    for f in fields(&s.fields).filter(|f| f.vis == ir::Vis::Public) {
                        external(&f.ty, &mut out);
                    }
                }
                ir::ItemKind::Enum(e) => {
                    for f in e.variants.iter().flat_map(|v| fields(&v.fields)) {
                        external(&f.ty, &mut out);
                    }
                }
                ir::ItemKind::Fn(f) => signature(&f.sig, &mut out),
                ir::ItemKind::Trait(t) => {
                    for i in &t.items {
                        if let ir::TraitItemKind::Fn { sig, .. } = &i.kind {
                            signature(sig, &mut out);
                        }
                    }
                }
                ir::ItemKind::Impl(i) => {
                    for i in i.items.iter().filter(|i| i.vis == ir::Vis::Public) {
                        if let ir::ImplItemKind::Fn(f) = &i.kind {
                            signature(&f.sig, &mut out);
                        }
                    }
                }
                _ => {}
            }
        }

        (out, errors)
    }
}

fn fields(fields: &ir::Fields) -> impl Iterator<Item = &ir::Field> {
    match fields {
        ir::Fields::Named(f) | ir::Fields::Unnamed(f) => f.as_slice(),
        ir::Fields::Unit => &[],
    }
    .iter()
}

fn signature(sig: &ir::Signature, out: &mut BTreeSet<Vec<String>>) {
    for p in &sig.params {
        external(&p.ty, out);
    }

    if let Some(t) = &sig.output {
        external(t, out);
    }
}

fn external(t: &ir::Type, out: &mut BTreeSet<Vec<String>>) {
    match t {
        ir::Type::Path(p) => path(p, out),
        ir::Type::Qualified(q) => external(&q.ty, out),
        ir::Type::Ref { ty, .. }
        | ir::Type::Slice(ty)
        | ir::Type::Array(ty, _)
        | ir::Type::Paren(ty) => external(ty, out),
        ir::Type::Tuple(types) => types.iter().for_each(|t| external(t, out)),
        ir::Type::ImplTrait(bounds) | ir::Type::DynTrait(bounds) => {
            for b in bounds {
                if let ir::Bound::Trait(p) = b {
                    args(p, out);
                }
            }
        }
        ir::Type::FnPtr(f) => fn_types(f, out),
        ir::Type::Infer => {}
    }
}

fn path(p: &ir::Path, out: &mut BTreeSet<Vec<String>>) {
    if let Target::Resolved(Resolved::External(segments)) = &p.target {
        out.insert(segments.clone());
    }

    args(p, out);
}

fn args(p: &ir::Path, out: &mut BTreeSet<Vec<String>>) {
    for s in &p.segments {
        match &s.args {
            ir::Args::Angle(args) => {
                for a in args {
                    if let ir::GenericArg::Type(t) = a {
                        external(t, out);
                    }
                }
            }
            ir::Args::Fn(f) => fn_types(f, out),
            ir::Args::None => {}
        }
    }
}

fn fn_types(f: &ir::FnTypes, out: &mut BTreeSet<Vec<String>>) {
    f.inputs.iter().for_each(|t| external(t, out));

    if let Some(t) = &f.output {
        external(t, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cfg;
    use crate::Target;
    use crate::TargetKind;
    use agplatform::Fs;
    use agplatform::Platform;
    use agplatform::test_platform::test_platform;

    mod source_crate {
        use super::*;

        mod external_api {
            use super::*;

            #[test]
            fn external_api_should_be_the_paths_into_other_crates_the_public_types_name() {
                let platform = test_platform();
                platform.fs().create_dir_all("/c/src").unwrap();
                platform
                    .fs()
                    .write(
                        "/c/src/lib.rs",
                        r#"
use dep::Wire;
pub struct Api {
    pub queries: Vec<dep::Query>,
    hidden: dep::Hidden,
    pub tuple: (dep::T1, &'static [dep::T2], [dep::T3; 1], (dep::Paren)),
}
pub struct Pair(pub dep::P, dep::Private);
pub struct Unit;
pub enum E {
    A(dep::Ea),
    B { b: Box<dep::Eb> },
    C,
}
pub fn f<B: dep::Bound>(x: impl Into<dep::Imp> + 'static, g: fn(dep::FnIn) -> dep::FnOut) -> Option<dep::Ret> {
    None
}
pub fn unit(f: fn(dep::NoOut)) {}
pub trait Tr {
    fn t(&self, d: &dyn AsRef<dep::Dyn>) -> <dep::Q as dep::QTr>::Out;
    type Out;
}
impl Api {
    pub fn m(&self, w: Wire) -> Box<dyn Fn(dep::FnArg) -> dep::FnRet> {
        panic!()
    }
    pub fn plain(&self) {}
    fn private(&self, x: dep::NotApi) {}
    pub type Alias = dep::NotFn;
}
impl dep::Ext for Api {
    fn e(&self, x: dep::TraitImpl) {}
}
impl Unknown {}
impl<T> Vec<T> {}
mod private {
    pub struct Hidden2 {
        pub x: dep::NotExported,
    }
}
pub const C: u8 = 1;
pub mod m {}
pub fn inferred(v: Vec<_>) {}
pub fn local(u: Unit) {}
pub fn cow(c: std::borrow::Cow<'static, str>) {}
"#,
                    )
                    .unwrap();
                let target = Target {
                    externs: vec!["dep".to_string()],
                    ..Target::new(TargetKind::Lib, "/c/src/lib.rs")
                };
                let c = SourceCrate::load(&platform, &target, &Cfg::All);

                let (paths, errors) = c.external_api();

                assert_eq!(
                    paths.iter().map(|p| p.join("::")).collect::<Vec<_>>(),
                    [
                        "dep::Dyn",
                        "dep::Ea",
                        "dep::Eb",
                        "dep::FnArg",
                        "dep::FnIn",
                        "dep::FnOut",
                        "dep::FnRet",
                        "dep::Imp",
                        "dep::NoOut",
                        "dep::P",
                        "dep::Paren",
                        "dep::Q",
                        "dep::Query",
                        "dep::Ret",
                        "dep::T1",
                        "dep::T2",
                        "dep::T3",
                        "dep::Wire",
                        "std::borrow::Cow",
                        "std::boxed::Box",
                        "std::option::Option",
                        "std::vec::Vec"
                    ]
                );
                assert_eq!(
                    errors.iter().map(ToString::to_string).collect::<Vec<_>>(),
                    Vec::<String>::new()
                );
            }
        }
    }
}
