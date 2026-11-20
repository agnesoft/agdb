//! The check of the IR against the source: the item printed from its IR,
//! parsed again, is the item as loaded. Both syntax trees are compared
//! without their spans (so without layout and comments) and normalized
//! where the IR keeps the meaning but not the spelling:
//!
//! - literals are their values: `0x10` is `16`, `r"a"` is `"a"`;
//! - lists (`use` groups too) have no trailing comma but in 1-tuples, nor
//!   do match arms;
//! - macro calls are their arguments as [`SourceCrate::macro_call`] parses
//!   them, in any delimiters.

use super::ir::Item;
use super::rust;
use crate::Error;
use crate::ItemId;
use crate::MacroCall;
use crate::ModuleId;
use crate::SourceCrate;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;

/// How many lines of the syntax trees a mismatch shows around the first
/// difference.
const CONTEXT: usize = 3;

/// Checks the item `id` of `c` against `item`, its IR.
pub fn compare(c: &SourceCrate, id: ItemId, item: &Item) -> Result<(), Error> {
    let original = c.item(id);
    let mismatch = |original, printed| Error::Mismatch {
        path: c.file(c.module(id.module()).file()).path().to_path_buf(),
        line: c.item(id).span().start().line,
        original,
        printed,
    };
    let text = rust::item(item);
    let printed: syn::Item = syn::parse_str(&text).map_err(|e| {
        mismatch(
            String::new(),
            format!("invalid Rust: {e}\n{}", text.trim_end()),
        )
    })?;
    let original = Normalized::new(c, id.module(), original.clone());
    let printed = Normalized::new(c, id.module(), printed);

    if original == printed {
        return Ok(());
    }

    let (original, printed) = difference(&original.dump(), &printed.dump());
    Err(mismatch(original, printed))
}

/// A syntax tree with the macro calls in it, normalized.
#[derive(PartialEq)]
struct Normalized {
    item: syn::Item,
    calls: Vec<Result<MacroCall, String>>,
}

impl Normalized {
    fn new(c: &SourceCrate, module: ModuleId, mut item: syn::Item) -> Self {
        let mut normalize = Normalize {
            c,
            module,
            calls: Vec::new(),
        };
        normalize.visit_item_mut(&mut item);
        Self {
            item,
            calls: normalize.calls,
        }
    }

    /// The trees without the spans, which differ.
    fn dump(&self) -> Vec<String> {
        format!("{:#?}\n{:#?}", self.item, self.calls)
            .lines()
            .filter(|line| !line.trim_start().starts_with("span:"))
            .map(str::to_string)
            .collect()
    }
}

struct Normalize<'a> {
    c: &'a SourceCrate,
    module: ModuleId,
    calls: Vec<Result<MacroCall, String>>,
}

/// Visits removing the trailing punctuation of the field of the node.
macro_rules! untrail {
    ($($visit:ident($node:ty) $field:ident;)*) => {
        $(
            fn $visit(&mut self, node: &mut $node) {
                untrail(&mut node.$field);
                syn::visit_mut::$visit(self, node);
            }
        )*
    };
}

/// [`untrail`] for tuples, a 1-tuple keeps its comma.
macro_rules! untrail_tuple {
    ($($visit:ident($node:ty);)*) => {
        $(
            fn $visit(&mut self, node: &mut $node) {
                if node.elems.len() > 1 {
                    untrail(&mut node.elems);
                }
                syn::visit_mut::$visit(self, node);
            }
        )*
    };
}

impl VisitMut for Normalize<'_> {
    untrail! {
        visit_fields_named_mut(syn::FieldsNamed) named;
        visit_fields_unnamed_mut(syn::FieldsUnnamed) unnamed;
        visit_item_enum_mut(syn::ItemEnum) variants;
        visit_generics_mut(syn::Generics) params;
        visit_where_clause_mut(syn::WhereClause) predicates;
        visit_angle_bracketed_generic_arguments_mut(syn::AngleBracketedGenericArguments) args;
        visit_parenthesized_generic_arguments_mut(syn::ParenthesizedGenericArguments) inputs;
        visit_signature_mut(syn::Signature) inputs;
        visit_type_fn_ptr_mut(syn::TypeFnPtr) inputs;
        visit_expr_array_mut(syn::ExprArray) elems;
        visit_expr_call_mut(syn::ExprCall) args;
        visit_expr_method_call_mut(syn::ExprMethodCall) args;
        visit_expr_closure_mut(syn::ExprClosure) inputs;
        visit_expr_struct_mut(syn::ExprStruct) fields;
        visit_pat_tuple_struct_mut(syn::PatTupleStruct) elems;
        visit_pat_struct_mut(syn::PatStruct) fields;
        visit_use_group_mut(syn::UseGroup) items;
    }

    untrail_tuple! {
        visit_expr_tuple_mut(syn::ExprTuple);
        visit_pat_tuple_mut(syn::PatTuple);
        visit_type_tuple_mut(syn::TypeTuple);
    }

    fn visit_arm_mut(&mut self, arm: &mut syn::Arm) {
        arm.comma = None;
        syn::visit_mut::visit_arm_mut(self, arm);
    }

    fn visit_lit_str_mut(&mut self, lit: &mut syn::LitStr) {
        *lit = syn::LitStr::new(&lit.value(), lit.span());
    }

    fn visit_lit_char_mut(&mut self, lit: &mut syn::LitChar) {
        *lit = syn::LitChar::new(lit.value(), lit.span());
    }

    fn visit_lit_int_mut(&mut self, lit: &mut syn::LitInt) {
        let text = format!("{}{}", lit.base10_digits(), lit.suffix());
        *lit = syn::LitInt::new(&text, lit.span());
    }

    fn visit_lit_float_mut(&mut self, lit: &mut syn::LitFloat) {
        let text = format!("{}{}", lit.base10_digits(), lit.suffix());
        *lit = syn::LitFloat::new(&text, lit.span());
    }

    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        let mut call = self.c.macro_call(self.module, mac);
        call.iter_mut().for_each(|call| call.visit_mut(self));
        self.calls.push(call);
        mac.tokens = TokenStream::new();
        mac.delimiter = syn::MacroDelimiter::Paren(syn::token::Paren::default());
        syn::visit_mut::visit_macro_mut(self, mac);
    }
}

fn untrail<T, P>(list: &mut syn::punctuated::Punctuated<T, P>) {
    list.pop_punct();
}

/// The lines around the first line where `original` and `printed` differ.
fn difference(original: &[String], printed: &[String]) -> (String, String) {
    let first = original
        .iter()
        .zip(printed)
        .take_while(|(a, b)| a == b)
        .count();
    let excerpt = |lines: &[String]| {
        lines[first.saturating_sub(CONTEXT)..(first + CONTEXT).min(lines.len())].join("\n")
    };
    (excerpt(original), excerpt(printed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cfg;
    use crate::Target;
    use crate::TargetKind;
    use crate::codegen::ir::Fields;
    use crate::codegen::ir::Generics;
    use crate::codegen::ir::ItemKind;
    use crate::codegen::ir::Struct;
    use crate::codegen::ir::Vis;
    use agplatform::Fs;
    use agplatform::Platform;
    use agplatform::test_platform::test_platform;

    /// The crate and the IR of all its items.
    fn ir(source: &str) -> (SourceCrate, Vec<(ItemId, Item)>) {
        let platform = test_platform();
        platform.fs().create_dir_all("/c/src").unwrap();
        platform.fs().write("/c/src/lib.rs", source).unwrap();
        let c = SourceCrate::load(
            &platform,
            &Target::new(TargetKind::Lib, "/c/src/lib.rs"),
            &Cfg::All,
        );
        let (items, errors) = c.ir(c.item_ids());
        assert!(errors.is_empty(), "{errors:?}");
        (c, items)
    }

    // WHY: joined paths display with `\` on Windows.
    fn message(error: Error) -> String {
        error.to_string().replace('\\', "/")
    }

    mod compare {
        use super::*;

        #[test]
        fn compare_should_accept_the_items_as_written() {
            let (c, items) = ir(r#"struct S { a: u8, }
struct T(u8,);
enum E { A, B, }
type V = Vec<u8,>;
type F = Box<dyn Fn(u8,) -> u8>;
type P = fn(u8,);
type U = ((u8, u8,), (u8,));
use std::{io, fmt,};
fn f<A, B,>(a: u8, b: u8,) where A: Copy, {
    let x = ([1, 2,], f(1, 2,), a.m(1,), |a, b,| a, S { a: 1, }, (1, 2,), (1,));
    let T(c,) = t;
    let S { a, } = s;
    let (d, e,) = x;
    let (g,) = y;
    let l = (0x10, 1_000u32, 1e3, 2.5_f64, r"a", "\x41", '\u{41}');
    match a {
        1 => {},
        _ => 2,
    }
    vec!(1, 2,);
    println!{"{}", 0x1};
}"#);

            let results: Vec<_> = items
                .iter()
                .map(|(id, item)| compare(&c, *id, item).map_err(message))
                .collect();

            assert_eq!(results, vec![Ok(()); 9]);
        }

        #[test]
        fn compare_should_show_the_lines_around_the_first_difference() {
            let (c, items) = ir("fn a() {}\nfn b() {}");

            let error = compare(&c, items[0].0, &items[1].1);

            assert_eq!(
                error.map_err(message),
                Err("/c/src/lib.rs:1: the printed item differs
--- original
        abi: None,
        fn_token: Token![fn],
        ident: Ident {
            sym: a,
        },
        generics: Generics {
--- printed
        abi: None,
        fn_token: Token![fn],
        ident: Ident {
            sym: b,
        },
        generics: Generics {"
                    .to_string())
            );
        }

        #[test]
        fn compare_should_report_an_item_printed_as_invalid_rust() {
            let (c, items) = ir("struct A;");
            let invalid = Item {
                attrs: Vec::new(),
                vis: Vis::Private,
                kind: ItemKind::Struct(Struct {
                    name: "1x".to_string(),
                    generics: Generics::default(),
                    fields: Fields::Unit,
                }),
            };

            let error = compare(&c, items[0].0, &invalid);

            assert_eq!(
                error.map_err(message),
                Err("/c/src/lib.rs:1: the printed item differs
--- original

--- printed
invalid Rust: expected identifier
struct 1x;"
                    .to_string())
            );
        }
    }
}
