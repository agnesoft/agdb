use super::ir::Args;
use super::ir::Arm;
use super::ir::Attr;
use super::ir::BinOp;
use super::ir::Block;
use super::ir::Bound;
use super::ir::Closure;
use super::ir::Const;
use super::ir::Enum;
use super::ir::Expr;
use super::ir::ExternCrate;
use super::ir::Field;
use super::ir::FieldPat;
use super::ir::FieldValue;
use super::ir::Fields;
use super::ir::Fn;
use super::ir::FnTypes;
use super::ir::FormatArg;
use super::ir::FormatArgs;
use super::ir::GenericArg;
use super::ir::GenericParam;
use super::ir::Generics;
use super::ir::Impl;
use super::ir::ImplItem;
use super::ir::ImplItemKind;
use super::ir::Item;
use super::ir::ItemKind;
use super::ir::Lit;
use super::ir::Macro;
use super::ir::MacroCall;
use super::ir::Member;
use super::ir::Param;
use super::ir::Pat;
use super::ir::Path;
use super::ir::Predicate;
use super::ir::Qualified;
use super::ir::Receiver;
use super::ir::Segment;
use super::ir::Signature;
use super::ir::Static;
use super::ir::Stmt;
use super::ir::Struct;
use super::ir::Target;
use super::ir::Trait;
use super::ir::TraitItem;
use super::ir::TraitItemKind;
use super::ir::Type;
use super::ir::TypeAlias;
use super::ir::UnOp;
use super::ir::Use;
use super::ir::UseTree;
use super::ir::Variant;
use super::ir::Vis;
use crate::Error;
use crate::ItemId;
use crate::ModuleId;
use crate::Namespace;
use crate::Resolved;
use crate::SourceCrate;
use crate::SourceFile;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;

impl SourceCrate {
    /// The items, in order. The IR has the constructs the items use; an
    /// item using another one is left out and the construct reported.
    /// `macro_rules!` are left out silently, they are no declarations, and so
    /// are `mod` items, the items of a module are items of their own.
    ///
    /// The source text comes from the spans of the syntax trees, which are
    /// valid on the thread that loaded the crate only.
    pub fn ir(&self, ids: impl IntoIterator<Item = ItemId>) -> (Vec<(ItemId, Item)>, Vec<Error>) {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        for id in ids {
            let mut load = Load {
                c: self,
                module: id.module(),
                file: self.file(self.module(id.module()).file()),
                generics: Vec::new(),
                scope: Vec::new(),
                barrier: 0,
                errors: Vec::new(),
            };

            if let Some(item) = load.item(self.item(id))
                && load.errors.is_empty()
            {
                items.push((id, item));
            }

            errors.append(&mut load.errors);
        }

        (items, errors)
    }
}

impl SourceCrate {
    /// The IR of a type written in `module` where `generics` are generic
    /// parameters, and the constructs it uses that the IR does not have.
    pub(super) fn ir_ty(
        &self,
        module: ModuleId,
        generics: &[String],
        ty: &syn::Type,
    ) -> (Type, Vec<Error>) {
        let mut load = Load {
            c: self,
            module,
            file: self.file(self.module(module).file()),
            generics: generics.to_vec(),
            scope: Vec::new(),
            barrier: 0,
            errors: Vec::new(),
        };
        let ty = load.ty(ty);
        (ty, load.errors)
    }
}

struct Load<'a> {
    c: &'a SourceCrate,
    module: ModuleId,
    file: &'a SourceFile,
    /// The names of the generic parameters in scope.
    generics: Vec<String>,
    /// The names the bodies being loaded declare, innermost last.
    scope: Vec<Scoped>,
    /// The bindings before it are of the bodies around the item being
    /// loaded, out of its scope.
    barrier: usize,
    errors: Vec<Error>,
}

enum Scoped {
    Binding(String),
    Item(String),
    /// The name a `use` of a block imports, with the path it imports.
    Use(String, syn::Path),
}

impl Load<'_> {
    fn item(&mut self, item: &syn::Item) -> Option<Item> {
        let (attrs, vis, kind) = match item {
            syn::Item::Struct(s) => {
                let generics = self.generics(&s.generics);
                let kind = ItemKind::Struct(Struct {
                    name: s.ident.to_string(),
                    generics,
                    fields: self.fields(&s.fields),
                });
                (&s.attrs, self.vis(&s.vis), kind)
            }
            syn::Item::Enum(e) => {
                let generics = self.generics(&e.generics);
                let variants = e.variants.iter().map(|v| self.variant(v)).collect();
                let kind = ItemKind::Enum(Enum {
                    name: e.ident.to_string(),
                    generics,
                    variants,
                });
                (&e.attrs, self.vis(&e.vis), kind)
            }
            syn::Item::Trait(t) => {
                if t.modifiers.auto_token.is_some() {
                    self.unsupported(t, "trait");
                }
                let generics = self.generics(&t.generics);
                let kind = ItemKind::Trait(Trait {
                    name: t.ident.to_string(),
                    generics,
                    supertraits: self.bounds(&t.supertraits),
                    items: t.items.iter().filter_map(|i| self.trait_item(i)).collect(),
                });
                (&t.attrs, self.vis(&t.vis), kind)
            }
            syn::Item::Impl(i) => {
                if i.modifiers.defaultness.is_some() || i.modifiers.polarity.is_some() {
                    self.unsupported(i, "impl");
                }
                let generics = self.generics(&i.generics);
                let kind = ItemKind::Impl(Impl {
                    generics,
                    trait_: i.trait_.as_ref().map(|(path, _)| self.type_path(path)),
                    self_ty: self.ty(&i.self_ty),
                    items: i.items.iter().filter_map(|i| self.impl_item(i)).collect(),
                });
                (&i.attrs, Vis::Private, kind)
            }
            syn::Item::Fn(f) => {
                let kind = ItemKind::Fn(self.function(&f.sig, &f.block));
                (&f.attrs, self.vis(&f.vis), kind)
            }
            syn::Item::Const(k) => {
                let kind = ItemKind::Const(Const {
                    name: k.ident.to_string(),
                    ty: self.ty(&k.ty),
                    expr: self.expr(&k.expr),
                });
                (&k.attrs, self.vis(&k.vis), kind)
            }
            syn::Item::Static(s) => {
                if matches!(s.mutability, syn::StaticMutability::Mut(_)) {
                    self.unsupported(s, "static");
                }
                let kind = ItemKind::Static(Static {
                    name: s.ident.to_string(),
                    ty: self.ty(&s.ty),
                    expr: self.expr(&s.expr),
                });
                (&s.attrs, self.vis(&s.vis), kind)
            }
            syn::Item::Type(t) => {
                let generics = self.generics(&t.generics);
                let kind = ItemKind::TypeAlias(TypeAlias {
                    name: t.ident.to_string(),
                    generics,
                    ty: self.ty(&t.ty),
                });
                (&t.attrs, self.vis(&t.vis), kind)
            }
            syn::Item::Use(u) => {
                let prefix = syn::Path {
                    leading_colon: u.leading_colon,
                    segments: Punctuated::new(),
                };
                let kind = ItemKind::Use(Use {
                    leading_colon: u.leading_colon.is_some(),
                    tree: self.use_tree(&prefix, &u.tree),
                });
                (&u.attrs, self.vis(&u.vis), kind)
            }
            syn::Item::ExternCrate(e) => {
                let kind = ItemKind::ExternCrate(ExternCrate {
                    name: e.ident.to_string(),
                    rename: e.rename.as_ref().map(|(_, r)| r.to_string()),
                });
                (&e.attrs, self.vis(&e.vis), kind)
            }
            syn::Item::Macro(_) | syn::Item::Mod(_) => return None,
            other => {
                self.unsupported(other, "item");
                return None;
            }
        };

        Some(Item {
            attrs: self.attrs(attrs),
            vis,
            kind,
        })
    }

    /// An item of a block: the bindings and generic parameters around it are
    /// not in its scope.
    fn nested(&mut self, item: &syn::Item) -> Option<Item> {
        let scope = self.scope.len();
        let barrier = std::mem::replace(&mut self.barrier, scope);
        let generics = std::mem::take(&mut self.generics);
        let item = self.item(item);
        self.generics = generics;
        self.barrier = barrier;
        self.scope.truncate(scope);
        item
    }

    /// The outer attributes. An inner one is unsupported, a function's in its
    /// body included.
    fn attrs(&mut self, attrs: &[syn::Attribute]) -> Vec<Attr> {
        attrs
            .iter()
            .filter_map(|attr| match (&attr.style, &attr.meta) {
                (syn::AttrStyle::Inner(_), _) => {
                    self.unsupported(attr, "attribute");
                    None
                }
                (
                    _,
                    syn::Meta::NameValue(syn::MetaNameValue {
                        path,
                        value:
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(doc),
                                ..
                            }),
                        ..
                    }),
                ) if path.is_ident("doc") => Some(Attr::Doc(doc.value())),
                // WHY: an applied `cfg_attr` keeps the brackets of the
                // original attribute, the meta has its own spans.
                (_, meta) => Some(Attr::Meta(self.text(meta))),
            })
            .collect()
    }

    fn vis(&mut self, vis: &syn::Visibility) -> Vis {
        match vis {
            syn::Visibility::Public(_) => Vis::Public,
            syn::Visibility::Inherited => Vis::Private,
            syn::Visibility::Restricted(r) if r.in_token.is_none() && r.path.is_ident("crate") => {
                Vis::Crate
            }
            syn::Visibility::Restricted(r) if r.in_token.is_none() && r.path.is_ident("super") => {
                Vis::Super
            }
            other => {
                self.unsupported(other, "visibility");
                Vis::Private
            }
        }
    }

    /// Brings the parameters into scope for the rest of the item.
    fn generics(&mut self, generics: &syn::Generics) -> Generics {
        for param in &generics.params {
            match param {
                syn::GenericParam::Type(t) => self.generics.push(t.ident.to_string()),
                syn::GenericParam::Const(c) => self.generics.push(c.ident.to_string()),
                syn::GenericParam::Lifetime(_) => {}
            }
        }

        let params = generics
            .params
            .iter()
            .map(|param| match param {
                syn::GenericParam::Lifetime(l) => {
                    if !l.attrs.is_empty() || !l.bounds.is_empty() {
                        self.unsupported(l, "lifetime parameter");
                    }
                    GenericParam::Lifetime(l.lifetime.to_string())
                }
                syn::GenericParam::Type(t) => {
                    if !t.attrs.is_empty() || t.default.is_some() {
                        self.unsupported(t, "type parameter");
                    }
                    GenericParam::Type {
                        name: t.ident.to_string(),
                        bounds: self.bounds(&t.bounds),
                    }
                }
                syn::GenericParam::Const(c) => {
                    if !c.attrs.is_empty() || c.default.is_some() {
                        self.unsupported(c, "const parameter");
                    }
                    GenericParam::Const {
                        name: c.ident.to_string(),
                        ty: self.ty(&c.ty),
                    }
                }
            })
            .collect();

        let predicates = generics
            .where_clause
            .iter()
            .flat_map(|w| &w.predicates)
            .filter_map(|predicate| match predicate {
                syn::WherePredicate::Type(p) if p.attrs.is_empty() && p.lifetimes.is_none() => {
                    Some(Predicate {
                        ty: self.ty(&p.bounded_ty),
                        bounds: self.bounds(&p.bounds),
                    })
                }
                other => {
                    self.unsupported(other, "where predicate");
                    None
                }
            })
            .collect();

        Generics { params, predicates }
    }

    fn bounds(&mut self, bounds: &Punctuated<syn::TypeParamBound, syn::Token![+]>) -> Vec<Bound> {
        bounds
            .iter()
            .filter_map(|bound| match bound {
                syn::TypeParamBound::Trait(t)
                    if t.paren_token.is_none() && t.lifetimes.is_none() && t.maybe.is_none() =>
                {
                    Some(Bound::Trait(self.type_path(&t.path)))
                }
                syn::TypeParamBound::Lifetime(l) => Some(Bound::Lifetime(l.to_string())),
                other => {
                    self.unsupported(other, "bound");
                    None
                }
            })
            .collect()
    }

    fn fields(&mut self, fields: &syn::Fields) -> Fields {
        match fields {
            syn::Fields::Named(named) => Fields::Named(self.field_list(&named.named)),
            syn::Fields::Unnamed(unnamed) => Fields::Unnamed(self.field_list(&unnamed.unnamed)),
            syn::Fields::Unit => Fields::Unit,
        }
    }

    fn field_list(&mut self, fields: &Punctuated<syn::Field, syn::Token![,]>) -> Vec<Field> {
        fields
            .iter()
            .map(|field| Field {
                attrs: self.attrs(&field.attrs),
                vis: self.vis(&field.vis),
                name: field.ident.as_ref().map(ToString::to_string),
                ty: self.ty(&field.ty),
            })
            .collect()
    }

    fn variant(&mut self, variant: &syn::Variant) -> Variant {
        Variant {
            attrs: self.attrs(&variant.attrs),
            name: variant.ident.to_string(),
            fields: self.fields(&variant.fields),
            discriminant: variant.discriminant.as_ref().map(|(_, e)| self.expr(e)),
        }
    }

    fn trait_item(&mut self, item: &syn::TraitItem) -> Option<TraitItem> {
        let (generics, scope) = (self.generics.len(), self.scope.len());
        let (attrs, kind) = match item {
            syn::TraitItem::Fn(f) => {
                let kind = TraitItemKind::Fn {
                    sig: self.signature(&f.sig),
                    body: f.default.as_ref().map(|b| self.block(b)),
                };
                (&f.attrs, kind)
            }
            syn::TraitItem::Type(t)
                if t.generics.params.is_empty()
                    && t.generics.where_clause.is_none()
                    && t.default.is_none() =>
            {
                let kind = TraitItemKind::Type {
                    name: t.ident.to_string(),
                    bounds: self.bounds(&t.bounds),
                };
                (&t.attrs, kind)
            }
            other => {
                self.unsupported(other, "trait item");
                return None;
            }
        };
        self.generics.truncate(generics);
        self.scope.truncate(scope);

        Some(TraitItem {
            attrs: self.attrs(attrs),
            kind,
        })
    }

    fn impl_item(&mut self, item: &syn::ImplItem) -> Option<ImplItem> {
        let (generics, scope) = (self.generics.len(), self.scope.len());
        let (attrs, vis, kind) = match item {
            syn::ImplItem::Fn(f) => {
                if f.modifiers.defaultness.is_some() {
                    self.unsupported(f, "fn");
                }
                (
                    &f.attrs,
                    &f.vis,
                    ImplItemKind::Fn(self.function(&f.sig, &f.block)),
                )
            }
            syn::ImplItem::Type(t)
                if t.modifiers.defaultness.is_none()
                    && t.generics.params.is_empty()
                    && t.generics.where_clause.is_none() =>
            {
                let kind = ImplItemKind::Type {
                    name: t.ident.to_string(),
                    ty: self.ty(&t.ty),
                };
                (&t.attrs, &t.vis, kind)
            }
            other => {
                self.unsupported(other, "impl item");
                return None;
            }
        };
        self.generics.truncate(generics);
        self.scope.truncate(scope);

        Some(ImplItem {
            attrs: self.attrs(attrs),
            vis: self.vis(vis),
            kind,
        })
    }

    fn function(&mut self, sig: &syn::Signature, block: &syn::Block) -> Fn {
        Fn {
            sig: self.signature(sig),
            body: self.block(block),
        }
    }

    /// Brings the receiver and the parameters into scope.
    fn signature(&mut self, sig: &syn::Signature) -> Signature {
        if !matches!(sig.safety, syn::Safety::Default)
            || sig.abi.is_some()
            || sig.variadic.is_some()
        {
            self.unsupported(sig, "signature");
        }

        let generics = self.generics(&sig.generics);
        let mut receiver = None;
        let mut params = Vec::new();

        for input in &sig.inputs {
            match input {
                syn::FnArg::Receiver(r) => receiver = self.receiver(r),
                syn::FnArg::Typed(t) => params.extend(self.param(t)),
            }
        }

        Signature {
            constness: sig.constness.is_some(),
            asyncness: sig.asyncness.is_some(),
            name: sig.ident.to_string(),
            generics,
            receiver,
            params,
            output: self.output(&sig.output),
        }
    }

    fn receiver(&mut self, r: &syn::Receiver) -> Option<Receiver> {
        self.scope.push(Scoped::Binding("self".to_string()));

        match (&r.kind, r.attrs.is_empty()) {
            (syn::ReceiverKind::Value, true) => Some(Receiver::Value {
                mutable: r.mutability.is_some(),
            }),
            (syn::ReceiverKind::Reference(_, lifetime, mutability), true) => Some(Receiver::Ref {
                lifetime: lifetime.as_ref().map(ToString::to_string),
                mutable: mutability.is_some(),
            }),
            (syn::ReceiverKind::Typed(_, ty), true) => Some(Receiver::Typed {
                mutable: r.mutability.is_some(),
                ty: Box::new(self.ty(ty)),
            }),
            _ => {
                self.unsupported(r, "receiver");
                None
            }
        }
    }

    fn param(&mut self, param: &syn::PatType) -> Option<Param> {
        if let syn::Pat::Ident(p) = &*param.pat
            && param.attrs.is_empty()
            && p.attrs.is_empty()
            && p.by_ref.is_none()
            && p.subpat.is_none()
        {
            self.scope.push(Scoped::Binding(p.ident.to_string()));
            Some(Param {
                mutable: p.mutability.is_some(),
                name: p.ident.to_string(),
                ty: self.ty(&param.ty),
            })
        } else {
            self.unsupported(param, "parameter");
            None
        }
    }

    fn ty(&mut self, ty: &syn::Type) -> Type {
        match ty {
            syn::Type::Path(p) if p.attrs.is_empty() => match &p.qself {
                None => Type::Path(self.type_path(&p.path)),
                Some(qself) => Type::Qualified(Box::new(self.qualified(qself, &p.path, false))),
            },
            syn::Type::Reference(r) if r.attrs.is_empty() => Type::Ref {
                lifetime: r.lifetime.as_ref().map(ToString::to_string),
                mutable: r.mutability.is_some(),
                ty: Box::new(self.ty(&r.elem)),
            },
            syn::Type::Tuple(t) if t.attrs.is_empty() => {
                Type::Tuple(t.elems.iter().map(|t| self.ty(t)).collect())
            }
            syn::Type::Slice(s) if s.attrs.is_empty() => Type::Slice(Box::new(self.ty(&s.elem))),
            syn::Type::Array(a) if a.attrs.is_empty() => {
                Type::Array(Box::new(self.ty(&a.elem)), self.boxed(&a.len))
            }
            syn::Type::ImplTrait(i) if i.attrs.is_empty() => {
                Type::ImplTrait(self.bounds(&i.bounds))
            }
            syn::Type::TraitObject(t) if t.attrs.is_empty() && t.dyn_token.is_some() => {
                Type::DynTrait(self.bounds(&t.bounds))
            }
            syn::Type::Paren(p) if p.attrs.is_empty() => Type::Paren(Box::new(self.ty(&p.elem))),
            syn::Type::FnPtr(f)
                if f.attrs.is_empty()
                    && f.lifetimes.is_none()
                    && f.unsafety.is_none()
                    && f.abi.is_none()
                    && f.variadic.is_none() =>
            {
                Type::FnPtr(self.fn_types(&f.inputs, &f.output))
            }
            syn::Type::Infer(i) if i.attrs.is_empty() => Type::Infer,
            other => {
                self.unsupported(other, "type");
                Type::Tuple(Vec::new())
            }
        }
    }

    /// The first `position` segments of the path are the trait, the rest
    /// are written as in an expression when `expr`.
    fn qualified(&mut self, qself: &syn::QSelf, path: &syn::Path, expr: bool) -> Qualified {
        let ty = self.ty(&qself.ty);
        let trait_ = (qself.position > 0).then(|| {
            self.type_path(&syn::Path {
                leading_colon: path.leading_colon,
                segments: path.segments.iter().take(qself.position).cloned().collect(),
            })
        });

        Qualified {
            ty,
            trait_,
            rest: path
                .segments
                .iter()
                .skip(qself.position)
                .map(|s| self.segment(s, expr))
                .collect(),
        }
    }

    fn fn_types(
        &mut self,
        inputs: &Punctuated<syn::NamedArg, syn::Token![,]>,
        output: &syn::ReturnType,
    ) -> FnTypes {
        FnTypes {
            inputs: inputs
                .iter()
                .filter_map(|arg| {
                    if arg.attrs.is_empty() && arg.name.is_none() {
                        Some(self.ty(&arg.ty))
                    } else {
                        self.unsupported(arg, "parameter");
                        None
                    }
                })
                .collect(),
            output: self.output(output).map(Box::new),
        }
    }

    fn output(&mut self, output: &syn::ReturnType) -> Option<Type> {
        match output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(self.ty(ty)),
        }
    }

    /// The tree of a `use`, after the path `prefix`.
    fn use_tree(&mut self, prefix: &syn::Path, tree: &syn::UseTree) -> UseTree {
        match tree {
            syn::UseTree::Path(p) => UseTree::Path {
                name: p.ident.to_string(),
                tree: Box::new(self.use_tree(&joined(prefix, &p.ident), &p.tree)),
            },
            syn::UseTree::Name(n) => UseTree::Name {
                name: n.ident.to_string(),
                rename: None,
                target: self.use_target(&imported(prefix, &n.ident)),
            },
            syn::UseTree::Rename(r) => UseTree::Name {
                name: r.ident.to_string(),
                rename: Some(r.rename.to_string()),
                target: self.use_target(&imported(prefix, &r.ident)),
            },
            syn::UseTree::Glob(_) => UseTree::Glob(self.target(prefix, Namespace::Type)),
            syn::UseTree::Group(g) => {
                UseTree::Group(g.items.iter().map(|t| self.use_tree(prefix, t)).collect())
            }
        }
    }

    /// What the path names in the first namespace it names something in.
    fn use_target(&self, path: &syn::Path) -> Target {
        [Namespace::Type, Namespace::Value, Namespace::Macro]
            .into_iter()
            .map(|namespace| self.target(path, namespace))
            .find(|target| *target != Target::Resolved(Resolved::Unresolved))
            .unwrap_or(Target::Resolved(Resolved::Unresolved))
    }

    fn type_path(&mut self, path: &syn::Path) -> Path {
        let target = self.target(path, Namespace::Type);
        self.path_with(path, target, false)
    }

    /// A path of an expression or a pattern, its generic arguments after
    /// `::`.
    fn expr_path(&mut self, path: &syn::Path, namespace: Namespace) -> Path {
        let target = self.target(path, namespace);
        self.path_with(path, target, true)
    }

    fn path_with(&mut self, path: &syn::Path, target: Target, expr: bool) -> Path {
        Path {
            leading_colon: path.leading_colon.is_some(),
            segments: path
                .segments
                .iter()
                .map(|s| self.segment(s, expr))
                .collect(),
            target,
        }
    }

    /// What the path names: a binding names a single value, the items and
    /// imports of the blocks in scope come before the generic parameters and
    /// the names of the module.
    fn target(&self, path: &syn::Path, namespace: Namespace) -> Target {
        if path.leading_colon.is_some() {
            return Target::Resolved(self.c.resolve(self.module, path, namespace));
        }

        let first = path.segments[0].ident.to_string();
        let value = path.segments.len() == 1 && namespace == Namespace::Value;

        if first == "Self" {
            return Target::SelfType;
        }

        for (index, scoped) in self.scope.iter().enumerate().rev() {
            match scoped {
                Scoped::Binding(name) if *name == first && value && index >= self.barrier => {
                    return Target::Local;
                }
                Scoped::Item(name) if *name == first => return Target::LocalItem,
                Scoped::Use(name, used) if *name == first => {
                    let mut path = path.clone();
                    path.segments = used
                        .segments
                        .iter()
                        .chain(path.segments.iter().skip(1))
                        .cloned()
                        .collect();
                    path.leading_colon = used.leading_colon;
                    return Target::Resolved(self.c.resolve(self.module, &path, namespace));
                }
                _ => {}
            }
        }

        if self.generics.contains(&first) {
            Target::Generic
        } else {
            Target::Resolved(self.c.resolve(self.module, path, namespace))
        }
    }

    fn segment(&mut self, segment: &syn::PathSegment, expr: bool) -> Segment {
        let args = match &segment.arguments {
            syn::PathArguments::None => Args::None,
            syn::PathArguments::AngleBracketed(a) if a.colon2_token.is_some() == expr => {
                Args::Angle(self.generic_args(&a.args))
            }
            syn::PathArguments::Parenthesized(p) => Args::Fn(self.fn_types(&p.inputs, &p.output)),
            other => {
                self.unsupported(other, "path arguments");
                Args::None
            }
        };

        Segment {
            name: segment.ident.to_string(),
            args,
        }
    }

    fn generic_args(
        &mut self,
        args: &Punctuated<syn::GenericArgument, syn::Token![,]>,
    ) -> Vec<GenericArg> {
        args.iter()
            .filter_map(|arg| {
                Some(match arg {
                    syn::GenericArgument::Lifetime(l) => GenericArg::Lifetime(l.to_string()),
                    syn::GenericArgument::Type(t) => GenericArg::Type(self.ty(t)),
                    syn::GenericArgument::Const(e) => GenericArg::Const(self.expr(e)),
                    syn::GenericArgument::AssocType(a) if a.generics.is_none() => {
                        GenericArg::AssocType {
                            name: a.ident.to_string(),
                            ty: self.ty(&a.ty),
                        }
                    }
                    other => {
                        self.unsupported(other, "generic argument");
                        return None;
                    }
                })
            })
            .collect()
    }

    /// The items and imports of the block are in scope in all of it, the
    /// bindings of its statements in those after them.
    fn block(&mut self, block: &syn::Block) -> Block {
        let scope = self.scope.len();

        for stmt in &block.stmts {
            if let syn::Stmt::Item(item) = stmt {
                self.scope.extend(declared(item));
            }
        }

        let stmts = block.stmts.iter().filter_map(|s| self.stmt(s)).collect();
        self.scope.truncate(scope);
        Block { stmts }
    }

    fn stmt(&mut self, stmt: &syn::Stmt) -> Option<Stmt> {
        Some(match stmt {
            syn::Stmt::Local(l) => {
                let init = l.init.as_ref().map(|i| self.expr(&i.expr));
                let diverge = l
                    .init
                    .as_ref()
                    .and_then(|i| i.diverge.as_ref())
                    .map(|(_, e)| self.expr(e));
                Stmt::Let {
                    attrs: self.attrs(&l.attrs),
                    pat: self.pat(&l.pat),
                    init,
                    diverge,
                }
            }
            syn::Stmt::Item(item @ (syn::Item::Macro(_) | syn::Item::Mod(_))) => {
                self.unsupported(item, "item");
                return None;
            }
            // WHY: the names a glob imports into a block are not in its scope.
            syn::Stmt::Item(syn::Item::Use(u)) if has_glob(&u.tree) => {
                self.unsupported(u, "use");
                return None;
            }
            syn::Stmt::Item(item) => Stmt::Item(self.nested(item)?),
            syn::Stmt::Expr(e, semi) => {
                let (attrs, e) = outer_attrs(e);
                Stmt::Expr {
                    attrs: self.attrs(&attrs),
                    expr: self.expr(&e),
                    semi: semi.is_some(),
                }
            }
            syn::Stmt::Macro(m) => Stmt::Expr {
                attrs: self.attrs(&m.attrs),
                expr: self.mac(&m.mac),
                semi: m.semi_token.is_some(),
            },
        })
    }

    fn expr(&mut self, expr: &syn::Expr) -> Expr {
        // WHY: the attributes of an expression are its first tokens.
        if self.text(expr).starts_with('#') {
            let (attrs, expr) = outer_attrs(expr);
            return Expr::Attributed {
                attrs: self.attrs(&attrs),
                expr: self.boxed(&expr),
            };
        }

        match expr {
            syn::Expr::Array(a) => Expr::Array(self.exprs(&a.elems)),
            syn::Expr::Assign(a) => Expr::Assign {
                left: self.boxed(&a.left),
                right: self.boxed(&a.right),
            },
            syn::Expr::Async(a) => Expr::Async {
                capture: a.capture.is_some(),
                block: self.block(&a.block),
            },
            syn::Expr::Await(a) => Expr::Await(self.boxed(&a.base)),
            syn::Expr::Binary(b) => Expr::Binary {
                left: self.boxed(&b.left),
                op: self.op(&BinOp::ALL, BinOp::token, &b.op),
                right: self.boxed(&b.right),
            },
            syn::Expr::Block(b) if b.label.is_none() => Expr::Block(self.block(&b.block)),
            syn::Expr::Break(b) if b.label.is_none() && b.expr.is_none() => Expr::Break,
            syn::Expr::Call(c) => Expr::Call {
                func: self.boxed(&c.func),
                args: self.exprs(&c.args),
            },
            syn::Expr::Cast(c) => Expr::Cast {
                expr: self.boxed(&c.expr),
                ty: Box::new(self.ty(&c.ty)),
            },
            syn::Expr::Closure(c)
                if c.lifetimes.is_none() && c.constness.is_none() && c.asyncness.is_none() =>
            {
                Expr::Closure(Box::new(self.closure(c)))
            }
            syn::Expr::Const(c) => Expr::Const(self.block(&c.block)),
            syn::Expr::Continue(c) if c.label.is_none() => Expr::Continue,
            syn::Expr::Field(f) => Expr::Field {
                expr: self.boxed(&f.base),
                member: member(&f.member),
            },
            syn::Expr::ForLoop(f) if f.label.is_none() => {
                let expr = self.boxed(&f.expr);
                let scope = self.scope.len();
                let pat = Box::new(self.pat(&f.pat));
                let body = self.block(&f.body);
                self.scope.truncate(scope);
                Expr::ForLoop { pat, expr, body }
            }
            syn::Expr::If(i) => {
                let scope = self.scope.len();
                let cond = self.boxed(&i.cond);
                let then_branch = self.block(&i.then_branch);
                self.scope.truncate(scope);
                Expr::If {
                    cond,
                    then_branch,
                    else_branch: i.else_branch.as_ref().map(|(_, e)| self.boxed(e)),
                }
            }
            syn::Expr::Index(i) => Expr::Index {
                expr: self.boxed(&i.expr),
                index: self.boxed(&i.index),
            },
            syn::Expr::Let(l) => {
                let expr = self.boxed(&l.expr);
                Expr::Let {
                    pat: Box::new(self.pat(&l.pat)),
                    expr,
                }
            }
            syn::Expr::Lit(l) => Expr::Lit(self.lit(&l.lit)),
            syn::Expr::Loop(l) if l.label.is_none() => Expr::Loop(self.block(&l.body)),
            syn::Expr::Macro(m) => self.mac(&m.mac),
            syn::Expr::Match(m) => Expr::Match {
                expr: self.boxed(&m.expr),
                arms: m.arms.iter().map(|a| self.arm(a)).collect(),
            },
            syn::Expr::MethodCall(m) => Expr::MethodCall {
                receiver: self.boxed(&m.receiver),
                method: m.method.to_string(),
                turbofish: m
                    .turbofish
                    .as_ref()
                    .map(|t| self.generic_args(&t.args))
                    .unwrap_or_default(),
                args: self.exprs(&m.args),
            },
            syn::Expr::Paren(p) => Expr::Paren(self.boxed(&p.expr)),
            syn::Expr::Path(p) => match &p.qself {
                None => Expr::Path(self.expr_path(&p.path, Namespace::Value)),
                Some(qself) => Expr::Qualified(Box::new(self.qualified(qself, &p.path, true))),
            },
            syn::Expr::Range(r) => Expr::Range {
                start: r.start.as_ref().map(|e| self.boxed(e)),
                closed: matches!(r.limits, syn::RangeLimits::Closed(_)),
                end: r.end.as_ref().map(|e| self.boxed(e)),
            },
            syn::Expr::Reference(r) => Expr::Reference {
                mutable: r.mutability.is_some(),
                expr: self.boxed(&r.expr),
            },
            syn::Expr::Repeat(r) => Expr::Repeat {
                expr: self.boxed(&r.expr),
                len: self.boxed(&r.len),
            },
            syn::Expr::Return(r) => Expr::Return(r.expr.as_ref().map(|e| self.boxed(e))),
            syn::Expr::Struct(s) if s.qself.is_none() && s.dot2_token.is_none() => Expr::Struct {
                path: self.expr_path(&s.path, Namespace::Type),
                fields: s.fields.iter().map(|f| self.field_value(f)).collect(),
            },
            syn::Expr::Try(t) => Expr::Try(self.boxed(&t.expr)),
            syn::Expr::Tuple(t) => Expr::Tuple(self.exprs(&t.elems)),
            syn::Expr::Unary(u) => Expr::Unary {
                op: self.op(&UnOp::ALL, UnOp::token, &u.op),
                expr: self.boxed(&u.expr),
            },
            syn::Expr::Unsafe(u) => Expr::Block(self.block(&u.block)),
            syn::Expr::While(w) if w.label.is_none() => {
                let scope = self.scope.len();
                let cond = self.boxed(&w.cond);
                let body = self.block(&w.body);
                self.scope.truncate(scope);
                Expr::While { cond, body }
            }
            other => {
                self.unsupported(other, "expression");
                Expr::Tuple(Vec::new())
            }
        }
    }

    fn boxed(&mut self, expr: &syn::Expr) -> Box<Expr> {
        Box::new(self.expr(expr))
    }

    fn exprs<'e>(&mut self, exprs: impl IntoIterator<Item = &'e syn::Expr>) -> Vec<Expr> {
        exprs.into_iter().map(|e| self.expr(e)).collect()
    }

    /// The operator of the token among all of them, `ops`.
    fn op<T: Copy>(&mut self, ops: &[T], token: fn(T) -> &'static str, node: &impl Spanned) -> T {
        let text = self.text(node);
        let op = ops.iter().copied().find(|&op| token(op) == text);
        // WHY: an operator syn has and the IR has not.
        let error = self.error(node, "operator");
        self.errors.extend(op.is_none().then_some(error));
        op.unwrap_or(ops[0])
    }

    fn closure(&mut self, closure: &syn::ExprClosure) -> Closure {
        let scope = self.scope.len();
        let inputs = closure.inputs.iter().map(|p| self.pat(p)).collect();
        let output = self.output(&closure.output);
        let body = self.expr(&closure.body);
        self.scope.truncate(scope);
        Closure {
            capture: closure.capture.is_some(),
            inputs,
            output,
            body,
        }
    }

    fn arm(&mut self, arm: &syn::Arm) -> Arm {
        let scope = self.scope.len();
        let (pat, guard) = match &arm.pat {
            syn::Pat::Guard(g) => (self.pat(&g.pat), Some(self.expr(&g.guard))),
            pat => (self.pat(pat), None),
        };
        let body = self.expr(&arm.body);
        self.scope.truncate(scope);
        Arm {
            attrs: self.attrs(&arm.attrs),
            pat,
            guard,
            body,
        }
    }

    fn field_value(&mut self, field: &syn::FieldValue) -> FieldValue {
        if !field.attrs.is_empty() {
            self.unsupported(field, "field");
        }

        FieldValue {
            member: member(&field.member),
            expr: self.expr(&field.expr),
        }
    }

    fn lit(&mut self, lit: &syn::Lit) -> Lit {
        match lit {
            syn::Lit::Str(s) => Lit::Str(s.value()),
            syn::Lit::Char(c) => Lit::Char(c.value()),
            syn::Lit::Int(i) => Lit::Int {
                digits: i.base10_digits().to_string(),
                suffix: i.suffix().to_string(),
            },
            syn::Lit::Float(f) => Lit::Float {
                digits: f.base10_digits().to_string(),
                suffix: f.suffix().to_string(),
            },
            syn::Lit::Bool(b) => Lit::Bool(b.value),
            other => {
                self.unsupported(other, "literal");
                Lit::Bool(false)
            }
        }
    }

    fn mac(&mut self, mac: &syn::Macro) -> Expr {
        let target = Target::Resolved(self.c.resolve(self.module, &mac.path, Namespace::Macro));
        let path = self.path_with(&mac.path, target, false);
        let call = match self.c.macro_call(self.module, mac) {
            Ok(crate::MacroCall::Format { exprs, format }) => MacroCall::Format {
                exprs: self.exprs(&exprs),
                format: format.map(|f| self.format(&f)),
            },
            Ok(crate::MacroCall::Vec(exprs)) => MacroCall::Vec(self.exprs(&exprs)),
            Ok(crate::MacroCall::VecRepeat(value, count)) => {
                MacroCall::VecRepeat(self.boxed(&value), self.boxed(&count))
            }
            Ok(crate::MacroCall::Matches { expr, pat, guard }) => {
                let expr = self.boxed(&expr);
                let scope = self.scope.len();
                let pat = Box::new(self.pat(&pat));
                let guard = guard.map(|g| self.boxed(&g));
                self.scope.truncate(scope);
                MacroCall::Matches { expr, pat, guard }
            }
            Ok(crate::MacroCall::ThreadLocal(statics)) => MacroCall::ThreadLocal(
                statics
                    .into_iter()
                    .filter_map(|s| {
                        self.scope.push(Scoped::Item(s.ident.to_string()));
                        self.nested(&syn::Item::Static(s))
                    })
                    .collect(),
            ),
            Ok(crate::MacroCall::NoArgs) => MacroCall::NoArgs,
            Ok(crate::MacroCall::Expanded(_)) | Err(_) => {
                self.unsupported(mac, "macro call");
                MacroCall::Vec(Vec::new())
            }
        };

        Expr::Macro(Box::new(Macro { path, call }))
    }

    fn format(&mut self, format: &crate::FormatArgs) -> FormatArgs {
        FormatArgs {
            string: format.string.value(),
            args: format
                .args
                .iter()
                .map(|arg| FormatArg {
                    name: arg.name.as_ref().map(ToString::to_string),
                    expr: self.expr(&arg.expr),
                })
                .collect(),
            captures: format
                .captures
                .iter()
                .map(|name| self.expr_path(&syn::Path::from(name.clone()), Namespace::Value))
                .collect(),
        }
    }

    /// Brings the bindings into scope.
    fn pat(&mut self, pat: &syn::Pat) -> Pat {
        // WHY: the attributes of a pattern are its first tokens.
        if self.text(pat).starts_with('#') {
            self.unsupported(pat, "attribute");
        }

        match pat {
            syn::Pat::Ident(i) => self.ident(i),
            syn::Pat::Lit(l) => Pat::Lit(self.lit(&l.lit)),
            syn::Pat::Or(o) => Pat::Or(self.pats(&o.cases)),
            syn::Pat::Paren(p) => Pat::Paren(Box::new(self.pat(&p.pat))),
            syn::Pat::Path(p) if p.qself.is_none() => {
                Pat::Path(self.expr_path(&p.path, Namespace::Value))
            }
            syn::Pat::Range(r) => Pat::Range {
                start: r.start.as_ref().map(|e| self.boxed(e)),
                closed: matches!(r.limits, syn::RangeLimits::Closed(_)),
                end: r.end.as_ref().map(|e| self.boxed(e)),
            },
            syn::Pat::Reference(r) => Pat::Ref {
                mutable: r.mutability.is_some(),
                pat: Box::new(self.pat(&r.pat)),
            },
            syn::Pat::Struct(s) if s.qself.is_none() => Pat::Struct {
                path: self.expr_path(&s.path, Namespace::Type),
                fields: s.fields.iter().map(|f| self.field_pat(f)).collect(),
                rest: s.rest.is_some(),
            },
            syn::Pat::Tuple(t) => Pat::Tuple(self.pats(&t.elems)),
            syn::Pat::TupleStruct(t) if t.qself.is_none() => Pat::TupleStruct {
                path: self.expr_path(&t.path, Namespace::Value),
                elems: self.pats(&t.elems),
            },
            syn::Pat::Type(t) => Pat::Type {
                pat: Box::new(self.pat(&t.pat)),
                ty: self.ty(&t.ty),
            },
            syn::Pat::Wild(_) => Pat::Wild,
            other => {
                self.unsupported(other, "pattern");
                Pat::Wild
            }
        }
    }

    fn pats<P>(&mut self, pats: &Punctuated<syn::Pat, P>) -> Vec<Pat> {
        pats.iter().map(|p| self.pat(p)).collect()
    }

    /// A path to a constant, a unit struct or a unit variant, a binding
    /// otherwise. A name of another crate is taken for a constant when it
    /// starts upper case.
    fn ident(&mut self, ident: &syn::PatIdent) -> Pat {
        let name = ident.ident.to_string();

        if ident.mutability.is_none() && ident.by_ref.is_none() && ident.subpat.is_none() {
            let path = self.expr_path(&syn::Path::from(ident.ident.clone()), Namespace::Value);
            let constant = match &path.target {
                Target::Resolved(Resolved::Item(id)) => {
                    matches!(self.c.item(*id), syn::Item::Const(_) | syn::Item::Struct(_))
                }
                Target::Resolved(Resolved::External(external)) => external
                    .last()
                    .is_some_and(|name| name.starts_with(char::is_uppercase)),
                target => matches!(
                    target,
                    Target::Generic | Target::LocalItem | Target::Resolved(Resolved::Variant(..))
                ),
            };

            if constant {
                return Pat::Path(path);
            }
        }

        let subpat = ident.subpat.as_ref().map(|(_, p)| Box::new(self.pat(p)));
        self.scope.push(Scoped::Binding(name.clone()));
        Pat::Binding {
            by_ref: ident.by_ref.is_some(),
            mutable: ident.mutability.is_some(),
            name,
            subpat,
        }
    }

    fn field_pat(&mut self, field: &syn::FieldPat) -> FieldPat {
        if !field.attrs.is_empty() {
            self.unsupported(field, "field");
        }

        FieldPat {
            member: member(&field.member),
            pat: self.pat(&field.pat),
        }
    }

    fn text(&self, node: &impl Spanned) -> String {
        self.file.text()[node.span().byte_range()].to_string()
    }

    fn error(&self, node: &impl Spanned, construct: &str) -> Error {
        let text = self.text(node);
        Error::Unsupported {
            path: self.file.path().to_path_buf(),
            line: node.span().start().line,
            construct: format!("{construct} `{}`", text.lines().next().unwrap_or_default()),
        }
    }

    fn unsupported(&mut self, node: &impl Spanned, construct: &str) {
        let error = self.error(node, construct);
        self.errors.push(error);
    }
}

fn member(member: &syn::Member) -> Member {
    match member {
        syn::Member::Named(name) => Member::Named(name.to_string()),
        syn::Member::Unnamed(index) => Member::Index(index.index),
    }
}

/// The path of a `use` without groups and globs, with its rename.
fn joined(prefix: &syn::Path, ident: &syn::Ident) -> syn::Path {
    let mut path = prefix.clone();
    path.segments.push(ident.clone().into());
    path
}

/// The path a name of a `use` imports, `self` imports the prefix.
fn imported(prefix: &syn::Path, ident: &syn::Ident) -> syn::Path {
    if ident == "self" {
        prefix.clone()
    } else {
        joined(prefix, ident)
    }
}

/// The names a `use` tree imports with the paths they import.
fn imports(prefix: &syn::Path, tree: &syn::UseTree, names: &mut Vec<Scoped>) {
    let (path, rename) = match tree {
        syn::UseTree::Path(p) => return imports(&joined(prefix, &p.ident), &p.tree, names),
        syn::UseTree::Name(n) => (imported(prefix, &n.ident), None),
        syn::UseTree::Rename(r) => (imported(prefix, &r.ident), Some(r.rename.to_string())),
        syn::UseTree::Glob(_) => return,
        syn::UseTree::Group(g) => {
            return g.items.iter().for_each(|t| imports(prefix, t, names));
        }
    };
    let name = rename.unwrap_or_else(|| path.segments[path.segments.len() - 1].ident.to_string());
    names.push(Scoped::Use(name, path));
}

fn has_glob(tree: &syn::UseTree) -> bool {
    match tree {
        syn::UseTree::Path(p) => has_glob(&p.tree),
        syn::UseTree::Name(_) | syn::UseTree::Rename(_) => false,
        syn::UseTree::Glob(_) => true,
        syn::UseTree::Group(g) => g.items.iter().any(has_glob),
    }
}

/// The expression without its attributes, which are taken.
fn outer_attrs(expr: &syn::Expr) -> (Vec<syn::Attribute>, syn::Expr) {
    struct Outer(Option<Vec<syn::Attribute>>);

    impl VisitMut for Outer {
        // WHY: an expression's own attributes are visited first.
        fn visit_attributes_mut(&mut self, attrs: &mut Vec<syn::Attribute>) {
            if self.0.is_none() {
                self.0 = Some(std::mem::take(attrs));
            }
        }
    }

    let mut expr = expr.clone();
    let mut outer = Outer(None);
    outer.visit_expr_mut(&mut expr);
    (outer.0.unwrap_or_default(), expr)
}

/// The names an item of a block declares.
fn declared(item: &syn::Item) -> Vec<Scoped> {
    let ident = match item {
        syn::Item::Const(i) => &i.ident,
        syn::Item::Enum(i) => &i.ident,
        syn::Item::Fn(i) => &i.sig.ident,
        syn::Item::Static(i) => &i.ident,
        syn::Item::Struct(i) => &i.ident,
        syn::Item::Trait(i) => &i.ident,
        syn::Item::Type(i) => &i.ident,
        syn::Item::Use(u) => {
            let mut names = Vec::new();
            let prefix = syn::Path {
                leading_colon: u.leading_colon,
                segments: Punctuated::new(),
            };
            imports(&prefix, &u.tree, &mut names);
            return names;
        }
        _ => return Vec::new(),
    };

    vec![Scoped::Item(ident.to_string())]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cfg;
    use crate::Resolved;
    use crate::Target as CrateTarget;
    use crate::TargetKind;
    use agplatform::Fs;
    use agplatform::Platform;
    use agplatform::test_platform::test_platform;

    /// The crate and the IR of its root items.
    fn ir(source: &str) -> (SourceCrate, Vec<(ItemId, Item)>, Vec<String>) {
        let platform = test_platform();
        platform.fs().create_dir_all("/c/src").unwrap();
        platform.fs().write("/c/src/lib.rs", source).unwrap();
        let c = SourceCrate::load(
            &platform,
            &CrateTarget::new(TargetKind::Lib, "/c/src/lib.rs"),
            &Cfg::All,
        );
        assert!(c.errors().is_empty(), "{:?}", c.errors());
        let ids = (0..c.items(c.root()).len()).map(|index| ItemId {
            module: c.root(),
            index,
        });
        let (items, errors) = c.ir(ids);
        let errors = errors.iter().map(ToString::to_string).collect();
        (c, items, errors)
    }

    fn ty(name: &str, args: Args, target: Target) -> Type {
        Type::Path(Path {
            leading_colon: false,
            segments: vec![Segment {
                name: name.to_string(),
                args,
            }],
            target,
        })
    }

    fn field(name: &str, ty: Type) -> Field {
        Field {
            attrs: Vec::new(),
            vis: Vis::Private,
            name: Some(name.to_string()),
            ty,
        }
    }

    fn generics(name: &str) -> Generics {
        Generics {
            params: vec![GenericParam::Type {
                name: name.to_string(),
                bounds: Vec::new(),
            }],
            predicates: Vec::new(),
        }
    }

    fn method(name: &str, generics: Generics, params: Vec<(&str, Type)>) -> ImplItem {
        ImplItem {
            attrs: Vec::new(),
            vis: Vis::Private,
            kind: ImplItemKind::Fn(Fn {
                sig: Signature {
                    constness: false,
                    asyncness: false,
                    name: name.to_string(),
                    generics,
                    receiver: None,
                    params: params
                        .into_iter()
                        .map(|(name, ty)| Param {
                            mutable: false,
                            name: name.to_string(),
                            ty,
                        })
                        .collect(),
                    output: None,
                },
                body: Block { stmts: Vec::new() },
            }),
        }
    }

    /// The crate and the paths of the bodies of its root functions, with
    /// their targets, in order.
    fn body_paths(source: &str) -> (SourceCrate, Vec<(String, Target)>) {
        let (c, items, errors) = ir(source);
        assert!(errors.is_empty(), "{errors:?}");
        let paths = items
            .iter()
            .filter_map(|(_, item)| match &item.kind {
                ItemKind::Fn(f) => Some(block_paths(&f.body)),
                _ => None,
            })
            .flatten()
            .collect();
        (c, paths)
    }

    fn block_paths(block: &Block) -> Vec<(String, Target)> {
        block
            .stmts
            .iter()
            .flat_map(|stmt| match stmt {
                Stmt::Let { pat, init, .. } => {
                    [init.iter().flat_map(expr_paths).collect(), pat_paths(pat)].concat()
                }
                Stmt::Expr { expr, .. } => expr_paths(expr),
                Stmt::Item(Item {
                    kind: ItemKind::Fn(f),
                    ..
                }) => block_paths(&f.body),
                Stmt::Item(_) => Vec::new(),
            })
            .collect()
    }

    fn expr_paths(expr: &Expr) -> Vec<(String, Target)> {
        match expr {
            Expr::Path(p) => vec![named(p)],
            Expr::Block(b) => block_paths(b),
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => [
                expr_paths(cond),
                block_paths(then_branch),
                else_branch.iter().flat_map(|e| expr_paths(e)).collect(),
            ]
            .concat(),
            Expr::Let { pat, expr } => [pat_paths(pat), expr_paths(expr)].concat(),
            Expr::Match { expr, arms } => [
                expr_paths(expr),
                arms.iter()
                    .flat_map(|a| {
                        [
                            pat_paths(&a.pat),
                            a.guard.iter().flat_map(expr_paths).collect(),
                            expr_paths(&a.body),
                        ]
                        .concat()
                    })
                    .collect(),
            ]
            .concat(),
            Expr::ForLoop { pat, expr, body } => {
                [pat_paths(pat), expr_paths(expr), block_paths(body)].concat()
            }
            Expr::While { cond, body } => [expr_paths(cond), block_paths(body)].concat(),
            Expr::Closure(c) => expr_paths(&c.body),
            Expr::Macro(m) => match &m.call {
                MacroCall::Format {
                    format: Some(f), ..
                } => f.captures.iter().map(named).collect(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    fn pat_paths(pat: &Pat) -> Vec<(String, Target)> {
        match pat {
            Pat::Path(p) => vec![named(p)],
            _ => Vec::new(),
        }
    }

    fn named(path: &Path) -> (String, Target) {
        let last = &path.segments[path.segments.len() - 1];
        (last.name.clone(), path.target.clone())
    }

    fn named_all(paths: &[(&str, Target)]) -> Vec<(String, Target)> {
        paths
            .iter()
            .map(|(name, target)| (name.to_string(), target.clone()))
            .collect()
    }

    mod source_crate {
        use super::*;

        mod ir {
            use super::*;

            #[test]
            fn ir_should_scope_the_bindings_to_their_blocks_arms_and_closures() {
                let (_, paths) = body_paths(
                    "fn f(p: u8) {
    let a = p;
    {
        let b = a;
        b;
    }
    b;
    if let c = a {
        c;
    } else {
        c;
    }
    match a {
        d if d => d,
        _ => d,
    }
    for e in a {
        e;
    }
    e;
    while let g = a {
        g;
    }
    g;
    |h| h;
    h;
    let mut z = 1;
}",
                );
                let local = Target::Local;
                let unresolved = Target::Resolved(Resolved::Unresolved);

                assert_eq!(
                    paths,
                    named_all(&[
                        ("p", local.clone()),
                        ("a", local.clone()),
                        ("b", local.clone()),
                        ("b", unresolved.clone()),
                        ("a", local.clone()),
                        ("c", local.clone()),
                        ("c", unresolved.clone()),
                        ("a", local.clone()),
                        ("d", local.clone()),
                        ("d", local.clone()),
                        ("d", unresolved.clone()),
                        ("a", local.clone()),
                        ("e", local.clone()),
                        ("e", unresolved.clone()),
                        ("a", local.clone()),
                        ("g", local.clone()),
                        ("g", unresolved.clone()),
                        ("h", local),
                        ("h", unresolved),
                    ])
                );
            }

            #[test]
            fn ir_should_resolve_the_items_and_imports_of_blocks_and_constant_patterns() {
                let (c, paths) = body_paths(
                    "pub enum E { A, B(u8) }
const K: u8 = 0;
struct Unit;
fn g() {}
use E::A;
fn f(p: u8) {
    use E::{B, self as F};
    struct Local;
    const C: u8 = 1;
    enum En {}
    static S: u8 = 0;
    trait Tr {}
    type Ty = u8;
    impl Local {}
    fn nested() {
        p;
    }
    B;
    F::A;
    Local;
    C;
    match p {
        A => {}
        E::A => {}
        K => {}
        None => {}
        Unit => {}
        C => {}
        g => g,
        drop => drop,
        x => x,
    }
    format!(\"{p} {K}\");
    vec![1];
}",
                );
                let id = |index| ItemId {
                    module: c.root(),
                    index,
                };
                let local = Target::Local;
                let item = Target::LocalItem;
                let a = Target::Resolved(Resolved::Variant(id(0), 0));
                let k = Target::Resolved(Resolved::Item(id(1)));
                let none = Target::Resolved(Resolved::External(
                    ["std", "option", "Option", "None"]
                        .map(String::from)
                        .to_vec(),
                ));

                assert_eq!(
                    paths,
                    named_all(&[
                        ("p", Target::Resolved(Resolved::Unresolved)),
                        ("B", Target::Resolved(Resolved::Variant(id(0), 1))),
                        ("A", a.clone()),
                        ("Local", item.clone()),
                        ("C", item.clone()),
                        ("p", local.clone()),
                        ("A", a.clone()),
                        ("A", a),
                        ("K", k.clone()),
                        ("None", none),
                        ("Unit", Target::Resolved(Resolved::Item(id(2)))),
                        ("C", item),
                        ("g", local.clone()),
                        ("drop", local.clone()),
                        ("x", local.clone()),
                        ("p", local),
                        ("K", k),
                    ])
                );
            }

            #[test]
            fn ir_should_resolve_every_name_and_glob_of_a_use() {
                let (c, items, errors) =
                    ir("pub enum E { A }\nuse crate::E::{self as F, A, *, B};");
                let id = ItemId {
                    module: c.root(),
                    index: 0,
                };
                let e = Target::Resolved(Resolved::Item(id));
                let name = |name: &str, rename: Option<&str>, target| UseTree::Name {
                    name: name.to_string(),
                    rename: rename.map(str::to_string),
                    target,
                };

                assert!(errors.is_empty(), "{errors:?}");
                assert_eq!(
                    items[1].1.kind,
                    ItemKind::Use(Use {
                        leading_colon: false,
                        tree: UseTree::Path {
                            name: "crate".to_string(),
                            tree: Box::new(UseTree::Path {
                                name: "E".to_string(),
                                tree: Box::new(UseTree::Group(vec![
                                    name("self", Some("F"), e.clone()),
                                    name("A", None, Target::Resolved(Resolved::Variant(id, 0))),
                                    UseTree::Glob(e),
                                    name(
                                        "B",
                                        None,
                                        Target::Resolved(Resolved::Associated(
                                            Box::new(Resolved::Item(id)),
                                            "B".to_string()
                                        ))
                                    ),
                                ])),
                            }),
                        },
                    })
                );
            }

            #[test]
            fn ir_should_resolve_the_paths_of_the_declarations() {
                let (c, items, errors) = ir("pub struct Local;
pub struct S<T> {
    a: T,
    b: Option<Self>,
    c: Local,
    d: u8,
}");
                let local = ItemId {
                    module: c.root(),
                    index: 0,
                };
                let option = Resolved::External(vec![
                    "std".to_string(),
                    "option".to_string(),
                    "Option".to_string(),
                ]);
                let self_type = GenericArg::Type(ty("Self", Args::None, Target::SelfType));

                assert!(errors.is_empty(), "{errors:?}");
                assert_eq!(
                    items[1].1.kind,
                    ItemKind::Struct(Struct {
                        name: "S".to_string(),
                        generics: generics("T"),
                        fields: Fields::Named(vec![
                            field("a", ty("T", Args::None, Target::Generic)),
                            field(
                                "b",
                                ty(
                                    "Option",
                                    Args::Angle(vec![self_type]),
                                    Target::Resolved(option)
                                )
                            ),
                            field(
                                "c",
                                ty("Local", Args::None, Target::Resolved(Resolved::Item(local)))
                            ),
                            field(
                                "d",
                                ty(
                                    "u8",
                                    Args::None,
                                    Target::Resolved(Resolved::Primitive("u8".to_string()))
                                )
                            ),
                        ]),
                    })
                );
            }

            #[test]
            fn ir_should_scope_the_generic_parameters_of_members_to_the_member() {
                let (_, items, _) = ir("impl<A> S<A> {
    fn f<B>(b: B) {}
    fn g(a: A, b: B) {}
}");
                let generic = |name| ty(name, Args::None, Target::Generic);
                let unresolved =
                    |name| ty(name, Args::None, Target::Resolved(Resolved::Unresolved));

                assert_eq!(
                    items[0].1.kind,
                    ItemKind::Impl(Impl {
                        generics: generics("A"),
                        trait_: None,
                        self_ty: ty(
                            "S",
                            Args::Angle(vec![GenericArg::Type(generic("A"))]),
                            Target::Resolved(Resolved::Unresolved)
                        ),
                        items: vec![
                            method("f", generics("B"), vec![("b", generic("B"))]),
                            method(
                                "g",
                                Generics::default(),
                                vec![("a", generic("A")), ("b", unresolved("B"))]
                            ),
                        ],
                    })
                );
            }

            #[test]
            fn ir_should_leave_out_macro_rules_and_modules() {
                let (_, items, errors) = ir("macro_rules! m { () => {} }\nmod a {}");

                assert!(items.is_empty());
                assert!(errors.is_empty(), "{errors:?}");
            }

            #[test]
            fn ir_should_report_unsupported_constructs_and_leave_their_items_out() {
                let cases = [
                    ("trait T { #![allow(x)] }", "attribute `#![allow(x)]`"),
                    ("impl !Send for S {}", "impl `impl !Send for S {}`"),
                    ("const C<T>: u8 = 1;", "item `const C<T>: u8 = 1;`"),
                    ("static mut S: u8 = 0;", "static `static mut S: u8 = 0;`"),
                    ("union U { a: u8 }", "item `union U { a: u8 }`"),
                    ("pub(in crate) struct S;", "visibility `pub(in crate)`"),
                    (
                        "struct S<'a: 'b, 'b>(&'a u8);",
                        "lifetime parameter `'a: 'b`",
                    ),
                    ("struct S<T = u8>(T);", "type parameter `T = u8`"),
                    (
                        "struct S<const N: u8 = 1>;",
                        "const parameter `const N: u8 = 1`",
                    ),
                    (
                        "struct S<'a, 'b>(&'a u8) where 'a: 'b;",
                        "where predicate `'a: 'b`",
                    ),
                    ("fn f<T: ?Sized>() {}", "bound `?Sized`"),
                    (
                        "trait T { default fn f(); }",
                        "trait item `default fn f();`",
                    ),
                    ("trait T { type A<B>; }", "trait item `type A<B>;`"),
                    ("trait T { const C: u8; }", "trait item `const C: u8;`"),
                    ("impl S { default fn f() {} }", "fn `default fn f() {}`"),
                    (
                        "impl S { const C: u8 = 1; }",
                        "impl item `const C: u8 = 1;`",
                    ),
                    (
                        "impl S { default type A = u8; }",
                        "impl item `default type A = u8;`",
                    ),
                    ("unsafe fn f() {}", "signature `unsafe fn f()`"),
                    ("impl S { fn f(#[x] self) {} }", "receiver `#[x] self`"),
                    ("fn f((a, b): (u8, u8)) {}", "parameter `(a, b): (u8, u8)`"),
                    ("type F = fn(x: u8);", "parameter `x: u8`"),
                    ("type P = *const u8;", "type `*const u8`"),
                    ("type V = Vec::<u8>;", "path arguments `::<u8>`"),
                    (
                        "type I = dyn Iterator<Item: Copy>;",
                        "generic argument `Item: Copy`",
                    ),
                    (
                        "fn f() { use std::{io as i, fmt::*}; }",
                        "use `use std::{io as i, fmt::*};`",
                    ),
                    ("fn f() { 'a: loop {} }", "expression `'a: loop {}`"),
                    ("fn f() { async || 1; }", "expression `async || 1`"),
                    ("fn f() { S { ..s }; }", "expression `S { ..s }`"),
                    ("fn f() { b\"x\"; }", "literal `b\"x\"`"),
                    ("fn f() { let [a] = x; }", "pattern `[a]`"),
                    ("fn f() { |#[x] a| a; }", "attribute `#[x] a`"),
                    ("fn f() { m!(); }", "macro call `m!()`"),
                    (
                        "fn f() { macro_rules! m { () => {} } }",
                        "item `macro_rules! m { () => {} }`",
                    ),
                    ("fn f() { mod m {} }", "item `mod m {}`"),
                    ("fn f() { union U { a: u8 } }", "item `union U { a: u8 }`"),
                    ("fn f() { S { #[x] a: 1 }; }", "field `#[x] a: 1`"),
                    ("fn f() { let S { #[x] a } = s; }", "field `#[x] a`"),
                ];

                for (source, construct) in cases {
                    let (_, items, errors) = ir(source);
                    let errors = errors
                        .iter()
                        .map(|e| e.replace('\\', "/"))
                        .collect::<Vec<_>>();

                    assert!(items.is_empty(), "{source}: {items:?}");
                    assert_eq!(
                        errors,
                        [format!("/c/src/lib.rs:1: unsupported {construct}")],
                        "{source}"
                    );
                }
            }
        }
    }
}
