use crate::Error;
use proc_macro2::Span;
use std::collections::BTreeSet;
use std::path::Path;
use syn::Attribute;
use syn::Meta;
use syn::Token;
use syn::ext::IdentExt;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::visit_mut;
use syn::visit_mut::VisitMut;

/// The code a [`SourceCrate`](crate::SourceCrate) is loaded with.
#[derive(Debug, Clone, Default)]
pub enum Cfg {
    /// All code: nothing is stripped and every `cfg_attr` applies, so a
    /// `#[path]` given by two `cfg_attr`s takes the first.
    #[default]
    All,
    /// The code rustc compiles with the options of the set: elements whose
    /// `#[cfg]` does not hold are stripped and a `cfg_attr` applies when its
    /// predicate holds.
    Eval(CfgSet),
}

/// The cfg options of a build: names such as `test` or `unix` and keys with
/// values such as `feature = "serde"`. A name or key the set does not know
/// is false and reported as [`Error::UnknownCfg`].
#[derive(Debug, Clone, Default)]
pub struct CfgSet {
    options: BTreeSet<(String, Option<String>)>,
    known: BTreeSet<String>,
}

enum Predicate {
    Bool(bool),
    Name(String, Span),
    Value(String, String, Span),
    All(Vec<Predicate>),
    Any(Vec<Predicate>),
    Not(Box<Predicate>),
}

struct Configure<'a> {
    cfg: &'a Cfg,
    path: &'a Path,
    errors: &'a mut Vec<Error>,
    // The next attribute list belongs to the element being kept or
    // stripped, as syn visits the attributes of a node before its children.
    own: bool,
    stripped: bool,
}

impl CfgSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name, e.g. `test` or `unix`.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.known.insert(name.clone());
        self.options.insert((name, None));
        self
    }

    /// Sets `key = "value"`, e.g. `target_os = "linux"`.
    pub fn value(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        self.known.insert(key.clone());
        self.options.insert((key, Some(value.into())));
        self
    }

    /// Sets `feature = "feature"`.
    pub fn feature(self, feature: impl Into<String>) -> Self {
        self.value("feature", feature)
    }

    /// Knows the name or key without setting it, so it is false without a
    /// report, e.g. `windows` on a unix target or `feature` with no features.
    pub fn unset(mut self, name: impl Into<String>) -> Self {
        self.known.insert(name.into());
        self
    }
}

/// Expands the `cfg_attr`s of `file` and, for [`Cfg::Eval`], strips the
/// elements whose `#[cfg]` does not hold. Returns whether the file's own
/// `#![cfg]` holds, its items are removed if not.
pub(crate) fn configure(
    cfg: &Cfg,
    path: &Path,
    file: &mut syn::File,
    errors: &mut Vec<Error>,
) -> bool {
    let mut configure = Configure {
        cfg,
        path,
        errors,
        own: false,
        stripped: false,
    };
    let holds = configure.keep(file, Configure::visit_file_mut);

    if !holds {
        file.items.clear();
    }

    holds
}

impl Configure<'_> {
    /// Visits `node` and returns whether its `#[cfg]` holds. Nothing is
    /// visited within a stripped element.
    fn keep<T>(&mut self, node: &mut T, visit: fn(&mut Self, &mut T)) -> bool {
        if self.stripped {
            return true;
        }

        let own = std::mem::replace(&mut self.own, true);
        visit(self, node);
        self.own = own;
        !std::mem::take(&mut self.stripped)
    }

    fn retain<T>(&mut self, list: &mut Vec<T>, visit: fn(&mut Self, &mut T)) {
        list.retain_mut(|node| self.keep(node, visit));
    }

    fn retain_punctuated<T, P: Default>(
        &mut self,
        list: &mut Punctuated<T, P>,
        visit: fn(&mut Self, &mut T),
    ) {
        let trailing = list.trailing_punct();
        let mut kept = Punctuated::new();

        for mut node in std::mem::take(list) {
            if self.keep(&mut node, visit) {
                kept.push(node);
            }
        }

        if trailing && !kept.is_empty() {
            kept.push_punct(P::default());
        }

        *list = kept;
    }

    fn expand(&mut self, attr: Attribute, expanded: &mut Vec<Attribute>) {
        if !attr.path().is_ident("cfg_attr") {
            expanded.push(attr);
            return;
        }

        let parsed = attr.parse_args_with(|input: ParseStream| {
            let predicate = predicate(input)?;
            input.parse::<Token![,]>()?;
            Ok((
                predicate,
                Punctuated::<Meta, Token![,]>::parse_terminated(input)?,
            ))
        });

        match parsed {
            Ok((predicate, metas)) => {
                if self.holds(&predicate) {
                    for meta in metas {
                        let attr = Attribute {
                            pound_token: attr.pound_token,
                            style: attr.style,
                            bracket_token: attr.bracket_token,
                            meta,
                        };
                        self.expand(attr, expanded);
                    }
                }
            }
            Err(e) => self.parse_error(&e),
        }
    }

    /// Whether every `#[cfg]` among `attrs` holds.
    fn holds_all(&mut self, attrs: &[Attribute]) -> bool {
        let mut holds = true;

        for attr in attrs.iter().filter(|attr| attr.path().is_ident("cfg")) {
            let parsed = attr.parse_args_with(|input: ParseStream| {
                let predicate = predicate(input)?;
                input.parse::<Token![,]>().ok();
                Ok(predicate)
            });

            holds &= match parsed {
                Ok(predicate) => self.holds(&predicate),
                Err(e) => {
                    self.parse_error(&e);
                    false
                }
            };
        }

        holds
    }

    fn holds(&mut self, predicate: &Predicate) -> bool {
        match self.cfg {
            Cfg::All => true,
            Cfg::Eval(set) => self.eval(set, predicate),
        }
    }

    /// Evaluates every operand, so each unknown name is reported.
    fn eval(&mut self, set: &CfgSet, predicate: &Predicate) -> bool {
        match predicate {
            Predicate::Bool(value) => *value,
            Predicate::Name(name, span) => self.option(set, name, None, *span),
            Predicate::Value(key, value, span) => self.option(set, key, Some(value), *span),
            Predicate::All(predicates) => {
                let mut all = true;

                for predicate in predicates {
                    all &= self.eval(set, predicate);
                }

                all
            }
            Predicate::Any(predicates) => {
                let mut any = false;

                for predicate in predicates {
                    any |= self.eval(set, predicate);
                }

                any
            }
            Predicate::Not(predicate) => !self.eval(set, predicate),
        }
    }

    fn option(&mut self, set: &CfgSet, name: &str, value: Option<&str>, span: Span) -> bool {
        if !set.known.contains(name) {
            self.errors.push(Error::UnknownCfg {
                path: self.path.to_path_buf(),
                line: span.start().line,
                name: name.to_string(),
            });
            return false;
        }

        set.options
            .contains(&(name.to_string(), value.map(str::to_string)))
    }

    fn parse_error(&mut self, e: &syn::Error) {
        let start = e.span().start();
        self.errors.push(Error::Parse {
            path: self.path.to_path_buf(),
            line: start.line,
            column: start.column + 1,
            message: e.to_string(),
        });
    }
}

impl VisitMut for Configure<'_> {
    fn visit_attributes_mut(&mut self, attrs: &mut Vec<Attribute>) {
        if self.stripped {
            return;
        }

        let mut expanded = Vec::with_capacity(attrs.len());

        for attr in std::mem::take(attrs) {
            self.expand(attr, &mut expanded);
        }

        *attrs = expanded;

        if std::mem::take(&mut self.own) {
            self.stripped = !self.holds_all(attrs);
        }
    }

    fn visit_file_mut(&mut self, node: &mut syn::File) {
        let mut items = std::mem::take(&mut node.items);
        visit_mut::visit_file_mut(self, node);
        self.retain(&mut items, Self::visit_item_mut);
        node.items = items;
    }

    fn visit_item_mod_mut(&mut self, node: &mut syn::ItemMod) {
        let mut content = node.content.take();
        visit_mut::visit_item_mod_mut(self, node);

        if let Some((_, items)) = &mut content {
            self.retain(items, Self::visit_item_mut);
        }

        node.content = content;
    }

    fn visit_item_impl_mut(&mut self, node: &mut syn::ItemImpl) {
        let mut items = std::mem::take(&mut node.items);
        visit_mut::visit_item_impl_mut(self, node);
        self.retain(&mut items, Self::visit_impl_item_mut);
        node.items = items;
    }

    fn visit_item_trait_mut(&mut self, node: &mut syn::ItemTrait) {
        let mut items = std::mem::take(&mut node.items);
        visit_mut::visit_item_trait_mut(self, node);
        self.retain(&mut items, Self::visit_trait_item_mut);
        node.items = items;
    }

    fn visit_item_foreign_mod_mut(&mut self, node: &mut syn::ItemForeignMod) {
        let mut items = std::mem::take(&mut node.items);
        visit_mut::visit_item_foreign_mod_mut(self, node);
        self.retain(&mut items, Self::visit_foreign_item_mut);
        node.items = items;
    }

    fn visit_item_enum_mut(&mut self, node: &mut syn::ItemEnum) {
        let mut variants = std::mem::take(&mut node.variants);
        visit_mut::visit_item_enum_mut(self, node);
        self.retain_punctuated(&mut variants, Self::visit_variant_mut);
        node.variants = variants;
    }

    fn visit_fields_named_mut(&mut self, node: &mut syn::FieldsNamed) {
        self.retain_punctuated(&mut node.named, Self::visit_field_mut);
    }

    fn visit_fields_unnamed_mut(&mut self, node: &mut syn::FieldsUnnamed) {
        self.retain_punctuated(&mut node.unnamed, Self::visit_field_mut);
    }

    fn visit_generics_mut(&mut self, node: &mut syn::Generics) {
        let mut params = std::mem::take(&mut node.params);
        visit_mut::visit_generics_mut(self, node);
        self.retain_punctuated(&mut params, Self::visit_generic_param_mut);
        node.params = params;
    }

    fn visit_signature_mut(&mut self, node: &mut syn::Signature) {
        let mut inputs = std::mem::take(&mut node.inputs);
        visit_mut::visit_signature_mut(self, node);
        self.retain_punctuated(&mut inputs, Self::visit_fn_arg_mut);
        node.inputs = inputs;
    }

    fn visit_block_mut(&mut self, node: &mut syn::Block) {
        self.retain(&mut node.stmts, Self::visit_stmt_mut);
    }

    fn visit_expr_match_mut(&mut self, node: &mut syn::ExprMatch) {
        let mut arms = std::mem::take(&mut node.arms);
        visit_mut::visit_expr_match_mut(self, node);
        self.retain(&mut arms, Self::visit_arm_mut);
        node.arms = arms;
    }

    fn visit_expr_struct_mut(&mut self, node: &mut syn::ExprStruct) {
        let mut fields = std::mem::take(&mut node.fields);
        visit_mut::visit_expr_struct_mut(self, node);
        self.retain_punctuated(&mut fields, Self::visit_field_value_mut);
        node.fields = fields;
    }

    fn visit_pat_struct_mut(&mut self, node: &mut syn::PatStruct) {
        let mut fields = std::mem::take(&mut node.fields);
        visit_mut::visit_pat_struct_mut(self, node);
        self.retain_punctuated(&mut fields, Self::visit_field_pat_mut);
        node.fields = fields;
    }

    fn visit_expr_array_mut(&mut self, node: &mut syn::ExprArray) {
        let mut elems = std::mem::take(&mut node.elems);
        visit_mut::visit_expr_array_mut(self, node);
        self.retain_punctuated(&mut elems, Self::visit_expr_mut);
        node.elems = elems;
    }

    fn visit_expr_call_mut(&mut self, node: &mut syn::ExprCall) {
        let mut args = std::mem::take(&mut node.args);
        visit_mut::visit_expr_call_mut(self, node);
        self.retain_punctuated(&mut args, Self::visit_expr_mut);
        node.args = args;
    }

    fn visit_expr_method_call_mut(&mut self, node: &mut syn::ExprMethodCall) {
        let mut args = std::mem::take(&mut node.args);
        visit_mut::visit_expr_method_call_mut(self, node);
        self.retain_punctuated(&mut args, Self::visit_expr_mut);
        node.args = args;
    }

    fn visit_expr_tuple_mut(&mut self, node: &mut syn::ExprTuple) {
        let mut elems = std::mem::take(&mut node.elems);
        visit_mut::visit_expr_tuple_mut(self, node);
        self.retain_punctuated(&mut elems, Self::visit_expr_mut);
        node.elems = elems;
    }

    fn visit_expr_closure_mut(&mut self, node: &mut syn::ExprClosure) {
        let mut inputs = std::mem::take(&mut node.inputs);
        visit_mut::visit_expr_closure_mut(self, node);
        self.retain_punctuated(&mut inputs, Self::visit_pat_mut);
        node.inputs = inputs;
    }
}

fn predicate(input: ParseStream) -> syn::Result<Predicate> {
    if input.peek(syn::LitBool) {
        return input
            .parse::<syn::LitBool>()
            .map(|value| Predicate::Bool(value.value));
    }

    let ident = input.call(syn::Ident::parse_any)?;
    let name = ident.to_string();

    if input.parse::<Token![=]>().is_ok() {
        let value = input.parse::<syn::LitStr>()?.value();
        return Ok(Predicate::Value(name, value, ident.span()));
    }

    if !input.peek(syn::token::Paren) {
        return Ok(Predicate::Name(name, ident.span()));
    }

    let content;
    syn::parenthesized!(content in input);
    let predicates: Vec<_> =
        Punctuated::<_, Token![,]>::parse_terminated_with(&content, predicate)?
            .into_iter()
            .collect();

    match name.as_str() {
        "all" => Ok(Predicate::All(predicates)),
        "any" => Ok(Predicate::Any(predicates)),
        "not" => match <[_; 1]>::try_from(predicates) {
            Ok([predicate]) => Ok(Predicate::Not(Box::new(predicate))),
            Err(_) => Err(syn::Error::new(
                ident.span(),
                "`not` takes exactly one predicate",
            )),
        },
        _ => Err(syn::Error::new(
            ident.span(),
            format!("unknown predicate `{name}`"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The idents of `source` after configuring, attributes shown as
    /// `#path`, with the errors and whether the file holds.
    fn configured(source: &str, cfg: &Cfg) -> (String, Vec<String>, bool) {
        struct Render(Vec<String>);

        impl VisitMut for Render {
            fn visit_attribute_mut(&mut self, attr: &mut Attribute) {
                let path: Vec<_> = attr
                    .path()
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                self.0.push(format!("#{}", path.join("::")));
            }

            fn visit_ident_mut(&mut self, ident: &mut syn::Ident) {
                self.0.push(ident.to_string());
            }
        }

        let mut file = syn::parse_file(source).unwrap();
        let mut errors = Vec::new();
        let holds = configure(cfg, Path::new("x.rs"), &mut file, &mut errors);
        let mut render = Render(Vec::new());
        render.visit_file_mut(&mut file);
        let errors = errors.iter().map(ToString::to_string).collect();
        (render.0.join(" "), errors, holds)
    }

    fn eval() -> Cfg {
        Cfg::Eval(CfgSet::new().name("on").unset("off").feature("f"))
    }

    mod configure {
        use super::*;

        #[test]
        fn configure_all_should_keep_everything_and_apply_every_cfg_attr() {
            let source = "#![cfg(off)]\n#[cfg(off)] fn a() {}\n#[cfg_attr(off, derive(Debug), allow(x))] struct B;";

            let (rendered, errors, holds) = configured(source, &Cfg::All);

            assert_eq!(rendered, "#cfg #cfg a #derive #allow B");
            assert!(errors.is_empty(), "{errors:?}");
            assert!(holds);
        }

        #[test]
        fn configure_eval_should_strip_items_whose_cfg_does_not_hold() {
            let source = "
                #[cfg(on)] fn a() {}
                #[cfg(off)] fn b() {}
                mod m { #[cfg(off)] fn c() {} fn d() {} }
                mod n { #![cfg(off)] fn e() {} }
                mod o;
                impl T { #[cfg(off)] fn f() {} fn g() {} }
                trait U { #[cfg(off)] fn h(); fn i(); }
                extern \"C\" { #[cfg(off)] fn j(); fn k(); }
            ";

            let (rendered, errors, _) = configured(source, &eval());

            assert_eq!(rendered, "#cfg a m d o T g U i k");
            assert!(errors.is_empty(), "{errors:?}");
        }

        #[test]
        fn configure_eval_should_strip_in_every_list_position() {
            let source = "
                enum E<#[cfg(off)] A, B> {
                    #[cfg(off)] V1,
                    V2(#[cfg(off)] u8, u16),
                    V3 { #[cfg(off)] x: u8, y: u16 },
                }
                fn f(#[cfg(off)] a: u8, b: u16) {
                    #[cfg(off)] let c = 1;
                    let d = match e { #[cfg(off)] F => 1, G => 2 };
                    let H { #[cfg(off)] i: _, j: _ } = K { #[cfg(off)] l: 1, m: 2 };
                    n([#[cfg(off)] o, p,], (#[cfg(off)] q, r), [#[cfg(off)] z,]);
                    s.t(#[cfg(off)] u, v);
                    |#[cfg(off)] w, x| ();
                }
            ";

            let (rendered, errors, _) = configured(source, &eval());

            assert_eq!(
                rendered,
                "E B V2 u16 V3 y u16 f b u16 d e G H j K m n p r s t v x"
            );
            assert!(errors.is_empty(), "{errors:?}");
        }

        #[test]
        fn configure_eval_should_evaluate_the_predicates() {
            let source = "
                #[cfg(all(on, feature = \"f\"))] fn a() {}
                #[cfg(all(on, off))] fn b() {}
                #[cfg(any(off, feature = \"f\"))] fn c() {}
                #[cfg(any(off, feature = \"g\"))] fn d() {}
                #[cfg(not(off))] fn e() {}
                #[cfg(true)] fn f() {}
                #[cfg(false)] fn g() {}
                #[cfg(on)] #[cfg(off)] fn h() {}
                #[cfg(all())] fn i() {}
                #[cfg(any())] fn j() {}
                #[cfg(on,)] fn k() {}
            ";

            let (rendered, errors, _) = configured(source, &eval());

            assert_eq!(rendered, "#cfg a #cfg c #cfg e #cfg f #cfg i #cfg k");
            assert!(errors.is_empty(), "{errors:?}");
        }

        #[test]
        fn configure_eval_unknown_names_should_be_false_and_reported() {
            let source = "#[cfg(unix)] fn a() {}
#[cfg(not(target_os = \"linux\"))] fn b() {}
#[cfg(any(on, windows))] fn c() {}
#[cfg(off)] mod d { #[cfg(hidden)] fn e() {} }";

            let (rendered, errors, _) = configured(source, &eval());

            assert_eq!(rendered, "#cfg b #cfg c");
            assert_eq!(
                errors,
                vec![
                    "x.rs:1: unknown cfg `unix`, taken as false",
                    "x.rs:2: unknown cfg `target_os`, taken as false",
                    "x.rs:3: unknown cfg `windows`, taken as false",
                ]
            );
        }

        #[test]
        fn configure_cfg_attr_should_apply_when_its_predicate_holds() {
            let source = "
                #![cfg_attr(on, allow(x))]
                #[cfg_attr(on, derive(A), cfg_attr(feature = \"f\", serde(default)), cfg_attr(off, inner))]
                struct S;
                #[cfg_attr(off, derive(B))] struct T;
                #[cfg_attr(on, cfg(off))] struct U;
                #[cfg_attr(on,)] struct V;
            ";

            let (rendered, errors, _) = configured(source, &eval());

            assert_eq!(rendered, "#allow #derive #serde S T V");
            assert!(errors.is_empty(), "{errors:?}");
        }

        #[test]
        fn configure_invalid_cfg_should_be_reported_and_false() {
            let source = "#[cfg(a(b))] fn a() {}
#[cfg(not(on, off))] fn b() {}
#[cfg] fn c() {}
#[cfg(on = 1)] fn d() {}
#[cfg_attr(on)] fn e() {}
#[cfg_attr(x(y), derive(Debug))] fn f() {}
#[cfg(\"s\")] fn g() {}
#[cfg(all(on off))] fn h() {}
#[cfg_attr(on, 1)] fn i() {}";

            let (rendered, errors, _) = configured(source, &eval());

            assert_eq!(rendered, "e f i");
            assert_eq!(
                errors,
                vec![
                    "x.rs:1:7: unknown predicate `a`",
                    "x.rs:2:7: `not` takes exactly one predicate",
                    "x.rs:3:3: expected attribute arguments in parentheses: #[cfg(...)]",
                    "x.rs:4:12: expected string literal",
                    "x.rs:5:14: expected `,`",
                    "x.rs:6:12: unknown predicate `x`",
                    "x.rs:7:7: expected ident",
                    "x.rs:8:14: expected `,`",
                    "x.rs:9:16: expected identifier",
                ]
            );
        }

        #[test]
        fn configure_file_cfg_should_remove_the_items() {
            let (rendered, errors, holds) = configured("#![cfg(off)]\nfn a() {}", &eval());

            assert_eq!(rendered, "#cfg");
            assert!(errors.is_empty(), "{errors:?}");
            assert!(!holds);
        }
    }
}
