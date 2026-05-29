//! Language-agnostic Normalized Expression IR (NExpr).
//!
//! This module transforms the Rust-flavored `Expression` AST into a simplified,
//! heap-allocated intermediate representation that strips all Rust-specific semantics
//! (ownership, borrowing, Result/Option wrappers, method ceremony).
//!
//! Each language backend consumes `Vec<NExpr>` and emits trivial syntax — no semantic
//! decisions remain in the emitter.

use agdb::type_def::{Expression, LiteralValue, MatchArm, Op, Pattern};

// ─────────────────────────────────────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Rules for how Rust methods map to target language constructs.
#[derive(Debug, Clone)]
pub struct MethodMapping {
    /// Methods to strip entirely (receiver passes through): `into`, `clone`, etc.
    pub strip: &'static [&'static str],
    /// Iterator chain methods to strip: `iter`, `collect`, `into_iter`
    pub strip_chain: &'static [&'static str],
    /// Method renames: `(rust_name, target_name)`
    pub rename: &'static [(&'static str, &'static str)],
    /// Methods converted to property access
    pub to_property: &'static [(&'static str, PropertyRule)],
}

/// How a Rust method becomes a property or expression in the target language.
#[derive(Debug, Clone, Copy)]
pub enum PropertyRule {
    /// `.len()` → `.length`
    Direct(&'static str),
    /// `.is_empty()` → `.length === 0`
    Comparison(&'static str, &'static str, &'static str),
    /// `.last()` → `.at(-1)`
    IndexAccess(i32),
}

/// Configuration for the normalization pass.
#[derive(Debug, Clone)]
pub struct NormalizeConfig {
    pub method_mapping: MethodMapping,
    /// Keyword for self-reference: `"this"` for TS/C#, `"self"` for Python
    pub self_keyword: &'static str,
    /// Keyword for null/none: `"null"` for TS/C#, `"None"` for Python
    pub none_keyword: &'static str,
}

/// Default TypeScript normalization configuration.
pub const TYPESCRIPT_NORMALIZE_CONFIG: NormalizeConfig = NormalizeConfig {
    method_mapping: MethodMapping {
        strip: &[
            "into",
            "to_string",
            "clone",
            "unwrap",
            "unwrap_or_default",
            "as_ref",
            "as_str",
            "to_owned",
            "expect",
            "reserve",
        ],
        strip_chain: &["iter", "collect", "into_iter"],
        rename: &[
            ("any", "some"),
            ("all", "every"),
            ("for_each", "forEach"),
            ("contains", "includes"),
            ("starts_with", "startsWith"),
            ("ends_with", "endsWith"),
            ("remove", "splice"),
        ],
        to_property: &[
            ("len", PropertyRule::Direct("length")),
            ("is_empty", PropertyRule::Comparison("length", "===", "0")),
            ("last", PropertyRule::IndexAccess(-1)),
            ("last_mut", PropertyRule::IndexAccess(-1)),
        ],
    },
    self_keyword: "this",
    none_keyword: "null",
};

// ─────────────────────────────────────────────────────────────────────────────
// Normalized Expression IR
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum NLiteral {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NAssignOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NExpr {
    Literal(NLiteral),
    Ident(String),
    Array(Vec<NExpr>),
    Tuple(Vec<NExpr>),
    Binary {
        op: NOp,
        left: Box<NExpr>,
        right: Box<NExpr>,
    },
    Unary {
        op: NOp,
        expr: Box<NExpr>,
    },
    Assign {
        target: Box<NExpr>,
        value: Box<NExpr>,
    },
    CompoundAssign {
        op: NAssignOp,
        target: Box<NExpr>,
        value: Box<NExpr>,
    },
    FieldAccess {
        base: Box<NExpr>,
        field: String,
    },
    Index {
        base: Box<NExpr>,
        index: Box<NExpr>,
    },
    TupleAccess {
        base: Box<NExpr>,
        index: u32,
    },
    /// Method or function call.
    Call {
        receiver: Option<Box<NExpr>>,
        method: String,
        args: Vec<NExpr>,
    },
    /// Property access (no parens): `.length`, `.at(-1)` etc.
    PropertyAccess {
        base: Box<NExpr>,
        property: String,
    },
    /// Struct literal: `new Foo({ field1: val1, ... })`
    Construct {
        name: String,
        args: Vec<NExpr>,
    },
    /// Closure / arrow function.
    Closure {
        params: Vec<String>,
        body: Vec<NExpr>,
        is_async: bool,
    },
    Block(Vec<NExpr>),

    // Control flow
    If {
        condition: Box<NExpr>,
        then_branch: Vec<NExpr>,
        else_branch: Option<Box<NExpr>>,
    },
    Match {
        scrutinee: Box<NExpr>,
        arms: Vec<NMatchArm>,
    },
    For {
        binding: String,
        iterable: Box<NExpr>,
        body: Vec<NExpr>,
    },
    While {
        condition: Box<NExpr>,
        body: Vec<NExpr>,
    },

    // Value-level
    Return(Option<Box<NExpr>>),
    Throw(Box<NExpr>),
    Await(Box<NExpr>),
    StringInterpolation {
        parts: Vec<StringPart>,
    },
    NullCheck(Box<NExpr>),

    // Declarations
    Let {
        name: String,
        value: Option<Box<NExpr>>,
    },
    LetTuple {
        names: Vec<String>,
        value: Box<NExpr>,
    },

    // Control
    Break,
    Continue,

    // Cast (for `.into()` calls where type coercion is needed)
    Cast {
        expr: Box<NExpr>,
        as_type: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct NMatchArm {
    pub pattern: NPattern,
    pub guard: Option<Box<NExpr>>,
    pub body: Vec<NExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NPattern {
    Literal(NLiteral),
    Ident(String),
    Constructor {
        name: String,
        fields: Vec<NPattern>,
    },
    Tuple(Vec<NPattern>),
    Or(Vec<NPattern>),
    Wild,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Expr(NExpr),
}

// ─────────────────────────────────────────────────────────────────────────────
// Normalization Pass
// ─────────────────────────────────────────────────────────────────────────────

/// Normalize a function body (slice of Expressions) into the language-agnostic IR.
pub fn normalize_body(body: &[Expression], config: &NormalizeConfig) -> Vec<NExpr> {
    body.iter().map(|expr| normalize_expr(expr, config)).collect()
}

fn normalize_expr(expr: &Expression, config: &NormalizeConfig) -> NExpr {
    match expr {
        Expression::Literal(lit) => NExpr::Literal(normalize_literal(lit)),
        Expression::Ident(name) => {
            if *name == "self" {
                NExpr::Ident(config.self_keyword.to_string())
            } else if *name == "None" {
                NExpr::Literal(NLiteral::Null)
            } else if *name == "Self" {
                NExpr::Ident(config.self_keyword.to_string())
            } else {
                NExpr::Ident(name.to_string())
            }
        }
        Expression::Wild => NExpr::Ident("_".to_string()),
        Expression::Array(elems) => {
            NExpr::Array(elems.iter().map(|e| normalize_expr(e, config)).collect())
        }
        Expression::Tuple(elems) => {
            NExpr::Tuple(elems.iter().map(|e| normalize_expr(e, config)).collect())
        }
        Expression::Binary { op, left, right } => normalize_binary(op, left, right, config),
        Expression::Unary { op, expr } => normalize_unary(op, expr, config),
        Expression::Assign { target, value } => NExpr::Assign {
            target: Box::new(normalize_expr(target, config)),
            value: Box::new(normalize_expr(value, config)),
        },
        Expression::Let { name, value, .. } => normalize_let(name, value, config),
        Expression::Call {
            recipient,
            function,
            args,
        } => normalize_call(recipient, function, args, config),
        Expression::FieldAccess { base, field } => NExpr::FieldAccess {
            base: Box::new(normalize_expr(base, config)),
            field: field.to_string(),
        },
        Expression::TupleAccess { base, index } => NExpr::TupleAccess {
            base: Box::new(normalize_expr(base, config)),
            index: *index,
        },
        Expression::Index { base, index } => NExpr::Index {
            base: Box::new(normalize_expr(base, config)),
            index: Box::new(normalize_expr(index, config)),
        },
        Expression::If {
            condition,
            then_branch,
            else_branch,
        } => normalize_if(condition, then_branch, else_branch, config),
        Expression::Match { scrutinee, arms } => normalize_match(scrutinee, arms, config),
        Expression::While { condition, body } => NExpr::While {
            condition: Box::new(normalize_expr(condition, config)),
            body: normalize_block_to_vec(body, config),
        },
        Expression::For {
            pattern,
            iterable,
            body,
        } => NExpr::For {
            binding: extract_ident(pattern),
            iterable: Box::new(normalize_expr(iterable, config)),
            body: normalize_block_to_vec(body, config),
        },
        Expression::Block(stmts) => {
            NExpr::Block(stmts.iter().map(|s| normalize_expr(s, config)).collect())
        }
        Expression::Break => NExpr::Break,
        Expression::Continue => NExpr::Continue,
        Expression::Return(value) => normalize_return(value, config),
        Expression::Await(expr) => NExpr::Await(Box::new(normalize_expr(expr, config))),
        Expression::Reference(expr) => normalize_expr(expr, config),
        Expression::Try(expr) => normalize_expr(expr, config),
        Expression::Closure(func) => NExpr::Closure {
            params: func
                .args
                .iter()
                .filter(|a| a.name != "self" && a.name != "&self")
                .map(|a| a.name.to_string())
                .collect(),
            body: func
                .body
                .iter()
                .map(|e| normalize_expr(e, config))
                .collect(),
            is_async: func.async_fn,
        },
        Expression::Format {
            format_string,
            args,
        } => normalize_format(format_string, args, config),
        Expression::Path {
            ident,
            parent,
            generics: _,
        } => normalize_path(ident, parent, config),
        Expression::Range { start, end, .. } => {
            let args: Vec<NExpr> = [start, end]
                .iter()
                .filter_map(|o| o.map(|e| normalize_expr(e, config)))
                .collect();
            NExpr::Call {
                receiver: None,
                method: "range".to_string(),
                args,
            }
        }
        Expression::Struct { name, fields } => {
            let type_name = extract_ident(name);
            NExpr::Construct {
                name: type_name,
                args: fields
                    .iter()
                    .map(|(_field_name, expr)| normalize_expr(expr, config))
                    .collect(),
            }
        }
        Expression::StructPattern { name, fields } => {
            let type_name = extract_ident(name);
            NExpr::Construct {
                name: type_name,
                args: fields
                    .iter()
                    .map(|expr| normalize_expr(expr, config))
                    .collect(),
            }
        }
        Expression::TupleStruct { name, expressions } => {
            normalize_tuple_struct(name, expressions, config)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn normalize_literal(lit: &LiteralValue) -> NLiteral {
    match lit {
        LiteralValue::Bool(v) => NLiteral::Bool(*v),
        LiteralValue::I8(v) => NLiteral::Integer(*v as i64),
        LiteralValue::I16(v) => NLiteral::Integer(*v as i64),
        LiteralValue::I32(v) => NLiteral::Integer(*v as i64),
        LiteralValue::I64(v) => NLiteral::Integer(*v),
        LiteralValue::U8(v) => NLiteral::Integer(*v as i64),
        LiteralValue::U16(v) => NLiteral::Integer(*v as i64),
        LiteralValue::U32(v) => NLiteral::Integer(*v as i64),
        LiteralValue::U64(v) => NLiteral::Integer(*v as i64),
        LiteralValue::Usize(v) => NLiteral::Integer(*v as i64),
        LiteralValue::F32(v) => NLiteral::Float(*v as f64),
        LiteralValue::F64(v) => NLiteral::Float(*v),
        LiteralValue::Str(s) => NLiteral::String(s.to_string()),
        LiteralValue::String(s) => NLiteral::String(s.clone()),
        LiteralValue::Unit => NLiteral::Null,
    }
}

fn normalize_binary(
    op: &Op,
    left: &Expression,
    right: &Expression,
    config: &NormalizeConfig,
) -> NExpr {
    if let Some(assign_op) = to_assign_op(op) {
        return NExpr::CompoundAssign {
            op: assign_op,
            target: Box::new(normalize_expr(left, config)),
            value: Box::new(normalize_expr(right, config)),
        };
    }
    let nop = to_nop(op);
    NExpr::Binary {
        op: nop,
        left: Box::new(normalize_expr(left, config)),
        right: Box::new(normalize_expr(right, config)),
    }
}

fn normalize_unary(op: &Op, expr: &Expression, config: &NormalizeConfig) -> NExpr {
    match op {
        Op::Deref => normalize_expr(expr, config),
        _ => NExpr::Unary {
            op: to_nop(op),
            expr: Box::new(normalize_expr(expr, config)),
        },
    }
}

fn normalize_let(
    name: &Expression,
    value: &Option<&'static Expression>,
    config: &NormalizeConfig,
) -> NExpr {
    match name {
        Expression::Tuple(elems) => {
            let names: Vec<String> = elems.iter().map(extract_ident).collect();
            NExpr::LetTuple {
                names,
                value: Box::new(
                    value
                        .map(|v| normalize_expr(v, config))
                        .unwrap_or(NExpr::Literal(NLiteral::Null)),
                ),
            }
        }
        _ => NExpr::Let {
            name: extract_ident(name),
            value: value.map(|v| Box::new(normalize_expr(v, config))),
        },
    }
}

fn normalize_call(
    recipient: &Option<&'static Expression>,
    function: &Expression,
    args: &[Expression],
    config: &NormalizeConfig,
) -> NExpr {
    let method_name = extract_method_name(function);

    if let Some(recv_expr) = recipient {
        // Method call
        let method = method_name.as_str();

        // Strip methods: return receiver only
        if config.method_mapping.strip.contains(&method) {
            if method == "into" {
                return NExpr::Cast {
                    expr: Box::new(normalize_expr(recv_expr, config)),
                    as_type: "any".to_string(),
                };
            }
            return normalize_expr(recv_expr, config);
        }

        // Strip chain methods
        if config.method_mapping.strip_chain.contains(&method) {
            return normalize_expr(recv_expr, config);
        }

        // Property methods
        if let Some((_, rule)) = config
            .method_mapping
            .to_property
            .iter()
            .find(|(name, _)| *name == method)
        {
            let base = normalize_expr(recv_expr, config);
            return apply_property_rule(base, rule);
        }

        // Rename
        let final_name = config
            .method_mapping
            .rename
            .iter()
            .find(|(from, _)| *from == method)
            .map(|(_, to)| *to)
            .unwrap_or(method);

        NExpr::Call {
            receiver: Some(Box::new(normalize_expr(recv_expr, config))),
            method: final_name.to_string(),
            args: args.iter().map(|a| normalize_expr(a, config)).collect(),
        }
    } else {
        // Free function call
        normalize_free_call(function, &method_name, args, config)
    }
}

fn normalize_free_call(
    function: &Expression,
    method_name: &str,
    args: &[Expression],
    config: &NormalizeConfig,
) -> NExpr {
    // Check for UFCS patterns: Into::into(x), From::from(x)
    if let Expression::Path {
        ident, parent: Some(parent), ..
    } = function
    {
        let parent_name = extract_ident(parent);
        if parent_name == "Into" && *ident == "into" && args.len() == 1 {
            return NExpr::Cast {
                expr: Box::new(normalize_expr(&args[0], config)),
                as_type: "any".to_string(),
            };
        }
        if parent_name == "Default" && *ident == "default" {
            return NExpr::Construct {
                name: "Default".to_string(),
                args: vec![],
            };
        }
        // Stripped UFCS methods
        if config.method_mapping.strip.contains(ident) && args.len() == 1 {
            return normalize_expr(&args[0], config);
        }
    }

    // Result constructors: Ok(x), Err(x), Some(x)
    match method_name {
        "Ok" | "Some" => {
            if args.len() == 1 {
                return normalize_expr(&args[0], config);
            }
            // Multiple args → tuple
            return NExpr::Tuple(args.iter().map(|a| normalize_expr(a, config)).collect());
        }
        "Err" => {
            if args.len() == 1 {
                return NExpr::Throw(Box::new(normalize_expr(&args[0], config)));
            }
            return NExpr::Throw(Box::new(NExpr::Literal(NLiteral::Null)));
        }
        _ => {}
    }

    NExpr::Call {
        receiver: None,
        method: method_name.to_string(),
        args: args.iter().map(|a| normalize_expr(a, config)).collect(),
    }
}

fn normalize_tuple_struct(
    name: &Expression,
    expressions: &[Expression],
    config: &NormalizeConfig,
) -> NExpr {
    let type_name = extract_ident(name);
    match type_name.as_str() {
        "Ok" | "Some" => {
            if expressions.len() == 1 {
                normalize_expr(&expressions[0], config)
            } else if expressions.is_empty() {
                NExpr::Literal(NLiteral::Null)
            } else {
                NExpr::Tuple(
                    expressions
                        .iter()
                        .map(|e| normalize_expr(e, config))
                        .collect(),
                )
            }
        }
        "Err" => {
            if expressions.len() == 1 {
                NExpr::Throw(Box::new(normalize_expr(&expressions[0], config)))
            } else {
                NExpr::Throw(Box::new(NExpr::Literal(NLiteral::Null)))
            }
        }
        _ => NExpr::Construct {
            name: type_name,
            args: expressions
                .iter()
                .map(|e| normalize_expr(e, config))
                .collect(),
        },
    }
}

fn normalize_return(
    value: &Option<&'static Expression>,
    config: &NormalizeConfig,
) -> NExpr {
    match value {
        Some(expr) => {
            // Check if returning Ok(x) or Err(x)
            match expr {
                Expression::TupleStruct { name, expressions } => {
                    let type_name = extract_ident(name);
                    match type_name.as_str() {
                        "Ok" | "Some" if expressions.len() == 1 => {
                            NExpr::Return(Some(Box::new(normalize_expr(&expressions[0], config))))
                        }
                        "Ok" | "Some" if expressions.is_empty() => NExpr::Return(None),
                        "Err" if expressions.len() == 1 => {
                            NExpr::Throw(Box::new(normalize_expr(&expressions[0], config)))
                        }
                        _ => NExpr::Return(Some(Box::new(normalize_tuple_struct(
                            name,
                            expressions,
                            config,
                        )))),
                    }
                }
                Expression::Call {
                    recipient: None,
                    function,
                    args,
                } => {
                    let name = extract_method_name(function);
                    match name.as_str() {
                        "Ok" | "Some" if args.len() == 1 => {
                            NExpr::Return(Some(Box::new(normalize_expr(&args[0], config))))
                        }
                        "Err" if args.len() == 1 => {
                            NExpr::Throw(Box::new(normalize_expr(&args[0], config)))
                        }
                        _ => NExpr::Return(Some(Box::new(normalize_expr(expr, config)))),
                    }
                }
                _ => NExpr::Return(Some(Box::new(normalize_expr(expr, config)))),
            }
        }
        None => NExpr::Return(None),
    }
}

fn normalize_if(
    condition: &Expression,
    then_branch: &Expression,
    else_branch: &Option<&'static Expression>,
    config: &NormalizeConfig,
) -> NExpr {
    let cond = match condition {
        Expression::Let { name: _, value: Some(val), .. } => {
            NExpr::NullCheck(Box::new(normalize_expr(val, config)))
        }
        _ => normalize_expr(condition, config),
    };
    let then_body = normalize_block_to_vec(then_branch, config);
    let else_body = else_branch.map(|eb| Box::new(normalize_expr(eb, config)));
    NExpr::If {
        condition: Box::new(cond),
        then_branch: then_body,
        else_branch: else_body,
    }
}

fn normalize_match(
    scrutinee: &Expression,
    arms: &[MatchArm],
    config: &NormalizeConfig,
) -> NExpr {
    NExpr::Match {
        scrutinee: Box::new(normalize_expr(scrutinee, config)),
        arms: arms.iter().map(|arm| normalize_match_arm(arm, config)).collect(),
    }
}

fn normalize_match_arm(arm: &MatchArm, config: &NormalizeConfig) -> NMatchArm {
    NMatchArm {
        pattern: normalize_pattern(&arm.pattern),
        guard: arm.guard.map(|g| Box::new(normalize_expr(g, config))),
        body: normalize_block_to_vec(arm.body, config),
    }
}

fn normalize_pattern(pat: &Pattern) -> NPattern {
    match pat {
        Pattern::Wild => NPattern::Wild,
        Pattern::Ident(name) => NPattern::Ident(name.to_string()),
        Pattern::Literal(lit) => NPattern::Literal(normalize_literal(lit)),
        Pattern::Constructor { name, fields } => NPattern::Constructor {
            name: name.to_string(),
            fields: fields.iter().map(normalize_pattern).collect(),
        },
        Pattern::Tuple(pats) => NPattern::Tuple(pats.iter().map(normalize_pattern).collect()),
        Pattern::Or(pats) => NPattern::Or(pats.iter().map(normalize_pattern).collect()),
        Pattern::Struct { name, fields } => NPattern::Constructor {
            name: name.to_string(),
            fields: fields
                .iter()
                .map(|(_field_name, pat)| normalize_pattern(pat))
                .collect(),
        },
    }
}

fn normalize_format(
    format_string: &str,
    args: &[Expression],
    config: &NormalizeConfig,
) -> NExpr {
    let mut parts = Vec::new();
    let mut arg_iter = args.iter();
    let mut current_literal = String::new();

    let mut chars = format_string.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                current_literal.push('{');
            } else {
                // Consume until '}'
                let mut placeholder = String::new();
                while let Some(c) = chars.next() {
                    if c == '}' {
                        break;
                    }
                    placeholder.push(c);
                }
                if !current_literal.is_empty() {
                    parts.push(StringPart::Literal(std::mem::take(&mut current_literal)));
                }
                // Check if placeholder is a named variable or positional
                let placeholder_name = placeholder.split(':').next().unwrap_or("").trim();
                if placeholder_name.is_empty() {
                    // Positional arg
                    if let Some(arg) = arg_iter.next() {
                        parts.push(StringPart::Expr(normalize_expr(arg, config)));
                    }
                } else {
                    // Named variable reference
                    parts.push(StringPart::Expr(NExpr::Ident(
                        placeholder_name.to_string(),
                    )));
                }
            }
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                chars.next();
                current_literal.push('}');
            } else {
                current_literal.push('}');
            }
        } else {
            current_literal.push(ch);
        }
    }
    if !current_literal.is_empty() {
        parts.push(StringPart::Literal(current_literal));
    }

    NExpr::StringInterpolation { parts }
}

fn normalize_path(
    ident: &str,
    parent: &Option<&'static Expression>,
    config: &NormalizeConfig,
) -> NExpr {
    if let Some(parent_expr) = parent {
        let parent_name = extract_ident(parent_expr);
        if parent_name == "Self" || parent_name == "self" {
            NExpr::FieldAccess {
                base: Box::new(NExpr::Ident(config.self_keyword.to_string())),
                field: ident.to_string(),
            }
        } else {
            NExpr::FieldAccess {
                base: Box::new(NExpr::Ident(parent_name)),
                field: ident.to_string(),
            }
        }
    } else {
        NExpr::Ident(ident.to_string())
    }
}

fn normalize_block_to_vec(expr: &Expression, config: &NormalizeConfig) -> Vec<NExpr> {
    match expr {
        Expression::Block(stmts) => stmts.iter().map(|s| normalize_expr(s, config)).collect(),
        _ => vec![normalize_expr(expr, config)],
    }
}

fn apply_property_rule(base: NExpr, rule: &PropertyRule) -> NExpr {
    match rule {
        PropertyRule::Direct(prop) => NExpr::PropertyAccess {
            base: Box::new(base),
            property: prop.to_string(),
        },
        PropertyRule::Comparison(prop, op, val) => NExpr::Binary {
            op: match *op {
                "===" => NOp::Eq,
                "!==" => NOp::Ne,
                _ => NOp::Eq,
            },
            left: Box::new(NExpr::PropertyAccess {
                base: Box::new(base),
                property: prop.to_string(),
            }),
            right: Box::new(NExpr::Literal(NLiteral::Integer(
                val.parse().unwrap_or(0),
            ))),
        },
        PropertyRule::IndexAccess(idx) => NExpr::Call {
            receiver: Some(Box::new(base)),
            method: "at".to_string(),
            args: vec![NExpr::Literal(NLiteral::Integer(*idx as i64))],
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Utility
// ─────────────────────────────────────────────────────────────────────────────

fn extract_ident(expr: &Expression) -> String {
    match expr {
        Expression::Ident(name) => name.to_string(),
        Expression::Path { ident, .. } => ident.to_string(),
        Expression::Tuple(elems) if elems.len() == 1 => extract_ident(&elems[0]),
        _ => "_".to_string(),
    }
}

fn extract_method_name(function: &Expression) -> String {
    match function {
        Expression::Ident(name) => name.to_string(),
        Expression::Path { ident, .. } => ident.to_string(),
        _ => "_unknown_".to_string(),
    }
}

fn to_nop(op: &Op) -> NOp {
    match op {
        Op::Add | Op::AddAssign => NOp::Add,
        Op::Sub | Op::SubAssign => NOp::Sub,
        Op::Mul | Op::MulAssign => NOp::Mul,
        Op::Div | Op::DivAssign => NOp::Div,
        Op::Rem | Op::RemAssign => NOp::Rem,
        Op::Eq => NOp::Eq,
        Op::Ne => NOp::Ne,
        Op::Lt => NOp::Lt,
        Op::Le => NOp::Le,
        Op::Gt => NOp::Gt,
        Op::Ge => NOp::Ge,
        Op::And => NOp::And,
        Op::Or => NOp::Or,
        Op::BitAnd | Op::BitAndAssign => NOp::BitAnd,
        Op::BitOr | Op::BitOrAssign => NOp::BitOr,
        Op::BitXor | Op::BitXorAssign => NOp::BitXor,
        Op::Shl | Op::ShlAssign => NOp::Shl,
        Op::Shr | Op::ShrAssign => NOp::Shr,
        Op::Not => NOp::Not,
        Op::Neg => NOp::Neg,
        Op::Deref => NOp::Neg, // shouldn't be reached
    }
}

fn to_assign_op(op: &Op) -> Option<NAssignOp> {
    match op {
        Op::AddAssign => Some(NAssignOp::Add),
        Op::SubAssign => Some(NAssignOp::Sub),
        Op::MulAssign => Some(NAssignOp::Mul),
        Op::DivAssign => Some(NAssignOp::Div),
        Op::RemAssign => Some(NAssignOp::Rem),
        Op::BitAndAssign => Some(NAssignOp::BitAnd),
        Op::BitOrAssign => Some(NAssignOp::BitOr),
        Op::BitXorAssign => Some(NAssignOp::BitXor),
        Op::ShlAssign => Some(NAssignOp::Shl),
        Op::ShrAssign => Some(NAssignOp::Shr),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_literal() {
        assert_eq!(normalize_literal(&LiteralValue::I32(42)), NLiteral::Integer(42));
        assert_eq!(normalize_literal(&LiteralValue::Bool(true)), NLiteral::Bool(true));
        assert_eq!(
            normalize_literal(&LiteralValue::Str("hello")),
            NLiteral::String("hello".to_string())
        );
        assert_eq!(normalize_literal(&LiteralValue::Unit), NLiteral::Null);
    }

    #[test]
    fn test_normalize_op_mapping() {
        assert_eq!(to_nop(&Op::Add), NOp::Add);
        assert_eq!(to_nop(&Op::Eq), NOp::Eq);
        assert_eq!(to_nop(&Op::And), NOp::And);
        assert_eq!(to_nop(&Op::Not), NOp::Not);
    }

    #[test]
    fn test_assign_op_mapping() {
        assert_eq!(to_assign_op(&Op::AddAssign), Some(NAssignOp::Add));
        assert_eq!(to_assign_op(&Op::SubAssign), Some(NAssignOp::Sub));
        assert_eq!(to_assign_op(&Op::Add), None);
    }

    #[test]
    fn test_property_rule_direct() {
        let base = NExpr::Ident("arr".to_string());
        let result = apply_property_rule(base, &PropertyRule::Direct("length"));
        assert_eq!(
            result,
            NExpr::PropertyAccess {
                base: Box::new(NExpr::Ident("arr".to_string())),
                property: "length".to_string(),
            }
        );
    }

    #[test]
    fn test_property_rule_comparison() {
        let base = NExpr::Ident("arr".to_string());
        let result = apply_property_rule(base, &PropertyRule::Comparison("length", "===", "0"));
        assert!(matches!(result, NExpr::Binary { op: NOp::Eq, .. }));
    }

    #[test]
    fn test_property_rule_index_access() {
        let base = NExpr::Ident("arr".to_string());
        let result = apply_property_rule(base, &PropertyRule::IndexAccess(-1));
        assert!(matches!(result, NExpr::Call { method, .. } if method == "at"));
    }
}
