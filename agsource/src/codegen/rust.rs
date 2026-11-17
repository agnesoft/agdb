//! Rust source from the IR, laid out like rustfmt for short lines: a member
//! per line, indented by four spaces, `where` clauses inline. Bodies and
//! expressions are printed as they are.

use super::ir::Args;
use super::ir::Attr;
use super::ir::Block;
use super::ir::Bound;
use super::ir::Expr;
use super::ir::Field;
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
use super::ir::Path;
use super::ir::Receiver;
use super::ir::Signature;
use super::ir::TraitItem;
use super::ir::TraitItemKind;
use super::ir::Type;
use super::ir::Variant;
use super::ir::Vis;
use crate::SourceCrate;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::path::PathBuf;
use syn::spanned::Spanned;

const INDENT: &str = "    ";

/// The files of the items, items of `c`, with each item replaced at its span
/// by the item printed; the rest of the files stays as it is.
pub fn rewrite(c: &SourceCrate, items: &[Item]) -> BTreeMap<PathBuf, String> {
    let mut edits = BTreeMap::<_, Vec<_>>::new();

    for i in items {
        let mut text = item(i);
        text.pop();
        edits
            .entry(c.module(i.id.module()).file())
            .or_default()
            .push((c.item(i.id).span().byte_range(), text));
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
    };

    format!("{}{decl}\n", attrs(&item.attrs, ""))
}

fn attrs(attrs: &[Attr], indent: &str) -> String {
    attrs
        .iter()
        .map(|attr| match attr {
            Attr::Doc(doc) => doc
                .split('\n')
                .map(|line| format!("{indent}///{line}\n"))
                .collect(),
            Attr::Meta(meta) => format!("{indent}#[{meta}]\n"),
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

/// The members in braces, each on its own indented line.
fn members<T>(members: &[T], member: fn(&T) -> String) -> String {
    if members.is_empty() {
        return "{}".to_string();
    }

    let lines: String = members.iter().map(member).collect();
    format!("{{\n{lines}}}")
}

fn named_field(field: &Field) -> String {
    format!(
        "{}{INDENT}{},\n",
        attrs(&field.attrs, INDENT),
        field_decl(field)
    )
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
        "{}{INDENT}{}{fields}{discriminant},\n",
        attrs(&variant.attrs, INDENT),
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
    format!("{}{INDENT}{decl}\n", attrs(&item.attrs, INDENT))
}

fn impl_item(item: &ImplItem) -> String {
    let decl = match &item.kind {
        ImplItemKind::Fn(f) => function(f),
        ImplItemKind::Type { name, ty: t } => format!("type {name} = {};", ty(t)),
    };
    format!(
        "{}{INDENT}{}{decl}\n",
        attrs(&item.attrs, INDENT),
        vis(item.vis)
    )
}

fn function(f: &Fn) -> String {
    format!("{} {}", signature(&f.sig), block(&f.body))
}

fn signature(sig: &Signature) -> String {
    let receiver = sig.receiver.as_ref().map(|r| match r {
        Receiver::Value { mutable } => format!("{}self", if *mutable { "mut " } else { "" }),
        Receiver::Ref { lifetime, mutable } => format!(
            "&{}{}self",
            lifetime.as_ref().map_or(String::new(), |l| format!("{l} ")),
            if *mutable { "mut " } else { "" }
        ),
    });
    let inputs: Vec<String> = receiver
        .into_iter()
        .chain(sig.params.iter().map(|p| {
            format!(
                "{}{}: {}",
                if p.mutable { "mut " } else { "" },
                p.name,
                ty(&p.ty)
            )
        }))
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
        Type::Ref {
            lifetime,
            mutable,
            ty: t,
        } => format!(
            "&{}{}{}",
            lifetime.as_ref().map_or(String::new(), |l| format!("{l} ")),
            if *mutable { "mut " } else { "" },
            ty(t)
        ),
        Type::Tuple(types) if types.len() == 1 => format!("({},)", ty(&types[0])),
        Type::Tuple(types) => format!("({})", types.iter().map(ty).collect::<Vec<_>>().join(", ")),
        Type::Slice(t) => format!("[{}]", ty(t)),
        Type::Array(t, len) => format!("[{}; {}]", ty(t), expr(len)),
        Type::ImplTrait(b) => format!("impl {}", bounds(b)),
        Type::DynTrait(b) => format!("dyn {}", bounds(b)),
        Type::Paren(t) => format!("({})", ty(t)),
        Type::FnPtr(f) => format!("fn{}", fn_types(f)),
    }
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
    let segments: Vec<String> = p
        .segments
        .iter()
        .map(|segment| match &segment.args {
            Args::None => segment.name.clone(),
            Args::Angle(args) => {
                let args: Vec<String> = args
                    .iter()
                    .map(|arg| match arg {
                        GenericArg::Type(t) => ty(t),
                        GenericArg::Lifetime(l) => l.clone(),
                        GenericArg::Const(e) => expr(e).to_string(),
                        GenericArg::AssocType { name, ty: t } => format!("{name} = {}", ty(t)),
                    })
                    .collect();
                format!("{}<{}>", segment.name, args.join(", "))
            }
            Args::Fn(f) => format!("{}{}", segment.name, fn_types(f)),
        })
        .collect();
    format!(
        "{}{}",
        if p.leading_colon { "::" } else { "" },
        segments.join("::")
    )
}

fn expr(e: &Expr) -> &str {
    match e {
        Expr::Verbatim(text) => text,
    }
}

fn block(b: &Block) -> &str {
    match b {
        Block::Verbatim(text) => text,
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

    fn ir(c: &SourceCrate, indices: &[usize]) -> Vec<Item> {
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
        ir(&c, &indices).iter().map(super::item).collect()
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
    #![allow(unused)]
    &a[*b as usize]
}
pub const C: [u8; 2] = [1, 2];
static S: &'static str = \"s\";
pub type Alias<T> = Result<T, Error<'static, 3, { 1 + 1 }, Item = u8>>;
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
