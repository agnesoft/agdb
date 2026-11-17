use super::ir::Args;
use super::ir::Attr;
use super::ir::Block;
use super::ir::Bound;
use super::ir::Const;
use super::ir::Enum;
use super::ir::Expr;
use super::ir::Field;
use super::ir::Fields;
use super::ir::Fn;
use super::ir::FnTypes;
use super::ir::GenericArg;
use super::ir::GenericParam;
use super::ir::Generics;
use super::ir::Impl;
use super::ir::ImplItem;
use super::ir::ImplItemKind;
use super::ir::Item;
use super::ir::ItemKind;
use super::ir::Param;
use super::ir::Path;
use super::ir::Predicate;
use super::ir::Qualified;
use super::ir::Receiver;
use super::ir::Segment;
use super::ir::Signature;
use super::ir::Static;
use super::ir::Struct;
use super::ir::Target;
use super::ir::Trait;
use super::ir::TraitItem;
use super::ir::TraitItemKind;
use super::ir::Type;
use super::ir::TypeAlias;
use super::ir::Variant;
use super::ir::Vis;
use crate::Error;
use crate::ItemId;
use crate::ModuleId;
use crate::Namespace;
use crate::SourceCrate;
use crate::SourceFile;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

impl SourceCrate {
    /// The declarations of the items, in order. The IR has the constructs
    /// the items use; an item using another one is left out and the
    /// construct reported. `macro_rules!` are left out silently, they are no
    /// declarations.
    ///
    /// The source text comes from the spans of the syntax trees, which are
    /// valid on the thread that loaded the crate only.
    pub fn ir(&self, ids: impl IntoIterator<Item = ItemId>) -> (Vec<Item>, Vec<Error>) {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        for id in ids {
            let mut load = Load {
                c: self,
                module: id.module(),
                file: self.file(self.module(id.module()).file()),
                generics: Vec::new(),
                errors: Vec::new(),
            };

            if let Some(item) = load.item(id)
                && load.errors.is_empty()
            {
                items.push(item);
            }

            errors.append(&mut load.errors);
        }

        (items, errors)
    }
}

struct Load<'a> {
    c: &'a SourceCrate,
    module: ModuleId,
    file: &'a SourceFile,
    /// The names of the generic parameters in scope.
    generics: Vec<String>,
    errors: Vec<Error>,
}

impl Load<'_> {
    fn item(&mut self, id: ItemId) -> Option<Item> {
        let (attrs, vis, kind) = match self.c.item(id) {
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
                if t.modifiers.auto_token.is_some() || t.unsafety.is_some() || inner(&t.attrs) {
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
                if i.modifiers.defaultness.is_some()
                    || i.modifiers.polarity.is_some()
                    || i.unsafety.is_some()
                    || inner(&i.attrs)
                {
                    self.unsupported(i, "impl");
                }
                let generics = self.generics(&i.generics);
                let kind = ItemKind::Impl(Impl {
                    generics,
                    trait_: i.trait_.as_ref().map(|(path, _)| self.path(path)),
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
            syn::Item::Macro(_) => return None,
            other => {
                self.unsupported(other, "item");
                return None;
            }
        };

        Some(Item {
            id,
            attrs: self.attrs(attrs),
            vis,
            kind,
        })
    }

    /// The outer attributes: the inner ones of a function are in its body.
    fn attrs(&mut self, attrs: &[syn::Attribute]) -> Vec<Attr> {
        attrs
            .iter()
            .filter(|attr| matches!(attr.style, syn::AttrStyle::Outer))
            .map(|attr| match &attr.meta {
                syn::Meta::NameValue(syn::MetaNameValue {
                    path,
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(doc),
                            ..
                        }),
                    ..
                }) if path.is_ident("doc") => Attr::Doc(doc.value()),
                // WHY: an applied `cfg_attr` keeps the brackets of the
                // original attribute, the meta has its own spans.
                meta => Attr::Meta(self.text(meta)),
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
                    Some(Bound::Trait(self.path(&t.path)))
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
        let scope = self.generics.len();
        let (attrs, kind) = match item {
            syn::TraitItem::Fn(f) => {
                let kind = TraitItemKind::Fn {
                    sig: self.signature(&f.sig),
                    body: f.default.as_ref().map(|b| Block::Verbatim(self.text(b))),
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
        self.generics.truncate(scope);

        Some(TraitItem {
            attrs: self.attrs(attrs),
            kind,
        })
    }

    fn impl_item(&mut self, item: &syn::ImplItem) -> Option<ImplItem> {
        let scope = self.generics.len();
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
        self.generics.truncate(scope);

        Some(ImplItem {
            attrs: self.attrs(attrs),
            vis: self.vis(vis),
            kind,
        })
    }

    fn function(&mut self, sig: &syn::Signature, block: &syn::Block) -> Fn {
        Fn {
            sig: self.signature(sig),
            body: Block::Verbatim(self.text(block)),
        }
    }

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
        match (&r.kind, r.attrs.is_empty()) {
            (syn::ReceiverKind::Value, true) => Some(Receiver::Value {
                mutable: r.mutability.is_some(),
            }),
            (syn::ReceiverKind::Reference(_, lifetime, mutability), true) => Some(Receiver::Ref {
                lifetime: lifetime.as_ref().map(ToString::to_string),
                mutable: mutability.is_some(),
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
                None => Type::Path(self.path(&p.path)),
                Some(qself) => self.qualified(qself, &p.path),
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
                Type::Array(Box::new(self.ty(&a.elem)), self.expr(&a.len))
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
            other => {
                self.unsupported(other, "type");
                Type::Tuple(Vec::new())
            }
        }
    }

    /// The first `position` segments of the path are the trait.
    fn qualified(&mut self, qself: &syn::QSelf, path: &syn::Path) -> Type {
        let ty = self.ty(&qself.ty);
        let trait_ = (qself.position > 0).then(|| {
            self.path(&syn::Path {
                leading_colon: path.leading_colon,
                segments: path.segments.iter().take(qself.position).cloned().collect(),
            })
        });

        Type::Qualified(Box::new(Qualified {
            ty,
            trait_,
            rest: path
                .segments
                .iter()
                .skip(qself.position)
                .map(|s| self.segment(s))
                .collect(),
        }))
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

    fn path(&mut self, path: &syn::Path) -> Path {
        let first = path.segments[0].ident.to_string();
        let target = if path.leading_colon.is_none() && first == "Self" {
            Target::SelfType
        } else if path.leading_colon.is_none() && self.generics.contains(&first) {
            Target::Generic
        } else {
            Target::Resolved(self.c.resolve(self.module, path, Namespace::Type))
        };

        Path {
            leading_colon: path.leading_colon.is_some(),
            segments: path.segments.iter().map(|s| self.segment(s)).collect(),
            target,
        }
    }

    fn segment(&mut self, segment: &syn::PathSegment) -> Segment {
        let args = match &segment.arguments {
            syn::PathArguments::None => Args::None,
            syn::PathArguments::AngleBracketed(a) if a.colon2_token.is_none() => Args::Angle(
                a.args
                    .iter()
                    .filter_map(|arg| self.generic_arg(arg))
                    .collect(),
            ),
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

    fn generic_arg(&mut self, arg: &syn::GenericArgument) -> Option<GenericArg> {
        Some(match arg {
            syn::GenericArgument::Lifetime(l) => GenericArg::Lifetime(l.to_string()),
            syn::GenericArgument::Type(t) => GenericArg::Type(self.ty(t)),
            syn::GenericArgument::Const(e) => GenericArg::Const(self.expr(e)),
            syn::GenericArgument::AssocType(a) if a.generics.is_none() => GenericArg::AssocType {
                name: a.ident.to_string(),
                ty: self.ty(&a.ty),
            },
            other => {
                self.unsupported(other, "generic argument");
                return None;
            }
        })
    }

    fn expr(&self, expr: &syn::Expr) -> Expr {
        Expr::Verbatim(self.text(expr))
    }

    fn text(&self, node: &impl Spanned) -> String {
        self.file.text()[node.span().byte_range()].to_string()
    }

    fn unsupported(&mut self, node: &impl Spanned, construct: &str) {
        let text = self.text(node);
        self.errors.push(Error::Unsupported {
            path: self.file.path().to_path_buf(),
            line: node.span().start().line,
            construct: format!("{construct} `{}`", text.lines().next().unwrap_or_default()),
        });
    }
}

fn inner(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| matches!(attr.style, syn::AttrStyle::Inner(_)))
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
    fn ir(source: &str) -> (SourceCrate, Vec<Item>, Vec<String>) {
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
                body: Block::Verbatim("{}".to_string()),
            }),
        }
    }

    mod source_crate {
        use super::*;

        mod ir {
            use super::*;

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
                    items[1].kind,
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
                    items[0].kind,
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
            fn ir_should_leave_out_macro_rules() {
                let (_, items, errors) = ir("macro_rules! m { () => {} }");

                assert!(items.is_empty());
                assert!(errors.is_empty(), "{errors:?}");
            }

            #[test]
            fn ir_should_report_unsupported_constructs_and_leave_their_items_out() {
                let cases = [
                    ("unsafe trait T {}", "trait `unsafe trait T {}`"),
                    (
                        "trait T { #![allow(x)] }",
                        "trait `trait T { #![allow(x)] }`",
                    ),
                    (
                        "unsafe impl Send for S {}",
                        "impl `unsafe impl Send for S {}`",
                    ),
                    ("impl !Send for S {}", "impl `impl !Send for S {}`"),
                    ("const C<T>: u8 = 1;", "item `const C<T>: u8 = 1;`"),
                    ("static mut S: u8 = 0;", "static `static mut S: u8 = 0;`"),
                    ("extern crate a;", "item `extern crate a;`"),
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
                    (
                        "impl S { fn f(self: Box<Self>) {} }",
                        "receiver `self: Box<Self>`",
                    ),
                    ("fn f((a, b): (u8, u8)) {}", "parameter `(a, b): (u8, u8)`"),
                    ("type F = fn(x: u8);", "parameter `x: u8`"),
                    ("type P = *const u8;", "type `*const u8`"),
                    ("type V = Vec::<u8>;", "path arguments `::<u8>`"),
                    (
                        "type I = dyn Iterator<Item: Copy>;",
                        "generic argument `Item: Copy`",
                    ),
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
