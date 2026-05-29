//! NExpr → TypeScript emitter.
//!
//! This module emits TypeScript code from the language-agnostic NExpr IR.
//! It contains NO semantic logic — just trivial syntax mapping.

use super::format::IndentWriter;
use crate::language::normalize_expr::*;

pub fn emit_body(body: &[NExpr], w: &mut IndentWriter) {
    for expr in body {
        emit_statement(expr, w);
    }
}

pub fn emit_statement(expr: &NExpr, w: &mut IndentWriter) {
    match expr {
        NExpr::If { .. } | NExpr::While { .. } | NExpr::For { .. } | NExpr::Match { .. } => {
            emit_nexpr(expr, w);
            w.newline();
        }
        NExpr::Block(stmts) => {
            emit_block(stmts, w);
            w.newline();
        }
        NExpr::Let { .. } | NExpr::LetTuple { .. } => {
            emit_nexpr(expr, w);
            w.write(";");
            w.newline();
        }
        NExpr::Throw(_) => {
            emit_nexpr(expr, w);
            w.write(";");
            w.newline();
        }
        NExpr::Return(_) | NExpr::Break | NExpr::Continue => {
            emit_nexpr(expr, w);
            w.write(";");
            w.newline();
        }
        _ => {
            emit_nexpr(expr, w);
            w.write(";");
            w.newline();
        }
    }
}

pub fn emit_nexpr(expr: &NExpr, w: &mut IndentWriter) {
    match expr {
        NExpr::Literal(lit) => emit_literal(lit, w),
        NExpr::Ident(name) => {
            w.write(name);
        }
        NExpr::Array(elems) => {
            w.write("[");
            for (i, elem) in elems.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                emit_nexpr(elem, w);
            }
            w.write("]");
        }
        NExpr::Tuple(elems) => {
            w.write("[");
            for (i, elem) in elems.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                emit_nexpr(elem, w);
            }
            w.write("]");
        }
        NExpr::Binary { op, left, right } => {
            emit_nexpr(left, w);
            w.write(" ");
            w.write(nop_str(op));
            w.write(" ");
            emit_nexpr(right, w);
        }
        NExpr::Unary { op, expr } => {
            w.write(nop_str(op));
            emit_nexpr(expr, w);
        }
        NExpr::Assign { target, value } => {
            emit_nexpr(target, w);
            w.write(" = ");
            emit_nexpr(value, w);
        }
        NExpr::CompoundAssign { op, target, value } => {
            emit_nexpr(target, w);
            w.write(" ");
            w.write(nassign_op_str(op));
            w.write(" ");
            emit_nexpr(value, w);
        }
        NExpr::FieldAccess { base, field } => {
            emit_nexpr(base, w);
            w.write(".");
            w.write(field);
        }
        NExpr::Index { base, index } => {
            emit_nexpr(base, w);
            w.write("[");
            emit_nexpr(index, w);
            w.write("]");
        }
        NExpr::TupleAccess { base, index } => {
            emit_nexpr(base, w);
            w.write("[");
            w.write(&index.to_string());
            w.write("]");
        }
        NExpr::Call {
            receiver,
            method,
            args,
        } => emit_call(receiver, method, args, w),
        NExpr::PropertyAccess { base, property } => {
            emit_nexpr(base, w);
            w.write(".");
            w.write(property);
        }
        NExpr::Construct { name, args } => {
            w.write("new ");
            w.write(name);
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                emit_nexpr(arg, w);
            }
            w.write(")");
        }
        NExpr::Closure {
            params,
            body,
            is_async,
        } => {
            if *is_async {
                w.write("async ");
            }
            w.write("(");
            w.write(&params.join(", "));
            w.write(") => ");
            if body.len() == 1 {
                emit_nexpr(&body[0], w);
            } else {
                emit_block(body, w);
            }
        }
        NExpr::Block(stmts) => emit_block_as_iife(stmts, w),
        NExpr::If {
            condition,
            then_branch,
            else_branch,
        } => emit_if(condition, then_branch, else_branch, w),
        NExpr::Match { scrutinee, arms } => emit_match(scrutinee, arms, w),
        NExpr::For {
            binding,
            iterable,
            body,
        } => {
            w.write("for (const ");
            w.write(binding);
            w.write(" of ");
            emit_nexpr(iterable, w);
            w.write(") ");
            emit_block(body, w);
        }
        NExpr::While { condition, body } => {
            w.write("while (");
            emit_nexpr(condition, w);
            w.write(") ");
            emit_block(body, w);
        }
        NExpr::Return(value) => {
            if let Some(val) = value {
                w.write("return ");
                emit_nexpr(val, w);
            } else {
                w.write("return");
            }
        }
        NExpr::Throw(expr) => {
            w.write("throw ");
            emit_nexpr(expr, w);
        }
        NExpr::Await(expr) => {
            w.write("await ");
            emit_nexpr(expr, w);
        }
        NExpr::StringInterpolation { parts } => emit_template_literal(parts, w),
        NExpr::NullCheck(expr) => {
            emit_nexpr(expr, w);
            w.write(" != null");
        }
        NExpr::Let { name, value } => {
            w.write("let ");
            w.write(name);
            if let Some(val) = value {
                w.write(" = ");
                emit_nexpr(val, w);
            }
        }
        NExpr::LetTuple { names, value } => {
            w.write("let [");
            w.write(&names.join(", "));
            w.write("] = ");
            emit_nexpr(value, w);
        }
        NExpr::Break => w.write("break"),
        NExpr::Continue => w.write("continue"),
        NExpr::Cast { expr, as_type } => {
            w.write("(");
            emit_nexpr(expr, w);
            w.write(" as ");
            w.write(as_type);
            w.write(")");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn emit_literal(lit: &NLiteral, w: &mut IndentWriter) {
    match lit {
        NLiteral::Bool(v) => w.write(if *v { "true" } else { "false" }),
        NLiteral::Integer(v) => w.write(&v.to_string()),
        NLiteral::Float(v) => {
            let s = v.to_string();
            if s.contains('.') {
                w.write(&s);
            } else {
                w.write(&format!("{s}.0"));
            }
        }
        NLiteral::String(s) => {
            w.write("\"");
            w.write(&s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"));
            w.write("\"");
        }
        NLiteral::Null => w.write("null"),
    }
}

fn emit_call(
    receiver: &Option<Box<NExpr>>,
    method: &str,
    args: &[NExpr],
    w: &mut IndentWriter,
) {
    if let Some(recv) = receiver {
        emit_nexpr(recv, w);
        w.write(".");
    }
    w.write(method);
    w.write("(");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        emit_nexpr(arg, w);
    }
    w.write(")");
}

fn emit_if(
    condition: &NExpr,
    then_branch: &[NExpr],
    else_branch: &Option<Box<NExpr>>,
    w: &mut IndentWriter,
) {
    w.write("if (");
    emit_nexpr(condition, w);
    w.write(") ");
    emit_block(then_branch, w);
    if let Some(else_expr) = else_branch {
        match else_expr.as_ref() {
            NExpr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                w.write(" else ");
                emit_if(condition, then_branch, else_branch, w);
            }
            NExpr::Block(stmts) => {
                w.write(" else ");
                emit_block(stmts, w);
            }
            _ => {
                w.write(" else ");
                emit_block(&[else_expr.as_ref().clone()], w);
            }
        }
    }
}

fn emit_match(scrutinee: &NExpr, arms: &[NMatchArm], w: &mut IndentWriter) {
    let mut first = true;
    let mut wild_arm: Option<&NMatchArm> = None;

    for arm in arms {
        if matches!(arm.pattern, NPattern::Wild) {
            wild_arm = Some(arm);
            continue;
        }
        if matches!(arm.pattern, NPattern::Ident(_)) && arm.guard.is_none() {
            wild_arm = Some(arm);
            continue;
        }
        // Err constructor arms become catch-all
        if let NPattern::Constructor { name, .. } = &arm.pattern {
            if name == "Err" {
                wild_arm = Some(arm);
                continue;
            }
        }

        if first {
            w.write("if (");
        } else {
            w.write(" else if (");
        }
        emit_match_condition(scrutinee, &arm.pattern, &arm.guard, w);
        w.write(") ");
        emit_block(&arm.body, w);
        first = false;
    }

    if let Some(wild) = wild_arm {
        if first {
            emit_block(&wild.body, w);
        } else {
            w.write(" else ");
            emit_block(&wild.body, w);
        }
    }
}

fn emit_match_condition(
    scrutinee: &NExpr,
    pattern: &NPattern,
    guard: &Option<Box<NExpr>>,
    w: &mut IndentWriter,
) {
    match pattern {
        NPattern::Literal(lit) => {
            emit_nexpr(scrutinee, w);
            w.write(" === ");
            emit_literal(lit, w);
        }
        NPattern::Constructor { name, fields } => {
            if name == "Some" && fields.len() == 1 {
                emit_nexpr(scrutinee, w);
                w.write(" != null");
            } else if name == "None" {
                emit_nexpr(scrutinee, w);
                w.write(" == null");
            } else if name == "Ok" && fields.len() == 1 {
                match &fields[0] {
                    NPattern::Literal(lit) => {
                        emit_nexpr(scrutinee, w);
                        w.write(" === ");
                        emit_literal(lit, w);
                    }
                    _ => w.write("true"),
                }
            } else {
                emit_nexpr(scrutinee, w);
            }
        }
        NPattern::Or(patterns) => {
            for (i, pat) in patterns.iter().enumerate() {
                if i > 0 {
                    w.write(" || ");
                }
                emit_match_condition(scrutinee, pat, &None, w);
            }
        }
        _ => w.write("true"),
    }
    if let Some(guard_expr) = guard {
        w.write(" && ");
        emit_nexpr(guard_expr, w);
    }
}

fn emit_template_literal(parts: &[StringPart], w: &mut IndentWriter) {
    w.write("`");
    for part in parts {
        match part {
            StringPart::Literal(s) => w.write(s),
            StringPart::Expr(expr) => {
                w.write("${");
                emit_nexpr(expr, w);
                w.write("}");
            }
        }
    }
    w.write("`");
}

fn emit_block(stmts: &[NExpr], w: &mut IndentWriter) {
    w.write("{");
    w.newline();
    w.indent();
    emit_body(stmts, w);
    w.dedent();
    w.write("}");
}

fn emit_block_as_iife(stmts: &[NExpr], w: &mut IndentWriter) {
    w.write("(() => ");
    emit_block(stmts, w);
    w.write(")()");
}

fn nop_str(op: &NOp) -> &'static str {
    match op {
        NOp::Add => "+",
        NOp::Sub => "-",
        NOp::Mul => "*",
        NOp::Div => "/",
        NOp::Rem => "%",
        NOp::Eq => "===",
        NOp::Ne => "!==",
        NOp::Lt => "<",
        NOp::Le => "<=",
        NOp::Gt => ">",
        NOp::Ge => ">=",
        NOp::And => "&&",
        NOp::Or => "||",
        NOp::BitAnd => "&",
        NOp::BitOr => "|",
        NOp::BitXor => "^",
        NOp::Shl => "<<",
        NOp::Shr => ">>",
        NOp::Not => "!",
        NOp::Neg => "-",
    }
}

fn nassign_op_str(op: &NAssignOp) -> &'static str {
    match op {
        NAssignOp::Add => "+=",
        NAssignOp::Sub => "-=",
        NAssignOp::Mul => "*=",
        NAssignOp::Div => "/=",
        NAssignOp::Rem => "%=",
        NAssignOp::BitAnd => "&=",
        NAssignOp::BitOr => "|=",
        NAssignOp::BitXor => "^=",
        NAssignOp::Shl => "<<=",
        NAssignOp::Shr => ">>=",
    }
}
