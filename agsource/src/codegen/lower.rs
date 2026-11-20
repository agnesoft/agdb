//! [`SourceCrate::lower`]: the lowered IR of a builder API from the IR of
//! its items.

use super::ir;
use super::ir::Target;
use super::lowered::*;
use crate::Error;
use crate::ItemId;
use crate::Namespace;
use crate::Resolved;
use crate::SourceCrate;
use std::collections::HashMap;
use std::collections::HashSet;
use syn::spanned::Spanned;

/// What [`SourceCrate::lower`] starts from.
pub struct Roots {
    /// The struct the builder API starts from, e.g. a `QueryBuilder`.
    pub root: ItemId,
    /// The traits whose impls of the types are loaded.
    pub traits: Vec<ItemId>,
    /// Types no builder reaches that the passes need.
    pub data: Vec<ItemId>,
}

impl Roots {
    /// The builder API making the `data` types. The queries are the types
    /// with a `From` impl into one of them, the makers the structs other
    /// than the queries with a `pub` method returning a query or a maker,
    /// and the root the one maker no maker returns. The traits are those of
    /// the crate bounding the generics of the builders from the root (see
    /// [`SourceCrate::lower`]), of their impls and of their `pub` methods.
    pub fn of(c: &SourceCrate, data: Vec<ItemId>) -> Result<Roots, Error> {
        Lower::new(c, &[]).roots(data)
    }
}

impl SourceCrate {
    /// The builder API from `roots.root`. The builders are the structs the
    /// return types of the builders' `pub` methods lead to that have `pub`
    /// methods with a receiver; the other types reached through fields,
    /// signatures, `roots.data` and the impls of `roots.traits` are data. A
    /// type has its inherent methods (the unused
    /// ones of data types are left to [`Prune`](super::pass::Prune)) and
    /// the methods of its impls of `roots.traits`.
    ///
    /// The bodies are syntactic: method calls by name and paths are left to
    /// [`Resolve`](super::pass::Resolve). A construct the lowered IR does
    /// not have is a [`K::Unsupported`] and among the [`Func::errors`].
    pub fn lower(&self, roots: &Roots) -> (Module, Vec<Error>) {
        let mut lower = Lower::new(self, &roots.traits);
        lower.run(roots);
        (lower.m, lower.errors)
    }

    /// The `#[test]` functions of this crate, a test crate of `source`
    /// whose library is `name` in its paths, into [`Module::tests`] of `m`
    /// lowered from `source`. The structs and enums the crate declares are
    /// data of `m` named in [`Module::test_types`]. A path of `source` is
    /// its item in `m`.
    pub fn lower_tests(&self, source: &SourceCrate, name: &str, m: &mut Module) -> Vec<Error> {
        let mut lower = Lower::new(self, &[]);
        lower.source = Some((source, name.to_owned()));
        lower.m = std::mem::take(m);
        lower.tests();
        *m = lower.m;
        lower.errors
    }
}

struct Lower<'a> {
    c: &'a SourceCrate,
    inherent: HashMap<ItemId, Vec<ItemId>>,
    trait_impls: HashMap<ItemId, Vec<(ItemId, ItemId)>>,
    /// The (type, impl) of the `From` impls in the order of the source.
    from_impls: Vec<(ItemId, ItemId)>,
    default_impls: HashMap<ItemId, ItemId>,
    items: HashMap<ItemId, Option<ir::Item>>,
    names: HashMap<String, ItemId>,
    builders: HashSet<ItemId>,
    queue: Vec<ItemId>,
    consts: Vec<ItemId>,
    fn_errors: Vec<String>,
    errors: Vec<Error>,
    m: Module,
    /// The crate a test crate tests, with its name in the paths.
    source: Option<(&'a SourceCrate, String)>,
}

impl<'a> Lower<'a> {
    fn new(c: &'a SourceCrate, traits: &[ItemId]) -> Self {
        let mut inherent: HashMap<ItemId, Vec<ItemId>> = HashMap::new();
        let mut trait_impls: HashMap<ItemId, Vec<(ItemId, ItemId)>> = HashMap::new();
        let mut from_impls = Vec::new();
        let mut default_impls = HashMap::new();

        for id in c.item_ids() {
            let syn::Item::Impl(i) = c.item(id) else {
                continue;
            };
            let syn::Type::Path(self_ty) = &*i.self_ty else {
                continue;
            };
            let Resolved::Item(ty) = c.resolve(id.module, &self_ty.path, Namespace::Type) else {
                continue;
            };

            match &i.trait_ {
                None => inherent.entry(ty).or_default().push(id),
                Some((path, _)) => match c.resolve(id.module, path, Namespace::Type) {
                    Resolved::Item(t) if traits.contains(&t) => {
                        trait_impls.entry(ty).or_default().push((t, id));
                    }
                    Resolved::External(p) if p.ends_with(&["convert".into(), "From".into()]) => {
                        from_impls.push((ty, id));
                    }
                    Resolved::External(p) if p.ends_with(&["default".into(), "Default".into()]) => {
                        default_impls.insert(ty, id);
                    }
                    _ => {}
                },
            }
        }

        Self {
            c,
            inherent,
            trait_impls,
            from_impls,
            default_impls,
            items: HashMap::new(),
            names: HashMap::new(),
            builders: HashSet::new(),
            queue: Vec::new(),
            consts: Vec::new(),
            fn_errors: Vec::new(),
            errors: Vec::new(),
            m: Module::default(),
            source: None,
        }
    }

    fn run(&mut self, roots: &Roots) {
        self.m.root = self.name(roots.root);
        self.m.data = roots.data.iter().map(|d| self.name(*d)).collect();
        self.find_builders(roots.root);

        for t in &roots.traits {
            self.load_trait(*t);
        }

        let mut implementors: Vec<ItemId> = self.trait_impls.keys().copied().collect();
        implementors.sort();
        self.queue.push(roots.root);
        self.queue.extend(implementors);
        self.drain();

        for d in &roots.data {
            self.reach(*d);
            self.drain();
        }

        for (ty, impl_) in self.from_impls.clone() {
            if let Some(c) = self.conversion(ty, impl_) {
                self.m.conversions.push(c);
            }
        }

        let mut loaded: Vec<(String, ItemId)> = self
            .names
            .iter()
            .filter(|(n, _)| self.m.classes.contains_key(*n) || self.m.enums.contains_key(*n))
            .map(|(n, id)| (n.clone(), *id))
            .collect();
        loaded.sort();

        for (name, id) in loaded {
            if let Some(default) = self.default(id) {
                self.m.defaults.insert(name.clone(), default);
            }

            self.skipped_defaults(&name, id);
        }

        let mut loaded = 0;

        while loaded < self.consts.len() {
            self.load_const(self.consts[loaded]);
            loaded += 1;
        }
    }

    fn tests(&mut self) {
        let ids: Vec<ItemId> = self.c.item_ids().collect();
        let mut types = Vec::new();

        for id in &ids {
            if !matches!(self.c.item(*id), syn::Item::Struct(_) | syn::Item::Enum(_)) {
                continue;
            }

            let name = self.name(*id);

            if self.m.classes.contains_key(&name) || self.m.enums.contains_key(&name) {
                self.error(*id, "a type of the tested crate has the name".into());
                continue;
            }

            self.m.test_types.insert(name.clone());
            self.reach(*id);
            self.drain();
            types.push((name, *id));
        }

        for (name, id) in types {
            if let Some(default) = self.default(id) {
                self.m.defaults.insert(name, default);
            }
        }

        for id in ids {
            let (
                Some(ir::Item {
                    attrs,
                    kind: ir::ItemKind::Fn(f),
                    ..
                }),
                syn::Item::Fn(syn_fn),
            ) = (self.item(id), self.c.item(id))
            else {
                continue;
            };

            if attrs
                .iter()
                .any(|a| matches!(a, ir::Attr::Meta(m) if m == "test"))
            {
                let at = self.location(id, syn_fn.sig.ident.span());
                let test = self.func(&f.sig, Some(&f.body), false, at, true);
                self.m.tests.push(test);
            }
        }
    }

    /// The names in the module of the item of the tested crate a path of
    /// another crate names.
    fn source_path(&self, p: &[String], namespace: Namespace) -> Option<Vec<String>> {
        let (source, name) = self.source.as_ref()?;

        match p {
            [first, rest @ ..] if first == name && !rest.is_empty() => {
                segments(source, &source.resolve_str(&rest.join("::"), namespace))
            }
            _ => None,
        }
    }

    // --- items -----------------------------------------------------------

    fn item(&mut self, id: ItemId) -> Option<ir::Item> {
        if !self.items.contains_key(&id) {
            let (mut items, mut errors) = self.c.ir([id]);
            self.errors.append(&mut errors);
            self.items.insert(id, items.pop().map(|(_, item)| item));
        }

        self.items[&id].clone()
    }

    fn name(&self, id: ItemId) -> String {
        item_name(self.c, id)
    }

    /// The name of a type of the module, an error if another type has it.
    fn type_name(&mut self, id: ItemId) -> String {
        let name = self.name(id);

        match self.names.get(&name) {
            Some(other) if *other != id => self.error(id, "another type has the name".into()),
            Some(_) => {}
            None => {
                self.names.insert(name.clone(), id);
            }
        }

        name
    }

    fn error(&mut self, id: ItemId, message: String) {
        let at = self.location(id, self.c.item(id).span());
        self.errors.push(Error::Lowering {
            path: at.path,
            line: at.line,
            item: self.name(id),
            message,
        });
    }

    fn location(&self, id: ItemId, span: proc_macro2::Span) -> Location {
        Location {
            path: self
                .c
                .file(self.c.module(id.module).file())
                .path()
                .to_owned(),
            line: span.start().line,
        }
    }

    /// The functions of the impl with their visibility and location, none
    /// if the IR does not have the impl.
    fn impl_fns(&mut self, impl_: ItemId) -> Vec<(ir::Vis, ir::Fn, Location)> {
        let (
            Some(ir::Item {
                kind: ir::ItemKind::Impl(i),
                ..
            }),
            syn::Item::Impl(syn_impl),
        ) = (self.item(impl_), self.c.item(impl_))
        else {
            return Vec::new();
        };
        let locations = syn_impl.items.iter().filter_map(|item| match item {
            syn::ImplItem::Fn(f) => Some(self.location(impl_, f.sig.ident.span())),
            _ => None,
        });

        i.items
            .into_iter()
            .filter_map(|item| match item.kind {
                ir::ImplItemKind::Fn(f) => Some((item.vis, f)),
                ir::ImplItemKind::Type { .. } => None,
            })
            .zip(locations)
            .map(|((vis, f), at)| (vis, f, at))
            .collect()
    }

    /// Whether the type has a `pub` method with a receiver, as the chaining
    /// methods of a builder and its `query(&self)` have.
    fn has_builder_method(&mut self, id: ItemId) -> bool {
        let impls = self.inherent.get(&id).cloned().unwrap_or_default();

        impls.into_iter().any(|i| {
            self.impl_fns(i)
                .iter()
                .any(|(vis, f, _)| *vis == ir::Vis::Public && f.sig.receiver.is_some())
        })
    }

    fn is_struct(&self, id: ItemId) -> bool {
        matches!(self.c.item(id), syn::Item::Struct(_))
    }

    fn roots(&mut self, data: Vec<ItemId>) -> Result<Roots, Error> {
        let mut made: HashSet<ItemId> = HashSet::new();

        for (to, impl_) in self.from_impls.clone() {
            if data.contains(&to)
                && let Some(source) = self.conversion_source(impl_)
            {
                made.insert(source);
            }
        }

        let mut structs: Vec<ItemId> = self
            .inherent
            .keys()
            .copied()
            .filter(|id| self.is_struct(*id) && !made.contains(id))
            .collect();
        structs.sort();
        let returns: HashMap<ItemId, Vec<ItemId>> =
            structs.iter().map(|s| (*s, self.pub_returns(*s))).collect();
        let mut makers: Vec<ItemId> = Vec::new();

        while let Some(s) = structs
            .iter()
            .find(|s| !makers.contains(s) && returns[s].iter().any(|t| made.contains(t)))
            .copied()
        {
            makers.push(s);
            made.insert(s);
        }

        let returned: HashSet<ItemId> = makers.iter().flat_map(|b| returns[b].clone()).collect();
        let roots: Vec<ItemId> = makers
            .iter()
            .copied()
            .filter(|b| !returned.contains(b))
            .collect();
        let [root] = roots[..] else {
            let mut names: Vec<String> = roots.iter().map(|r| self.name(*r)).collect();
            names.sort();

            return Err(Error::Lowering {
                path: Default::default(),
                line: 0,
                item: data
                    .iter()
                    .map(|d| self.name(*d))
                    .collect::<Vec<_>>()
                    .join(", "),
                message: format!("not one root of the makers but [{}]", names.join(", ")),
            });
        };
        self.find_builders(root);
        let mut builders: Vec<ItemId> = self.builders.iter().copied().collect();
        builders.sort();
        let mut traits = Vec::new();

        for b in &builders {
            for t in self.bounding_traits(*b) {
                if !traits.contains(&t) {
                    traits.push(t);
                }
            }
        }

        Ok(Roots { root, traits, data })
    }

    /// The type `S` of the `impl From<S>`, refs removed.
    fn conversion_source(&mut self, impl_: ItemId) -> Option<ItemId> {
        let Some(ir::Item {
            kind: ir::ItemKind::Impl(i),
            ..
        }) = self.item(impl_)
        else {
            return None;
        };
        let mut ty = trait_arg(&i)?;

        while let ir::Type::Ref { ty: inner, .. } | ir::Type::Paren(inner) = ty {
            ty = *inner;
        }

        match ty {
            ir::Type::Path(ir::Path {
                target: Target::Resolved(Resolved::Item(id)),
                ..
            }) => Some(id),
            _ => None,
        }
    }

    /// The named types of the returns of the type's `pub` inherent methods.
    fn pub_returns(&mut self, id: ItemId) -> Vec<ItemId> {
        let mut out = Vec::new();

        for i in self.inherent.get(&id).cloned().unwrap_or_default() {
            for (vis, f, _) in self.impl_fns(i) {
                if vis == ir::Vis::Public {
                    named_types(f.sig.output.as_ref(), &mut out);
                }
            }
        }

        out
    }

    /// The crate's traits bounding the generics of the struct, of its
    /// inherent impls and of their `pub` methods.
    fn bounding_traits(&mut self, id: ItemId) -> Vec<ItemId> {
        let mut generics = Vec::new();

        if let Some(ir::Item {
            kind: ir::ItemKind::Struct(s),
            ..
        }) = self.item(id)
        {
            generics.push(s.generics);
        }

        for i in self.inherent.get(&id).cloned().unwrap_or_default() {
            if let Some(ir::Item {
                kind: ir::ItemKind::Impl(imp),
                ..
            }) = self.item(i)
            {
                generics.push(imp.generics);
            }

            for (vis, f, _) in self.impl_fns(i) {
                if vis == ir::Vis::Public {
                    generics.push(f.sig.generics);
                }
            }
        }

        let mut out = Vec::new();

        for g in &generics {
            let params = g.params.iter().flat_map(|p| match p {
                ir::GenericParam::Type { bounds, .. } => bounds.as_slice(),
                _ => &[],
            });

            for b in params.chain(g.predicates.iter().flat_map(|p| p.bounds.iter())) {
                if let ir::Bound::Trait(path) = b
                    && let Target::Resolved(Resolved::Item(t)) = path.target
                    && matches!(self.c.item(t), syn::Item::Trait(_))
                    && !out.contains(&t)
                {
                    out.push(t);
                }
            }
        }

        out
    }

    /// The structs the returns of the `pub` methods of the builders lead to
    /// that have `pub` methods with a receiver.
    fn find_builders(&mut self, root: ItemId) {
        self.builders.insert(root);
        let mut stack = vec![root];

        while let Some(id) = stack.pop() {
            for i in self.inherent.get(&id).cloned().unwrap_or_default() {
                for (vis, f, _) in self.impl_fns(i) {
                    if vis != ir::Vis::Public {
                        continue;
                    }

                    let mut types = Vec::new();
                    named_types(f.sig.output.as_ref(), &mut types);

                    for ty in types {
                        if self.is_struct(ty)
                            && !self.builders.contains(&ty)
                            && self.has_builder_method(ty)
                        {
                            self.builders.insert(ty);
                            stack.push(ty);
                        }
                    }
                }
            }
        }
    }

    /// Loads the structs of the queue, last first.
    fn drain(&mut self) {
        while let Some(id) = self.queue.pop() {
            let name = self.type_name(id);

            if self.m.classes.contains_key(&name) {
                continue;
            }

            let class = self.class(id);

            if self.builders.contains(&id) {
                self.m.builders.push(name.clone());
            }

            self.m.classes.insert(name, class);
        }
    }

    /// A type the module needs: a struct is queued, an enum loaded.
    fn reach(&mut self, id: ItemId) {
        match self.c.item(id) {
            syn::Item::Struct(_) => {
                let name = self.name(id);

                if !self.m.classes.contains_key(&name) {
                    self.queue.push(id);
                }
            }
            syn::Item::Enum(_) => self.load_enum(id),
            _ => {}
        }
    }

    fn class(&mut self, id: ItemId) -> Class {
        let name = self.name(id);
        let Some(ir::Item {
            kind: ir::ItemKind::Struct(s),
            ..
        }) = self.item(id)
        else {
            return empty_class(name);
        };
        let generics = self.generics(&s.generics, true);
        let fields = self.fields(&s.fields, true);
        let builder = self.builders.contains(&id);
        let mut methods = Vec::new();

        for i in self.inherent.get(&id).cloned().unwrap_or_default() {
            methods.extend(self.impl_methods(i, builder));
        }

        let mut trait_impls: std::collections::BTreeMap<String, Vec<Func>> = Default::default();

        for (t, i) in self.trait_impls.get(&id).cloned().unwrap_or_default() {
            let trait_name = self.name(t);
            let funcs = self.impl_methods(i, true);
            trait_impls.entry(trait_name).or_default().extend(funcs);
        }

        Class {
            name,
            generics,
            fields,
            methods,
            trait_impls,
        }
    }

    fn impl_methods(&mut self, impl_: ItemId, follow: bool) -> Vec<Func> {
        let trait_impl = matches!(self.c.item(impl_), syn::Item::Impl(i) if i.trait_.is_some());

        self.impl_fns(impl_)
            .into_iter()
            .map(|(vis, f, at)| {
                let private = !trait_impl && vis != ir::Vis::Public;
                self.func(&f.sig, Some(&f.body), private, at, follow)
            })
            .collect()
    }

    fn load_enum(&mut self, id: ItemId) {
        let name = self.type_name(id);

        if self.m.enums.contains_key(&name) {
            return;
        }

        self.m.enums.insert(
            name.clone(),
            Enum {
                name: name.clone(),
                variants: Vec::new(),
                methods: Vec::new(),
            },
        );

        let Some(ir::Item {
            kind: ir::ItemKind::Enum(e),
            ..
        }) = self.item(id)
        else {
            return;
        };
        let variants = e
            .variants
            .iter()
            .map(|v| match &v.fields {
                ir::Fields::Unit => Variant::Unit(v.name.clone()),
                ir::Fields::Unnamed(fields) if fields.len() == 1 => {
                    Variant::Tuple(v.name.clone(), self.ty(&fields[0].ty, true))
                }
                ir::Fields::Unnamed(fields) => Variant::Tuple(
                    v.name.clone(),
                    Ty::Tuple(fields.iter().map(|f| self.ty(&f.ty, true)).collect()),
                ),
                ir::Fields::Named(_) => {
                    Variant::Struct(v.name.clone(), self.fields(&v.fields, true))
                }
            })
            .collect();
        let mut methods = Vec::new();

        for i in self.inherent.get(&id).cloned().unwrap_or_default() {
            methods.extend(self.impl_methods(i, false));
        }

        let e = self.m.enums.get_mut(&name).expect("inserted above");
        e.variants = variants;
        e.methods = methods;
    }

    fn load_trait(&mut self, id: ItemId) {
        let name = self.name(id);
        let (
            Some(ir::Item {
                kind: ir::ItemKind::Trait(t),
                ..
            }),
            syn::Item::Trait(syn_trait),
        ) = (self.item(id), self.c.item(id))
        else {
            return;
        };
        let locations: Vec<Location> = syn_trait
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Fn(f) => Some(self.location(id, f.sig.ident.span())),
                _ => None,
            })
            .collect();
        let fns = t
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                ir::TraitItemKind::Fn { sig, body } => Some((sig, body)),
                ir::TraitItemKind::Type { .. } => None,
            })
            .zip(locations)
            .map(|((sig, body), at)| self.func(sig, body.as_ref(), false, at, true))
            .collect();

        self.m.traits.insert(name, fns);
    }

    /// The `From` impl into a type of the module. Left out are the impls
    /// from a type the module does not have and the ones with a generic
    /// bounded by a trait of the crate the module does not load.
    fn conversion(&mut self, ty: ItemId, impl_: ItemId) -> Option<Conversion> {
        let to = self.name(ty);

        if !self.m.classes.contains_key(&to) && !self.m.enums.contains_key(&to) {
            return None;
        }

        let Some(ir::Item {
            kind: ir::ItemKind::Impl(i),
            ..
        }) = self.item(impl_)
        else {
            return None;
        };
        let bounds = i
            .generics
            .params
            .iter()
            .flat_map(|p| match p {
                ir::GenericParam::Type { bounds, .. } => bounds.as_slice(),
                _ => &[],
            })
            .chain(i.generics.predicates.iter().flat_map(|p| p.bounds.iter()));

        for b in bounds {
            if let ir::Bound::Trait(path) = b
                && let Target::Resolved(Resolved::Item(t)) = path.target
                && !self.m.traits.contains_key(&self.name(t))
            {
                return None;
            }
        }

        let source = trait_arg(&i)?;
        let from = self.ty(&source, false);

        if !self.representable(&from) {
            return None;
        }

        let generics = self.generics(&i.generics, false);
        let (_, f, at) = self
            .impl_fns(impl_)
            .into_iter()
            .find(|(_, f, _)| f.sig.name == "from")?;
        let param = f.sig.params.first()?.name.clone();
        let mut body = self.block(&f.body);
        let mut body = match body.len() {
            1 => body.pop().expect("one statement"),
            _ => Expr::new(self.unsupported("a conversion of more than one statement")),
        };
        if param != "v" && binds(&body, &param) {
            body = Expr::new(self.unsupported("a binding shadowing the parameter"));
        } else if param != "v" {
            rename_bindings(&mut body, "v");
            body.rewrite(&mut |e| {
                if matches!(&e.k, K::Local(n) if *n == param) {
                    e.k = K::Local("v".into());
                }
            });
        }

        Some(Conversion {
            to,
            from,
            generics,
            body,
            at: Some(at),
            errors: std::mem::take(&mut self.fn_errors),
        })
    }

    /// The `Default` value of the type: the body of its `impl Default`, the
    /// fields of a struct deriving it with their defaults or the
    /// `#[default]` variant of an enum deriving it.
    fn default(&mut self, id: ItemId) -> Option<Expr> {
        if let Some(impl_) = self.default_impls.get(&id).copied() {
            let (_, f, _) = self
                .impl_fns(impl_)
                .into_iter()
                .find(|(_, f, _)| f.sig.name == "default")?;
            let mut body = self.block(&f.body);
            self.fn_errors.clear();

            return (body.len() == 1).then(|| body.pop().expect("one statement"));
        }

        let item = self.item(id)?;

        if !item.attrs.iter().any(|a| derives(a, "Default")) {
            return None;
        }

        let name = self.name(id);
        let default = || {
            Expr::new(K::Call {
                callee: Callee::Path(vec!["Default".into(), "default".into()], Vec::new()),
                recv: None,
                args: Vec::new(),
            })
        };

        match item.kind {
            ir::ItemKind::Struct(s) => {
                let fields = match &s.fields {
                    ir::Fields::Named(fields) | ir::Fields::Unnamed(fields) => fields
                        .iter()
                        .enumerate()
                        .map(|(i, f)| (f.name.clone().unwrap_or_else(|| i.to_string()), default()))
                        .collect(),
                    ir::Fields::Unit => Vec::new(),
                };

                Some(Expr::new(K::Construct {
                    class: name,
                    fields,
                }))
            }
            ir::ItemKind::Enum(e) => e
                .variants
                .iter()
                .find(|v| {
                    v.attrs
                        .iter()
                        .any(|a| matches!(a, ir::Attr::Meta(m) if m == "default"))
                })
                .map(|v| Expr::new(K::Path(vec![name, v.name.clone()]))),
            _ => None,
        }
    }

    /// The fields of the struct serde leaves out of the serialized value
    /// (`skip_serializing_if`) with the value its predicate holds for.
    fn skipped_defaults(&mut self, class: &str, id: ItemId) {
        let Some(ir::Item {
            kind: ir::ItemKind::Struct(s),
            ..
        }) = self.item(id)
        else {
            return;
        };
        let ir::Fields::Named(fields) = &s.fields else {
            return;
        };

        for f in fields {
            let Some(predicate) = f.attrs.iter().find_map(skip_serializing_if) else {
                continue;
            };
            let name = f.name.clone().unwrap_or_default();

            match self.skipped_value(id, &predicate) {
                Some(value) => self
                    .m
                    .skipped_defaults
                    .push((class.to_owned(), name, value)),
                None => self.error(
                    id,
                    format!("unsupported `skip_serializing_if = \"{predicate}\"` of `{name}`"),
                ),
            }
        }
    }

    /// The value the predicate holds for: the pattern of a method that is
    /// a `matches!(self, ..)` of a unit variant, `None` for
    /// `Option::is_none` and an empty list for `Vec::is_empty`.
    fn skipped_value(&mut self, id: ItemId, predicate: &str) -> Option<Expr> {
        let path = syn::parse_str::<syn::Path>(predicate).ok()?;

        match self.c.resolve(id.module, &path, Namespace::Value) {
            Resolved::External(p) => match p.as_slice() {
                [.., ty, f] if ty == "Option" && f == "is_none" => {
                    Some(Expr::variant("Option", "None", None))
                }
                [.., ty, f] if ty == "Vec" && f == "is_empty" => {
                    Some(Expr::new(K::Array(Vec::new())))
                }
                _ => None,
            },
            Resolved::Associated(base, name) => match *base {
                Resolved::Item(ty) => {
                    let impls = self.inherent.get(&ty).cloned().unwrap_or_default();
                    let f = impls
                        .into_iter()
                        .flat_map(|i| self.impl_fns(i))
                        .find(|(_, f, _)| f.sig.name == name)?
                        .1;
                    let [ir::Stmt::Expr { expr, .. }] = f.body.stmts.as_slice() else {
                        return None;
                    };
                    let ir::Expr::Macro(m) = expr else {
                        return None;
                    };
                    let ir::MacroCall::Matches {
                        pat, guard: None, ..
                    } = &m.call
                    else {
                        return None;
                    };

                    match self.pat(pat) {
                        Pat::Variant { en, var, inner } if inner.is_empty() => {
                            Some(Expr::variant(&en, &var, None))
                        }
                        _ => None,
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Whether the type has only types the module has.
    fn representable(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Unknown => false,
            Ty::Named(n, args) => {
                (self.m.classes.contains_key(n) || self.m.enums.contains_key(n))
                    && args.iter().all(|a| self.representable(a))
            }
            Ty::List(t) | Ty::Option(t) | Ty::Ref(t) => self.representable(t),
            Ty::Tuple(items) => items.iter().all(|t| self.representable(t)),
            _ => true,
        }
    }

    fn load_const(&mut self, id: ItemId) {
        let Some(ir::Item {
            kind: ir::ItemKind::Const(c),
            ..
        }) = self.item(id)
        else {
            return;
        };
        let ty = self.ty(&c.ty, false).strip_ref().clone();
        let mut value = self.expr(&c.expr);
        value.ty = ty;
        self.fn_errors.clear();
        self.m.consts.insert(c.name, value);
    }

    // --- signatures ------------------------------------------------------

    fn func(
        &mut self,
        sig: &ir::Signature,
        body: Option<&ir::Block>,
        private: bool,
        at: Location,
        follow: bool,
    ) -> Func {
        let generics = self.generics(&sig.generics, follow);
        let recv = sig.receiver.as_ref().map(|r| match r {
            ir::Receiver::Value { mutable: false } => Recv::Value,
            ir::Receiver::Value { mutable: true } => Recv::ValueMut,
            ir::Receiver::Ref { mutable: false, .. } => Recv::Ref,
            ir::Receiver::Ref { mutable: true, .. } => Recv::RefMut,
            ir::Receiver::Typed { .. } => {
                self.fn_errors.push("unsupported a typed `self`".into());
                Recv::Value
            }
        });
        let params = sig
            .params
            .iter()
            .map(|p| (p.name.clone(), self.ty(&p.ty, follow)))
            .collect();
        let ret = sig
            .output
            .as_ref()
            .map(|t| self.ty(t, follow))
            .unwrap_or(Ty::Unit);
        let body = body.map(|b| self.block(b)).unwrap_or_default();

        Func {
            name: sig.name.clone(),
            generics,
            recv,
            params,
            ret,
            body,
            private,
            at: Some(at),
            errors: std::mem::take(&mut self.fn_errors),
        }
    }

    fn generics(&mut self, generics: &ir::Generics, follow: bool) -> Vec<Generic> {
        let mut out: Vec<Generic> = generics
            .params
            .iter()
            .filter_map(|p| match p {
                ir::GenericParam::Type { name, bounds } => Some(Generic {
                    name: name.clone(),
                    bounds: self.bounds(bounds, follow),
                }),
                ir::GenericParam::Lifetime(_) | ir::GenericParam::Const { .. } => None,
            })
            .collect();

        for p in &generics.predicates {
            let ir::Type::Path(path) = &p.ty else {
                continue;
            };
            let bounds = self.bounds(&p.bounds, follow);

            if let Some(g) = out.iter_mut().find(|g| {
                path.target == Target::Generic
                    && path.segments.len() == 1
                    && path.segments[0].name == g.name
            }) {
                g.bounds.extend(bounds);
            }
        }

        out
    }

    fn bounds(&mut self, bounds: &[ir::Bound], follow: bool) -> Vec<(String, Option<Ty>)> {
        bounds
            .iter()
            .filter_map(|b| match b {
                ir::Bound::Trait(path) => {
                    let last = path.segments.last().expect("a path has a segment");
                    let arg = match &last.args {
                        ir::Args::Angle(args) => args.iter().find_map(|a| match a {
                            ir::GenericArg::Type(t) => Some(t),
                            _ => None,
                        }),
                        _ => None,
                    };
                    let arg = arg.map(|t| self.ty(t, follow));
                    Some((last.name.clone(), arg))
                }
                ir::Bound::Lifetime(_) => None,
            })
            .collect()
    }

    fn fields(&mut self, fields: &ir::Fields, follow: bool) -> Vec<(String, Ty)> {
        match fields {
            ir::Fields::Named(fields) | ir::Fields::Unnamed(fields) => fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let name = f.name.clone().unwrap_or_else(|| i.to_string());
                    (name, self.ty(&f.ty, follow))
                })
                .collect(),
            ir::Fields::Unit => Vec::new(),
        }
    }

    /// The type; with `follow` the types of the module it names are
    /// reached.
    fn ty(&mut self, t: &ir::Type, follow: bool) -> Ty {
        match t {
            ir::Type::Path(path) => self.path_ty(path, follow),
            ir::Type::Ref { ty, .. } => Ty::Ref(Box::new(self.ty(ty, follow))),
            ir::Type::Tuple(types) if types.is_empty() => Ty::Unit,
            ir::Type::Tuple(types) => Ty::Tuple(types.iter().map(|t| self.ty(t, follow)).collect()),
            ir::Type::Slice(ty) | ir::Type::Array(ty, _) => Ty::List(Box::new(self.ty(ty, follow))),
            ir::Type::Paren(ty) => self.ty(ty, follow),
            ir::Type::Qualified(_)
            | ir::Type::ImplTrait(_)
            | ir::Type::DynTrait(_)
            | ir::Type::FnPtr(_)
            | ir::Type::Infer => Ty::Unknown,
        }
    }

    fn path_ty(&mut self, path: &ir::Path, follow: bool) -> Ty {
        let last = path.segments.last().expect("a path has a segment");
        let args: Vec<Ty> = match &last.args {
            ir::Args::Angle(args) => args
                .iter()
                .filter_map(|a| match a {
                    ir::GenericArg::Type(t) => Some(self.ty(t, follow)),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        let first = |args: Vec<Ty>| Box::new(args.into_iter().next().unwrap_or(Ty::Unknown));

        match &path.target {
            Target::Generic => Ty::Param(last.name.clone()),
            Target::SelfType => Ty::SelfTy,
            Target::Resolved(Resolved::Primitive(p)) => primitive(p),
            Target::Resolved(Resolved::External(p)) => {
                if let Some(segments) = self.source_path(p, Namespace::Type) {
                    return Ty::Named(segments.join("::"), args);
                }

                match p.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
                    [_, "vec", "Vec"] => Ty::List(first(args)),
                    [_, "option", "Option"] => Ty::Option(first(args)),
                    [_, "string", "String"] => Ty::Str,
                    _ => Ty::Unknown,
                }
            }
            Target::Resolved(Resolved::Item(id)) => match self.c.item(*id) {
                syn::Item::Type(_) => match self.item(*id) {
                    Some(ir::Item {
                        kind: ir::ItemKind::TypeAlias(alias),
                        ..
                    }) => self.ty(&alias.ty, follow),
                    _ => Ty::Unknown,
                },
                _ => {
                    if follow {
                        self.reach(*id);
                    }

                    Ty::Named(self.type_name(*id), args)
                }
            },
            Target::Local | Target::LocalItem | Target::Resolved(_) => Ty::Unknown,
        }
    }

    // --- bodies ----------------------------------------------------------

    /// An `if`, with the conditions of a let chain as nested ones that each
    /// have the `else`.
    fn condition(&mut self, cond: &ir::Expr, then: Vec<Expr>, els: Option<Vec<Expr>>) -> K {
        match cond {
            ir::Expr::Binary {
                left,
                op: ir::BinOp::And,
                right,
            } if has_let(cond) => {
                let inner = Expr::new(self.condition(right, then, els.clone()));
                self.condition(left, vec![inner], els)
            }
            ir::Expr::Let { pat, expr } => K::IfLet {
                pat: self.pat(pat),
                value: self.expr(expr).boxed(),
                then,
                els,
            },
            cond => K::If {
                cond: self.expr(cond).boxed(),
                then,
                els,
            },
        }
    }

    fn unsupported(&mut self, construct: &str) -> K {
        self.fn_errors.push(format!("unsupported {construct}"));
        K::Unsupported(construct.to_owned())
    }

    fn block(&mut self, b: &ir::Block) -> Vec<Expr> {
        b.stmts.iter().map(|s| self.stmt(s)).collect()
    }

    fn stmt(&mut self, s: &ir::Stmt) -> Expr {
        match s {
            ir::Stmt::Let {
                diverge: Some(_), ..
            } => Expr::new(self.unsupported("`let`-`else`")),
            ir::Stmt::Let { pat, init, .. } => match binding(pat) {
                Some(name) => {
                    let ty = match pat {
                        ir::Pat::Type { ty, .. } => self.ty(ty, false),
                        _ => Ty::Unknown,
                    };
                    let value = init.as_ref().map(|v| Expr { ty, ..self.expr(v) });
                    Expr::new(K::Let {
                        name,
                        value: value.map(Expr::boxed),
                    })
                }
                None => Expr::new(self.unsupported("a destructuring `let`")),
            },
            ir::Stmt::Item(_) => Expr::new(self.unsupported("an item in a body")),
            ir::Stmt::Expr { expr, .. } => self.expr(expr),
        }
    }

    /// A block or `if` of an `else` or an arm as statements.
    fn stmts(&mut self, e: &ir::Expr) -> Vec<Expr> {
        match e {
            ir::Expr::Block(b) => self.block(b),
            e => vec![self.expr(e)],
        }
    }

    fn exprs(&mut self, v: &[ir::Expr]) -> Vec<Expr> {
        v.iter().map(|e| self.expr(e)).collect()
    }

    fn expr(&mut self, e: &ir::Expr) -> Expr {
        let k = match e {
            ir::Expr::Array(items) => K::Array(self.exprs(items)),
            ir::Expr::Assign { left, right } => K::Assign {
                place: self.expr(left).boxed(),
                value: self.expr(right).boxed(),
            },
            ir::Expr::Attributed { expr, .. } | ir::Expr::Paren(expr) => return self.expr(expr),
            ir::Expr::Binary { left, op, right } => {
                let l = self.expr(left);
                let r = self.expr(right);

                match assigned(*op) {
                    Some(op) => K::Assign {
                        place: l.clone().boxed(),
                        value: Expr::new(K::Binary {
                            op: op.token(),
                            l: l.boxed(),
                            r: r.boxed(),
                        })
                        .boxed(),
                    },
                    None => K::Binary {
                        op: op.token(),
                        l: l.boxed(),
                        r: r.boxed(),
                    },
                }
            }
            ir::Expr::Block(b) => K::If {
                cond: Expr::new(K::Lit(Lit::Bool(true))).boxed(),
                then: self.block(b),
                els: None,
            },
            ir::Expr::Call { func, args } => match &**func {
                ir::Expr::Path(path) => K::Call {
                    callee: Callee::Path(self.segments(path), self.path_args(path)),
                    recv: None,
                    args: self.exprs(args),
                },
                _ => self.unsupported("a call of an expression"),
            },
            ir::Expr::Closure(c) => {
                let params: Option<Vec<String>> = c.inputs.iter().map(binding).collect();

                match params {
                    Some(params) => K::Closure {
                        params,
                        body: self.stmts(&c.body),
                    },
                    None => self.unsupported("a closure parameter pattern"),
                }
            }
            ir::Expr::Field { expr, member } => K::Field {
                base: self.expr(expr).boxed(),
                name: member_name(member),
            },
            ir::Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let then = self.block(then_branch);
                let els = else_branch.as_ref().map(|e| self.stmts(e));
                self.condition(cond, then, els)
            }
            ir::Expr::Index { expr, index } => K::Index {
                base: self.expr(expr).boxed(),
                index: self.expr(index).boxed(),
            },
            ir::Expr::Lit(l) => match lit(l) {
                Some(l) => K::Lit(l),
                None => self.unsupported("an integer beyond `i64`"),
            },
            ir::Expr::Macro(m) => self.macro_call(m),
            ir::Expr::Match { expr, arms } if arms.iter().all(|a| a.guard.is_none()) => K::Match {
                value: self.expr(expr).boxed(),
                arms: arms
                    .iter()
                    .map(|a| (self.pat(&a.pat), self.stmts(&a.body)))
                    .collect(),
            },
            ir::Expr::Match { .. } => self.unsupported("a match guard"),
            ir::Expr::MethodCall {
                receiver,
                method,
                args,
                ..
            } => K::Call {
                callee: Callee::Method(method.clone()),
                recv: Some(self.expr(receiver).boxed()),
                args: self.exprs(args),
            },
            ir::Expr::Path(path) => self.path_expr(path),
            ir::Expr::Reference { expr, .. } => K::Ref(self.expr(expr).boxed()),
            ir::Expr::Return(value) => K::Return(value.as_ref().map(|v| self.expr(v).boxed())),
            ir::Expr::Struct { path, .. } if is_variant(path) => {
                self.unsupported("a struct variant")
            }
            ir::Expr::Struct { path, fields } => K::Construct {
                class: self.class_name(path),
                fields: fields
                    .iter()
                    .map(|f| (member_name(&f.member), self.expr(&f.expr)))
                    .collect(),
            },
            ir::Expr::Tuple(items) if items.is_empty() => K::Lit(Lit::Unit),
            ir::Expr::Tuple(items) => K::Tuple(self.exprs(items)),
            ir::Expr::Unary { op, expr } => {
                let e = self.expr(expr).boxed();

                match op {
                    ir::UnOp::Not => K::Not(e),
                    ir::UnOp::Deref => K::Deref(e),
                    ir::UnOp::Neg => K::Binary {
                        op: "-",
                        l: Expr::new(K::Lit(Lit::Int(0))).boxed(),
                        r: e,
                    },
                }
            }
            ir::Expr::While { cond, .. } if matches!(**cond, ir::Expr::Let { .. }) => {
                self.unsupported("`while let`")
            }
            ir::Expr::While { cond, body } => K::While {
                cond: self.expr(cond).boxed(),
                body: self.block(body),
            },
            ir::Expr::Async { .. } => self.unsupported("`async`"),
            ir::Expr::Await(_) => self.unsupported("`.await`"),
            ir::Expr::Break => self.unsupported("`break`"),
            ir::Expr::Cast { expr, ty } => {
                let value = self.expr(expr).boxed();
                return Expr::typed(K::Cast(value), self.ty(ty, false));
            }
            ir::Expr::Const(_) => self.unsupported("a `const` block"),
            ir::Expr::Continue => self.unsupported("`continue`"),
            ir::Expr::ForLoop { .. } => self.unsupported("`for`"),
            ir::Expr::Let { .. } => self.unsupported("`let` in an expression"),
            ir::Expr::Loop(_) => self.unsupported("`loop`"),
            ir::Expr::Qualified(_) => self.unsupported("a qualified path"),
            ir::Expr::Range { .. } => self.unsupported("a range"),
            ir::Expr::Repeat { .. } => self.unsupported("`[x; n]`"),
            ir::Expr::Try(_) => self.unsupported("`?`"),
        };

        Expr::new(k)
    }

    fn macro_call(&mut self, m: &ir::Macro) -> K {
        let name = m
            .path
            .segments
            .last()
            .map(|s| s.name.clone())
            .unwrap_or_default();

        match &m.call {
            ir::MacroCall::Vec(items) => K::Array(self.exprs(items)),
            ir::MacroCall::Format {
                exprs,
                format: None,
            } if name == "assert_eq" && exprs.len() == 2 => K::Call {
                callee: Callee::Intrinsic(Intrinsic::AssertEq),
                recv: None,
                args: self.exprs(exprs),
            },
            ir::MacroCall::Format { format, .. } if name == "panic" => K::Panic(
                format
                    .as_ref()
                    .map(|f| f.string.clone())
                    .unwrap_or_else(|| "panic".into()),
            ),
            _ => self.unsupported(&format!("`{name}!`")),
        }
    }

    fn path_expr(&mut self, path: &ir::Path) -> K {
        match &path.target {
            Target::Local if path.segments[0].name == "self" => K::This,
            Target::Local => K::Local(path.segments[0].name.clone()),
            _ => K::Path(self.segments(path)),
        }
    }

    /// The struct a construction or pattern names, also through a type
    /// alias.
    fn class_name(&mut self, path: &ir::Path) -> String {
        match self.path_ty(path, false) {
            Ty::Named(name, _) => name,
            _ => path
                .segments
                .last()
                .map(|s| s.name.clone())
                .unwrap_or_default(),
        }
    }

    /// The names of the path by what it names: an item by its name, a
    /// variant with its enum, a path of another crate without the crate
    /// (`mem::take`) and `Some`/`None` alone.
    fn segments(&mut self, path: &ir::Path) -> Vec<String> {
        let resolved = match &path.target {
            Target::Resolved(r) => self.resolved_segments(r),
            _ => None,
        };

        resolved
            .filter(|segments| !segments.is_empty())
            .unwrap_or_else(|| path.segments.iter().map(|s| s.name.clone()).collect())
    }

    fn resolved_segments(&mut self, r: &Resolved) -> Option<Vec<String>> {
        match r {
            Resolved::Item(id) => {
                if matches!(self.c.item(*id), syn::Item::Const(_)) && !self.consts.contains(id) {
                    self.consts.push(*id);
                }

                Some(vec![self.name(*id)])
            }
            Resolved::Variant(id, index) => {
                Some(vec![self.name(*id), variant_name(self.c.item(*id), *index)])
            }
            Resolved::Associated(base, name) => self.resolved_segments(base).map(|mut segments| {
                segments.push(name.clone());
                segments
            }),
            Resolved::Primitive(p) => Some(vec![p.clone()]),
            Resolved::External(p) => {
                if let Some(segments) = self
                    .source_path(p, Namespace::Value)
                    .or_else(|| self.source_path(p, Namespace::Type))
                {
                    return Some(segments);
                }

                match p.as_slice() {
                    [_, module, ty, variant] if module == "option" && ty == "Option" => {
                        Some(vec![variant.clone()])
                    }
                    p => Some(p[1..].to_vec()),
                }
            }
            Resolved::Module(_) | Resolved::Unresolved => None,
        }
    }

    /// The type arguments of the segments.
    fn path_args(&mut self, path: &ir::Path) -> Vec<Ty> {
        let mut out = Vec::new();

        for s in &path.segments {
            if let ir::Args::Angle(args) = &s.args {
                for a in args {
                    if let ir::GenericArg::Type(t) = a {
                        out.push(self.ty(t, false));
                    }
                }
            }
        }

        out
    }

    fn pat(&mut self, p: &ir::Pat) -> Pat {
        match p {
            ir::Pat::Binding {
                name, subpat: None, ..
            } => Pat::Bind(name.clone()),
            ir::Pat::Lit(l) => match lit(l) {
                Some(l) => Pat::Lit(l),
                None => self.unsupported_pat("an integer beyond `i64`"),
            },
            ir::Pat::Paren(p) | ir::Pat::Ref { pat: p, .. } | ir::Pat::Type { pat: p, .. } => {
                self.pat(p)
            }
            ir::Pat::Path(path) => self.variant_pat(path, Vec::new()),
            ir::Pat::TupleStruct { path, elems } => {
                let inner = elems.iter().map(|p| self.pat(p)).collect();
                self.variant_pat(path, inner)
            }
            ir::Pat::Struct { path, .. } if is_variant(path) => {
                self.unsupported_pat("a struct variant pattern")
            }
            ir::Pat::Struct { path, fields, .. } => Pat::Struct {
                name: self.class_name(path),
                fields: fields
                    .iter()
                    .map(|f| (member_name(&f.member), self.pat(&f.pat)))
                    .collect(),
            },
            ir::Pat::Wild => Pat::Wild,
            ir::Pat::Binding { .. } => self.unsupported_pat("a `@` pattern"),
            ir::Pat::Or(_) => self.unsupported_pat("an or pattern"),
            ir::Pat::Range { .. } => self.unsupported_pat("a range pattern"),
            ir::Pat::Tuple(_) => self.unsupported_pat("a tuple pattern"),
        }
    }

    fn unsupported_pat(&mut self, construct: &str) -> Pat {
        self.fn_errors.push(format!("unsupported {construct}"));
        Pat::Wild
    }

    fn variant_pat(&mut self, path: &ir::Path, inner: Vec<Pat>) -> Pat {
        let mut segments = self.segments(path);
        let var = segments.pop().unwrap_or_default();

        Pat::Variant {
            en: segments.pop().unwrap_or_default(),
            var,
            inner,
        }
    }
}

/// The names of what a path of `c` resolves to.
fn segments(c: &SourceCrate, r: &Resolved) -> Option<Vec<String>> {
    match r {
        Resolved::Item(id) => Some(vec![item_name(c, *id)]),
        Resolved::Variant(id, index) => {
            Some(vec![item_name(c, *id), variant_name(c.item(*id), *index)])
        }
        Resolved::Associated(base, name) => segments(c, base).map(|mut segments| {
            segments.push(name.clone());
            segments
        }),
        _ => None,
    }
}

fn item_name(c: &SourceCrate, id: ItemId) -> String {
    match c.item(id) {
        syn::Item::Struct(s) => s.ident.to_string(),
        syn::Item::Enum(e) => e.ident.to_string(),
        syn::Item::Trait(t) => t.ident.to_string(),
        syn::Item::Const(c) => c.ident.to_string(),
        syn::Item::Static(s) => s.ident.to_string(),
        syn::Item::Type(t) => t.ident.to_string(),
        syn::Item::Fn(f) => f.sig.ident.to_string(),
        syn::Item::Union(u) => u.ident.to_string(),
        _ => String::new(),
    }
}

/// Whether the attribute derives the trait.
fn derives(attr: &ir::Attr, trait_: &str) -> bool {
    let ir::Attr::Meta(text) = attr else {
        return false;
    };
    let Ok(syn::Meta::List(list)) = syn::parse_str::<syn::Meta>(text) else {
        return false;
    };

    list.path.is_ident("derive")
        && list
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|paths| {
                paths
                    .iter()
                    .any(|p| p.segments.last().is_some_and(|s| s.ident == trait_))
            })
}

/// The first type argument of the trait of an impl, `S` of `From<S>`.
fn trait_arg(i: &ir::Impl) -> Option<ir::Type> {
    let Some(ir::Segment {
        args: ir::Args::Angle(args),
        ..
    }) = i.trait_.as_ref().and_then(|t| t.segments.last())
    else {
        return None;
    };

    args.iter().find_map(|a| match a {
        ir::GenericArg::Type(t) => Some(t.clone()),
        _ => None,
    })
}

/// The predicate of a `#[serde(skip_serializing_if = "..")]`.
fn skip_serializing_if(attr: &ir::Attr) -> Option<String> {
    let ir::Attr::Meta(text) = attr else {
        return None;
    };
    let Ok(syn::Meta::List(list)) = syn::parse_str::<syn::Meta>(text) else {
        return None;
    };

    if !list.path.is_ident("serde") {
        return None;
    }

    let mut predicate = None;
    list.parse_nested_meta(|meta| {
        if meta.path.is_ident("skip_serializing_if") {
            predicate = Some(meta.value()?.parse::<syn::LitStr>()?.value());
        } else if let Ok(value) = meta.value() {
            value.parse::<syn::Expr>()?;
        }

        Ok(())
    })
    .ok()?;

    predicate
}

/// The closure parameters, `let`s and pattern bindings named `name` renamed
/// to a name the expression does not use, so that `name` refers to one value
/// only.
fn rename_bindings(e: &mut Expr, name: &str) {
    let mut fresh = "x".to_owned();
    let mut n = 0;

    while e.any(&|x| matches!(&x.k, K::Local(l) if *l == fresh)) {
        fresh = format!("x{n}");
        n += 1;
    }

    if !binds(e, name) {
        return;
    }

    e.rewrite(&mut |x| match &mut x.k {
        K::Closure { params, .. } => params
            .iter_mut()
            .filter(|p| *p == name)
            .for_each(|p| *p = fresh.clone()),
        K::Let { name: l, .. } if l == name => *l = fresh.clone(),
        K::Match { arms, .. } => arms
            .iter_mut()
            .for_each(|(p, _)| rename_pat(p, name, &fresh)),
        K::IfLet { pat, .. } => rename_pat(pat, name, &fresh),
        K::Local(l) if l == name => *l = fresh.clone(),
        _ => {}
    });
}

/// Whether a closure, `let` or pattern of `e` binds `name`.
fn binds(e: &Expr, name: &str) -> bool {
    e.any(&|x| match &x.k {
        K::Closure { params, .. } => params.iter().any(|p| p == name),
        K::Let { name: l, .. } => l == name,
        K::Match { arms, .. } => arms.iter().any(|(p, _)| pat_binds(p, name)),
        K::IfLet { pat, .. } => pat_binds(pat, name),
        _ => false,
    })
}

fn pat_binds(p: &Pat, name: &str) -> bool {
    match p {
        Pat::Bind(b) => b == name,
        Pat::Variant { inner, .. } => inner.iter().any(|p| pat_binds(p, name)),
        Pat::Struct { fields, .. } => fields.iter().any(|(_, p)| pat_binds(p, name)),
        Pat::Wild | Pat::Lit(_) => false,
    }
}

fn rename_pat(p: &mut Pat, name: &str, to: &str) {
    match p {
        Pat::Bind(b) if b == name => *b = to.to_owned(),
        Pat::Variant { inner, .. } => inner.iter_mut().for_each(|p| rename_pat(p, name, to)),
        Pat::Struct { fields, .. } => fields.iter_mut().for_each(|(_, p)| rename_pat(p, name, to)),
        _ => {}
    }
}

/// The name of the variant of the enum, empty for another item.
fn variant_name(item: &syn::Item, index: usize) -> String {
    match item {
        syn::Item::Enum(e) => e.variants[index].ident.to_string(),
        _ => String::new(),
    }
}

fn empty_class(name: String) -> Class {
    Class {
        name,
        generics: Vec::new(),
        fields: Vec::new(),
        methods: Vec::new(),
        trait_impls: Default::default(),
    }
}

/// The types of the module the type names, its arguments too.
fn named_types(t: Option<&ir::Type>, out: &mut Vec<ItemId>) {
    let Some(t) = t else {
        return;
    };

    match t {
        ir::Type::Path(path) => {
            if let Target::Resolved(Resolved::Item(id)) = &path.target {
                out.push(*id);
            }

            for s in &path.segments {
                if let ir::Args::Angle(args) = &s.args {
                    for a in args {
                        if let ir::GenericArg::Type(t) = a {
                            named_types(Some(t), out);
                        }
                    }
                }
            }
        }
        ir::Type::Ref { ty, .. } | ir::Type::Slice(ty) | ir::Type::Paren(ty) => {
            named_types(Some(ty), out);
        }
        ir::Type::Tuple(types) => {
            for t in types {
                named_types(Some(t), out);
            }
        }
        _ => {}
    }
}

/// The name of a pattern that binds a single name.
fn binding(p: &ir::Pat) -> Option<String> {
    match p {
        ir::Pat::Binding {
            name, subpat: None, ..
        } => Some(name.clone()),
        ir::Pat::Type { pat, .. } => binding(pat),
        _ => None,
    }
}

fn member_name(m: &ir::Member) -> String {
    match m {
        ir::Member::Named(name) => name.clone(),
        ir::Member::Index(index) => index.to_string(),
    }
}

/// The literal, none for an integer an `i64` cannot hold.
fn lit(l: &ir::Lit) -> Option<Lit> {
    Some(match l {
        ir::Lit::Str(s) => Lit::Str(s.clone()),
        ir::Lit::Char(c) => Lit::Str(c.to_string()),
        ir::Lit::Int { digits, .. } => return digits.parse().ok().map(Lit::Int),
        ir::Lit::Float { digits, .. } => Lit::Float(digits.parse().expect("a float literal")),
        ir::Lit::Bool(b) => Lit::Bool(*b),
    })
}

fn is_variant(path: &ir::Path) -> bool {
    matches!(path.target, Target::Resolved(Resolved::Variant(..)))
}

fn primitive(name: &str) -> Ty {
    const INTS: [&str; 12] = [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
    ];

    match name {
        "bool" => Ty::Bool,
        "f32" | "f64" => Ty::Float,
        "str" | "char" => Ty::Str,
        _ => INTS
            .iter()
            .find(|i| **i == name)
            .map(|i| Ty::Int(i))
            .unwrap_or(Ty::Unknown),
    }
}

/// The operation of a compound assignment.
fn assigned(op: ir::BinOp) -> Option<ir::BinOp> {
    use ir::BinOp::*;

    Some(match op {
        AddAssign => Add,
        SubAssign => Sub,
        MulAssign => Mul,
        DivAssign => Div,
        RemAssign => Rem,
        BitXorAssign => BitXor,
        BitAndAssign => BitAnd,
        BitOrAssign => BitOr,
        ShlAssign => Shl,
        ShrAssign => Shr,
        _ => return None,
    })
}

/// Whether the condition is a let chain.
fn has_let(cond: &ir::Expr) -> bool {
    match cond {
        ir::Expr::Let { .. } => true,
        ir::Expr::Binary {
            left,
            op: ir::BinOp::And,
            right,
        } => has_let(left) || has_let(right),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::id;
    use crate::codegen::test_utilities::load;
    use crate::codegen::test_utilities::lower;
    use crate::codegen::test_utilities::tested;

    fn dump(m: &Module) -> String {
        let mut out = print::module(m, "lower");

        for (name, c) in &m.classes {
            out += &format!(
                "class {name}<{}> {{ {} }}\n",
                c.generics
                    .iter()
                    .map(|g| g.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
                c.fields
                    .iter()
                    .map(|(n, t)| format!("{n}: {}", print::ty(t)))
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            for f in &c.methods {
                out += &format!("  {}{}\n", if f.private { "priv " } else { "" }, f.name);
            }
        }

        for (name, e) in &m.enums {
            out += &format!(
                "enum {name} {:?} {:?}\n",
                e.variants,
                e.methods.iter().map(|f| &f.name).collect::<Vec<_>>()
            );
        }

        for (name, fs) in &m.traits {
            for f in fs {
                out += &format!("trait {name}: {}\n", print::func(f));
            }
        }

        for (name, v) in &m.consts {
            out += &format!("const {name}: {} = {}\n", print::ty(&v.ty), print::expr(v));
        }

        out
    }

    fn func<'a>(m: &'a Module, class: &str, name: &str) -> &'a Func {
        m.method(class, name).unwrap()
    }

    mod roots {
        use super::*;

        mod of {
            use super::*;

            fn names(c: &SourceCrate, ids: &[ItemId]) -> Vec<String> {
                ids.iter().map(|id| Lower::new(c, &[]).name(*id)).collect()
            }

            #[test]
            fn of_should_be_the_root_making_the_data_types_and_the_traits_bounding_the_builders() {
                let c = load(
                    r#"pub enum Wire { A(Qa), B(Qb) }
pub struct Qa;
pub struct Qb;
impl From<Qa> for Wire { fn from(value: Qa) -> Self { Wire::A(value) } }
impl From<&Qb> for Wire { fn from(value: &Qb) -> Self { Wire::B(Qb) } }
impl From for Wire { fn from(value: Qb) -> Self { Wire::B(Qb) } }
impl From<'static> for Wire { fn from(value: Qb) -> Self { Wire::B(Qb) } }
impl From<Other> for Wire { const X: u8 = 1; fn from(value: Other) -> Self { Wire::B(Qb) } }
impl From<Vec<u8>> for Wire { fn from(value: Vec<u8>) -> Self { Wire::B(Qb) } }
impl From<u8> for Other { fn from(value: u8) -> Self { Other } }
pub struct Other;
pub trait Tr {}
pub trait Unused {}
pub struct Root;
impl Root {
    pub fn a<'a>(&self, x: &'a u8) -> A { A }
    pub fn g<T: Tr>(&self, t: T) -> G<T> { G(t) }
    pub fn b(&self) -> B { B(0) }
    fn hidden(&self) -> Other { Other }
}
pub struct B(m!());
impl B { pub fn q(&self) -> Qa { Qa } }
impl B { m!(); }
pub struct A;
impl A { pub fn q(&self) -> Qa { Qa } }
pub struct G<T: Tr>(T);
impl<T: Tr> G<T> where T: Clone { pub fn q(self) -> T { self.0 } }
impl Qa { pub fn new() -> Qa { Qa } }
pub struct Data;
impl Data { pub fn x(&self) -> u8 { 0 } }
"#,
                );

                let roots = Roots::of(&c, vec![id(&c, "Wire")]).unwrap();

                assert_eq!(names(&c, &[roots.root]), ["Root"]);
                assert_eq!(names(&c, &roots.traits), ["Tr"]);
                assert_eq!(names(&c, &roots.data), ["Wire"]);
            }

            #[test]
            fn of_should_fail_without_one_root() {
                let c = load(
                    r#"pub enum Wire { A(Qa) }
pub struct Qa;
impl From<Qa> for Wire { fn from(value: Qa) -> Self { Wire::A(value) } }
pub struct R1;
impl R1 { pub fn q(&self) -> Qa { Qa } }
pub struct R2;
impl R2 { pub fn q(&self) -> Qa { Qa } }
pub struct Plain;
"#,
                );

                let two = Roots::of(&c, vec![id(&c, "Wire")]).err().unwrap();
                let none = Roots::of(&c, vec![id(&c, "Plain")]).err().unwrap();

                assert_eq!(
                    two.to_string(),
                    "`Wire`: not one root of the makers but [R1, R2]"
                );
                assert_eq!(
                    none.to_string(),
                    "`Plain`: not one root of the makers but []"
                );
            }
        }
    }

    mod source_crate {
        use super::*;

        mod lower {
            use super::*;

            #[test]
            fn lower_should_take_the_structs_the_builders_return_as_builders() {
                let (m, errors) = lower(
                    r#"pub struct Root;
impl Root {
    pub fn select() -> Select { Select(Query { ids: Ids::All, kind: Kind::A }) }
    fn hidden() -> Hidden { Hidden }
}
pub struct Select(pub Query);
impl Select {
    pub fn ids(self, ids: Ids) -> SelectIds { SelectIds(self.0) }
    pub fn indexes(self) -> Indexes { Indexes }
}
pub struct SelectIds(pub Query);
impl SelectIds { pub fn query(&self) -> Query { self.0.clone() } }
pub struct Indexes;
impl Indexes {
    pub fn query(&self) -> Option<Query> { None }
    pub fn a(&self) -> std::borrow::Cow<'static, str> { todo() }
    pub fn b(&self) -> (&'static Query, [Kind; 1], fn()) { todo() }
}
pub struct Query { pub ids: Ids, kind: Kind }
impl Query { fn helper(&self) -> NotLoaded { NotLoaded } }
pub enum Ids { All, Some(Vec<u64>) }
pub enum Kind { A }
pub struct Hidden;
impl Hidden { pub fn x(self) {} }
pub struct NotLoaded;
pub struct Extra { pub a: u8 }
"#,
                    &[],
                    &["Extra"],
                );

                assert_eq!(errors, Vec::<String>::new());
                assert_eq!(
                    dump(&m),
                    "// ===== after lower\n// Root\nfn select() -> Select {\n    Select(Query { ids: Ids::All, kind: Kind::A })/*unresolved*/;\n}\n\n// Root\nfn hidden() -> Hidden {\n    Hidden;\n}\n\n// Select\nfn ids(self, ids: Ids) -> SelectIds {\n    SelectIds(self.0)/*unresolved*/;\n}\n\n// Select\nfn indexes(self) -> Indexes {\n    Indexes;\n}\n\n// Indexes\nfn query(&self) -> Option<Query> {\n    None;\n}\n\n// Indexes\nfn a(&self) -> ? {\n    todo()/*unresolved*/;\n}\n\n// Indexes\nfn b(&self) -> (&Query, Vec<Kind>, ?) {\n    todo()/*unresolved*/;\n}\n\n// SelectIds\nfn query(&self) -> Query {\n    self.0.clone()/*unresolved*/;\n}\n\nclass Extra<> { a: u8 }\nclass Hidden<> {  }\n  x\nclass Indexes<> {  }\n  query\n  a\n  b\nclass Query<> { ids: Ids, kind: Kind }\n  priv helper\nclass Root<> {  }\n  select\n  priv hidden\nclass Select<> { 0: Query }\n  ids\n  indexes\nclass SelectIds<> { 0: Query }\n  query\nenum Ids [Unit(\"All\"), Tuple(\"Some\", List(Int(\"u64\")))] []\nenum Kind [Unit(\"A\")] []\n"
                );
            }

            #[test]
            fn lower_should_load_the_conversions_defaults_and_skipped_defaults() {
                let (m, errors) = lower(
                    r#"pub struct Root {
    pub v: V, pub d: D, pub ats: Attrs, pub t: T, pub u: U, pub n: NoDefault, pub s: S, pub ms: Ms, pub nf: Nf,
    pub un: Unnamed, pub nd: NotDefault, pub odd: Odd, pub gen: Gen<u8>,
    #[serde(skip_serializing_if = "E::is_none")] pub a: E,
    #[serde(skip_serializing_if = "Option::is_none")] pub o: Option<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty")] pub l: Vec<u8>,
    #[serde(skip_serializing_if = "Root::other")] pub b: u8,
    #[serde(skip_serializing_if = "Root::two")] pub c: u8,
    #[serde(skip_serializing_if = "Root::guarded")] pub g: E,
    #[serde(skip_serializing_if = "E::payload")] pub p: E,
    #[serde(skip_serializing_if = "E::missing")] pub mi: E,
    #[serde(skip_serializing_if = "String::is_empty")] pub st: String,
    #[serde(skip_serializing_if = "is_zero")] pub z: u8,
    #[serde(skip_serializing_if = "1 +")] pub bad: u8,
    #[serde(rename = "x")] pub r: u8,
    #[serde(skip_serializing_if = "u8::is_power_of_two")] pub pw: u8,
    /// A doc.
    #[deprecated] #[allow(unused)] #[serde(default)] pub at: u8,
    #[serde(skip_serializing_if = 1)] pub lit: u8,
}
impl Clone for Root { fn clone(&self) -> Self { Root::default() } }
/// A doc.
#[must_use]
pub struct Attrs;
pub struct Unnamed(#[serde(skip_serializing_if = "Option::is_none")] pub Option<u8>);
#[derive(Default)] pub struct D { pub a: u8, pub b: Option<u8> }
#[derive(Default)] pub struct T(u8, u8);
#[derive(Default)] pub struct U;
#[derive(Clone, Default)] pub enum E { #[default] A, B(u8) }
#[derive(Default)] pub enum NoDefault { A }
#[derive(Clone)] pub struct NotDefault;
pub struct S;
impl Default for S { fn default() -> Self { S } }
pub struct Ms;
impl Default for Ms { fn default() -> Self { let x = 1; Ms } }
pub struct Nf;
impl Default for Nf { }
pub struct V(u64);
impl From<u64> for V { fn from(value: u64) -> Self { V(value) } }
impl From<u8> for V { fn from(v: u8) -> Self { V(v.into()) } }
impl<'a> From<&'a str> for V { fn from(s: &'a str) -> Self { let v = 1; V(v) } }
impl From<(u64, u8)> for V { fn from(value: (u64, u8)) -> Self { V(value.0) } }
impl From<Vec<u8>> for V { fn from(value: Vec<u8>) -> Self { V(value.iter().map(|v| { let x = *v; let v = x; v }).sum()) } }
impl From for V { fn from(value: u8) -> Self { V(1) } }
impl From<'static> for V { fn from(value: u8) -> Self { V(1) } }
impl From<i8> for V { const X: u8 = 1; fn from(value: i8) -> Self { V(1) } }
impl From<Vec<u64>> for V { fn from(value: Vec<u64>) -> Self { V(value.iter().map(|v| *v).sum()) } }
impl From<Unknown> for V { fn from(value: Unknown) -> Self { V(1) } }
impl From<V> for Unloaded { fn from(value: V) -> Self { Unloaded } }
impl<X: NotLoaded> From<X> for V { fn from(value: X) -> Self { V(1) } }
impl<X: Into<u64>> From<Vec<X>> for D where X: Clone { fn from(value: Vec<X>) -> Self { D::default() } }
impl From<u32> for V { }
pub struct Unloaded;
pub trait NotLoaded {}
impl E {
    pub fn is_none(&self) -> bool { matches!(self, E::A) }
    pub fn payload(&self) -> bool { matches!(self, E::B(_)) }
}
impl Root {
    pub fn other(&self) -> bool { self.b == 0 }
    pub fn two(&self) -> bool { let x = 1; true }
    pub fn guarded(&self) -> bool { matches!(self.g, E::A if true) }
}
pub fn is_zero(v: &u8) -> bool { *v == 0 }
pub struct Odd { #[serde(skip_serializing_if)] pub s: u8, #[serde(rename = )] pub r: u8 }
pub struct Gen<T>(T);
impl From<E> for V { fn from(value: E) -> Self { V(2) } }
impl From<Gen<u8>> for V { fn from(value: Gen<u8>) -> Self { V(3) } }
impl From<u16> for V { fn from() -> Self { V(0) } }
impl From<Option<u8>> for V { fn from(value: Option<u8>) -> Self { if let Some(v) = value { V(v) } else { V(0) } } }
impl From<Option<u16>> for V { fn from(value: Option<u16>) -> Self { match value { Some(v) => V(v), _ => V(0) } } }
impl From<Odd> for V { fn from(o: Odd) -> Self { match o { Odd { s: v, r: 1 } => V(v) } } }
impl From<Option<u32>> for V { fn from(x: Option<u32>) -> Self { if let Some(x) = x { V(1) } else { V(0) } } }
"#,
                    &[],
                    &[],
                );

                assert_eq!(
                    errors,
                    [
                        "/c/src/lib.rs:46: unsupported impl item `const X: u8 = 1;`",
                        "/c/src/lib.rs:1: `Root`: unsupported `skip_serializing_if = \"Root::other\"` of `b`",
                        "/c/src/lib.rs:1: `Root`: unsupported `skip_serializing_if = \"Root::two\"` of `c`",
                        "/c/src/lib.rs:1: `Root`: unsupported `skip_serializing_if = \"Root::guarded\"` of `g`",
                        "/c/src/lib.rs:1: `Root`: unsupported `skip_serializing_if = \"E::payload\"` of `p`",
                        "/c/src/lib.rs:1: `Root`: unsupported `skip_serializing_if = \"E::missing\"` of `mi`",
                        "/c/src/lib.rs:1: `Root`: unsupported `skip_serializing_if = \"String::is_empty\"` of `st`",
                        "/c/src/lib.rs:1: `Root`: unsupported `skip_serializing_if = \"is_zero\"` of `z`",
                        "/c/src/lib.rs:1: `Root`: unsupported `skip_serializing_if = \"1 +\"` of `bad`",
                        "/c/src/lib.rs:1: `Root`: unsupported `skip_serializing_if = \"u8::is_power_of_two\"` of `pw`"
                    ]
                );
                assert_eq!(
                    m.conversions
                        .iter()
                        .map(|c| format!(
                            "{} from {}{}: {} {:?}",
                            c.to,
                            print::ty(&c.from),
                            c.generics
                                .iter()
                                .map(|g| format!(" {}: {}", g.name, g.bounds.len()))
                                .collect::<String>(),
                            print::expr(&c.body),
                            c.errors
                        ))
                        .collect::<Vec<_>>(),
                    [
                        "V from u64: V(v)/*unresolved*/ []",
                        "V from u8: V(v.into()/*unresolved*/)/*unresolved*/ []",
                        "V from &str: /* unsupported: a conversion of more than one statement */ [\"unsupported a conversion of more than one statement\"]",
                        "V from (u64, u8): V(v.0)/*unresolved*/ []",
                        "V from Vec<u8>: V(v.iter()/*unresolved*/.map(|x0| {\n    let x: ? = *x0;\n    let x0: ? = x;\n    x0;\n})/*unresolved*/.sum()/*unresolved*/)/*unresolved*/ []",
                        "V from Vec<u64>: V(v.iter()/*unresolved*/.map(|x| {\n    *x;\n})/*unresolved*/.sum()/*unresolved*/)/*unresolved*/ []",
                        "D from Vec<X> X: 2: D::default()/*unresolved*/ []",
                        "V from E: V(2)/*unresolved*/ []",
                        "V from Gen<u8>: V(3)/*unresolved*/ []",
                        "V from Option<u8>: if let Some(x) = v {\n    V(x)/*unresolved*/;\n} else {\n    V(0)/*unresolved*/;\n} []",
                        "V from Option<u16>: match v {\n    Some(x) => {\n        V(x)/*unresolved*/;\n    },\n    _ => {\n        V(0)/*unresolved*/;\n    },\n} []",
                        "V from Odd: match v {\n    Odd { s: x, r: 1 } => {\n        V(x)/*unresolved*/;\n    },\n} []",
                        "V from Option<u32>: /* unsupported: a binding shadowing the parameter */ [\"unsupported a binding shadowing the parameter\"]"
                    ]
                );
                assert_eq!(
                    m.defaults
                        .iter()
                        .map(|(n, e)| format!("{n}: {}", print::expr(e)))
                        .collect::<Vec<_>>(),
                    [
                        "D: D { a: Default::default()/*unresolved*/, b: Default::default()/*unresolved*/ }",
                        "E: E::A",
                        "S: S",
                        "T: T(Default::default()/*unresolved*/, Default::default()/*unresolved*/)",
                        "U: U {  }"
                    ]
                );
                assert_eq!(
                    m.skipped_defaults
                        .iter()
                        .map(|(c, f, e)| format!("{c}.{f}: {}", print::expr(e)))
                        .collect::<Vec<_>>(),
                    ["Root.a: E::A", "Root.o: Option::None", "Root.l: vec![]"]
                );
            }

            #[test]
            fn lower_should_load_the_impls_of_the_traits() {
                let (m, errors) = lower(
                    r#"pub struct Root;
impl Root { pub fn s(self) -> S<Q> { S(Q) } }
pub struct S<T: Tr>(pub T);
impl<T: Tr> S<T> { pub fn get(self) -> T { self.0 } }
pub trait Tr {
    type Out;
    fn name(&self) -> String;
    fn other() -> String { String::new() }
}
pub struct Q;
impl Tr for Q {
    type Out = u8;
    fn name(&self) -> String { "q".to_string() }
}
impl Default for Q { fn default() -> Self { Q } }
impl Tr for u8 {
    type Out = u8;
    fn name(&self) -> String { String::new() }
}
impl Tr for &'static str {
    type Out = u8;
    fn name(&self) -> String { String::new() }
}
"#,
                    &["Tr"],
                    &[],
                );

                assert_eq!(errors, Vec::<String>::new());
                assert_eq!(
                    dump(&m),
                    "// ===== after lower\n// Root\nfn s(self) -> S<Q> {\n    S(Q)/*unresolved*/;\n}\n\n// S\nfn get(self) -> T {\n    self.0;\n}\n\n// impl Tr for Q\nfn name(&self) -> str {\n    \"q\".to_string()/*unresolved*/;\n}\n\nclass Q<> {  }\nclass Root<> {  }\n  s\nclass S<T> { 0: T }\n  get\ntrait Tr: fn name(&self) -> str {\n}\ntrait Tr: fn other() -> str {\n    string::String::new()/*unresolved*/;\n}\n"
                );
            }

            #[test]
            fn lower_should_lower_the_types() {
                let (m, errors) = lower(
                    r#"pub struct Root<T> {
    a: bool, b: i8, c: u128, d: f32, e: f64, f: String, g: &'static str, h: char,
    i: Vec<u8>, j: Option<u8>, k: Alias, l: Bad, m: (), n: (u8, bool), o: [u8; 4],
    p: &'static [u8], q: (u8), r: <u8 as Tr>::Out, s: fn(u8), t: Box<dyn Tr>,
    u: std::borrow::Cow<'static, str>, v: U, w: Al, x: Option<Box<Self>>, y: E,
    z: Unknown, aa: Q::Assoc, ab: T, ac: Q, ad: Q, ae: impl Tr,
}
impl<T> Root<T> { pub fn f(self) -> Self { self } }
type Alias = Vec<u8>;
type Bad = m!();
pub trait Tr { type Out; }
trait Al = Tr;
pub union U { a: u8 }
pub enum E { A }
pub struct Q;
mod a { pub struct Q; }
pub struct Dup(a::Q);
"#,
                    &[],
                    &["Dup"],
                );

                assert_eq!(
                    errors,
                    vec![
                        "/c/src/lib.rs:10: unsupported type `m!()`",
                        "/c/src/lib.rs:16: `Q`: another type has the name"
                    ]
                );
                assert_eq!(
                    dump(&m),
                    "// ===== after lower\n// Root\nfn f(self) -> Self {\n    self;\n}\n\nclass Dup<> { 0: Q }\nclass Q<> {  }\nclass Root<T> { a: bool, b: i8, c: u128, d: f64, e: f64, f: str, g: &str, h: str, i: Vec<u8>, j: Option<u8>, k: Vec<u8>, l: ?, m: (), n: (u8, bool), o: Vec<u8>, p: &Vec<u8>, q: u8, r: ?, s: ?, t: ?, u: ?, v: U, w: , x: Option<?>, y: E, z: ?, aa: ?, ab: T, ac: Q, ad: Q, ae: ? }\n  f\nenum E [Unit(\"A\")] []\n"
                );
            }

            #[test]
            fn lower_should_lower_the_generics_and_receivers() {
                let (m, errors) = lower(
                    r#"pub struct Root;
pub struct Q;
impl Root {
    pub fn g<'a, T: Into<Q> + Clone + 'a, const N: usize, I: Iterator<Item = u8>, F: Fn(u8)>(self, t: T) -> Self
    where
        T: Default,
        (T,): Clone,
        Self: Sized,
    {
        self
    }
    pub fn a(mut self) {}
    pub fn b(&self) {}
    pub fn c(&mut self) {}
    pub fn d(self: Box<Self>) {}
}
"#,
                    &[],
                    &[],
                );

                assert_eq!(errors, Vec::<String>::new());
                assert_eq!(
                    dump(&m),
                    "// ===== after lower\n// Root\nfn g<T: Into<Q> + Clone + Default, I: Iterator, F: Fn>(self, t: T) -> Self {\n    self;\n}\n\n// Root\nfn a(mut self) -> () {\n}\n\n// Root\nfn b(&self) -> () {\n}\n\n// Root\nfn c(&mut self) -> () {\n}\n\n// Root\nfn d(self) -> () {\n}\n\nclass Q<> {  }\nclass Root<> {  }\n  g\n  a\n  b\n  c\n  d\n"
                );
                assert_eq!(
                    func(&m, "Root", "d").errors,
                    vec!["unsupported a typed `self`"]
                );
            }

            #[test]
            fn lower_should_lower_the_bodies() {
                let (m, errors) = lower(
                    r#"pub struct Root { pub v: Vec<u8>, pub o: Option<u8> }
pub struct S { a: u8, b: u8 }
pub struct T(u8);
pub enum E { A, B(u8), C }
const A: &str = "a";
static ST: u8 = 1;
type Alias = S;
mod md {}
impl Root {
    pub fn new() -> Self { Root { v: vec![], o: None } }
    pub fn body(mut self, a: u8, e: E) -> u8 {
        let mut x = vec![1, 2];
        let y = 'c' == 'c';
        let t = (a, 1.5, "s", A, u64::MAX, 0x10);
        let u = ();
        let z;
        z = -a;
        x[0] += 1; x[0] -= 1; x[0] *= 1; x[0] /= 1; x[0] %= 1;
        x[0] ^= 1; x[0] &= 1; x[0] |= 1; x[0] <<= 1; x[0] >>= 1;
        self.v.push(a);
        let f = |b| b + 1;
        let g = |b: u8| { b };
        let r = &self.v;
        let d = *r;
        let s = S { a: 1, b: t.0 };
        let n = Self::new();
        let m = std::mem::take(&mut self.v);
        let o = Some(T(1));
        let i = Into::<u64>::into(a);
        let w = #[allow(unused)] (a);
        let q = Self { v: vec![], o: None };
        let p = E::B;
        let arr = [true, !false];
        let al = Alias { a: ST, b: 1 };
        break;
        continue;
        unknown();
        md;
        Vec::<'static, u8>::new();
        match s { S { a, .. } => a };
        if a > 1 { return 1; } else if a < 1 { 2; } else { 3; }
        if let Some(v) = self.o { v; }
        while a != 0 { a; }
        match e {
            E::A => 1,
            E::B(v) => { v }
            E::C => 3,
        };
        match o { None => 0, Some(T(v)) => v, Some(&(v)) => v };
        match a { 1 => 1, _ => 0 };
        { a; }
        panic!("no");
        panic!();
        return;
        a
    }
}
"#,
                    &[],
                    &[],
                );

                assert_eq!(errors, Vec::<String>::new());
                assert_eq!(
                    print::func(func(&m, "Root", "body")),
                    "fn body(mut self, a: u8, e: E) -> u8 {\n    let x: ? = vec![1, 2];\n    let y: ? = (\"c\" == \"c\");\n    let t: ? = (a, 1.5, \"s\", A, u64::MAX, 16);\n    let u: ? = ();\n    let z;\n    z = (0 - a);\n    x[0] = (x[0] + 1);\n    x[0] = (x[0] - 1);\n    x[0] = (x[0] * 1);\n    x[0] = (x[0] / 1);\n    x[0] = (x[0] % 1);\n    x[0] = (x[0] ^ 1);\n    x[0] = (x[0] & 1);\n    x[0] = (x[0] | 1);\n    x[0] = (x[0] << 1);\n    x[0] = (x[0] >> 1);\n    self.v.push(a)/*unresolved*/;\n    let f: ? = |b| {\n    (b + 1);\n};\n    let g: ? = |b| {\n    b;\n};\n    let r: ? = &self.v;\n    let d: ? = *r;\n    let s: ? = S { a: 1, b: t.0 };\n    let n: ? = Self::new()/*unresolved*/;\n    let m: ? = mem::take(&self.v)/*unresolved*/;\n    let o: ? = Some(T(1)/*unresolved*/)/*unresolved*/;\n    let i: ? = convert::Into::into(a)/*unresolved*/;\n    let w: ? = a;\n    let q: ? = Self { v: vec![], o: None };\n    let p: ? = E::B;\n    let arr: ? = vec![true, !false];\n    let al: ? = S { a: ST, b: 1 };\n    /* unsupported: `break` */;\n    /* unsupported: `continue` */;\n    unknown()/*unresolved*/;\n    md;\n    vec::Vec::new()/*unresolved*/;\n    match s {\n        S { a: a } => {\n            a;\n        },\n    };\n    if (a > 1) {\n        return 1;\n    } else {\n        if (a < 1) {\n            2;\n        } else {\n            3;\n        };\n    };\n    if let Some(v) = self.o {\n        v;\n    };\n    while (a != 0) {\n        a;\n    };\n    match e {\n        E::A => {\n            1;\n        },\n        E::B(v) => {\n            v;\n        },\n        E::C => {\n            3;\n        },\n    };\n    match o {\n        None => {\n            0;\n        },\n        Some(T(v)) => {\n            v;\n        },\n        Some(v) => {\n            v;\n        },\n    };\n    match a {\n        1 => {\n            1;\n        },\n        _ => {\n            0;\n        },\n    };\n    if true {\n        a;\n    };\n    panic!(\"no\");\n    panic!(\"panic\");\n    return ;\n    a;\n}"
                );
                assert_eq!(
                    func(&m, "Root", "body").errors,
                    vec!["unsupported `break`", "unsupported `continue`"]
                );
            }

            #[test]
            fn lower_should_report_the_constructs_it_does_not_lower() {
                let (m, errors) = lower(
                    r#"pub struct Root;
pub enum E { C { x: u8 } }
impl Root {
    pub fn f(self, a: u8, o: Option<u8>, v: Vec<u8>, e: E) -> u8 {
        async { 1 };
        a.await;
        loop { break; }
        for i in v { continue; }
        const { 1 };
        <Root>::f;
        0..1;
        [0; 3];
        o?;
        while let Some(x) = o {}
        match a { x if x > 0 => 1, _ => 0 };
        (f)();
        |(p, q)| p;
        let (p, q) = (1, 2);
        let Some(p) = o else { return 0 };
        fn inner() {}
        if a > 0 || let Some(x) = o {}
        assert!(a > 0);
        matches!(o, None);
        match a { x @ 1 => 0, 1 | 2 => 0, 3..=4 => 0, _ => 0 };
        match (a, a) { (p, q) => 0 };
        18446744073709551615;
        match a { 18446744073709551615 => 0, _ => 0 };
        E::C { x: 1 };
        match e { E::C { x } => x };
        0
    }
}
"#,
                    &[],
                    &[],
                );

                assert_eq!(errors, Vec::<String>::new());
                assert_eq!(
                    func(&m, "Root", "f").errors,
                    vec![
                        "unsupported `async`",
                        "unsupported `.await`",
                        "unsupported `loop`",
                        "unsupported `for`",
                        "unsupported a `const` block",
                        "unsupported a qualified path",
                        "unsupported a range",
                        "unsupported `[x; n]`",
                        "unsupported `?`",
                        "unsupported `while let`",
                        "unsupported a match guard",
                        "unsupported a call of an expression",
                        "unsupported a closure parameter pattern",
                        "unsupported a destructuring `let`",
                        "unsupported `let`-`else`",
                        "unsupported an item in a body",
                        "unsupported `let` in an expression",
                        "unsupported `assert!`",
                        "unsupported `matches!`",
                        "unsupported a `@` pattern",
                        "unsupported an or pattern",
                        "unsupported a range pattern",
                        "unsupported a tuple pattern",
                        "unsupported an integer beyond `i64`",
                        "unsupported an integer beyond `i64`",
                        "unsupported a struct variant",
                        "unsupported a struct variant pattern"
                    ]
                );
                assert_eq!(
                    print::func(func(&m, "Root", "f")),
                    "fn f(self, a: u8, o: Option<u8>, v: Vec<u8>, e: E) -> u8 {\n    /* unsupported: `async` */;\n    /* unsupported: `.await` */;\n    /* unsupported: `loop` */;\n    /* unsupported: `for` */;\n    /* unsupported: a `const` block */;\n    /* unsupported: a qualified path */;\n    /* unsupported: a range */;\n    /* unsupported: `[x; n]` */;\n    /* unsupported: `?` */;\n    /* unsupported: `while let` */;\n    /* unsupported: a match guard */;\n    /* unsupported: a call of an expression */;\n    /* unsupported: a closure parameter pattern */;\n    /* unsupported: a destructuring `let` */;\n    /* unsupported: `let`-`else` */;\n    /* unsupported: an item in a body */;\n    if ((a > 0) || /* unsupported: `let` in an expression */) {\n    };\n    /* unsupported: `assert!` */;\n    /* unsupported: `matches!` */;\n    match a {\n        _ => {\n            0;\n        },\n        _ => {\n            0;\n        },\n        _ => {\n            0;\n        },\n        _ => {\n            0;\n        },\n    };\n    match (a, a) {\n        _ => {\n            0;\n        },\n    };\n    /* unsupported: an integer beyond `i64` */;\n    match a {\n        _ => {\n            0;\n        },\n        _ => {\n            0;\n        },\n    };\n    /* unsupported: a struct variant */;\n    match e {\n        _ => {\n            x;\n        },\n    };\n    0;\n}"
                );
            }

            #[test]
            fn lower_should_load_the_consts_the_bodies_use() {
                let (m, errors) = lower(
                    r#"pub struct Root;
const A: &str = "a";
const B: u8 = C;
const C: u8 = 1;
const D: u8 = m!();
impl Root { pub fn f(self) -> u8 { A; A; D; B } }
"#,
                    &[],
                    &[],
                );

                assert_eq!(
                    errors,
                    vec!["/c/src/lib.rs:5: unsupported macro call `m!()`"]
                );
                assert_eq!(
                    dump(&m),
                    "// ===== after lower\n// Root\nfn f(self) -> u8 {\n    A;\n    A;\n    D;\n    B;\n}\n\nclass Root<> {  }\n  f\nconst A: str = \"a\"\nconst B: u8 = C\nconst C: u8 = 1\n"
                );
            }

            #[test]
            fn lower_should_leave_out_the_items_the_ir_does_not_have() {
                let (m, errors) = lower(
                    r#"pub struct Root { a: Bad, e: BadEnum, f: E }
pub struct Bad { a: m!() }
pub enum BadEnum { A(m!()) }
pub enum E { A, B(u8), C(u8, bool), D { x: u8 }, F(Vec<E>) }
impl E { pub fn f(&self) -> NotLoaded { NotLoaded } }
pub struct NotLoaded;
impl Root { pub fn f(self) { m!() } }
pub unsafe trait Tr {}
"#,
                    &["Tr"],
                    &[],
                );

                assert_eq!(
                    errors,
                    vec![
                        "/c/src/lib.rs:7: unsupported macro call `m!()`",
                        "/c/src/lib.rs:8: unsupported trait `pub unsafe trait Tr {}`",
                        "/c/src/lib.rs:3: unsupported type `m!()`",
                        "/c/src/lib.rs:2: unsupported type `m!()`"
                    ]
                );
                assert_eq!(
                    dump(&m),
                    "// ===== after lower\nclass Bad<> {  }\nclass Root<> { a: Bad, e: BadEnum, f: E }\nenum BadEnum [] []\nenum E [Unit(\"A\"), Tuple(\"B\", Int(\"u8\")), Tuple(\"C\", Tuple([Int(\"u8\"), Bool])), Struct(\"D\", [(\"x\", Int(\"u8\"))]), Tuple(\"F\", List(Named(\"E\", [])))] [\"f\"]\n"
                );
            }
        }

        mod lower_tests {
            use super::*;

            #[test]
            fn lower_tests_should_lower_the_test_functions_and_the_types_of_a_test_crate() {
                let (m, errors) = tested(
                    r#"pub struct Root { pub e: E }
pub enum E { A, B(u8) }
pub use std::string::String as Text;
impl Root {
    pub fn new() -> Root { Root { e: E::A } }
}"#,
                    r#"use c::Root;
#[derive(Default)]
struct T { n: u8 }
struct N;
enum E { X }
#[test]
fn builds() {
    let r: c::Root = Root::new();
    let e = c::E::B(1);
    let o: Option<u8> = None;
    let p: c = c::nothing::X;
    let s: c::Text;
    assert_eq!(r.e, T::default().n);
    assert_eq!(1, 2, "m");
}
fn helper() {}"#,
                    &[],
                );

                assert_eq!(
                    dump(&m),
                    "// ===== after lower\n// Root\nfn new() -> Root {\n    Root { e: E::A };\n}\n\n// test\nfn builds() -> () {\n    let r: Root = Root::new()/*unresolved*/;\n    let e: ? = E::B(1)/*unresolved*/;\n    let o: Option<u8> = None;\n    let p: ? = nothing::X;\n    let s;\n    .AssertEq(r.e, T::default()/*unresolved*/.n);\n    /* unsupported: `assert_eq!` */;\n}\n\nclass N<> {  }\nclass Root<> { e: E }\n  new\nclass T<> { n: u8 }\nenum E [Unit(\"A\"), Tuple(\"B\", Int(\"u8\"))] []\n"
                );
                assert_eq!(
                    (
                        m.test_types.iter().cloned().collect::<Vec<_>>(),
                        m.defaults.keys().cloned().collect::<Vec<_>>()
                    ),
                    (
                        vec!["N".to_string(), "T".to_string()],
                        vec!["T".to_string()]
                    )
                );
                assert_eq!(
                    errors,
                    ["/c/tests/t.rs:5: `E`: a type of the tested crate has the name"]
                );
            }
        }
    }

    mod lower {
        use super::*;

        mod name {
            use super::*;

            #[test]
            fn name_should_be_the_identifier_of_the_item() {
                let c = load(
                    "pub struct Root; impl Root {} pub fn f() {} pub union U { a: u8 } pub type Al = u8;",
                );
                let roots = Roots {
                    root: id(&c, "Root"),
                    traits: Vec::new(),
                    data: Vec::new(),
                };
                let lower = Lower::new(&c, &roots.traits);
                let item = |index| ItemId {
                    module: c.root(),
                    index,
                };

                assert_eq!(
                    (0..5).map(|i| lower.name(item(i))).collect::<Vec<_>>(),
                    ["Root", "", "f", "U", "Al"]
                );
            }
        }

        mod default {
            use super::*;

            #[test]
            fn default_should_be_none_for_a_deriving_item_other_than_a_struct_or_an_enum() {
                let c = load("#[derive(Default)] pub type Al = u8;");
                let mut lower = Lower::new(&c, &[]);

                let default = lower.default(id(&c, "Al"));

                assert_eq!(default, None);
            }
        }
    }

    mod variant_name {
        use super::*;

        #[test]
        fn variant_name_should_be_empty_for_another_item() {
            let c = load("pub struct Root;");

            assert_eq!(variant_name(c.item(id(&c, "Root")), 0), "");
        }
    }
}
