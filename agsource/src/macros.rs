use crate::ModuleId;
use crate::Namespace;
use crate::Resolved;
use crate::SourceCrate;
use proc_macro2::Delimiter;
use proc_macro2::Group;
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use std::collections::HashMap;
use syn::Expr;
use syn::Item;
use syn::ItemStatic;
use syn::LitStr;
use syn::Pat;
use syn::Stmt;
use syn::Token;
use syn::ext::IdentExt;
use syn::parse::ParseStream;
use syn::parse::Parser;
use syn::parse::discouraged::Speculative;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;

/// A macro call, parsed by what the macro does with its arguments.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroCall {
    /// A std macro taking a format string: `format!`, `format_args!`,
    /// `print!`, `println!`, `eprint!`, `eprintln!`, `write!`, `writeln!`,
    /// `panic!`, `unreachable!`, `todo!`, `unimplemented!`, `assert!`,
    /// `assert_eq!`, `assert_ne!` and their `debug_` forms. The expressions
    /// are those before the format string: the destination of `write!`, the
    /// condition of `assert!`, both sides of `assert_eq!`.
    Format {
        exprs: Vec<Expr>,
        format: Option<FormatArgs>,
    },
    /// `vec![a, b]`.
    Vec(Vec<Expr>),
    /// `vec![value; count]`.
    VecRepeat(Box<Expr>, Box<Expr>),
    /// `matches!(expr, pat if guard)`.
    Matches {
        expr: Box<Expr>,
        pat: Box<Pat>,
        guard: Option<Box<Expr>>,
    },
    /// The statics of `thread_local!`.
    ThreadLocal(Vec<ItemStatic>),
    /// `file!()`, `line!()`, `column!()`, `module_path!()`.
    NoArgs,
    /// The statements a `macro_rules!` of the crate expands to. `$crate` is
    /// `crate`, the other paths are as written, to be resolved from the
    /// module of the call.
    Expanded(Vec<Stmt>),
}

/// A format string with its arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct FormatArgs {
    pub string: LitStr,
    pub args: Vec<FormatArg>,
    /// The names the string uses that no argument gives (`{name}`,
    /// `{:width$}`), in order and once each.
    pub captures: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatArg {
    /// `name = expr`.
    pub name: Option<Ident>,
    pub expr: Expr,
}

impl SourceCrate {
    /// Parses the call of a macro named from the module: a std macro whose
    /// arguments are known, or a `macro_rules!` of the crate, which is
    /// expanded. The error says why the call is not parsed: a macro of
    /// another crate, or arguments that do not fit the macro.
    pub fn macro_call(&self, from: ModuleId, mac: &syn::Macro) -> Result<MacroCall, String> {
        self.call(&self.resolve(from, &mac.path, Namespace::Macro), mac)
    }

    pub(crate) fn call(&self, target: &Resolved, mac: &syn::Macro) -> Result<MacroCall, String> {
        if let Resolved::Item(id) = target
            && let Item::Macro(def) = self.item(*id)
            && def.mac.path.is_ident("macro_rules")
        {
            return expand(&def.mac.tokens, &mac.tokens);
        }

        match target {
            Resolved::External(path) if path.len() == 2 && STD.contains(&path[0].as_str()) => {
                std_call(&path[1], mac).unwrap_or_else(|| Err(unknown(path)))
            }
            Resolved::External(path) => Err(unknown(path)),
            _ => Err("names no macro".to_string()),
        }
    }
}

impl MacroCall {
    pub(crate) fn visit<'ast>(&'ast self, v: &mut impl Visit<'ast>) {
        match self {
            MacroCall::Format { exprs, format } => {
                exprs.iter().for_each(|e| v.visit_expr(e));
                format
                    .iter()
                    .flat_map(|f| &f.args)
                    .for_each(|a| v.visit_expr(&a.expr));
            }
            MacroCall::Vec(exprs) => exprs.iter().for_each(|e| v.visit_expr(e)),
            MacroCall::VecRepeat(value, count) => {
                v.visit_expr(value);
                v.visit_expr(count);
            }
            MacroCall::Matches { expr, pat, guard } => {
                v.visit_expr(expr);
                v.visit_pat(pat);
                guard.iter().for_each(|g| v.visit_expr(g));
            }
            MacroCall::ThreadLocal(statics) => statics.iter().for_each(|s| v.visit_item_static(s)),
            MacroCall::NoArgs => {}
            MacroCall::Expanded(stmts) => stmts.iter().for_each(|s| v.visit_stmt(s)),
        }
    }

    pub(crate) fn visit_mut(&mut self, v: &mut impl VisitMut) {
        match self {
            MacroCall::Format { exprs, format } => {
                exprs.iter_mut().for_each(|e| v.visit_expr_mut(e));
                format.iter_mut().for_each(|f| {
                    v.visit_lit_str_mut(&mut f.string);
                    f.args
                        .iter_mut()
                        .for_each(|a| v.visit_expr_mut(&mut a.expr));
                });
            }
            MacroCall::Vec(exprs) => exprs.iter_mut().for_each(|e| v.visit_expr_mut(e)),
            MacroCall::VecRepeat(value, count) => {
                v.visit_expr_mut(value);
                v.visit_expr_mut(count);
            }
            MacroCall::Matches { expr, pat, guard } => {
                v.visit_expr_mut(expr);
                v.visit_pat_mut(pat);
                guard.iter_mut().for_each(|g| v.visit_expr_mut(g));
            }
            MacroCall::ThreadLocal(statics) => {
                statics.iter_mut().for_each(|s| v.visit_item_static_mut(s));
            }
            MacroCall::NoArgs => {}
            MacroCall::Expanded(stmts) => stmts.iter_mut().for_each(|s| v.visit_stmt_mut(s)),
        }
    }
}

const STD: [&str; 3] = ["std", "core", "alloc"];

fn unknown(path: &[String]) -> String {
    format!("the arguments of {}! are not known", path.join("::"))
}

/// The call of a std macro, none for a macro whose arguments are not known.
fn std_call(name: &str, mac: &syn::Macro) -> Option<Result<MacroCall, String>> {
    let result = match name {
        "format" | "format_args" | "print" | "println" | "eprint" | "eprintln" | "panic"
        | "unreachable" | "todo" | "unimplemented" => mac.parse_body_with(format_call(0)),
        "write" | "writeln" | "assert" | "debug_assert" => mac.parse_body_with(format_call(1)),
        "assert_eq" | "assert_ne" | "debug_assert_eq" | "debug_assert_ne" => {
            mac.parse_body_with(format_call(2))
        }
        "vec" => mac.parse_body_with(vec_call),
        "matches" => mac.parse_body_with(matches_call),
        "thread_local" => mac.parse_body_with(thread_local_call),
        "file" | "line" | "column" | "module_path" => {
            mac.parse_body_with(|_: ParseStream| Ok(MacroCall::NoArgs))
        }
        _ => return None,
    };
    Some(result.map_err(|e| e.to_string()))
}

/// The format call with `exprs` expressions before the format string.
fn format_call(exprs: usize) -> impl FnOnce(ParseStream) -> syn::Result<MacroCall> {
    move |input| {
        let mut parsed = Vec::new();

        while parsed.len() < exprs && !input.is_empty() {
            parsed.push(input.parse()?);
            eat::<Token![,]>(input);
        }

        let format = if input.is_empty() {
            None
        } else {
            Some(format_args(input)?)
        };

        Ok(MacroCall::Format {
            exprs: parsed,
            format,
        })
    }
}

fn format_args(input: ParseStream) -> syn::Result<FormatArgs> {
    let string: LitStr = input.parse()?;
    let mut args = Vec::new();

    while !input.is_empty() {
        input.parse::<Token![,]>()?;

        if !input.is_empty() {
            args.push(format_arg(input.parse()?));
        }
    }

    let named: Vec<String> = args
        .iter()
        .filter_map(|a| a.name.as_ref().map(Ident::to_string))
        .collect();
    let captures = captures(&string.value())
        .into_iter()
        .filter(|name| !named.contains(name))
        .map(|name| Ident::new(&name, string.span()))
        .collect();

    Ok(FormatArgs {
        string,
        args,
        captures,
    })
}

fn format_arg(expr: Expr) -> FormatArg {
    if let Expr::Assign(assign) = &expr
        && let Expr::Path(path) = &*assign.left
        && let Some(name) = path.path.get_ident()
    {
        return FormatArg {
            name: Some(name.clone()),
            expr: (*assign.right).clone(),
        };
    }

    FormatArg { name: None, expr }
}

/// The names of the placeholders of a format string: the arguments and the
/// `name$` widths and precisions.
fn captures(string: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut chars = string.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '{' || chars.next_if_eq(&'{').is_some() {
            continue;
        }

        let placeholder: String = chars.by_ref().take_while(|&c| c != '}').collect();
        let (arg, spec) = placeholder.split_once(':').unwrap_or((&placeholder, ""));
        let counts = spec
            .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
            .filter_map(|word| word.strip_suffix('$'));

        for name in std::iter::once(arg.trim()).chain(counts) {
            if name.starts_with(|c: char| c.is_alphabetic() || c == '_')
                && !names.iter().any(|n| n == name)
            {
                names.push(name.to_string());
            }
        }
    }

    names
}

fn vec_call(input: ParseStream) -> syn::Result<MacroCall> {
    if input.is_empty() {
        return Ok(MacroCall::Vec(Vec::new()));
    }

    let first: Expr = input.parse()?;

    if eat::<Token![;]>(input) {
        return Ok(MacroCall::VecRepeat(Box::new(first), input.parse()?));
    }

    let mut exprs = vec![first];

    while !input.is_empty() {
        input.parse::<Token![,]>()?;

        if !input.is_empty() {
            exprs.push(input.parse()?);
        }
    }

    Ok(MacroCall::Vec(exprs))
}

/// Parses the token if it is next.
fn eat<T: syn::parse::Parse + syn::token::Token>(input: ParseStream) -> bool {
    input.parse::<Option<T>>().ok().flatten().is_some()
}

fn matches_call(input: ParseStream) -> syn::Result<MacroCall> {
    let expr = input.parse()?;
    input.parse::<Token![,]>()?;
    let pat = Pat::parse_multi_with_leading_vert(input)?;
    let guard = if eat::<Token![if]>(input) {
        Some(input.parse()?)
    } else {
        None
    };
    eat::<Token![,]>(input);

    Ok(MacroCall::Matches {
        expr,
        pat: Box::new(pat),
        guard,
    })
}

fn thread_local_call(input: ParseStream) -> syn::Result<MacroCall> {
    let mut statics = Vec::new();

    while !input.is_empty() {
        statics.push(input.parse()?);
    }

    Ok(MacroCall::ThreadLocal(statics))
}

/// A token of a `macro_rules!` matcher (`V` is `$name:kind`) or
/// transcriber (`V` is `$name`).
enum Pattern<V> {
    Token(TokenTree),
    Group(Delimiter, Vec<Pattern<V>>),
    Var(V),
    /// `$( ... ) sep op`, the operator is `*`, `+` or `?`.
    Repeat(Vec<Pattern<V>>, Option<TokenTree>, char),
}

type Matcher = Pattern<(String, Fragment)>;
type Transcriber = Pattern<String>;

trait Var {
    fn name(&self) -> &str;
}

impl Var for String {
    fn name(&self) -> &str {
        self
    }
}

impl Var for (String, Fragment) {
    fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
enum Fragment {
    Block,
    Expr,
    Ident,
    Item,
    Lifetime,
    Literal,
    Meta,
    Pat,
    PatParam,
    Path,
    Tt,
    Ty,
    Vis,
}

#[derive(Debug, Clone)]
enum Binding {
    One(TokenStream),
    Repeated(Vec<HashMap<String, Binding>>),
}

struct Rule {
    matcher: Vec<Matcher>,
    transcriber: Vec<Transcriber>,
}

/// Expands the call `input` of the `macro_rules!` with the `rules` by the
/// first rule that matches it.
fn expand(rules: &TokenStream, input: &TokenStream) -> Result<MacroCall, String> {
    let rules = parse_rules(rules)?;

    for rule in &rules {
        let mut bindings = HashMap::new();
        let matched = (|stream: ParseStream| match_patterns(stream, &rule.matcher, &mut bindings))
            .parse2(input.clone());

        if matched.is_ok() {
            let mut tokens = TokenStream::new();
            transcribe(&rule.transcriber, &bindings, &mut tokens)?;
            return syn::Block::parse_within
                .parse2(tokens)
                .map(MacroCall::Expanded)
                .map_err(|e| format!("the expansion is not statements: {e}"));
        }
    }

    Err("no rule matches".to_string())
}

fn parse_rules(tokens: &TokenStream) -> Result<Vec<Rule>, String> {
    let groups = (|input: ParseStream| {
        let mut groups = Vec::new();

        while !input.is_empty() {
            let matcher: Group = input.parse()?;
            input.parse::<Token![=>]>()?;
            let transcriber: Group = input.parse()?;
            eat::<Token![;]>(input);
            groups.push((matcher.stream(), transcriber.stream()));
        }

        Ok(groups)
    })
    .parse2(tokens.clone())
    .map_err(|e| format!("invalid macro_rules!: {e}"))?;

    groups
        .into_iter()
        .map(|(matcher, transcriber)| {
            Ok(Rule {
                matcher: patterns(matcher, matcher_var)?,
                transcriber: patterns(transcriber, |name, _| Ok(name.unraw().to_string()))?,
            })
        })
        .collect()
}

fn matcher_var(
    name: Ident,
    tokens: &mut proc_macro2::token_stream::IntoIter,
) -> Result<(String, Fragment), String> {
    match (tokens.next(), tokens.next()) {
        (Some(TokenTree::Punct(colon)), Some(TokenTree::Ident(kind))) if colon.as_char() == ':' => {
            Fragment::new(&kind.to_string())
                .map(|fragment| (name.unraw().to_string(), fragment))
                .ok_or_else(|| format!("unsupported fragment specifier {kind}"))
        }
        _ => Err(format!("${name} has no fragment specifier")),
    }
}

fn patterns<V>(
    tokens: TokenStream,
    var: fn(Ident, &mut proc_macro2::token_stream::IntoIter) -> Result<V, String>,
) -> Result<Vec<Pattern<V>>, String> {
    let mut patterns = Vec::new();
    let mut tokens = tokens.into_iter();

    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Punct(p) if p.as_char() == '$' => match tokens.next() {
                Some(TokenTree::Ident(name)) => {
                    patterns.push(Pattern::Var(var(name, &mut tokens)?))
                }
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
                    let inner = self::patterns(group.stream(), var)?;
                    let (separator, op) = match tokens.next() {
                        Some(TokenTree::Punct(op)) if "*+?".contains(op.as_char()) => {
                            (None, op.as_char())
                        }
                        separator => match tokens.next() {
                            Some(TokenTree::Punct(op)) if "*+".contains(op.as_char()) => {
                                (separator, op.as_char())
                            }
                            _ => return Err("a repetition has no operator".to_string()),
                        },
                    };
                    patterns.push(Pattern::Repeat(inner, separator, op));
                }
                _ => return Err("`$` is not followed by a name or a repetition".to_string()),
            },
            TokenTree::Group(group) => patterns.push(Pattern::Group(
                group.delimiter(),
                self::patterns(group.stream(), var)?,
            )),
            token => patterns.push(Pattern::Token(token)),
        }
    }

    Ok(patterns)
}

impl Fragment {
    fn new(kind: &str) -> Option<Self> {
        Some(match kind {
            "block" => Fragment::Block,
            "expr" | "expr_2021" => Fragment::Expr,
            "ident" => Fragment::Ident,
            "item" => Fragment::Item,
            "lifetime" => Fragment::Lifetime,
            "literal" => Fragment::Literal,
            "meta" => Fragment::Meta,
            "pat" => Fragment::Pat,
            "pat_param" => Fragment::PatParam,
            "path" => Fragment::Path,
            "tt" => Fragment::Tt,
            "ty" => Fragment::Ty,
            "vis" => Fragment::Vis,
            _ => return None,
        })
    }

    fn parse(self, input: ParseStream) -> syn::Result<()> {
        match self {
            Fragment::Block => input.parse::<syn::Block>().map(drop),
            Fragment::Expr => input.parse::<Expr>().map(drop),
            Fragment::Ident => Ident::parse_any(input).map(drop),
            Fragment::Item => input.parse::<Item>().map(drop),
            Fragment::Lifetime => input.parse::<syn::Lifetime>().map(drop),
            Fragment::Literal => {
                eat::<Token![-]>(input);
                input.parse::<syn::Lit>().map(drop)
            }
            Fragment::Meta => input.parse::<syn::Meta>().map(drop),
            Fragment::Pat => Pat::parse_multi_with_leading_vert(input).map(drop),
            Fragment::PatParam => Pat::parse_single(input).map(drop),
            Fragment::Path => input.parse::<syn::Path>().map(drop),
            Fragment::Tt => input.parse::<TokenTree>().map(drop),
            Fragment::Ty => input.parse::<syn::Type>().map(drop),
            Fragment::Vis => input.parse::<syn::Visibility>().map(drop),
        }
    }

    /// The tokens of the fragment at the start of `input`. An expression is
    /// kept in an invisible group, as rustc does, so that it stays one
    /// operand where it is substituted.
    fn take(self, input: ParseStream) -> syn::Result<TokenStream> {
        let fork = input.fork();
        self.parse(&fork)?;
        let end = fork.cursor();
        input.step(|cursor| {
            let mut tokens = TokenStream::new();
            let mut rest = *cursor;

            while let Some((token, next)) = rest.token_tree().filter(|_| rest != end) {
                tokens.extend([token]);
                rest = next;
            }

            let tokens = match self {
                Fragment::Expr => TokenTree::Group(Group::new(Delimiter::None, tokens)).into(),
                _ => tokens,
            };
            Ok((tokens, rest))
        })
    }
}

fn match_patterns(
    input: ParseStream,
    patterns: &[Matcher],
    bindings: &mut HashMap<String, Binding>,
) -> syn::Result<()> {
    for pattern in patterns {
        match pattern {
            Pattern::Token(expected) => match_token(input, expected)?,
            Pattern::Group(delimiter, inner) => {
                let group: Group = input.parse()?;

                if group.delimiter() != *delimiter {
                    return Err(syn::Error::new(group.span(), "another delimiter"));
                }

                (|stream: ParseStream| match_patterns(stream, inner, bindings))
                    .parse2(group.stream())?;
            }
            Pattern::Var((name, fragment)) => {
                let tokens = fragment.take(input)?;
                bindings.insert(name.clone(), Binding::One(tokens));
            }
            Pattern::Repeat(inner, separator, op) => {
                let mut repeated = Vec::new();

                while !(*op == '?' && repeated.len() == 1) {
                    let fork = input.fork();

                    if !repeated.is_empty()
                        && let Some(separator) = separator
                        && match_token(&fork, separator).is_err()
                    {
                        break;
                    }

                    let mut inner_bindings = HashMap::new();

                    if match_patterns(&fork, inner, &mut inner_bindings).is_err()
                        || fork.cursor() == input.cursor()
                    {
                        break;
                    }

                    input.advance_to(&fork);
                    repeated.push(inner_bindings);
                }

                if *op == '+' && repeated.is_empty() {
                    return Err(input.error("expected at least one repetition"));
                }

                for name in vars(inner) {
                    bindings.insert(name, Binding::Repeated(repeated.clone()));
                }
            }
        }
    }

    Ok(())
}

fn match_token(input: ParseStream, expected: &TokenTree) -> syn::Result<()> {
    input.step(|cursor| match cursor.token_tree() {
        Some((token, rest)) if same(&token, expected) => Ok(((), rest)),
        _ => Err(syn::Error::new(
            cursor.span(),
            format!("expected `{expected}`"),
        )),
    })
}

fn same(token: &TokenTree, expected: &TokenTree) -> bool {
    match (token, expected) {
        (TokenTree::Ident(a), TokenTree::Ident(b)) => a == b,
        (TokenTree::Punct(a), TokenTree::Punct(b)) => a.as_char() == b.as_char(),
        (TokenTree::Literal(a), TokenTree::Literal(b)) => a.to_string() == b.to_string(),
        _ => false,
    }
}

/// The names of the variables in the patterns, those in repetitions
/// included.
fn vars<V: Var>(patterns: &[Pattern<V>]) -> Vec<String> {
    patterns
        .iter()
        .flat_map(|pattern| match pattern {
            Pattern::Var(var) => vec![var.name().to_string()],
            Pattern::Group(_, inner) | Pattern::Repeat(inner, _, _) => vars(inner),
            Pattern::Token(_) => Vec::new(),
        })
        .collect()
}

fn transcribe(
    patterns: &[Transcriber],
    bindings: &HashMap<String, Binding>,
    out: &mut TokenStream,
) -> Result<(), String> {
    for pattern in patterns {
        match pattern {
            Pattern::Token(token) => out.extend([token.clone()]),
            Pattern::Group(delimiter, inner) => {
                let mut tokens = TokenStream::new();
                transcribe(inner, bindings, &mut tokens)?;
                out.extend([TokenTree::Group(Group::new(*delimiter, tokens))]);
            }
            Pattern::Var(name) if name == "crate" => {
                out.extend([TokenTree::Ident(Ident::new("crate", Span::call_site()))]);
            }
            Pattern::Var(name) => match bindings.get(name) {
                Some(Binding::One(tokens)) => out.extend(tokens.clone()),
                Some(Binding::Repeated(_)) => return Err(format!("${name} is still repeating")),
                None => return Err(format!("${name} is not bound")),
            },
            Pattern::Repeat(inner, separator, _) => {
                let repeated: Vec<&Vec<HashMap<String, Binding>>> = vars(inner)
                    .iter()
                    .filter_map(|name| match bindings.get(name) {
                        Some(Binding::Repeated(repeated)) => Some(repeated),
                        _ => None,
                    })
                    .collect();
                let Some(count) = repeated.first().map(|r| r.len()) else {
                    return Err("a repetition repeats no variable".to_string());
                };

                if repeated.iter().any(|r| r.len() != count) {
                    return Err("the variables of a repetition repeat differently".to_string());
                }

                for i in 0..count {
                    if i > 0 {
                        out.extend(separator.clone());
                    }

                    let mut inner_bindings = bindings.clone();

                    for r in &repeated {
                        inner_bindings.extend(r[i].clone());
                    }

                    transcribe(inner, &inner_bindings, out)?;
                }
            }
        }
    }

    Ok(())
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

    fn call(c: &SourceCrate, text: &str) -> Result<MacroCall, String> {
        c.macro_call(c.root(), &syn::parse_str(text).unwrap())
    }

    fn expr(text: &str) -> Expr {
        syn::parse_str(text).unwrap()
    }

    fn stmts(text: &str) -> Vec<Stmt> {
        syn::Block::parse_within.parse_str(text).unwrap()
    }

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    fn format(string: &str, args: &[(Option<&str>, &str)], captures: &[&str]) -> FormatArgs {
        FormatArgs {
            string: syn::parse_str(string).unwrap(),
            args: args
                .iter()
                .map(|(name, e)| FormatArg {
                    name: name.map(ident),
                    expr: expr(e),
                })
                .collect(),
            captures: captures.iter().map(|name| ident(name)).collect(),
        }
    }

    mod macro_call {
        use super::*;

        mod visit_mut {
            use super::*;

            struct Names(Vec<String>);

            impl VisitMut for Names {
                fn visit_ident_mut(&mut self, ident: &mut Ident) {
                    self.0.push(ident.to_string());
                }

                fn visit_lit_str_mut(&mut self, lit: &mut LitStr) {
                    self.0.push(lit.value());
                }
            }

            #[test]
            fn visit_mut_should_visit_every_argument() {
                let c = lib("macro_rules! m { () => { let a = b; } }");
                let calls = [
                    r#"assert_eq!(a, b, "{}", c)"#,
                    "assert!(a)",
                    "vec![a]",
                    "vec![a; b]",
                    "matches!(a, b if c)",
                    "matches!(a, b)",
                    "thread_local!(static A: B = c;)",
                    "m!()",
                    "file!()",
                ];

                let names: Vec<Vec<String>> = calls
                    .iter()
                    .map(|text| {
                        let mut names = Names(Vec::new());
                        call(&c, text).unwrap().visit_mut(&mut names);
                        names.0
                    })
                    .collect();

                assert_eq!(
                    names,
                    [
                        vec!["a", "b", "{}", "c"],
                        vec!["a"],
                        vec!["a"],
                        vec!["a", "b"],
                        vec!["a", "b", "c"],
                        vec!["a", "b"],
                        vec!["A", "B", "c"],
                        vec!["a", "b"],
                        vec![],
                    ]
                );
            }
        }
    }

    mod source_crate {
        use super::*;

        mod macro_call {
            use super::*;

            #[test]
            fn macro_call_should_parse_the_format_macros() {
                let c = lib("");

                assert_eq!(
                    call(
                        &c,
                        r#"format!("{a} {b:>w$.p$} {{x}} {0} {a} {named} {:?}", x, named = y,)"#
                    ),
                    Ok(MacroCall::Format {
                        exprs: Vec::new(),
                        format: Some(format(
                            r#""{a} {b:>w$.p$} {{x}} {0} {a} {named} {:?}""#,
                            &[(None, "x"), (Some("named"), "y")],
                            &["a", "b", "w", "p"],
                        )),
                    })
                );
                assert_eq!(
                    call(&c, "println!()"),
                    Ok(MacroCall::Format {
                        exprs: Vec::new(),
                        format: None,
                    })
                );
                assert_eq!(
                    call(&c, r#"core::write!(f, "{}", 1)"#),
                    Ok(MacroCall::Format {
                        exprs: vec![expr("f")],
                        format: Some(format(r#""{}""#, &[(None, "1")], &[])),
                    })
                );
                assert_eq!(
                    call(&c, "assert!(ok)"),
                    Ok(MacroCall::Format {
                        exprs: vec![expr("ok")],
                        format: None,
                    })
                );
                assert_eq!(
                    call(&c, r#"assert_eq!(a, b + 1, "{a}")"#),
                    Ok(MacroCall::Format {
                        exprs: vec![expr("a"), expr("b + 1")],
                        format: Some(format(r#""{a}""#, &[], &["a"])),
                    })
                );
            }

            #[test]
            fn macro_call_should_parse_vec_matches_and_thread_local() {
                let c = lib("");

                assert_eq!(call(&c, "vec![]"), Ok(MacroCall::Vec(Vec::new())));
                assert_eq!(
                    call(&c, "vec![a, b + 1,]"),
                    Ok(MacroCall::Vec(vec![expr("a"), expr("b + 1")]))
                );
                assert_eq!(
                    call(&c, "vec![0; n]"),
                    Ok(MacroCall::VecRepeat(
                        Box::new(expr("0")),
                        Box::new(expr("n"))
                    ))
                );
                assert_eq!(
                    call(&c, "matches!(x, Some(_) | None if y,)"),
                    Ok(MacroCall::Matches {
                        expr: Box::new(expr("x")),
                        pat: Box::new(
                            Pat::parse_multi_with_leading_vert
                                .parse_str("Some(_) | None")
                                .unwrap()
                        ),
                        guard: Some(Box::new(expr("y"))),
                    })
                );
                assert_eq!(
                    call(
                        &c,
                        "thread_local! { static A: u8 = 1; pub static B: u8 = const { 2 }; }"
                    ),
                    Ok(MacroCall::ThreadLocal(vec![
                        syn::parse_str("static A: u8 = 1;").unwrap(),
                        syn::parse_str("pub static B: u8 = const { 2 };").unwrap(),
                    ]))
                );
                assert_eq!(
                    call(&c, "matches!(x, A)"),
                    Ok(MacroCall::Matches {
                        expr: Box::new(expr("x")),
                        pat: Box::new(Pat::parse_single.parse_str("A").unwrap()),
                        guard: None,
                    })
                );
            }

            #[test]
            fn macro_call_should_report_unknown_macros_and_invalid_arguments() {
                let c = lib("");

                assert_eq!(
                    call(&c, "dbg!(x)"),
                    Err("the arguments of std::dbg! are not known".to_string())
                );
                assert_eq!(
                    call(&c, "dep::m!()"),
                    Err("the arguments of dep::m! are not known".to_string())
                );
                assert_eq!(call(&c, "gone!()"), Err("names no macro".to_string()));
                assert_eq!(call(&c, "vec![1 2]"), Err("expected `,`".to_string()));
                assert_eq!(
                    call(&c, r#"format!("a" b)"#),
                    Err("expected `,`".to_string())
                );

                for text in [
                    "assert!(=)",
                    "format!(1)",
                    r#"format!("a", =)"#,
                    "vec![=]",
                    "vec![0; =]",
                    "vec![1, =]",
                    "matches!(=, x)",
                    "matches!(x y)",
                    "matches!(x, =)",
                    "matches!(x, A if =)",
                    "thread_local!(static)",
                ] {
                    assert!(call(&c, text).is_err(), "{text}");
                }
            }

            #[test]
            fn macro_call_should_expand_macro_rules_by_the_first_rule_that_matches() {
                let c = lib("macro_rules! m {
    (group [a]) => { 0 };
    (one $x:expr) => { $x * 2 };
    (list $($x:literal),* ; $($y:ident)?) => { [$($x),*]; $($y;)? };
    (some $($x:ident)+) => { $($x;)+ };
    (nested $([$($x:ident)*])*) => { $($(let $x = 0;)*)* };
    (krate) => { $crate::f() };
    (tokens (+ 1)) => { 1 };
    (outer $y:ident $($x:ident)*) => { $($y + $x;)* };
}");

                assert_eq!(
                    call(&c, "m!(group (a))"),
                    Err("no rule matches".to_string())
                );
                assert_eq!(
                    call(&c, "m!{group [a]}"),
                    Ok(MacroCall::Expanded(stmts("0")))
                );
                assert!(matches!(
                    &call(&c, "m!(one 1 + 2)").unwrap(),
                    MacroCall::Expanded(stmts)
                        if matches!(&stmts[..], [Stmt::Expr(Expr::Binary(b), None)]
                            if matches!(&*b.left, Expr::Group(_)))
                ));
                assert_eq!(
                    call(&c, "m!(list 1, 2 ; y)"),
                    Ok(MacroCall::Expanded(stmts("[1, 2]; y;")))
                );
                assert_eq!(
                    call(&c, "m!(list ;)"),
                    Ok(MacroCall::Expanded(stmts("[];")))
                );
                assert_eq!(call(&c, "m!(some)"), Err("no rule matches".to_string()));
                assert_eq!(
                    call(&c, "m!(some a b)"),
                    Ok(MacroCall::Expanded(stmts("a; b;")))
                );
                assert_eq!(
                    call(&c, "m!(nested [a b] [] [c])"),
                    Ok(MacroCall::Expanded(stmts(
                        "let a = 0; let b = 0; let c = 0;"
                    )))
                );
                assert_eq!(
                    call(&c, "m!(krate)"),
                    Ok(MacroCall::Expanded(stmts("crate::f()")))
                );
                assert_eq!(
                    call(&c, "m!(tokens (1 1))"),
                    Err("no rule matches".to_string())
                );
                assert_eq!(
                    call(&c, "m!(tokens (+ 1))"),
                    Ok(MacroCall::Expanded(stmts("1")))
                );
                assert_eq!(
                    call(&c, "m!(outer y a b)"),
                    Ok(MacroCall::Expanded(stmts("y + a; y + b;")))
                );
            }

            #[test]
            fn macro_call_should_match_every_fragment_specifier() {
                let c = lib("macro_rules! kinds {
    (block $x:block) => { $x };
    (expr_2021 $x:expr_2021) => { $x };
    (ident $x:ident) => { $x };
    (item $x:item) => { $x };
    (lifetime $x:lifetime) => { let _: &$x u8; };
    (literal $x:literal) => { $x };
    (meta $x:meta) => { #[$x] fn f() {} };
    (pat $x:pat) => { match 0 { $x => {} } };
    (pat_param $x:pat_param) => { let $x = 0; };
    (path $x:path) => { $x };
    (tt $x:tt) => { $x };
    (ty $x:ty) => { let _: $x; };
    (vis $x:vis) => { $x fn g() {} };
}");
                let cases = [
                    ("kinds!(block { 1 })", "{ 1 }"),
                    ("kinds!(ident x)", "x"),
                    ("kinds!(item struct S;)", "struct S;"),
                    ("kinds!(lifetime 'a)", "let _: &'a u8;"),
                    ("kinds!(literal -1)", "-1"),
                    ("kinds!(meta inline)", "#[inline] fn f() {}"),
                    ("kinds!(pat A | B)", "match 0 { A | B => {} }"),
                    ("kinds!(pat_param x)", "let x = 0;"),
                    ("kinds!(path a::B)", "a::B"),
                    ("kinds!(tt (a, b))", "(a, b)"),
                    ("kinds!(ty Vec<u8>)", "let _: Vec<u8>;"),
                    ("kinds!(vis pub)", "pub fn g() {}"),
                ];

                for (text, expansion) in cases {
                    assert_eq!(
                        call(&c, text),
                        Ok(MacroCall::Expanded(stmts(expansion))),
                        "{text}"
                    );
                }

                assert!(matches!(
                    call(&c, "kinds!(expr_2021 a)"),
                    Ok(MacroCall::Expanded(stmts))
                        if matches!(&stmts[..], [Stmt::Expr(Expr::Group(_), None)])
                ));
            }

            #[test]
            fn macro_call_should_report_invalid_macro_rules() {
                let rules = [
                    ("(a) {}", "invalid macro_rules!: expected `=>`"),
                    ("($x) => {}", "$x has no fragment specifier"),
                    ("($x:stmt) => {}", "unsupported fragment specifier stmt"),
                    ("($(a)) => {}", "a repetition has no operator"),
                    ("($(a),) => {}", "a repetition has no operator"),
                    (
                        "($ 1) => {}",
                        "`$` is not followed by a name or a repetition",
                    ),
                    ("($x:ident) => { let }", "the expansion is not statements: "),
                    ("a => {}", "invalid macro_rules!: "),
                    ("(a) => b", "invalid macro_rules!: "),
                    (
                        "(a) => { $ 1 }",
                        "`$` is not followed by a name or a repetition",
                    ),
                    ("($($x)*) => {}", "$x has no fragment specifier"),
                    ("(($x)) => {}", "$x has no fragment specifier"),
                    ("(a) => { ($y) }", "$y is not bound"),
                    ("($($x:ident)*) => { $($x $y)* }", "$y is not bound"),
                    ("($($x:ident)*) => { $x }", "$x is still repeating"),
                    ("(a) => { $y }", "$y is not bound"),
                    ("(a) => { $(a)* }", "a repetition repeats no variable"),
                    (
                        "($($x:ident)* ; $($y:ident)*) => { $($x $y)* }",
                        "the variables of a repetition repeat differently",
                    ),
                ];

                for (rules, error) in rules {
                    let c = lib(&format!("macro_rules! m {{ {rules} }}"));
                    let input = if rules.contains(';') {
                        "m!(a ; b c)"
                    } else {
                        "m!(a)"
                    };
                    let result = call(&c, input);

                    assert!(
                        result.as_ref().is_err_and(|e| e.starts_with(error)),
                        "{rules}: {result:?}"
                    );
                }
            }
        }
    }
}
