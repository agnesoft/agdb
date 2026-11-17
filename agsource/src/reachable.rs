use crate::ItemId;
use crate::MacroCall;
use crate::ModuleId;
use crate::Namespace;
use crate::Resolved;
use crate::SourceCrate;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use syn::Item;
use syn::ext::IdentExt;
use syn::visit::Visit;

/// A public name of the crate: its path from the root through `pub`
/// modules and re-exports only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    path: Vec<String>,
    namespace: Namespace,
    target: Resolved,
}

/// What [`SourceCrate::reachable`] reaches from its roots.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Reachable {
    /// The items and impls, the opaque ones included.
    pub items: BTreeSet<ItemId>,
    /// The item that first reached an item, none for the roots.
    pub reached_from: BTreeMap<ItemId, ItemId>,
    /// The paths into other crates.
    pub external: BTreeSet<Vec<String>>,
    /// The paths that name nothing, with the item they are in: the items and
    /// imports of blocks, which are not resolved.
    pub unresolved: BTreeSet<(ItemId, String)>,
    /// The macro calls that are not parsed, with the item they are in and
    /// why (see [`SourceCrate::macro_call`]).
    pub unparsed: BTreeSet<(ItemId, String)>,
}

impl Export {
    /// The segments after the crate name.
    pub fn path(&self) -> &[String] {
        &self.path
    }

    pub fn namespace(&self) -> Namespace {
        self.namespace
    }

    pub fn target(&self) -> &Resolved {
        &self.target
    }
}

impl SourceCrate {
    /// The public names of the crate, sorted. The names of a module reached
    /// by several paths are listed under the shortest one, and the names of
    /// glob imports of other crates are unknown.
    pub fn exports(&self) -> Vec<Export> {
        let mut exports = Vec::new();
        let mut visited = HashSet::from([self.root()]);
        let mut queue = VecDeque::from([(self.root(), Vec::new())]);

        while let Some((module, prefix)) = queue.pop_front() {
            for ((namespace, name), target) in self.scopes.public_names(self, module) {
                let mut path: Vec<String> = prefix.clone();
                path.push(name);

                if let Resolved::Module(m) = target
                    && visited.insert(m)
                {
                    queue.push_back((m, path.clone()));
                }

                exports.push(Export {
                    path,
                    namespace,
                    target,
                });
            }
        }

        exports.sort_by(|a, b| (&a.path, a.namespace).cmp(&(&b.path, b.namespace)));
        exports
    }

    /// The items the `roots` lead to through the paths in them: field,
    /// variant and signature types, bounds, and the paths in bodies, plus the
    /// impls of the types reached, inherent or of a trait that is reached,
    /// exported or of another crate. A module root is its `pub` functions and those of
    /// its `pub` submodules. An `opaque` item is reached but not followed,
    /// and neither are the impls of an opaque trait. Macro calls are
    /// followed as [`SourceCrate::macro_call`] parses them, the expansions of
    /// `macro_rules!` from the module of the call.
    pub fn reachable(&self, roots: &[&Export], opaque: &[ItemId]) -> Reachable {
        let opaque: HashSet<ItemId> = opaque.iter().copied().collect();
        let exported: HashSet<ItemId> = self
            .exports()
            .iter()
            .filter_map(|export| match export.target {
                Resolved::Item(id) => Some(id),
                _ => None,
            })
            .collect();
        let impls = self.impls();
        let mut reach = Reach::default();
        // The impls of local traits not reached yet, with their type.
        let mut pending: HashMap<ItemId, Vec<(ItemId, ItemId)>> = HashMap::new();

        for root in roots {
            match &root.target {
                Resolved::Module(module) => self.module_root(*module, &mut reach),
                target => reach.add(target, None),
            }
        }

        while let Some(id) = reach.queue.pop() {
            if opaque.contains(&id) {
                continue;
            }

            let item = self.item(id);

            if !matches!(item, Item::Macro(_)) {
                let mut paths = Paths::new(self, id, item);
                paths.visit_item(item);

                for (target, path) in paths.found {
                    if target == Resolved::Unresolved {
                        reach.unresolved.insert((id, path));
                    } else {
                        reach.add(&target, Some(id));
                    }
                }

                reach
                    .unparsed
                    .extend(paths.unparsed.into_iter().map(|reason| (id, reason)));
            }

            if matches!(item, Item::Trait(_)) {
                for (imp, self_ty) in pending.remove(&id).into_iter().flatten() {
                    reach.push(imp, Some(self_ty));
                }
            }

            if matches!(item, Item::Struct(_) | Item::Enum(_) | Item::Union(_)) {
                for (imp, trait_) in impls.get(&id).into_iter().flatten() {
                    match trait_ {
                        Some(t) if opaque.contains(t) => {}
                        Some(t) if !exported.contains(t) && !reach.items.contains(t) => {
                            pending.entry(*t).or_default().push((*imp, id));
                        }
                        _ => reach.push(*imp, Some(id)),
                    }
                }
            }
        }

        Reachable {
            items: reach.items,
            reached_from: reach.reached_from,
            external: reach.external,
            unresolved: reach.unresolved,
            unparsed: reach.unparsed,
        }
    }

    fn module_root(&self, module: ModuleId, reach: &mut Reach) {
        for (index, item) in self.items(module).iter().enumerate() {
            if let Item::Fn(f) = item
                && matches!(f.vis, syn::Visibility::Public(_))
            {
                reach.push(ItemId { module, index }, None);
            }
        }

        for &child in self.module(module).children() {
            if let Some(Item::Mod(m)) = self.module(child).item().map(|id| self.item(id))
                && matches!(m.vis, syn::Visibility::Public(_))
            {
                self.module_root(child, reach);
            }
        }
    }

    /// The impls of the module items by the local type they are for, with
    /// their local trait.
    fn impls(&self) -> HashMap<ItemId, Vec<(ItemId, Option<ItemId>)>> {
        let mut impls: HashMap<ItemId, Vec<(ItemId, Option<ItemId>)>> = HashMap::new();

        for (module, _) in self.modules() {
            for (index, item) in self.items(module).iter().enumerate() {
                let Item::Impl(imp) = item else {
                    continue;
                };
                let local = |path: &syn::Path| match self.resolve(module, path, Namespace::Type) {
                    Resolved::Item(id) => Some(id),
                    _ => None,
                };

                if let Some(self_ty) = type_path(&imp.self_ty).and_then(local) {
                    let trait_ = imp.trait_.as_ref().and_then(|(path, _)| local(path));
                    impls
                        .entry(self_ty)
                        .or_default()
                        .push((ItemId { module, index }, trait_));
                }
            }
        }

        impls
    }
}

#[derive(Default)]
struct Reach {
    items: BTreeSet<ItemId>,
    reached_from: BTreeMap<ItemId, ItemId>,
    queue: Vec<ItemId>,
    external: BTreeSet<Vec<String>>,
    unresolved: BTreeSet<(ItemId, String)>,
    unparsed: BTreeSet<(ItemId, String)>,
}

impl Reach {
    fn add(&mut self, target: &Resolved, from: Option<ItemId>) {
        match target {
            Resolved::Item(id) | Resolved::Variant(id, _) => self.push(*id, from),
            Resolved::Associated(base, _) => self.add(base, from),
            Resolved::External(path) => {
                self.external.insert(path.clone());
            }
            Resolved::Module(_) | Resolved::Primitive(_) | Resolved::Unresolved => {}
        }
    }

    fn push(&mut self, id: ItemId, from: Option<ItemId>) {
        if self.items.insert(id) {
            self.queue.push(id);

            if let Some(from) = from {
                self.reached_from.insert(id, from);
            }
        }
    }
}

/// The paths of an item, resolved from its module, apart from those that
/// start with `Self` or a generic parameter of the item and the single
/// names of its local bindings, as far as it binds them anywhere. The
/// macro calls are parsed and their parts visited.
struct Paths<'a> {
    c: &'a SourceCrate,
    module: ModuleId,
    generics: HashSet<String>,
    locals: HashSet<String>,
    found: Vec<(Resolved, String)>,
    unparsed: Vec<String>,
    /// The macro calls being visited, which expand in one another.
    depth: usize,
}

/// rustc's default `recursion_limit`.
const RECURSION_LIMIT: usize = 128;

#[derive(Default)]
struct Bindings {
    generics: HashSet<String>,
    locals: HashSet<String>,
}

impl<'a> Paths<'a> {
    fn new(c: &'a SourceCrate, id: ItemId, item: &Item) -> Self {
        let mut bindings = Bindings::default();
        bindings.visit_item(item);

        Self {
            c,
            module: id.module(),
            generics: bindings.generics,
            locals: bindings.locals,
            found: Vec::new(),
            unparsed: Vec::new(),
            depth: 0,
        }
    }

    fn path(&mut self, qself: Option<&syn::QSelf>, path: &syn::Path, namespace: Namespace) {
        // `<T as Trait>::name` names an item of the trait.
        let (path, namespace) = match qself {
            Some(qself) if qself.position == 0 => return,
            Some(qself) => {
                let mut segments = path.segments.clone();

                while segments.len() > qself.position {
                    segments.pop();
                }

                let trait_path = syn::Path {
                    leading_colon: path.leading_colon,
                    segments: segments.into_pairs().map(|p| p.into_value()).collect(),
                };
                (std::borrow::Cow::Owned(trait_path), Namespace::Type)
            }
            None => (std::borrow::Cow::Borrowed(path), namespace),
        };
        let first = path.segments[0].ident.unraw().to_string();

        if first == "Self"
            || self.generics.contains(&first)
            || (path.segments.len() == 1
                && namespace == Namespace::Value
                && self.locals.contains(&first))
        {
            return;
        }

        let target = self.c.resolve(self.module, &path, namespace);
        self.found.push((target, display(&path)));
    }

    fn call(&mut self, m: &syn::Macro, target: &Resolved) {
        let name = display(&m.path);

        if self.depth == RECURSION_LIMIT {
            self.unparsed
                .push(format!("{name}!: the recursion limit is reached"));
            return;
        }

        match self.c.call(target, m) {
            Ok(call) => {
                let mut bindings = Bindings::default();
                call.visit(&mut bindings);
                self.locals.extend(bindings.locals);
                self.depth += 1;
                call.visit(self);
                self.depth -= 1;

                if let MacroCall::Format {
                    format: Some(format),
                    ..
                } = &call
                {
                    for capture in &format.captures {
                        self.path(None, &syn::Path::from(capture.clone()), Namespace::Value);
                    }
                }
            }
            Err(reason) => self.unparsed.push(format!("{name}!: {reason}")),
        }
    }
}

impl<'ast> Visit<'ast> for Paths<'_> {
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {}

    fn visit_visibility(&mut self, _: &'ast syn::Visibility) {}

    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {}

    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let target = self.c.resolve(self.module, &m.path, Namespace::Macro);

        if target != Resolved::Unresolved {
            self.call(m, &target);
        }

        self.found.push((target, display(&m.path)));
    }

    fn visit_type_path(&mut self, t: &'ast syn::TypePath) {
        self.path(t.qself.as_ref(), &t.path, Namespace::Type);
        syn::visit::visit_type_path(self, t);
    }

    fn visit_expr_path(&mut self, e: &'ast syn::ExprPath) {
        self.path(e.qself.as_ref(), &e.path, Namespace::Value);
        syn::visit::visit_expr_path(self, e);
    }

    fn visit_expr_struct(&mut self, e: &'ast syn::ExprStruct) {
        self.path(e.qself.as_ref(), &e.path, Namespace::Type);
        syn::visit::visit_expr_struct(self, e);
    }

    fn visit_pat_struct(&mut self, p: &'ast syn::PatStruct) {
        self.path(p.qself.as_ref(), &p.path, Namespace::Type);
        syn::visit::visit_pat_struct(self, p);
    }

    fn visit_pat_tuple_struct(&mut self, p: &'ast syn::PatTupleStruct) {
        self.path(p.qself.as_ref(), &p.path, Namespace::Value);
        syn::visit::visit_pat_tuple_struct(self, p);
    }

    fn visit_trait_bound(&mut self, b: &'ast syn::TraitBound) {
        self.path(None, &b.path, Namespace::Type);
        syn::visit::visit_trait_bound(self, b);
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        if let Some((path, _)) = &i.trait_ {
            self.path(None, path, Namespace::Type);
        }

        syn::visit::visit_item_impl(self, i);
    }
}

impl<'ast> Visit<'ast> for Bindings {
    fn visit_generic_param(&mut self, p: &'ast syn::GenericParam) {
        match p {
            syn::GenericParam::Type(t) => {
                self.generics.insert(t.ident.unraw().to_string());
            }
            syn::GenericParam::Const(c) => {
                self.generics.insert(c.ident.unraw().to_string());
            }
            syn::GenericParam::Lifetime(_) => {}
        }

        syn::visit::visit_generic_param(self, p);
    }

    fn visit_pat_ident(&mut self, p: &'ast syn::PatIdent) {
        self.locals.insert(p.ident.unraw().to_string());
        syn::visit::visit_pat_ident(self, p);
    }
}

/// The path of a type, through references and parentheses.
fn type_path(ty: &syn::Type) -> Option<&syn::Path> {
    match ty {
        syn::Type::Path(t) if t.qself.is_none() => Some(&t.path),
        syn::Type::Reference(r) => type_path(&r.elem),
        syn::Type::Paren(p) => type_path(&p.elem),
        _ => None,
    }
}

fn display(path: &syn::Path) -> String {
    let segments: Vec<String> = path
        .segments
        .iter()
        .map(|s| s.ident.unraw().to_string())
        .collect();
    let leading = if path.leading_colon.is_some() {
        "::"
    } else {
        ""
    };
    format!("{leading}{}", segments.join("::"))
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

    fn lib(source: &str) -> SourceCrate {
        let platform = test_platform();
        platform.fs().create_dir_all("/c/src").unwrap();
        platform.fs().write("/c/src/lib.rs", source).unwrap();
        let target = Target {
            externs: vec!["dep".to_string()],
            ..Target::new(TargetKind::Lib, "/c/src/lib.rs")
        };
        SourceCrate::load(&platform, &target, &Cfg::All)
    }

    /// The item named `name`, the first of its modules in order.
    fn id(c: &SourceCrate, name: &str) -> ItemId {
        c.modules()
            .flat_map(|(module, _)| {
                c.items(module)
                    .iter()
                    .enumerate()
                    .map(move |(index, item)| (ItemId { module, index }, item))
            })
            .find(|(_, item)| {
                let ident = match item {
                    Item::Struct(s) => &s.ident,
                    Item::Enum(e) => &e.ident,
                    Item::Trait(t) => &t.ident,
                    Item::Fn(f) => &f.sig.ident,
                    Item::Const(k) => &k.ident,
                    Item::Static(s) => &s.ident,
                    Item::Type(t) => &t.ident,
                    Item::Macro(m) => m.ident.as_ref().unwrap(),
                    _ => return false,
                };
                ident == name
            })
            .unwrap()
            .0
    }

    /// The impl for the type named `self_ty`, of the trait named `trait_`.
    fn imp(c: &SourceCrate, self_ty: &str, trait_: Option<&str>) -> ItemId {
        let last = |path: &syn::Path| path.segments.last().unwrap().ident.to_string();
        c.modules()
            .flat_map(|(module, _)| {
                c.items(module)
                    .iter()
                    .enumerate()
                    .map(move |(index, item)| (ItemId { module, index }, item))
            })
            .find(|(_, item)| {
                matches!(item, Item::Impl(i)
                    if type_path(&i.self_ty).is_some_and(|p| last(p) == self_ty)
                        && i.trait_.as_ref().map(|(p, _)| last(p)).as_deref() == trait_)
            })
            .unwrap()
            .0
    }

    fn export<'a>(exports: &'a [Export], path: &str) -> &'a Export {
        exports
            .iter()
            .find(|e| e.path().join("::") == path && e.namespace() == Namespace::Type)
            .unwrap()
    }

    fn reachable(c: &SourceCrate, roots: &[&str], opaque: &[&str]) -> Reachable {
        let exports = c.exports();
        let roots: Vec<&Export> = roots.iter().map(|root| export(&exports, root)).collect();
        let opaque: Vec<ItemId> = opaque.iter().map(|name| id(c, name)).collect();
        c.reachable(&roots, &opaque)
    }

    mod source_crate {
        use super::*;

        mod exports {
            use super::*;

            fn paths(c: &SourceCrate) -> Vec<(String, Namespace)> {
                c.exports()
                    .iter()
                    .map(|e| (e.path().join("::"), e.namespace()))
                    .collect()
            }

            #[test]
            fn exports_should_list_the_pub_names_through_pub_modules() {
                let c = lib("pub struct A;
struct Hidden;
pub mod m {
    pub fn f() {}
    pub(crate) fn g() {}
    pub mod inner { pub const C: u8 = 0; }
}
mod private { pub struct P {} }
pub use private::P as Q;
#[macro_export]
macro_rules! mac { () => {} }");

                assert_eq!(
                    paths(&c),
                    vec![
                        ("A".to_string(), Namespace::Type),
                        ("A".to_string(), Namespace::Value),
                        ("Q".to_string(), Namespace::Type),
                        ("m".to_string(), Namespace::Type),
                        ("m::f".to_string(), Namespace::Value),
                        ("m::inner".to_string(), Namespace::Type),
                        ("m::inner::C".to_string(), Namespace::Value),
                        ("mac".to_string(), Namespace::Macro),
                    ]
                );
                assert_eq!(
                    export(&c.exports(), "Q").target(),
                    &Resolved::Item(id(&c, "P"))
                );
            }

            #[test]
            fn exports_should_expand_the_pub_globs_of_modules_and_enums() {
                let c = lib("mod a { pub struct X {} pub(crate) struct Y {} }
mod b { pub struct Z {} }
pub use a::*;
use b::*;
pub enum E { V, W(u8) }
pub use E::*;
pub use dep::*;
pub struct S {}
pub use S::*;
pub mod cycle { pub use super::*; }");

                assert_eq!(
                    paths(&c)
                        .into_iter()
                        .filter(|(path, _)| !path.starts_with("cycle::"))
                        .collect::<Vec<_>>(),
                    vec![
                        ("E".to_string(), Namespace::Type),
                        ("S".to_string(), Namespace::Type),
                        ("V".to_string(), Namespace::Type),
                        ("W".to_string(), Namespace::Type),
                        ("X".to_string(), Namespace::Type),
                        ("cycle".to_string(), Namespace::Type),
                    ]
                );
                assert_eq!(
                    export(&c.exports(), "V").target(),
                    &Resolved::Variant(id(&c, "E"), 0)
                );
            }

            #[test]
            fn exports_of_a_module_with_several_paths_should_list_its_names_once() {
                let c = lib("pub mod a { pub mod b { pub fn f() {} } }
pub use a::b;");

                assert_eq!(
                    paths(&c),
                    vec![
                        ("a".to_string(), Namespace::Type),
                        ("a::b".to_string(), Namespace::Type),
                        ("b".to_string(), Namespace::Type),
                        ("b::f".to_string(), Namespace::Value),
                    ]
                );
            }
        }

        mod reachable {
            use super::*;

            #[test]
            fn reachable_should_follow_fields_signatures_bounds_and_bodies() {
                let c = lib("pub struct Root<'a> { field: &'a Field }
pub struct Field;
pub struct Unused;
pub trait Bound {}
pub struct Arg;
impl Arg {
    fn make() {}
}
pub enum Ret { A(u8), B { b: u8 } }
pub struct Lit { x: u8 }
fn helper() {}
const CONST: u8 = 0;
static STATIC: u8 = 0;
pub type Alias = Field;
macro_rules! mac { () => {} }
impl<'a> Root<'a> {
    #[inline]
    pub fn new<T: Bound>(arg: Arg) -> Ret {
        helper();
        Arg::make();
        let _ = (CONST, STATIC, Lit { x: 0 });
        let _: Alias;
        mac!();
        match Ret::A(0) {
            Ret::A(_) => {}
            Ret::B { .. } => {}
        }
        Ret::A(0)
    }
}");

                let reachable = reachable(&c, &["Root"], &[]);

                assert_eq!(
                    reachable.items,
                    BTreeSet::from([
                        id(&c, "Root"),
                        id(&c, "Field"),
                        id(&c, "Bound"),
                        id(&c, "Arg"),
                        id(&c, "Ret"),
                        id(&c, "Lit"),
                        id(&c, "helper"),
                        id(&c, "CONST"),
                        id(&c, "STATIC"),
                        id(&c, "Alias"),
                        id(&c, "mac"),
                        imp(&c, "Root", None),
                        imp(&c, "Arg", None),
                    ])
                );
                assert_eq!(reachable.reached_from[&id(&c, "Field")], id(&c, "Root"));
                assert_eq!(
                    reachable.reached_from[&id(&c, "helper")],
                    imp(&c, "Root", None)
                );
                assert!(!reachable.reached_from.contains_key(&id(&c, "Root")));
                assert!(
                    reachable.unresolved.is_empty(),
                    "{:?}",
                    reachable.unresolved
                );
            }

            #[test]
            fn reachable_should_follow_the_impls_of_exported_and_external_traits() {
                let c = lib("pub struct Root;
pub trait Public { fn f(&self) -> A; }
trait Internal { fn g(&self) -> B; }
pub struct A;
pub struct B;
pub struct C;
pub struct D;
impl Public for Root { fn f(&self) -> A { A } }
impl Internal for Root { fn g(&self) -> B { B } }
impl std::fmt::Display for Root {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { C; Ok(()) }
}
impl Public for &(D) { fn f(&self) -> A { A } }
impl Public for [D] { fn f(&self) -> A { A } }
impl Public for u8 { fn f(&self) -> A { A } }");

                let reachable = reachable(&c, &["Root", "D"], &[]);

                assert_eq!(
                    reachable.items,
                    BTreeSet::from([
                        id(&c, "Root"),
                        id(&c, "Public"),
                        id(&c, "A"),
                        id(&c, "C"),
                        id(&c, "D"),
                        imp(&c, "Root", Some("Public")),
                        imp(&c, "Root", Some("Display")),
                        imp(&c, "D", Some("Public")),
                    ])
                );
            }

            #[test]
            fn reachable_should_follow_the_impls_of_local_traits_once_reached() {
                let c = lib("pub struct Root { search: Search<A> }
pub struct Search<T: Builder>(T);
trait Builder { fn next(&self) -> Next; }
pub struct A;
pub struct Next;
pub struct Unused;
impl Builder for A { fn next(&self) -> Next { Next } }
impl Builder for Next { fn next(&self) -> Next { Next } }
impl Builder for Unused { fn next(&self) -> Next { Next } }");

                let reachable = reachable(&c, &["Root"], &[]);

                // WHY: A is followed before the bound of Search reaches
                // Builder, Next after.
                assert_eq!(
                    reachable.items,
                    BTreeSet::from([
                        id(&c, "Root"),
                        id(&c, "Search"),
                        id(&c, "Builder"),
                        id(&c, "A"),
                        id(&c, "Next"),
                        imp(&c, "A", Some("Builder")),
                        imp(&c, "Next", Some("Builder")),
                    ])
                );
                assert_eq!(
                    reachable.reached_from[&imp(&c, "A", Some("Builder"))],
                    id(&c, "A")
                );
            }

            #[test]
            fn reachable_opaque_items_should_be_reached_but_not_followed() {
                let c = lib("pub struct Root { opaque: Opaque }
pub struct Opaque { behind: Behind }
pub struct Behind;
pub trait OpaqueTrait { fn hidden(&self) -> Behind; }
impl Opaque { fn f() -> Behind { Behind } }
impl OpaqueTrait for Root { fn hidden(&self) -> Behind { Behind } }");

                let reachable = reachable(&c, &["Root"], &["Opaque", "OpaqueTrait"]);

                assert_eq!(
                    reachable.items,
                    BTreeSet::from([id(&c, "Root"), id(&c, "Opaque")])
                );
            }

            #[test]
            fn reachable_module_root_should_be_its_pub_fns_and_those_of_its_pub_submodules() {
                let c = lib("pub mod tests {
    pub fn a() {}
    fn private() {}
    pub mod inner { pub fn b() {} }
    mod hidden { pub fn c() {} }
}");

                let reachable = reachable(&c, &["tests"], &[]);

                assert_eq!(reachable.items, BTreeSet::from([id(&c, "a"), id(&c, "b")]));
            }

            #[test]
            fn reachable_should_skip_self_generic_parameters_and_local_bindings() {
                let c = lib("pub struct Root;
pub trait Bound { fn make() -> u8; }
fn local() {}
fn x() {}
fn closure() {}
impl Root {
    pub fn f<T: Bound, const N: usize>(local: u8) -> Self {
        let x = local;
        let _ = (x, N, T::make(), <T as Bound>::make(), <T>::make());
        let _ = |closure: u8| closure;
        Self::g();
        Self
    }
    fn g() {}
}");

                let reachable = reachable(&c, &["Root"], &[]);

                assert_eq!(
                    reachable.items,
                    BTreeSet::from([id(&c, "Root"), id(&c, "Bound"), imp(&c, "Root", None)])
                );
                assert!(
                    reachable.unresolved.is_empty(),
                    "{:?}",
                    reachable.unresolved
                );
            }

            #[test]
            fn reachable_should_report_external_and_unresolved_paths() {
                let c = lib("pub fn root() {
    ::std::mem::drop(Vec::<u8>::new());
    dep::f();
    println!();
    {
        use dep::Local;
        Local::g();
    }
    gone();
}");

                let reachable = reachable(&c, &[], &[]);
                let exports = c.exports();
                let reachable_root = c.reachable(&[export_value(&exports, "root")], &[]);

                assert_eq!(reachable, Reachable::default());
                assert_eq!(
                    reachable_root.external,
                    BTreeSet::from([
                        vec!["dep".to_string(), "f".to_string()],
                        vec!["std".to_string(), "mem".to_string(), "drop".to_string()],
                        vec!["std".to_string(), "println".to_string()],
                        vec![
                            "std".to_string(),
                            "vec".to_string(),
                            "Vec".to_string(),
                            "new".to_string()
                        ],
                    ])
                );
                assert_eq!(
                    reachable_root.unresolved,
                    BTreeSet::from([
                        (id(&c, "root"), "Local::g".to_string()),
                        (id(&c, "root"), "gone".to_string()),
                    ])
                );
            }

            #[test]
            fn reachable_should_follow_macro_arguments_and_expansions() {
                let c = lib(r#"pub struct Root;
pub struct Failure;
impl Failure { fn new(_: String) -> Self { Failure } }
const HOST: &str = "h";
const WIDTH: usize = 1;
const A: u8 = 0;
const N: usize = 1;
fn helper() -> u8 { 0 }
pub enum E { V(u8) }
pub struct Unused;
macro_rules! bail {
    ($fmt:literal) => { return Err(crate::Failure::new(format!($fmt))) };
}
macro_rules! forever { () => { forever!() }; }
impl Root {
    pub fn f(local: u8, e: E) -> Result<(), Failure> {
        let _ = format!("{HOST:WIDTH$} {local} {}", helper());
        assert_eq!(helper(), A);
        let _ = (vec![A], vec![A; N]);
        assert!(matches!(e, E::V(x) if x > local));
        dbg!(Unused);
        forever!();
        gone!();
        bail!("{HOST}");
    }
}"#);

                let reachable = reachable(&c, &["Root"], &[]);

                assert_eq!(
                    reachable.items,
                    BTreeSet::from([
                        id(&c, "Root"),
                        id(&c, "Failure"),
                        id(&c, "HOST"),
                        id(&c, "WIDTH"),
                        id(&c, "A"),
                        id(&c, "N"),
                        id(&c, "helper"),
                        id(&c, "E"),
                        id(&c, "bail"),
                        id(&c, "forever"),
                        imp(&c, "Root", None),
                        imp(&c, "Failure", None),
                    ])
                );
                assert_eq!(
                    reachable.unparsed,
                    BTreeSet::from([
                        (
                            imp(&c, "Root", None),
                            "dbg!: the arguments of std::dbg! are not known".to_string()
                        ),
                        (
                            imp(&c, "Root", None),
                            "forever!: the recursion limit is reached".to_string()
                        ),
                    ])
                );
                assert_eq!(
                    reachable.unresolved,
                    BTreeSet::from([(imp(&c, "Root", None), "gone".to_string())])
                );
            }

            fn export_value<'a>(exports: &'a [Export], path: &str) -> &'a Export {
                exports
                    .iter()
                    .find(|e| e.path().join("::") == path && e.namespace() == Namespace::Value)
                    .unwrap()
            }
        }
    }
}
