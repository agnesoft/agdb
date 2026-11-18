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

impl SourceCrate {
    /// The builder API from `roots.root`. The builders are the structs the
    /// return types of the builders' `pub` methods lead to that have `pub`
    /// methods with a receiver; the other types reached through fields, signatures and
    /// `roots.data` are data. A type has its inherent methods (the unused
    /// ones of data types are left to [`Prune`](super::pass::Prune)) and
    /// the methods of its impls of `roots.traits`.
    ///
    /// The bodies are syntactic: method calls by name and paths are left to
    /// [`Resolve`](super::pass::Resolve). A construct the lowered IR does
    /// not have is a [`K::Unsupported`] and among the [`Func::errors`].
    pub fn lower(&self, roots: &Roots) -> (Module, Vec<Error>) {
        let mut lower = Lower::new(self, roots);
        lower.run(roots);
        (lower.m, lower.errors)
    }
}

struct Lower<'a> {
    c: &'a SourceCrate,
    inherent: HashMap<ItemId, Vec<ItemId>>,
    trait_impls: HashMap<ItemId, Vec<(ItemId, ItemId)>>,
    items: HashMap<ItemId, Option<ir::Item>>,
    names: HashMap<String, ItemId>,
    builders: HashSet<ItemId>,
    queue: Vec<ItemId>,
    consts: Vec<ItemId>,
    fn_errors: Vec<String>,
    errors: Vec<Error>,
    m: Module,
}

impl<'a> Lower<'a> {
    fn new(c: &'a SourceCrate, roots: &Roots) -> Self {
        let mut inherent: HashMap<ItemId, Vec<ItemId>> = HashMap::new();
        let mut trait_impls: HashMap<ItemId, Vec<(ItemId, ItemId)>> = HashMap::new();

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
                Some((path, _)) => {
                    if let Resolved::Item(t) = c.resolve(id.module, path, Namespace::Type)
                        && roots.traits.contains(&t)
                    {
                        trait_impls.entry(ty).or_default().push((t, id));
                    }
                }
            }
        }

        Self {
            c,
            inherent,
            trait_impls,
            items: HashMap::new(),
            names: HashMap::new(),
            builders: HashSet::new(),
            queue: Vec::new(),
            consts: Vec::new(),
            fn_errors: Vec::new(),
            errors: Vec::new(),
            m: Module::default(),
        }
    }

    fn run(&mut self, roots: &Roots) {
        self.find_builders(roots.root);

        for t in &roots.traits {
            self.load_trait(*t);
        }

        self.queue.push(roots.root);
        self.drain();

        for d in &roots.data {
            self.reach(*d);
            self.drain();
        }

        let mut loaded = 0;

        while loaded < self.consts.len() {
            self.load_const(self.consts[loaded]);
            loaded += 1;
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
        match self.c.item(id) {
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
                Some(name) => Expr::new(K::Let {
                    name,
                    value: init.as_ref().map(|v| self.expr(v).boxed()),
                }),
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

                match &**cond {
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
            ir::Expr::Cast { .. } => self.unsupported("`as`"),
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
            Resolved::External(p) => match p.as_slice() {
                [_, module, ty, variant] if module == "option" && ty == "Option" => {
                    Some(vec![variant.clone()])
                }
                p => Some(p[1..].to_vec()),
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::id;
    use crate::codegen::test_utilities::load;
    use crate::codegen::test_utilities::lower;

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
        a as u64;
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
        if a > 0 && let Some(x) = o {}
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
                        "unsupported `as`",
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
                    "fn f(self, a: u8, o: Option<u8>, v: Vec<u8>, e: E) -> u8 {\n    /* unsupported: `async` */;\n    /* unsupported: `.await` */;\n    /* unsupported: `loop` */;\n    /* unsupported: `for` */;\n    /* unsupported: `as` */;\n    /* unsupported: a `const` block */;\n    /* unsupported: a qualified path */;\n    /* unsupported: a range */;\n    /* unsupported: `[x; n]` */;\n    /* unsupported: `?` */;\n    /* unsupported: `while let` */;\n    /* unsupported: a match guard */;\n    /* unsupported: a call of an expression */;\n    /* unsupported: a closure parameter pattern */;\n    /* unsupported: a destructuring `let` */;\n    /* unsupported: `let`-`else` */;\n    /* unsupported: an item in a body */;\n    if ((a > 0) && /* unsupported: `let` in an expression */) {\n    };\n    /* unsupported: `assert!` */;\n    /* unsupported: `matches!` */;\n    match a {\n        _ => {\n            0;\n        },\n        _ => {\n            0;\n        },\n        _ => {\n            0;\n        },\n        _ => {\n            0;\n        },\n    };\n    match (a, a) {\n        _ => {\n            0;\n        },\n    };\n    /* unsupported: an integer beyond `i64` */;\n    match a {\n        _ => {\n            0;\n        },\n        _ => {\n            0;\n        },\n    };\n    /* unsupported: a struct variant */;\n    match e {\n        _ => {\n            x;\n        },\n    };\n    0;\n}"
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
                let lower = Lower::new(&c, &roots);
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
