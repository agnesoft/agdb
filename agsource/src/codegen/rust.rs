//! Rust source from the IR, laid out like rustfmt for short lines: a member
//! or a statement per line, indented by four spaces, `where` clauses inline.
//! The expressions are printed with the parentheses of the IR.

use super::ir::Args;
use super::ir::Arm;
use super::ir::Attr;
use super::ir::Block;
use super::ir::Bound;
use super::ir::Expr;
use super::ir::Field;
use super::ir::FieldValue;
use super::ir::Fields;
use super::ir::Fn;
use super::ir::FnTypes;
use super::ir::GenericArg;
use super::ir::GenericParam;
use super::ir::Generics;
use super::ir::ImplItem;
use super::ir::ImplItemKind;
use super::ir::Item;
use super::ir::ItemKind;
use super::ir::Lit;
use super::ir::Macro;
use super::ir::MacroCall;
use super::ir::Member;
use super::ir::Pat;
use super::ir::Path;
use super::ir::Qualified;
use super::ir::Receiver;
use super::ir::Segment;
use super::ir::Signature;
use super::ir::Stmt;
use super::ir::TraitItem;
use super::ir::TraitItemKind;
use super::ir::Type;
use super::ir::UseTree;
use super::ir::Variant;
use super::ir::Vis;
use crate::ItemId;
use crate::SourceCrate;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::path::PathBuf;
use syn::spanned::Spanned;

const INDENT: &str = "    ";

/// The files of the items, items of `c`, with each item replaced at its span
/// by the item printed; the rest of the files stays as it is.
pub fn rewrite(c: &SourceCrate, items: &[(ItemId, Item)]) -> BTreeMap<PathBuf, String> {
    let mut edits = BTreeMap::<_, Vec<_>>::new();

    for (id, i) in items {
        edits
            .entry(c.module(id.module()).file())
            .or_default()
            .push((c.item(*id).span().byte_range(), decl(i)));
    }

    edits
        .into_iter()
        .map(|(file, mut edits)| {
            let file = c.file(file);
            let mut text = file.text().to_string();
            edits.sort_by_key(|(range, _)| Reverse(range.start));

            for (range, item) in edits {
                text.replace_range(range, &item);
            }

            (file.path().to_path_buf(), text)
        })
        .collect()
}

/// The item ending with a newline.
pub fn item(item: &Item) -> String {
    format!("{}\n", decl(item))
}

fn decl(item: &Item) -> String {
    let vis = vis(item.vis);
    let decl = match &item.kind {
        ItemKind::Struct(s) => {
            let head = format!("{vis}struct {}{}", s.name, params(&s.generics));
            let clause = where_clause(&s.generics);

            match &s.fields {
                Fields::Named(fields) => format!("{head}{clause} {}", members(fields, named_field)),
                Fields::Unnamed(fields) => format!("{head}({}){clause};", inline_fields(fields)),
                Fields::Unit => format!("{head}{clause};"),
            }
        }
        ItemKind::Enum(e) => format!(
            "{vis}enum {}{}{} {}",
            e.name,
            params(&e.generics),
            where_clause(&e.generics),
            members(&e.variants, variant)
        ),
        ItemKind::Trait(t) => {
            let supertraits = if t.supertraits.is_empty() {
                String::new()
            } else {
                format!(": {}", bounds(&t.supertraits))
            };
            format!(
                "{vis}trait {}{}{supertraits}{} {}",
                t.name,
                params(&t.generics),
                where_clause(&t.generics),
                members(&t.items, trait_item)
            )
        }
        ItemKind::Impl(i) => {
            let trait_ = i
                .trait_
                .as_ref()
                .map_or(String::new(), |t| format!("{} for ", path(t)));
            format!(
                "impl{} {trait_}{}{} {}",
                params(&i.generics),
                ty(&i.self_ty),
                where_clause(&i.generics),
                members(&i.items, impl_item)
            )
        }
        ItemKind::Fn(f) => format!("{vis}{}", function(f)),
        ItemKind::Const(k) => format!("{vis}const {}: {} = {};", k.name, ty(&k.ty), expr(&k.expr)),
        ItemKind::Static(s) => {
            format!("{vis}static {}: {} = {};", s.name, ty(&s.ty), expr(&s.expr))
        }
        ItemKind::TypeAlias(t) => format!(
            "{vis}type {}{}{} = {};",
            t.name,
            params(&t.generics),
            where_clause(&t.generics),
            ty(&t.ty)
        ),
        ItemKind::Use(u) => format!(
            "{vis}use {}{};",
            if u.leading_colon { "::" } else { "" },
            use_tree(&u.tree)
        ),
        ItemKind::ExternCrate(e) => format!("{vis}extern crate {}{};", e.name, rename(&e.rename)),
    };

    format!("{}{decl}", attrs(&item.attrs))
}

fn use_tree(tree: &UseTree) -> String {
    match tree {
        UseTree::Path { name, tree } => format!("{name}::{}", use_tree(tree)),
        UseTree::Name {
            name, rename: r, ..
        } => format!("{name}{}", rename(r)),
        UseTree::Glob(_) => "*".to_string(),
        UseTree::Group(trees) => {
            let trees: Vec<String> = trees.iter().map(use_tree).collect();
            format!("{{{}}}", trees.join(", "))
        }
    }
}

fn rename(rename: &Option<String>) -> String {
    rename
        .as_ref()
        .map_or(String::new(), |r| format!(" as {r}"))
}

/// The attributes before an expression on its line.
fn inline_attrs(attrs: &[Attr]) -> String {
    attrs
        .iter()
        .map(|attr| match attr {
            Attr::Doc(doc) => format!("#[doc = {doc:?}] "),
            Attr::Meta(meta) => format!("#[{meta}] "),
        })
        .collect()
}

/// A line per attribute.
fn attrs(attrs: &[Attr]) -> String {
    attrs
        .iter()
        .map(|attr| match attr {
            Attr::Doc(doc) => doc.split('\n').map(|line| format!("///{line}\n")).collect(),
            Attr::Meta(meta) => format!("#[{meta}]\n"),
        })
        .collect()
}

fn vis(vis: Vis) -> &'static str {
    match vis {
        Vis::Private => "",
        Vis::Public => "pub ",
        Vis::Crate => "pub(crate) ",
        Vis::Super => "pub(super) ",
    }
}

/// Each line indented.
fn indent(text: &str) -> String {
    text.lines()
        .map(|line| format!("{INDENT}{line}\n"))
        .collect()
}

/// The members in braces, each on its own indented lines.
fn members<T>(members: &[T], member: fn(&T) -> String) -> String {
    if members.is_empty() {
        return "{}".to_string();
    }

    let lines: String = members.iter().map(|m| indent(&member(m))).collect();
    format!("{{\n{lines}}}")
}

fn named_field(field: &Field) -> String {
    format!("{}{},", attrs(&field.attrs), field_decl(field))
}

fn field_decl(field: &Field) -> String {
    let name = field
        .name
        .as_ref()
        .map_or(String::new(), |name| format!("{name}: "));
    format!("{}{name}{}", vis(field.vis), ty(&field.ty))
}

/// The fields on one line, a doc comment as `#[doc = ".."]`.
fn inline_fields(fields: &[Field]) -> String {
    fields
        .iter()
        .map(|field| {
            let attrs: String = field
                .attrs
                .iter()
                .map(|attr| match attr {
                    Attr::Doc(doc) => format!("#[doc = {doc:?}] "),
                    Attr::Meta(meta) => format!("#[{meta}] "),
                })
                .collect();
            format!("{attrs}{}", field_decl(field))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn variant(variant: &Variant) -> String {
    let fields = match &variant.fields {
        Fields::Named(fields) => format!(" {{ {} }}", inline_fields(fields)),
        Fields::Unnamed(fields) => format!("({})", inline_fields(fields)),
        Fields::Unit => String::new(),
    };
    let discriminant = variant
        .discriminant
        .as_ref()
        .map_or(String::new(), |e| format!(" = {}", expr(e)));
    format!(
        "{}{}{fields}{discriminant},",
        attrs(&variant.attrs),
        variant.name
    )
}

fn trait_item(item: &TraitItem) -> String {
    let decl = match &item.kind {
        TraitItemKind::Fn { sig, body } => match body {
            Some(body) => format!("{} {}", signature(sig), block(body)),
            None => format!("{};", signature(sig)),
        },
        TraitItemKind::Type { name, bounds: b } if b.is_empty() => format!("type {name};"),
        TraitItemKind::Type { name, bounds: b } => format!("type {name}: {};", bounds(b)),
    };
    format!("{}{decl}", attrs(&item.attrs))
}

fn impl_item(item: &ImplItem) -> String {
    let decl = match &item.kind {
        ImplItemKind::Fn(f) => function(f),
        ImplItemKind::Type { name, ty: t } => format!("type {name} = {};", ty(t)),
    };
    format!("{}{}{decl}", attrs(&item.attrs), vis(item.vis))
}

fn function(f: &Fn) -> String {
    format!("{} {}", signature(&f.sig), block(&f.body))
}

fn signature(sig: &Signature) -> String {
    let receiver = sig.receiver.as_ref().map(|r| match r {
        Receiver::Value { mutable } => format!("{}self", mutability(*mutable)),
        Receiver::Ref { lifetime, mutable } => format!(
            "&{}{}self",
            lifetime.as_ref().map_or(String::new(), |l| format!("{l} ")),
            mutability(*mutable)
        ),
        Receiver::Typed { mutable, ty: t } => format!("{}self: {}", mutability(*mutable), ty(t)),
    });
    let inputs: Vec<String> = receiver
        .into_iter()
        .chain(
            sig.params
                .iter()
                .map(|p| format!("{}{}: {}", mutability(p.mutable), p.name, ty(&p.ty))),
        )
        .collect();
    let output = sig
        .output
        .as_ref()
        .map_or(String::new(), |t| format!(" -> {}", ty(t)));

    format!(
        "{}{}fn {}{}({}){output}{}",
        if sig.constness { "const " } else { "" },
        if sig.asyncness { "async " } else { "" },
        sig.name,
        params(&sig.generics),
        inputs.join(", "),
        where_clause(&sig.generics)
    )
}

fn mutability(mutable: bool) -> &'static str {
    if mutable { "mut " } else { "" }
}

fn params(generics: &Generics) -> String {
    if generics.params.is_empty() {
        return String::new();
    }

    let params: Vec<String> = generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Lifetime(name) => name.clone(),
            GenericParam::Type { name, bounds: b } if b.is_empty() => name.clone(),
            GenericParam::Type { name, bounds: b } => format!("{name}: {}", bounds(b)),
            GenericParam::Const { name, ty: t } => format!("const {name}: {}", ty(t)),
        })
        .collect();
    format!("<{}>", params.join(", "))
}

fn where_clause(generics: &Generics) -> String {
    if generics.predicates.is_empty() {
        return String::new();
    }

    let predicates: Vec<String> = generics
        .predicates
        .iter()
        .map(|p| format!("{}: {}", ty(&p.ty), bounds(&p.bounds)))
        .collect();
    format!(" where {}", predicates.join(", "))
}

fn bounds(bounds: &[Bound]) -> String {
    bounds
        .iter()
        .map(|bound| match bound {
            Bound::Trait(p) => path(p),
            Bound::Lifetime(l) => l.clone(),
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn ty(t: &Type) -> String {
    match t {
        Type::Path(p) => path(p),
        Type::Qualified(q) => qualified(q, ""),
        Type::Ref {
            lifetime,
            mutable,
            ty: t,
        } => format!(
            "&{}{}{}",
            lifetime.as_ref().map_or(String::new(), |l| format!("{l} ")),
            mutability(*mutable),
            ty(t)
        ),
        Type::Tuple(types) => tuple(&types.iter().map(ty).collect::<Vec<_>>()),
        Type::Slice(t) => format!("[{}]", ty(t)),
        Type::Array(t, len) => format!("[{}; {}]", ty(t), expr(len)),
        Type::ImplTrait(b) => format!("impl {}", bounds(b)),
        Type::DynTrait(b) => format!("dyn {}", bounds(b)),
        Type::Paren(t) => format!("({})", ty(t)),
        Type::FnPtr(f) => format!("fn{}", fn_types(f)),
        Type::Infer => "_".to_string(),
    }
}

/// `(a, b)`, `(a,)`.
fn tuple(elems: &[String]) -> String {
    match elems {
        [one] => format!("({one},)"),
        elems => format!("({})", elems.join(", ")),
    }
}

/// The generic arguments of the segments after `separator`: `::` in
/// expressions and patterns.
fn qualified(q: &Qualified, separator: &str) -> String {
    format!(
        "<{}{}>::{}",
        ty(&q.ty),
        q.trait_
            .as_ref()
            .map_or(String::new(), |t| format!(" as {}", path(t))),
        q.rest
            .iter()
            .map(|s| segment(s, separator))
            .collect::<Vec<_>>()
            .join("::")
    )
}

/// `(A, B) -> C`.
fn fn_types(f: &FnTypes) -> String {
    let inputs: Vec<String> = f.inputs.iter().map(ty).collect();
    let output = f
        .output
        .as_ref()
        .map_or(String::new(), |t| format!(" -> {}", ty(t)));
    format!("({}){output}", inputs.join(", "))
}

fn path(p: &Path) -> String {
    path_with(p, "")
}

fn expr_path(p: &Path) -> String {
    path_with(p, "::")
}

fn path_with(p: &Path, separator: &str) -> String {
    let segments: Vec<String> = p.segments.iter().map(|s| segment(s, separator)).collect();
    format!(
        "{}{}",
        if p.leading_colon { "::" } else { "" },
        segments.join("::")
    )
}

fn segment(segment: &Segment, separator: &str) -> String {
    match &segment.args {
        Args::None => segment.name.clone(),
        Args::Angle(args) => format!("{}{separator}<{}>", segment.name, generic_args(args)),
        Args::Fn(f) => format!("{}{}", segment.name, fn_types(f)),
    }
}

fn generic_args(args: &[GenericArg]) -> String {
    args.iter()
        .map(|arg| match arg {
            GenericArg::Type(t) => ty(t),
            GenericArg::Lifetime(l) => l.clone(),
            GenericArg::Const(e) => expr(e),
            GenericArg::AssocType { name, ty: t } => format!("{name} = {}", ty(t)),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn block(b: &Block) -> String {
    if b.stmts.is_empty() {
        return "{}".to_string();
    }

    let lines: String = b.stmts.iter().map(|s| indent(&stmt(s))).collect();
    format!("{{\n{lines}}}")
}

fn stmt(s: &Stmt) -> String {
    match s {
        Stmt::Let {
            attrs: a,
            pat: p,
            init,
            diverge,
        } => format!(
            "{}let {}{}{};",
            attrs(a),
            pat(p),
            init.as_ref()
                .map_or(String::new(), |e| format!(" = {}", expr(e))),
            diverge
                .as_ref()
                .map_or(String::new(), |e| format!(" else {}", expr(e)))
        ),
        Stmt::Item(i) => decl(i),
        Stmt::Expr {
            attrs: a,
            expr: e,
            semi,
        } => format!("{}{}{}", attrs(a), expr(e), if *semi { ";" } else { "" }),
    }
}

fn exprs(exprs: &[Expr]) -> String {
    exprs.iter().map(expr).collect::<Vec<_>>().join(", ")
}

fn expr(e: &Expr) -> String {
    match e {
        Expr::Array(elems) => format!("[{}]", exprs(elems)),
        Expr::Assign { left, right } => format!("{} = {}", expr(left), expr(right)),
        Expr::Async { capture, block: b } => {
            format!("async {}{}", if *capture { "move " } else { "" }, block(b))
        }
        Expr::Await(e) => format!("{}.await", expr(e)),
        Expr::Attributed { attrs, expr: e } => format!("{}{}", inline_attrs(attrs), expr(e)),
        Expr::Binary { left, op, right } => {
            format!("{} {} {}", expr(left), op.token(), expr(right))
        }
        Expr::Block(b) => block(b),
        Expr::Break => "break".to_string(),
        Expr::Call { func, args } => format!("{}({})", expr(func), exprs(args)),
        Expr::Cast { expr: e, ty: t } => format!("{} as {}", expr(e), ty(t)),
        Expr::Closure(c) => {
            let inputs: Vec<String> = c.inputs.iter().map(pat).collect();
            let output = c
                .output
                .as_ref()
                .map_or(String::new(), |t| format!(" -> {}", ty(t)));
            format!(
                "{}|{}|{output} {}",
                if c.capture { "move " } else { "" },
                inputs.join(", "),
                expr(&c.body)
            )
        }
        Expr::Const(b) => format!("const {}", block(b)),
        Expr::Continue => "continue".to_string(),
        Expr::Field { expr: e, member: m } => format!("{}.{}", expr(e), member(m)),
        Expr::ForLoop {
            pat: p,
            expr: e,
            body,
        } => format!("for {} in {} {}", pat(p), expr(e), block(body)),
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => format!(
            "if {} {}{}",
            expr(cond),
            block(then_branch),
            else_branch
                .as_ref()
                .map_or(String::new(), |e| format!(" else {}", expr(e)))
        ),
        Expr::Index { expr: e, index } => format!("{}[{}]", expr(e), expr(index)),
        Expr::Let { pat: p, expr: e } => format!("let {} = {}", pat(p), expr(e)),
        Expr::Lit(l) => lit(l),
        Expr::Loop(b) => format!("loop {}", block(b)),
        Expr::Macro(m) => mac(m),
        Expr::Match { expr: e, arms } => format!("match {} {}", expr(e), members(arms, arm)),
        Expr::MethodCall {
            receiver,
            method,
            turbofish,
            args,
        } => {
            let turbofish = if turbofish.is_empty() {
                String::new()
            } else {
                format!("::<{}>", generic_args(turbofish))
            };
            format!("{}.{method}{turbofish}({})", expr(receiver), exprs(args))
        }
        Expr::Paren(e) => format!("({})", expr(e)),
        Expr::Path(p) => expr_path(p),
        Expr::Qualified(q) => qualified(q, "::"),
        Expr::Range { start, closed, end } => range(start, *closed, end),
        Expr::Reference { mutable, expr: e } => format!("&{}{}", mutability(*mutable), expr(e)),
        Expr::Repeat { expr: e, len } => format!("[{}; {}]", expr(e), expr(len)),
        Expr::Return(e) => match e {
            Some(e) => format!("return {}", expr(e)),
            None => "return".to_string(),
        },
        Expr::Struct { path: p, fields } if fields.is_empty() => format!("{} {{}}", expr_path(p)),
        Expr::Struct { path: p, fields } => {
            let fields: Vec<String> = fields.iter().map(field_value).collect();
            format!("{} {{ {} }}", expr_path(p), fields.join(", "))
        }
        Expr::Try(e) => format!("{}?", expr(e)),
        Expr::Tuple(elems) => tuple(&elems.iter().map(expr).collect::<Vec<_>>()),
        Expr::Unary { op, expr: e } => format!("{}{}", op.token(), expr(e)),
        Expr::While { cond, body } => format!("while {} {}", expr(cond), block(body)),
    }
}

fn member(m: &Member) -> String {
    match m {
        Member::Named(name) => name.clone(),
        Member::Index(index) => index.to_string(),
    }
}

/// `name` for `name: name`.
fn field_value(f: &FieldValue) -> String {
    let m = member(&f.member);

    match &f.expr {
        Expr::Path(p) if expr_path(p) == m => m,
        e => format!("{m}: {}", expr(e)),
    }
}

/// Without a comma after a block.
fn arm(a: &Arm) -> String {
    let guard = a
        .guard
        .as_ref()
        .map_or(String::new(), |g| format!(" if {}", expr(g)));
    let comma = if matches!(a.body, Expr::Block(_)) {
        ""
    } else {
        ","
    };
    format!(
        "{}{}{guard} => {}{comma}",
        attrs(&a.attrs),
        pat(&a.pat),
        expr(&a.body)
    )
}

fn lit(l: &Lit) -> String {
    match l {
        Lit::Str(s) => format!("{s:?}"),
        Lit::Char(c) => format!("{c:?}"),
        Lit::Int { digits, suffix } | Lit::Float { digits, suffix } => format!("{digits}{suffix}"),
        Lit::Bool(b) => b.to_string(),
    }
}

fn mac(m: &Macro) -> String {
    let args = match &m.call {
        MacroCall::Format { exprs: e, format } => {
            let format = format.iter().flat_map(|f| {
                std::iter::once(format!("{:?}", f.string)).chain(f.args.iter().map(
                    |a| match &a.name {
                        Some(name) => format!("{name} = {}", expr(&a.expr)),
                        None => expr(&a.expr),
                    },
                ))
            });
            let args: Vec<String> = e.iter().map(expr).chain(format).collect();
            format!("({})", args.join(", "))
        }
        MacroCall::Vec(e) => format!("[{}]", exprs(e)),
        MacroCall::NoArgs => "()".to_string(),
        MacroCall::VecRepeat(value, count) => format!("[{}; {}]", expr(value), expr(count)),
        MacroCall::Matches {
            expr: e,
            pat: p,
            guard,
        } => format!(
            "({}, {}{})",
            expr(e),
            pat(p),
            guard
                .as_ref()
                .map_or(String::new(), |g| format!(" if {}", expr(g)))
        ),
        MacroCall::ThreadLocal(statics) => format!(" {}", members(statics, decl)),
    };
    format!("{}!{args}", path(&m.path))
}

fn range(start: &Option<Box<Expr>>, closed: bool, end: &Option<Box<Expr>>) -> String {
    format!(
        "{}{}{}",
        start.as_ref().map_or(String::new(), |e| expr(e)),
        if closed { "..=" } else { ".." },
        end.as_ref().map_or(String::new(), |e| expr(e))
    )
}

fn pat(p: &Pat) -> String {
    match p {
        Pat::Binding {
            by_ref,
            mutable,
            name,
            subpat,
        } => format!(
            "{}{}{name}{}",
            if *by_ref { "ref " } else { "" },
            mutability(*mutable),
            subpat
                .as_ref()
                .map_or(String::new(), |p| format!(" @ {}", pat(p)))
        ),
        Pat::Lit(l) => lit(l),
        Pat::Or(cases) => cases.iter().map(pat).collect::<Vec<_>>().join(" | "),
        Pat::Path(p) => expr_path(p),
        Pat::Paren(p) => format!("({})", pat(p)),
        Pat::Range { start, closed, end } => range(start, *closed, end),
        Pat::Ref { mutable, pat: p } => format!("&{}{}", mutability(*mutable), pat(p)),
        Pat::Struct {
            path: p,
            fields,
            rest,
        } => {
            let fields = fields.iter().map(|f| {
                let m = member(&f.member);

                match &f.pat {
                    Pat::Binding {
                        name, subpat: None, ..
                    } if *name == m => pat(&f.pat),
                    p => format!("{m}: {}", pat(p)),
                }
            });
            let fields: Vec<String> = fields.chain(rest.then(|| "..".to_string())).collect();

            if fields.is_empty() {
                format!("{} {{}}", expr_path(p))
            } else {
                format!("{} {{ {} }}", expr_path(p), fields.join(", "))
            }
        }
        Pat::Tuple(elems) => tuple(&elems.iter().map(pat).collect::<Vec<_>>()),
        Pat::TupleStruct { path: p, elems } => {
            let elems: Vec<String> = elems.iter().map(pat).collect();
            format!("{}({})", expr_path(p), elems.join(", "))
        }
        Pat::Type { pat: p, ty: t } => format!("{}: {}", pat(p), ty(t)),
        Pat::Wild => "_".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cfg;
    use crate::ItemId;
    use crate::SourceCrate;
    use crate::Target;
    use crate::TargetKind;
    use agplatform::Fs;
    use agplatform::Platform;
    use agplatform::test_platform::test_platform;

    fn load(source: &str) -> SourceCrate {
        let platform = test_platform();
        platform.fs().create_dir_all("/c/src").unwrap();
        platform.fs().write("/c/src/lib.rs", source).unwrap();
        SourceCrate::load(
            &platform,
            &Target::new(TargetKind::Lib, "/c/src/lib.rs"),
            &Cfg::All,
        )
    }

    fn ir(c: &SourceCrate, indices: &[usize]) -> Vec<(ItemId, Item)> {
        let ids = indices.iter().map(|&index| ItemId {
            module: c.root(),
            index,
        });
        let (items, errors) = c.ir(ids);
        assert!(errors.is_empty(), "{errors:?}");
        items
    }

    /// The root items of the crate printed from the IR.
    fn print(source: &str) -> String {
        let c = load(source);
        let indices: Vec<usize> = (0..c.items(c.root()).len()).collect();
        ir(&c, &indices)
            .iter()
            .map(|(_, i)| super::item(i))
            .collect()
    }

    mod rewrite {
        use super::*;

        #[test]
        fn rewrite_should_replace_the_items_and_keep_the_rest() {
            let c = load(
                "pub struct A;

fn keep() {}
#[derive(Debug)]
pub struct B { a: u8 }
",
            );
            let items = ir(&c, &[0, 2]);

            let files = super::super::rewrite(&c, &items);

            assert_eq!(
                files,
                BTreeMap::from([(
                    PathBuf::from("/c/src/lib.rs"),
                    "pub struct A;

fn keep() {}
#[derive(Debug)]
pub struct B {
    a: u8,
}
"
                    .to_string()
                )])
            );
        }
    }

    mod item {
        use super::*;

        #[test]
        fn item_should_print_structs_as_written() {
            let source = "/// A doc.
#[derive(Debug)]
pub struct Named<'a, T: Clone + 'a, const N: usize> where T: Default {
    /// A field.
    pub(crate) a: &'a T,
    b: [u8; N],
}
pub(super) struct Tuple(#[doc = \" x\"] pub u8, #[allow(dead_code)] (u8,));
struct Bounded<T>(T) where T: Copy;
struct Unit;
struct Empty {}
";

            assert_eq!(print(source), source);
        }

        #[test]
        fn item_should_print_enums_as_written() {
            let source = "pub enum E<T> {
    #[default]
    A,
    B(T, u8),
    C { a: u8, b: Vec<T> },
    D = 1 + 2,
}
enum Empty {}
";

            assert_eq!(print(source), source);
        }

        #[test]
        fn item_should_print_traits_as_written() {
            let source = "pub trait T<A>: Clone + Send where A: Copy {
    /// An associated type.
    type Item: Into<A>;
    type Plain;
    fn required(&self, a: A) -> Self::Item;
    async fn provided<'a>(&'a mut self, mut b: impl Fn(&A) -> bool) -> Option<&'a dyn Send> {
        None
    }
}
trait Empty {}
";

            assert_eq!(print(source), source);
        }

        #[test]
        fn item_should_print_impls_as_written() {
            let source = "impl<T: Clone> Trait for Wrapper<T> where T: Send {
    type Item = (T, u8);
    #[inline]
    pub(crate) const fn f(self, x: fn(u8) -> u8, y: fn()) {}
    fn g(mut self) -> ::std::boxed::Box<(dyn Send)> {
        Box::new(())
    }
}
impl S {}
";

            assert_eq!(print(source), source);
        }

        #[test]
        fn item_should_print_functions_constants_statics_and_aliases_as_written() {
            let source = "pub fn f<'a, T>(a: &'a [T], b: &mut u8) -> &'a T where T: Copy {
    &a[*b as usize]
}
pub const C: [u8; 2] = [1, 2];
static S: &'static str = \"s\";
pub type Alias<T> = Result<T, Error<'static, 3, Item = u8>>;
type Handle<P> = <<P as Platform>::Fs as Fs>::WriteHandle;
type Output<T> = <T>::Output<u8>;
";

            assert_eq!(print(source), source);
        }

        #[test]
        fn item_should_print_bodies_as_written() {
            let source = r#"fn f<T: Default>(a: u8, mut b: Vec<u8>) -> u8 {
    let c = [1, 2];
    let d;
    let (e, _): (u8, u8) = (1, 2);
    d = a;
    {
        let j = 1;
    }
    let g = |x: u8| -> u8 {
        x
    };
    let h = |y, (z, _)| y;
    let i = const {
        1
    };
    for k in 0..a {
        continue;
    }
    while let Some(l) = b.pop() {
        break;
    }
    loop {}
    if a > 1 {
        return a;
    } else if a < 1 {
        return;
    } else {
        a;
    }
    let m = match b.len() {
        0 => 1,
        n if n > 1 => {
            2
        }
        _ => 3,
    };
    match a {}
    let n = S { a, b: 1 };
    let o = S {};
    let S { a, b: _, .. } = n;
    let S {} = o;
    let S { 0: x } = o;
    let q = Vec::<u8>::new();
    let r = b.iter().map(|x| *x).collect::<Vec<_>>();
    let s = <T as Default>::default();
    let t = (a as u64, (a), !true, -1, &a, &mut b, a.0, b?, b[0], (a,));
    let u = [0; 4];
    let v = (1..=2, ..2, 1.., ..);
    let w = ("s\n\"", 'c', '\'', 1.5f64, 2u8, 0, false, true);
    let z = format!("{a} {}", b, c = 1);
    println!();
    let aa = vec![1, 2];
    let ab = vec![0; 2];
    let ac = matches!(a, 1 | 2 if b);
    let ad = matches!(a, _);
    assert_eq!(a, 1);
    thread_local! {
        static TL: u8 = 1;
    }
    match a {
        A::B(x) | A::C { x } => {}
        A::D => {}
        &x => {}
        &mut x => {}
        #[cfg(all())]
        1 => {}
    }
    struct Local;
    use std::mem::swap as sw;
    use std::mem::take;
    if let A::B(x) = a {}
    T::default()
}
"#;

            assert_eq!(print(source), source);
        }

        #[test]
        fn item_should_print_the_constructs_of_tests_as_written() {
            let source = r#"use ::std::io;
pub use std::{self as s, fmt::{Debug, Display as D}, collections::*};
extern crate alloc;
pub extern crate self as c;
impl S {
    fn f(self: Box<Self>) {}
    fn g(mut self: Rc<Self>) {}
}
async fn h(a: Option<u8>) -> u8 {
    let Some(b) = a else {
        return 0;
    };
    #[allow(x)]
    let c = 1;
    #[allow(x)]
    a;
    #[allow(x)]
    println!();
    let d = async move {
        1
    }.await;
    let e = async {};
    let f = move || b;
    let g = (file!(), line!(), column!(), module_path!());
    let i = vec![#[allow(x)] 1, #[doc = " d"] 2];
    match a {
        Some(ref mut j @ 1..=2) => {}
        Some(k @ (3 | 4)) => {}
        Some(..=5 | 6..) => {}
        _ => {}
    }
    b
}
"#;

            assert_eq!(print(source), source);
        }

        #[test]
        fn item_should_print_every_operator() {
            let source = "fn f() {
    (a + b, a - b, a * b, a / b, a % b, a && b, a || b, a ^ b, a & b, a | b, a << b, a >> b);
    (a == b, a < b, a <= b, a != b, a >= b, a > b);
    (*a, !a, -a);
    a += b;
    a -= b;
    a *= b;
    a /= b;
    a %= b;
    a ^= b;
    a &= b;
    a |= b;
    a <<= b;
    a >>= b;
}
";

            assert_eq!(print(source), source);
        }

        #[test]
        fn item_should_print_attributes_after_cfg_attr_and_docs_by_line() {
            let source = "/** a
 b */
#[cfg_attr(all(), derive(Debug), allow(x))]
struct S;";

            assert_eq!(
                print(source),
                "/// a
/// b 
#[derive(Debug)]
#[allow(x)]
struct S;
"
            );
        }
    }
}
