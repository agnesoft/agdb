use std::collections::HashMap;
use std::path::PathBuf;

use agplatform::Fs;
use agplatform::Platform;
use proc_macro2::Delimiter;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use syn::Attribute;
use syn::ImplItem;
use syn::Item;
use syn::ItemFn;
use syn::ItemImpl;
use syn::ItemMod;
use syn::ItemTrait;
use syn::Meta;
use syn::Token;
use syn::TraitItem;
use syn::Type;
use syn::TypeParamBound;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use crate::sources::warn_unreadable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceTypeKind {
    Type,
    Trait,
    Function,
    Macro,
    Method,
    Module(ModuleKind),
    Test,
    /// A contract case: a function of test code named like a test that
    /// the implementers of a trait call from their tests.
    Case,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModuleKind {
    Source,
    Test,
    TestHelper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Comment {
    pub(crate) line: usize,
    pub(crate) text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceType {
    pub(crate) kind: SourceTypeKind,
    pub(crate) name: String,
    pub(crate) line: usize,
    pub(crate) types: Vec<SourceType>,
    pub(crate) comments: Vec<Comment>,
    /// The functions and methods a test calls, by their last path segment.
    pub(crate) calls: Vec<String>,
    /// A test whose body is only WHY comments, explaining why it does not
    /// call its contract case.
    pub(crate) waived: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct SourceTypes {
    pub(crate) types: HashMap<String, Vec<SourceType>>,
    pub(crate) crate_dir: Option<PathBuf>,
    pub(crate) code_after_tests: Vec<usize>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct SourceMap {
    pub(crate) files: HashMap<PathBuf, SourceTypes>,
    pub(crate) errors: HashMap<PathBuf, String>,
}

pub(crate) fn source_map(p: &impl Platform, sources: &[PathBuf]) -> SourceMap {
    let mut map = SourceMap::default();

    for source in sources {
        let crate_dir = source
            .ancestors()
            .skip(1)
            .find(|dir| p.fs().open_read(dir.join("Cargo.toml")).is_ok());
        let integration_test = crate_dir.is_some_and(|dir| source.starts_with(dir.join("tests")));

        match p
            .fs()
            .read_to_string(source)
            .map(|content| parse_source(&content, integration_test))
        {
            Ok(Ok(mut types)) => {
                types.crate_dir = crate_dir.map(PathBuf::from);
                map.files.insert(source.clone(), types);
            }
            Ok(Err(e)) => {
                map.errors.insert(source.clone(), e);
            }
            Err(e) => warn_unreadable(&e),
        }
    }

    map
}

/// A test file, e.g. `#[cfg(test)] mod tests;` or an integration test, is
/// mapped as the body of a `tests` module.
pub(crate) fn parse_source(content: &str, test_file: bool) -> Result<SourceTypes, String> {
    let without_bom = content.strip_prefix('\u{feff}').unwrap_or(content);
    let result = syn::parse_file(content)
        .map_err(|e| parse_error(without_bom, &e))
        .map(|file| {
            // Spans are relative to the content without the BOM and the
            // shebang that syn strips.
            let content = &without_bom[file.shebang.as_ref().map_or(0, String::len)..];
            let test_file = test_file || is_cfg_test(&file.attrs) || is_test_file(&file.items);
            let types = parse_items(&file.items, test_file, content);
            let mut code_after_tests = Vec::new();

            if test_file {
                (vec![test_module("tests", 0, types)], code_after_tests)
            } else {
                find_code_after_tests(&file.items, &mut code_after_tests);
                (types, code_after_tests)
            }
        });

    // With span-locations every parsed source is otherwise kept in a
    // thread local store until the thread exits.
    proc_macro2::extra::invalidate_current_thread_spans();

    let (mut types, code_after_tests) = result?;
    sort(&mut types);

    let mut source_types = SourceTypes {
        code_after_tests,
        ..SourceTypes::default()
    };

    for source_type in types {
        source_types
            .types
            .entry(source_type.name.clone())
            .or_default()
            .push(source_type);
    }

    Ok(source_types)
}

fn parse_error(content: &str, e: &syn::Error) -> String {
    let span = e.span();

    // Errors at the end of input carry an empty call site span.
    let (line, column) = if span.byte_range().is_empty() {
        let last_line = content.rsplit('\n').next().unwrap_or_default();
        (content.matches('\n').count() + 1, last_line.chars().count())
    } else {
        (span.start().line, span.start().column)
    };

    format!("{line}:{}: {e}", column + 1)
}

fn parse_items(items: &[Item], in_test: bool, content: &str) -> Vec<SourceType> {
    let mut types = Vec::new();
    let mut impls = Vec::new();

    for item in items {
        let source_type = match item {
            Item::Fn(i) if is_test_fn(i, in_test) => {
                let mut test = SourceType::new(SourceTypeKind::Test, &i.sig.ident);
                test.comments = test_comments(i, content);
                test.calls = test_calls(i, content);
                test.waived = is_waiver(i, &test.comments);
                test
            }
            Item::Mod(
                i @ ItemMod {
                    content: Some((_, items)),
                    ..
                },
            ) => module_type(&i.ident, items, in_test || is_cfg_test(&i.attrs), content),
            Item::Fn(i) if in_test && is_case_name(&name(&i.sig.ident)) => {
                let mut case = SourceType::new(SourceTypeKind::Case, &i.sig.ident);
                case.comments = test_comments(i, content);
                case
            }
            _ if in_test || is_cfg_test(item_attrs(item)) => continue,
            Item::Enum(i) => SourceType::new(SourceTypeKind::Type, &i.ident),
            Item::Struct(i) => SourceType::new(SourceTypeKind::Type, &i.ident),
            Item::Union(i) => SourceType::new(SourceTypeKind::Type, &i.ident),
            Item::Trait(i) => trait_type(i),
            Item::Fn(i) => SourceType::new(SourceTypeKind::Function, &i.sig.ident),
            Item::Macro(syn::ItemMacro {
                ident: Some(ident), ..
            }) => SourceType::new(SourceTypeKind::Macro, ident),
            Item::Impl(i) => {
                impls.push(i);
                continue;
            }
            _ => continue,
        };

        types.push(source_type);
    }

    // Trait impls are grouped per target by the trait path without generics:
    // `From<A>` and `From<B>` share a module, `fmt::Write` and `io::Write` do
    // not.
    let mut trait_impls: Vec<(ImplTarget, SourceType)> = Vec::new();
    let defined: Vec<String> = types
        .iter()
        .filter(|t| matches!(t.kind, SourceTypeKind::Type | SourceTypeKind::Trait))
        .map(|t| t.name.clone())
        .collect();

    for i in impls {
        let items: Vec<SourceType> = impl_types(i).collect();

        let Some(target) = impl_target(i) else {
            continue;
        };

        // A trait implemented for a type defined elsewhere (`impl MyTrait for
        // Vec<u8>`) is tested under the trait, like a blanket impl.
        let target = match target.trait_path {
            Some(path)
                if target.kind == SourceTypeKind::Type
                    && !defined.contains(&name(target.ident)) =>
            {
                ImplTarget {
                    ident: last_segment(path),
                    kind: SourceTypeKind::Trait,
                    trait_path: None,
                }
            }
            _ => target,
        };

        match target.trait_path {
            Some(path) => {
                let trait_name = path_string(path);

                match trait_impls.iter_mut().find(|(t, trait_type)| {
                    t.ident == target.ident && trait_type.name == trait_name
                }) {
                    Some((_, trait_type)) => trait_type.types.extend(items),
                    None => {
                        let mut trait_type =
                            SourceType::new(SourceTypeKind::Trait, last_segment(path));
                        trait_type.name = trait_name;
                        trait_type.types = items;
                        trait_impls.push((target, trait_type));
                    }
                }
            }
            None => target.find_or_insert(&mut types).types.extend(items),
        }
    }

    for (target, trait_type) in trait_impls {
        target.find_or_insert(&mut types).types.push(trait_type);
    }

    types
}

/// Required methods are kept for the contract cases of the trait. Impls
/// always override them so they never count as defaults.
fn trait_type(t: &ItemTrait) -> SourceType {
    let mut source_type = SourceType::new(SourceTypeKind::Trait, &t.ident);

    source_type
        .types
        .extend(t.items.iter().filter_map(|item| match item {
            TraitItem::Fn(f) if !is_cfg_test(&f.attrs) => {
                Some(SourceType::new(SourceTypeKind::Method, &f.sig.ident))
            }
            TraitItem::Type(t) if t.default.is_some() && !is_cfg_test(&t.attrs) => {
                Some(SourceType::new(SourceTypeKind::Type, &t.ident))
            }
            _ => None,
        }));

    source_type
}

fn impl_types(i: &ItemImpl) -> impl Iterator<Item = SourceType> + '_ {
    i.items.iter().filter_map(|item| match item {
        ImplItem::Fn(f) if !is_cfg_test(&f.attrs) => {
            Some(SourceType::new(SourceTypeKind::Method, &f.sig.ident))
        }
        ImplItem::Type(t) if !is_cfg_test(&t.attrs) => {
            Some(SourceType::new(SourceTypeKind::Type, &t.ident))
        }
        _ => None,
    })
}

fn module_type(ident: &syn::Ident, items: &[Item], in_test: bool, content: &str) -> SourceType {
    let in_test = in_test
        || items
            .iter()
            .any(|item| matches!(item, Item::Fn(f) if is_test_fn(f, false)));
    let types = parse_items(items, in_test, content);

    if in_test {
        return test_module(&name(ident), line(ident), types);
    }

    let mut source_type = SourceType::new(SourceTypeKind::Module(ModuleKind::Source), ident);
    source_type.types = types;
    source_type
}

fn test_module(name: &str, line: usize, types: Vec<SourceType>) -> SourceType {
    let kind = if types.iter().any(SourceType::has_tests) {
        ModuleKind::Test
    } else {
        ModuleKind::TestHelper
    };

    SourceType {
        kind: SourceTypeKind::Module(kind),
        name: name.to_string(),
        line,
        types,
        comments: Vec::new(),
        calls: Vec::new(),
        waived: false,
    }
}

/// A file with tests outside of `#[cfg(test)]` modules and without its own
/// `#[cfg(test)]` test module is the body of a test module declared
/// elsewhere. Any attribute ending in `test` counts as nothing marks the file
/// as test code.
fn is_test_file(items: &[Item]) -> bool {
    let has_test_module = items.iter().any(|item| match item {
        Item::Mod(ItemMod {
            attrs,
            content: Some((_, items)),
            ..
        }) => is_cfg_test(attrs) && items.iter().any(has_tests),
        _ => false,
    });

    !has_test_module && items.iter().any(has_ungated_tests)
}

/// Test code is gated by `#[cfg(test)]`, a `#[test]` or a module holding
/// `#[test]` functions. Only test code holding tests starts the tests so
/// test helpers may come before production code.
fn find_code_after_tests(items: &[Item], lines: &mut Vec<usize>) {
    let mut after_tests = false;

    for item in items {
        let is_test = is_cfg_test(item_attrs(item))
            || match item {
                Item::Fn(f) => is_test_fn(f, false),
                Item::Mod(ItemMod {
                    content: Some((_, items)),
                    ..
                }) => items
                    .iter()
                    .any(|item| matches!(item, Item::Fn(f) if is_test_fn(f, false))),
                _ => false,
            };

        if is_test {
            after_tests |= has_tests(item);
        } else if after_tests {
            lines.push(item.span().start().line);
            return;
        } else if let Item::Mod(ItemMod {
            content: Some((_, items)),
            ..
        }) = item
        {
            find_code_after_tests(items, lines);
        }
    }
}

fn has_tests(item: &Item) -> bool {
    match item {
        Item::Fn(f) => is_test_fn(f, true),
        Item::Mod(ItemMod {
            content: Some((_, items)),
            ..
        }) => items.iter().any(has_tests),
        _ => false,
    }
}

fn has_ungated_tests(item: &Item) -> bool {
    match item {
        Item::Fn(f) => is_test_fn(f, true),
        Item::Mod(ItemMod {
            attrs,
            content: Some((_, items)),
            ..
        }) => !is_cfg_test(attrs) && items.iter().any(has_ungated_tests),
        _ => false,
    }
}

fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(i) => &i.attrs,
        Item::Enum(i) => &i.attrs,
        Item::ExternCrate(i) => &i.attrs,
        Item::Fn(i) => &i.attrs,
        Item::ForeignMod(i) => &i.attrs,
        Item::Impl(i) => &i.attrs,
        Item::Macro(i) => &i.attrs,
        Item::Mod(i) => &i.attrs,
        Item::Static(i) => &i.attrs,
        Item::Struct(i) => &i.attrs,
        Item::Trait(i) => &i.attrs,
        Item::TraitAlias(i) => &i.attrs,
        Item::Type(i) => &i.attrs,
        Item::Union(i) => &i.attrs,
        Item::Use(i) => &i.attrs,
        _ => &[],
    }
}

/// The tokenizer drops comments so they are what is left between the tokens
/// of the body, apart from doc comments that become `#[doc = "..."]` with
/// every token spanning the comment.
fn test_comments(f: &ItemFn, content: &str) -> Vec<Comment> {
    let braces = f.block.brace_token.span;
    let body = body(f, content);
    let mut scan = BodyScan {
        body,
        end: 0,
        end_line: 1,
        comments: Vec::new(),
    };
    scan.tokens(body.parse().expect("syn already tokenized this body"));
    scan.gap(body.len());

    // Lines of the body are relative to the line of its opening brace.
    let first_line = braces.open().end().line - 1;

    merge_line_comments(body, scan.comments)
        .into_iter()
        .filter_map(|c| {
            comment_text(&body[c.start..c.end]).map(|(skipped, text)| Comment {
                line: first_line + c.line + skipped,
                text,
            })
        })
        .collect()
}

fn body<'a>(f: &ItemFn, content: &'a str) -> &'a str {
    let braces = f.block.brace_token.span;
    &content[braces.open().byte_range().end..braces.close().byte_range().start]
}

/// Scanning the tokens rather than the syntax tree finds the calls inside
/// macros such as `assert!(case(..))` too.
fn test_calls(f: &ItemFn, content: &str) -> Vec<String> {
    let mut calls = Vec::new();
    scan_calls(
        body(f, content)
            .parse()
            .expect("syn already tokenized this body"),
        &mut calls,
    );
    calls
}

fn scan_calls(stream: TokenStream, calls: &mut Vec<String>) {
    let mut tokens = stream.into_iter().peekable();

    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Ident(ident) if matches!(tokens.peek(), Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis) =>
            {
                calls.push(name(&ident));
            }
            TokenTree::Group(g) => scan_calls(g.stream(), calls),
            _ => {}
        }
    }
}

/// A test without code whose `comments` are all WHY, starting with it and
/// starting no other step, e.g. one of an implementer that cannot support
/// its contract case. Unlike an ignored test it runs, so it does not leave
/// its body uncovered.
fn is_waiver(f: &ItemFn, comments: &[Comment]) -> bool {
    fn first_word(line: &str) -> &str {
        line.split(|c: char| !c.is_ascii_alphanumeric())
            .next()
            .unwrap_or_default()
    }

    f.block.stmts.is_empty()
        && !comments.is_empty()
        && comments.iter().all(|c| {
            first_word(&c.text) == "WHY"
                && c.text
                    .lines()
                    .all(|line| !["GIVEN", "WHEN", "THEN"].contains(&first_word(line)))
        })
}

fn is_case_name(name: &str) -> bool {
    name.split_once("_should_")
        .is_some_and(|(subject, behaviour)| !subject.is_empty() && !behaviour.is_empty())
}

struct RawComment {
    start: usize,
    end: usize,
    line: usize,
}

struct BodyScan<'a> {
    body: &'a str,
    end: usize,
    end_line: usize,
    comments: Vec<RawComment>,
}

impl BodyScan<'_> {
    fn tokens(&mut self, stream: TokenStream) {
        for token in stream {
            match token {
                TokenTree::Group(g) => {
                    self.token(g.span_open(), false);
                    self.tokens(g.stream());
                    self.token(g.span_close(), false);
                }
                TokenTree::Punct(p) => self.token(p.span(), p.as_char() == '#'),
                t => self.token(t.span(), false),
            }
        }
    }

    fn token(&mut self, span: Span, pound: bool) {
        let range = span.byte_range();

        if range.start < self.end {
            return;
        }

        self.gap(range.start);

        if pound && self.body[range.clone()].starts_with('/') {
            self.comments.push(RawComment {
                start: range.start,
                end: range.end,
                line: span.start().line,
            });
        }

        self.end = range.end;
        self.end_line = span.end().line;
    }

    fn gap(&mut self, until: usize) {
        let gap = &self.body[self.end..until];
        let mut line = self.end_line;
        let mut i = 0;

        while let Some(pos) = gap[i..].find(|c: char| !c.is_whitespace()) {
            let start = i + pos;
            let len = if gap[start..].starts_with("/*") {
                block_comment_len(&gap[start..])
            } else {
                gap[start..]
                    .find('\n')
                    .expect("line comment should end before the next token")
            };

            line += gap[i..start].matches('\n').count();
            self.comments.push(RawComment {
                start: self.end + start,
                end: self.end + start + len,
                line,
            });
            line += gap[start..start + len].matches('\n').count();
            i = start + len;
        }
    }
}

fn block_comment_len(text: &str) -> usize {
    let mut depth = 0;
    let mut skip = false;

    text.as_bytes()
        .windows(2)
        .position(|pair| {
            if std::mem::take(&mut skip) {
                return false;
            }

            match pair {
                b"/*" => depth += 1,
                b"*/" => depth -= 1,
                _ => return false,
            }

            skip = true;
            depth == 0
        })
        .expect("block comment should be terminated")
        + 2
}

fn merge_line_comments(body: &str, comments: Vec<RawComment>) -> Vec<RawComment> {
    let mut merged: Vec<RawComment> = Vec::new();

    for c in comments {
        match merged.last_mut() {
            Some(last)
                if body[last.start..].starts_with("//")
                    && body[..last.start]
                        .rfind('\n')
                        .is_some_and(|i| body[i + 1..last.start].trim().is_empty())
                    && body[c.start..].starts_with("//")
                    && body[last.end..c.start].trim().is_empty()
                    && body[last.end..c.start].matches('\n').count() == 1 =>
            {
                last.end = c.end;
            }
            _ => merged.push(c),
        }
    }

    merged
}

/// Empty lines inside the comment are kept so that each line of the text
/// stays on its source line. The number of skipped leading empty lines is
/// returned with the text.
fn comment_text(comment: &str) -> Option<(usize, String)> {
    let comment = comment.strip_prefix("/*").map_or(comment, |block| {
        block
            .strip_suffix("*/")
            .expect("block comment should be terminated")
    });
    let lines: Vec<&str> = comment
        .lines()
        .map(|l| l.trim().trim_start_matches(['/', '*', '!']).trim())
        .collect();
    let first = lines.iter().position(|l| !l.is_empty())?;
    let last = lines.iter().rposition(|l| !l.is_empty()).unwrap_or(first);

    Some((first, lines[first..=last].join("\n")))
}

fn sort(types: &mut [SourceType]) {
    types.sort_by_key(|t| t.line);

    for source_type in types {
        sort(&mut source_type.types);
    }
}

impl SourceType {
    fn has_tests(&self) -> bool {
        matches!(
            self.kind,
            SourceTypeKind::Test | SourceTypeKind::Case | SourceTypeKind::Module(ModuleKind::Test)
        )
    }

    fn new(kind: SourceTypeKind, ident: &syn::Ident) -> Self {
        Self {
            kind,
            name: name(ident),
            line: line(ident),
            types: Vec::new(),
            comments: Vec::new(),
            calls: Vec::new(),
            waived: false,
        }
    }
}

/// The type or trait an impl's items belong under, the trait for blanket
/// impls.
struct ImplTarget<'a> {
    ident: &'a syn::Ident,
    kind: SourceTypeKind,
    /// The implemented trait when its items nest under the target.
    trait_path: Option<&'a syn::Path>,
}

impl ImplTarget<'_> {
    /// Impls of types defined elsewhere (e.g. in another file) have no
    /// definition so the first impl stands in for it.
    fn find_or_insert<'t>(&self, types: &'t mut Vec<SourceType>) -> &'t mut SourceType {
        let name = name(self.ident);
        let index = match types.iter().position(|t| {
            matches!(t.kind, SourceTypeKind::Type | SourceTypeKind::Trait) && t.name == name
        }) {
            Some(index) => index,
            None => {
                types.push(SourceType::new(self.kind.clone(), self.ident));
                types.len() - 1
            }
        };

        &mut types[index]
    }
}

/// Impls for a named type or `dyn Trait` belong under it. Blanket impls
/// (`impl<T: Bound> Trait for T`) and trait impls for types without a name
/// (slices, tuples etc.) belong under the trait itself.
fn impl_target(i: &ItemImpl) -> Option<ImplTarget<'_>> {
    let trait_path = i.trait_.as_ref().map(|(path, _)| path);

    match self_type(&i.self_ty) {
        Some((ident, kind)) if !i.generics.type_params().any(|p| p.ident == *ident) => {
            Some(ImplTarget {
                ident,
                kind,
                trait_path,
            })
        }
        _ => trait_path.map(|path| ImplTarget {
            ident: last_segment(path),
            kind: SourceTypeKind::Trait,
            trait_path: None,
        }),
    }
}

fn self_type(ty: &Type) -> Option<(&syn::Ident, SourceTypeKind)> {
    match ty {
        Type::Path(p) if p.qself.is_none() => p
            .path
            .segments
            .last()
            .map(|s| (&s.ident, SourceTypeKind::Type)),
        Type::TraitObject(t) => t.bounds.iter().find_map(|b| match b {
            TypeParamBound::Trait(t) => Some((last_segment(&t.path), SourceTypeKind::Trait)),
            _ => None,
        }),
        Type::Reference(r) => self_type(&r.elem),
        Type::Paren(p) => self_type(&p.elem),
        _ => None,
    }
}

fn last_segment(path: &syn::Path) -> &syn::Ident {
    &path
        .segments
        .last()
        .expect("path should have at least one segment")
        .ident
}

fn path_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| name(&s.ident))
        .collect::<Vec<_>>()
        .join("::")
}

fn is_test_fn(f: &ItemFn, in_test: bool) -> bool {
    let in_test = in_test || is_cfg_test(&f.attrs);

    f.attrs.iter().any(|a| {
        let path = a.path();
        path.is_ident("test")
            || (in_test && path.segments.last().is_some_and(|s| s.ident == "test"))
    })
}

fn is_cfg_test(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|a| a.path().is_ident("cfg") && a.parse_args::<Meta>().is_ok_and(|m| implies_test(&m)))
}

/// Whether the cfg predicate can only hold in test builds, e.g. `test` or
/// `all(test, unix)` but not `any(test, feature = "x")` or `not(test)`.
fn implies_test(meta: &Meta) -> bool {
    match meta {
        Meta::Path(p) => p.is_ident("test"),
        Meta::List(l) if l.path.is_ident("all") || l.path.is_ident("any") => l
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .is_ok_and(|predicates| {
                if l.path.is_ident("all") {
                    predicates.iter().any(implies_test)
                } else {
                    predicates.iter().all(implies_test)
                }
            }),
        _ => false,
    }
}

fn name(ident: &syn::Ident) -> String {
    ident.unraw().to_string()
}

fn line(ident: &syn::Ident) -> usize {
    ident.span().start().line
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::test_platform::test_platform;

    fn ty(kind: SourceTypeKind, name: &str, line: usize, types: Vec<SourceType>) -> SourceType {
        SourceType {
            kind,
            name: name.to_string(),
            line,
            types,
            comments: Vec::new(),
            calls: Vec::new(),
            waived: false,
        }
    }

    const MODULE: SourceTypeKind = SourceTypeKind::Module(ModuleKind::Source);
    const TEST_MODULE: SourceTypeKind = SourceTypeKind::Module(ModuleKind::Test);
    const HELPER_MODULE: SourceTypeKind = SourceTypeKind::Module(ModuleKind::TestHelper);

    fn comments_of_t(source: &str) -> Vec<Comment> {
        fn find(types: &[SourceType]) -> Option<Vec<Comment>> {
            types.iter().find_map(|t| {
                if t.kind == SourceTypeKind::Test && t.name == "t" {
                    Some(t.comments.clone())
                } else {
                    find(&t.types)
                }
            })
        }

        let types: Vec<SourceType> = parse_source(source, false)
            .unwrap()
            .types
            .into_values()
            .flatten()
            .collect();
        find(&types).expect("source should have test t")
    }

    fn comment(line: usize, text: &str) -> Comment {
        Comment {
            line,
            text: text.to_string(),
        }
    }

    fn leaf(kind: SourceTypeKind, name: &str, line: usize) -> SourceType {
        ty(kind, name, line, Vec::new())
    }

    fn types(types: Vec<SourceType>) -> SourceTypes {
        let mut source_types = SourceTypes::default();

        for t in types {
            source_types
                .types
                .entry(t.name.clone())
                .or_default()
                .push(t);
        }

        source_types
    }

    mod source_map {
        use super::*;

        #[test]
        fn source_map_should_map_each_source() {
            let platform = test_platform();
            platform.fs().create_dir_all("src").unwrap();
            platform.fs().write("src/main.rs", b"fn main() {}").unwrap();
            platform.fs().write("src/lib.rs", b"struct S;").unwrap();

            let map = source_map(
                &platform,
                &[PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")],
            );

            assert_eq!(map.errors, HashMap::new());
            assert_eq!(
                map.files,
                HashMap::from([
                    (
                        PathBuf::from("src/main.rs"),
                        types(vec![leaf(SourceTypeKind::Function, "main", 1)])
                    ),
                    (
                        PathBuf::from("src/lib.rs"),
                        types(vec![leaf(SourceTypeKind::Type, "S", 1)])
                    ),
                ])
            );
        }

        #[test]
        fn source_map_should_map_crate_dir_and_integration_tests() {
            let platform = test_platform();

            for dir in ["a/src", "a/tests/common", "b/tests"] {
                platform.fs().create_dir_all(dir).unwrap();
            }

            for (file, content) in [
                ("a/Cargo.toml", ""),
                ("a/src/lib.rs", "struct S;"),
                ("a/tests/common/mod.rs", "\npub fn helper() {}"),
                ("b/tests/x.rs", "fn x() {}"),
            ] {
                platform.fs().write(file, content.as_bytes()).unwrap();
            }

            let map = source_map(
                &platform,
                &[
                    PathBuf::from("a/src/lib.rs"),
                    PathBuf::from("a/tests/common/mod.rs"),
                    PathBuf::from("b/tests/x.rs"),
                ],
            );

            let in_crate = |types: SourceTypes| SourceTypes {
                crate_dir: Some(PathBuf::from("a")),
                ..types
            };
            assert_eq!(map.errors, HashMap::new());
            assert_eq!(
                map.files,
                HashMap::from([
                    (
                        PathBuf::from("a/src/lib.rs"),
                        in_crate(types(vec![leaf(SourceTypeKind::Type, "S", 1)]))
                    ),
                    (
                        PathBuf::from("a/tests/common/mod.rs"),
                        in_crate(types(vec![leaf(HELPER_MODULE, "tests", 0)]))
                    ),
                    (
                        PathBuf::from("b/tests/x.rs"),
                        types(vec![leaf(SourceTypeKind::Function, "x", 1)])
                    ),
                ])
            );
        }

        #[test]
        fn source_map_should_record_unparsable_and_skip_unreadable_sources() {
            let platform = test_platform();
            platform.fs().write("invalid.rs", b"fn () {}").unwrap();

            let map = source_map(
                &platform,
                &[PathBuf::from("missing.rs"), PathBuf::from("invalid.rs")],
            );

            assert_eq!(map.files, HashMap::new());
            assert_eq!(
                map.errors,
                HashMap::from([(
                    PathBuf::from("invalid.rs"),
                    "1:4: expected identifier".to_string()
                )])
            );
        }
    }

    mod parse_source {
        use super::*;

        #[test]
        fn parse_source_should_map_types_traits_and_functions() {
            let source = "\
enum MyEnum { A }
struct MyStruct;
union MyUnion { a: u32 }
trait MyTrait {}
fn my_fn() {}
fn r#type() {}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    leaf(SourceTypeKind::Type, "MyEnum", 1),
                    leaf(SourceTypeKind::Type, "MyStruct", 2),
                    leaf(SourceTypeKind::Type, "MyUnion", 3),
                    leaf(SourceTypeKind::Trait, "MyTrait", 4),
                    leaf(SourceTypeKind::Function, "my_fn", 5),
                    leaf(SourceTypeKind::Function, "type", 6),
                ])
            );
        }

        #[test]
        fn parse_source_should_nest_impl_items_under_type() {
            let source = "\
impl Cache {
    fn get(&self) {}
}
struct Cache;
impl Deref for Cache {
    type Target = u32;
    fn deref(&self) -> &u32 { &0 }
}
impl<T> Wrapper<T> {
    fn new() {}
}
impl Display for &Cache {
    fn fmt(&self) {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    ty(
                        SourceTypeKind::Type,
                        "Cache",
                        4,
                        vec![
                            leaf(SourceTypeKind::Method, "get", 2),
                            ty(
                                SourceTypeKind::Trait,
                                "Deref",
                                5,
                                vec![
                                    leaf(SourceTypeKind::Type, "Target", 6),
                                    leaf(SourceTypeKind::Method, "deref", 7),
                                ]
                            ),
                            ty(
                                SourceTypeKind::Trait,
                                "Display",
                                12,
                                vec![leaf(SourceTypeKind::Method, "fmt", 13)]
                            ),
                        ]
                    ),
                    ty(
                        SourceTypeKind::Type,
                        "Wrapper",
                        9,
                        vec![leaf(SourceTypeKind::Method, "new", 10)]
                    ),
                ])
            );
        }

        #[test]
        fn parse_source_should_nest_trait_impl_items_under_trait() {
            let source = "\
struct S;
impl S {
    fn fmt(&self) {}
}
impl Display for S {
    fn fmt(&self) {}
}
impl From<A> for S {
    fn from(a: A) -> Self { S }
}
impl From<B> for S {
    fn from(b: B) -> Self { S }
}
impl fmt::Write for S {
    fn write_str(&mut self) {}
}
impl io::Write for S {
    fn write(&mut self) {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![ty(
                    SourceTypeKind::Type,
                    "S",
                    1,
                    vec![
                        leaf(SourceTypeKind::Method, "fmt", 3),
                        ty(
                            SourceTypeKind::Trait,
                            "Display",
                            5,
                            vec![leaf(SourceTypeKind::Method, "fmt", 6)]
                        ),
                        ty(
                            SourceTypeKind::Trait,
                            "From",
                            8,
                            vec![
                                leaf(SourceTypeKind::Method, "from", 9),
                                leaf(SourceTypeKind::Method, "from", 12),
                            ]
                        ),
                        ty(
                            SourceTypeKind::Trait,
                            "fmt::Write",
                            14,
                            vec![leaf(SourceTypeKind::Method, "write_str", 15)]
                        ),
                        ty(
                            SourceTypeKind::Trait,
                            "io::Write",
                            17,
                            vec![leaf(SourceTypeKind::Method, "write", 18)]
                        ),
                    ]
                )])
            );
        }

        #[test]
        fn parse_source_should_nest_methods_defaults_and_blanket_items_under_trait() {
            let source = "\
trait Fs {
    type In;
    type Out = u8;
    fn read(&self);
    fn read_to_string(&self) {}
}
impl<T: Read> Fs for T {
    type In = u8;
    fn read(&self) {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![ty(
                    SourceTypeKind::Trait,
                    "Fs",
                    1,
                    vec![
                        leaf(SourceTypeKind::Type, "Out", 3),
                        leaf(SourceTypeKind::Method, "read", 4),
                        leaf(SourceTypeKind::Method, "read_to_string", 5),
                        leaf(SourceTypeKind::Type, "In", 8),
                        leaf(SourceTypeKind::Method, "read", 9),
                    ]
                )])
            );
        }

        #[test]
        fn parse_source_should_map_blanket_impl_of_external_trait_as_trait() {
            let source = "\
impl<T: Display> MyExt for T {
    fn ext(&self) {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![ty(
                    SourceTypeKind::Trait,
                    "MyExt",
                    1,
                    vec![leaf(SourceTypeKind::Method, "ext", 2)]
                )])
            );
        }

        #[test]
        fn parse_source_should_map_impls_for_unnamed_types_under_trait() {
            let source = "\
impl MyTrait for [u8] {
    fn slice(&self) {}
}
impl MyTrait for (A, B) {
    fn tuple(&self) {}
}
impl MyTrait for <S as T>::Assoc {
    fn assoc(&self) {}
}
impl dyn MyTrait {
    fn object(&self) {}
}
impl Display for dyn 'static + MyTrait + Send {
    fn fmt(&self) {}
}
impl [u8] {
    fn inherent(&self) {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![ty(
                    SourceTypeKind::Trait,
                    "MyTrait",
                    1,
                    vec![
                        leaf(SourceTypeKind::Method, "slice", 2),
                        leaf(SourceTypeKind::Method, "tuple", 5),
                        leaf(SourceTypeKind::Method, "assoc", 8),
                        leaf(SourceTypeKind::Method, "object", 11),
                        ty(
                            SourceTypeKind::Trait,
                            "Display",
                            13,
                            vec![leaf(SourceTypeKind::Method, "fmt", 14)]
                        ),
                    ]
                )])
            );
        }

        #[test]
        fn parse_source_should_map_trait_impls_for_undefined_types_under_trait() {
            let source = "\
impl MyTrait for Vec<u8> {
    fn m(&self) {}
}
impl MyTrait for String {
    fn m(&self) {}
}
impl std::fmt::Display for External {
    fn fmt(&self) {}
}
impl External {
    fn run() {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    ty(
                        SourceTypeKind::Trait,
                        "MyTrait",
                        1,
                        vec![
                            leaf(SourceTypeKind::Method, "m", 2),
                            leaf(SourceTypeKind::Method, "m", 5),
                        ]
                    ),
                    ty(
                        SourceTypeKind::Trait,
                        "Display",
                        7,
                        vec![leaf(SourceTypeKind::Method, "fmt", 8)]
                    ),
                    ty(
                        SourceTypeKind::Type,
                        "External",
                        10,
                        vec![leaf(SourceTypeKind::Method, "run", 11)]
                    ),
                ])
            );
        }

        #[test]
        fn parse_source_should_map_impls_without_items() {
            let source = "\
impl Error for MyError {}
unsafe impl Send for MyError {}
impl Y {}
impl<T> Marker for T {}
impl Z {
    const C: u32 = 0;
}
struct MyError;
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    ty(
                        SourceTypeKind::Type,
                        "MyError",
                        8,
                        vec![
                            leaf(SourceTypeKind::Trait, "Error", 1),
                            leaf(SourceTypeKind::Trait, "Send", 2),
                        ]
                    ),
                    leaf(SourceTypeKind::Type, "Y", 3),
                    leaf(SourceTypeKind::Trait, "Marker", 4),
                    leaf(SourceTypeKind::Type, "Z", 5),
                ])
            );
        }

        #[test]
        fn parse_source_should_keep_items_of_same_name() {
            let source = "\
fn foo() {}
mod foo {}
struct Foo;
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    leaf(SourceTypeKind::Function, "foo", 1),
                    leaf(MODULE, "foo", 2),
                    leaf(SourceTypeKind::Type, "Foo", 3),
                ])
            );
        }

        #[test]
        fn parse_source_should_attach_impl_to_type_not_function_of_same_name() {
            let source = "\
fn bar() {}
struct bar;
impl bar {
    fn run() {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    leaf(SourceTypeKind::Function, "bar", 1),
                    ty(
                        SourceTypeKind::Type,
                        "bar",
                        2,
                        vec![leaf(SourceTypeKind::Method, "run", 4)]
                    ),
                ])
            );
        }

        #[test]
        fn parse_source_should_nest_inline_modules() {
            let source = "\
mod outer {
    struct Inner;
    fn inner() {}
    mod nested {
        fn deep() {}
    }
}
mod empty {}
mod external;
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    ty(
                        MODULE,
                        "outer",
                        1,
                        vec![
                            leaf(SourceTypeKind::Type, "Inner", 2),
                            leaf(SourceTypeKind::Function, "inner", 3),
                            ty(
                                MODULE,
                                "nested",
                                4,
                                vec![leaf(SourceTypeKind::Function, "deep", 5)]
                            ),
                        ]
                    ),
                    leaf(MODULE, "empty", 8),
                ])
            );
        }

        #[test]
        fn parse_source_should_map_macro_rules() {
            let source = "\
macro_rules! my_macro { () => {} }
#[macro_export]
macro_rules! exported { () => {} }
#[cfg(test)]
macro_rules! test_only { () => {} }
println!(\"not a definition\");
macro m2() {}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    leaf(SourceTypeKind::Macro, "my_macro", 1),
                    leaf(SourceTypeKind::Macro, "exported", 3),
                ])
            );
        }

        #[test]
        fn parse_source_should_ignore_test_and_unmapped_items() {
            let source = "\
#[cfg(test)]
fn test_helper() {}
#[cfg(test)]
struct TestOnly;
#[cfg(test)]
impl S {
    fn test_impl() {}
}
#[cfg(all(test, unix))]
fn test_unix() {}
#[cfg(any(test, all(test, feature = \"x\")))]
fn test_any() {}
#[cfg(any())]
fn never() {}
#[cfg(all(test,))]
struct TrailingComma;
struct S;
impl S {
    #[cfg(test)]
    fn test_helper() {}
    #[cfg(test)]
    type TestType = u8;
    const C: u32 = 0;
}
trait T {
    #[cfg(test)]
    fn test_default() {}
    #[cfg(test)]
    type TestType = u8;
    const C: u32;
}
const C: u32 = 0;
static ST: u32 = 0;
type Alias = S;
use std::path::Path;
extern crate alloc;
extern \"C\" {}
trait Alias = T;
macro m2() {}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    leaf(SourceTypeKind::Type, "S", 17),
                    leaf(SourceTypeKind::Trait, "T", 25),
                ])
            );
        }

        #[test]
        fn parse_source_should_keep_items_not_exclusive_to_tests() {
            let source = "\
#[cfg(feature = \"x\")]
fn featured() {}
#[cfg(any(test, feature = \"x\"))]
fn test_or_featured() {}
#[cfg(not(test))]
fn not_test() {}
#[cfg(all(unix, feature = \"x\"))]
fn unix_featured() {}
#[cfg_attr(test, derive(Debug))]
struct Derived;
#[cfg]
fn bare() {}
#[cfg(all(1))]
fn invalid() {}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    leaf(SourceTypeKind::Function, "featured", 2),
                    leaf(SourceTypeKind::Function, "test_or_featured", 4),
                    leaf(SourceTypeKind::Function, "not_test", 6),
                    leaf(SourceTypeKind::Function, "unix_featured", 8),
                    leaf(SourceTypeKind::Type, "Derived", 10),
                    leaf(SourceTypeKind::Function, "bare", 12),
                    leaf(SourceTypeKind::Function, "invalid", 14),
                ])
            );
        }

        #[test]
        fn parse_source_should_map_impl_of_external_type() {
            let source = "\
impl External {
    fn run() {}
}
impl (External) {
    fn walk() {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![ty(
                    SourceTypeKind::Type,
                    "External",
                    1,
                    vec![
                        leaf(SourceTypeKind::Method, "run", 2),
                        leaf(SourceTypeKind::Method, "walk", 5),
                    ]
                )])
            );
        }

        #[test]
        fn parse_source_should_map_test_modules_and_tests() {
            let source = "\
struct S;
#[cfg(test)]
mod tests {
    use super::*;
    fn helper() {}
    struct Mock;
    #[test]
    fn top_should_work() {}
    mod s {
        #[test]
        fn s_should_work() {}
        #[tokio::test]
        async fn s_async_should_work() {}
    }
    mod fixtures {
        pub fn data() {}
    }
    mod outer {
        mod inner {
            #[test]
            fn inner_should_work() {}
        }
    }
    mod external;
    #[my::deep::test]
    fn deep() {}
}
#[cfg(test)]
mod empty {}
#[cfg(test)]
#[test]
fn loose_should_work() {}
#[cfg(all(test, unix))]
mod unix_tests;
#[cfg(test)]
mod test_utils {
    pub fn make() {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    leaf(SourceTypeKind::Type, "S", 1),
                    ty(
                        TEST_MODULE,
                        "tests",
                        3,
                        vec![
                            leaf(SourceTypeKind::Test, "top_should_work", 8),
                            ty(
                                TEST_MODULE,
                                "s",
                                9,
                                vec![
                                    leaf(SourceTypeKind::Test, "s_should_work", 11),
                                    leaf(SourceTypeKind::Test, "s_async_should_work", 13),
                                ]
                            ),
                            leaf(HELPER_MODULE, "fixtures", 15),
                            ty(
                                TEST_MODULE,
                                "outer",
                                18,
                                vec![ty(
                                    TEST_MODULE,
                                    "inner",
                                    19,
                                    vec![leaf(SourceTypeKind::Test, "inner_should_work", 21)]
                                )]
                            ),
                            leaf(SourceTypeKind::Test, "deep", 26),
                        ]
                    ),
                    leaf(HELPER_MODULE, "empty", 29),
                    leaf(SourceTypeKind::Test, "loose_should_work", 32),
                    leaf(HELPER_MODULE, "test_utils", 36),
                ])
            );
        }

        #[test]
        fn parse_source_should_map_contract_cases_in_test_code() {
            let source = "\
struct S;
fn outside_should_work() {}
#[cfg(test)]
mod tests {
    fn helper() {}
    fn s_should_conform() {
        // GIVEN s
    }
    fn should_conform() {}
    fn s_should_() {}
    mod s {
        fn s_only_should_conform() {}
    }
}
";
            let mut case = leaf(SourceTypeKind::Case, "s_should_conform", 6);
            case.comments = vec![comment(7, "GIVEN s")];

            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![
                    leaf(SourceTypeKind::Type, "S", 1),
                    leaf(SourceTypeKind::Function, "outside_should_work", 2),
                    ty(
                        TEST_MODULE,
                        "tests",
                        4,
                        vec![
                            case,
                            ty(
                                TEST_MODULE,
                                "s",
                                11,
                                vec![leaf(SourceTypeKind::Case, "s_only_should_conform", 12)]
                            ),
                        ]
                    ),
                ])
            );
        }

        #[test]
        fn parse_source_should_map_calls_and_waiver_of_tests() {
            let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn a() {
        // WHY: code makes no waiver.
        crate::cases::case_a(&x);
        assert!(check(1).is_ok());
        let f = |y| inner(y);
        println!("{}", 1);
        let s = S { f: 1 };
        x.method();
        r#match();
    }
    #[test]
    fn b() {}
    #[test]
    fn c() {
        // WHY: the reason
        // AND more of it.
        /* WHY: another */
    }
    #[test]
    fn d() {
        // WHY: the reason
        // GIVEN a step
    }
    #[test]
    fn e() {
        // GIVEN a step
    }
}
"#;
            let tests: Vec<(String, Vec<String>, bool)> =
                parse_source(source, false).unwrap().types["tests"][0]
                    .types
                    .iter()
                    .map(|t| (t.name.clone(), t.calls.clone(), t.waived))
                    .collect();
            let calls = |calls: &[&str]| calls.iter().map(|c| c.to_string()).collect();

            assert_eq!(
                tests,
                vec![
                    (
                        "a".to_string(),
                        calls(&["case_a", "check", "is_ok", "inner", "method", "match"]),
                        false
                    ),
                    ("b".to_string(), vec![], false),
                    ("c".to_string(), vec![], true),
                    ("d".to_string(), vec![], false),
                    ("e".to_string(), vec![], false),
                ]
            );
        }

        #[test]
        fn parse_source_should_treat_inner_attribute_after_comment_as_attribute() {
            let source = "#! // not a shebang\n[cfg(test)]\n#[test]\nfn t() {}\n";

            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![ty(
                    TEST_MODULE,
                    "tests",
                    0,
                    vec![leaf(SourceTypeKind::Test, "t", 4)]
                )])
            );
        }

        #[test]
        fn parse_source_should_map_comments_in_test_body() {
            for (body, expected) in [
                ("    // GIVEN a value", vec![(3, "GIVEN a value")]),
                ("    let x = 1; // trailing", vec![(3, "trailing")]),
                (
                    "    let x = 1; // GIVEN x\n    // note",
                    vec![(3, "GIVEN x"), (4, "note")],
                ),
                ("    /* a /* nested */ */", vec![(3, "a /* nested */")]),
                ("    assert!(true /* in macro */);", vec![(3, "in macro")]),
                (
                    "    /*\n     * first\n     * second\n     */",
                    vec![(4, "first\nsecond")],
                ),
                (
                    "    // GIVEN a\n    //\n    // AND b",
                    vec![(3, "GIVEN a\n\nAND b")],
                ),
                (
                    "    /* GIVEN a\n       continued */",
                    vec![(3, "GIVEN a\ncontinued")],
                ),
                (
                    "    /* GIVEN */ /* note */ let x = 1;",
                    vec![(3, "GIVEN"), (3, "note")],
                ),
                (
                    "    // GIVEN a value that is\n    // long enough to wrap",
                    vec![(3, "GIVEN a value that is\nlong enough to wrap")],
                ),
                (
                    "    /// GIVEN a\n    /// b\n    let x = 1;",
                    vec![(3, "GIVEN a\nb")],
                ),
                ("    // a\n\n    // b", vec![(3, "a"), (5, "b")]),
                (
                    "    // a\n    let x = 1;\n    // b",
                    vec![(3, "a"), (5, "b")],
                ),
                ("    /* a */\n    // b", vec![(3, "a"), (4, "b")]),
                ("    /// GIVEN\n    let x = 1;", vec![(3, "GIVEN")]),
                ("    /** WHEN */\n    fn nested() {}", vec![(3, "WHEN")]),
                ("    //! inner doc", vec![(3, "inner doc")]),
                (
                    "    mod m {\n        //! nested docs\n    }",
                    vec![(4, "nested docs")],
                ),
                ("    #![allow(unused)]\n    // after", vec![(4, "after")]),
            ] {
                let source = format!("#[test]\nfn t() {{\n{body}\n}}");
                let expected: Vec<Comment> = expected
                    .into_iter()
                    .map(|(line, text)| comment(line, text))
                    .collect();

                assert_eq!(comments_of_t(&source), expected, "{body:?}");
            }
        }

        #[test]
        fn parse_source_should_not_map_comments_outside_test_body() {
            let source = "\
// above
/// doc
#[test]
// between
pub /* vis */ fn t(/* arg */) /* ret */ {} // after
";
            assert_eq!(comments_of_t(source), vec![]);
        }

        #[test]
        fn parse_source_should_skip_code_and_empty_comment_lines() {
            let source = r####"
#[test]
fn t() {
    let s = "// not \" /* still string";
    let raw = r#"// raw " /* not "#;
    let c = cr"// c string";
    let chars = ['"', '\'', '/', b'/'];
    let x = 4 / 2;
    let mut y = 1;
    y /= 2;
    //
    /* */
    /**/
    ///
    let z = 1;
}
"####;
            assert_eq!(comments_of_t(source), vec![]);
        }

        #[test]
        fn parse_source_should_map_comment_lines_after_shebang_and_bom() {
            for source in [
                "#!/usr/bin/env run-cargo-script\n#[test]\nfn t() {\n    // é body\n}\n",
                "\u{feff}\n#[test]\nfn t() {\n    // é body\n}\n",
            ] {
                assert_eq!(
                    comments_of_t(source),
                    vec![comment(4, "é body")],
                    "{source:?}"
                );
            }
        }

        #[test]
        fn parse_source_should_not_map_comments_of_test_helpers() {
            let source = "\
#[cfg(test)]
mod tests {
    fn helper() {
        // helper
    }
    #[test]
    fn t() {}
}
";
            assert_eq!(comments_of_t(source), vec![]);
        }

        #[test]
        fn parse_source_should_map_tests_outside_test_context() {
            let source = "\
#[test]
fn smoke() {}
mod tests {
    #[test]
    fn a() {}
    mod b {
        #[test]
        fn b_should_work() {}
    }
}
mod mixed {
    fn setup() {}
    #[test]
    fn c() {}
}
#[my::deep::test]
fn deep() {}
mod runtime_only {
    #[tokio::test]
    async fn t() {}
}
#[cfg(test)]
#[tokio::test]
async fn gated() {}
mod config {
    pub const MAX: u32 = 5;
    pub mod parser;
    #[cfg(test)]
    mod tests {
        #[test]
        fn x() {}
    }
}
pub mod store {
    pub use inner::*;
    #[cfg(test)]
    pub mod mocks {
        pub fn m() {}
    }
}
mod empty {}
#[cfg(test)]
mod gated_tests {
    #[test]
    fn g() {}
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                SourceTypes {
                    code_after_tests: vec![16],
                    ..types(vec![
                        leaf(SourceTypeKind::Test, "smoke", 2),
                        ty(
                            TEST_MODULE,
                            "tests",
                            3,
                            vec![
                                leaf(SourceTypeKind::Test, "a", 5),
                                ty(
                                    TEST_MODULE,
                                    "b",
                                    6,
                                    vec![leaf(SourceTypeKind::Test, "b_should_work", 8)]
                                ),
                            ]
                        ),
                        ty(
                            TEST_MODULE,
                            "mixed",
                            11,
                            vec![leaf(SourceTypeKind::Test, "c", 14)]
                        ),
                        leaf(SourceTypeKind::Function, "deep", 17),
                        ty(
                            MODULE,
                            "runtime_only",
                            18,
                            vec![leaf(SourceTypeKind::Function, "t", 20)]
                        ),
                        leaf(SourceTypeKind::Test, "gated", 24),
                        ty(
                            MODULE,
                            "config",
                            25,
                            vec![ty(
                                TEST_MODULE,
                                "tests",
                                29,
                                vec![leaf(SourceTypeKind::Test, "x", 31)]
                            )]
                        ),
                        ty(MODULE, "store", 34, vec![leaf(HELPER_MODULE, "mocks", 37)]),
                        leaf(MODULE, "empty", 41),
                        ty(
                            TEST_MODULE,
                            "gated_tests",
                            43,
                            vec![leaf(SourceTypeKind::Test, "g", 45)]
                        ),
                    ])
                }
            );
        }

        #[test]
        fn parse_source_should_map_tests_of_inline_modules_in_module() {
            let source = "\
mod outer {
    struct Inner;
    #[cfg(test)]
    mod tests {
        #[test]
        fn inner_should_work() {}
    }
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![ty(
                    MODULE,
                    "outer",
                    1,
                    vec![
                        leaf(SourceTypeKind::Type, "Inner", 2),
                        ty(
                            TEST_MODULE,
                            "tests",
                            4,
                            vec![leaf(SourceTypeKind::Test, "inner_should_work", 6)]
                        ),
                    ]
                )])
            );
        }

        #[test]
        fn parse_source_test_file_should_map_file_as_tests_module() {
            let body =
                "struct Mock;\n#[test]\nfn file_should_work() {}\nmod nested { mod deeper {} }\n";

            for (source, test_file) in [
                (format!("#![cfg(test)]\n{body}"), false),
                (format!("\n{body}"), true),
            ] {
                assert_eq!(
                    parse_source(&source, test_file).unwrap(),
                    types(vec![ty(
                        TEST_MODULE,
                        "tests",
                        0,
                        vec![
                            leaf(SourceTypeKind::Test, "file_should_work", 4),
                            ty(
                                HELPER_MODULE,
                                "nested",
                                5,
                                vec![leaf(HELPER_MODULE, "deeper", 5)]
                            ),
                        ]
                    )]),
                    "{source}"
                );
            }
        }

        #[test]
        fn parse_source_with_ungated_tests_should_map_file_as_tests_module() {
            let source = "\
use super::*;
fn helper() {}
#[tokio::test]
async fn loose() {}
mod sik {
    mod new {
        #[test]
        fn new_should_work() {}
    }
}
";
            assert_eq!(
                parse_source(source, false).unwrap(),
                types(vec![ty(
                    TEST_MODULE,
                    "tests",
                    0,
                    vec![
                        leaf(SourceTypeKind::Test, "loose", 4),
                        ty(
                            TEST_MODULE,
                            "sik",
                            5,
                            vec![ty(
                                TEST_MODULE,
                                "new",
                                6,
                                vec![leaf(SourceTypeKind::Test, "new_should_work", 8)]
                            )]
                        ),
                    ]
                )])
            );
        }

        #[test]
        fn parse_source_should_map_first_code_after_tests_of_each_module() {
            let source = "\
struct S;
#[cfg(test)]
mod fixtures {}
fn f() {}
mod outer {
    mod tests {
        #[test]
        fn t() {}
    }
    /// Docs
    impl S {}
    fn g() {}
}
#[cfg(test)]
mod tests {
    #[test]
    fn t() {}
}
#[cfg(test)]
fn helper() {}
#[test]
fn loose() {}
use std::fmt;
mod later;
";
            assert_eq!(
                parse_source(source, false).unwrap().code_after_tests,
                [10, 23]
            );
        }

        #[test]
        fn parse_source_test_file_should_not_map_code_after_tests() {
            let source = "#[test]\nfn t() {}\nfn helper() {}\n";

            assert_eq!(
                parse_source(source, true).unwrap().code_after_tests,
                Vec::<usize>::new()
            );
        }

        #[test]
        fn parse_source_empty_should_return_no_types() {
            assert_eq!(parse_source("", false).unwrap(), SourceTypes::default());
        }

        #[test]
        fn parse_source_invalid_should_fail_with_location() {
            for (source, location) in [
                ("struct S;\nfn () {}", "2:4: expected identifier"),
                ("struct S;\nstruct T", "2:9: unexpected end of input"),
                ("struct S;\nfn\n", "3:1: unexpected end of input"),
                ("\u{feff}struct T", "1:9: unexpected end of input"),
            ] {
                let error = parse_source(source, false).unwrap_err();
                assert!(error.starts_with(location), "{source:?}: {error}");
            }
        }
    }
}
