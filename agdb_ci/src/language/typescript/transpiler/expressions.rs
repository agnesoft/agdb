use agdb::type_def::Expression;
use agdb::type_def::LiteralValue;
use agdb::type_def::Op;

use super::format::IndentWriter;
use super::types::type_annotation;

pub fn emit_expression(expr: &Expression, w: &mut IndentWriter) {
    match expr {
        Expression::Literal(lit) => emit_literal(lit, w),
        Expression::Ident(name) => w.write(name),
        Expression::Wild => w.write("_"),
        Expression::Binary { op, left, right } => emit_binary(op, left, right, w),
        Expression::Unary { op, expr } => emit_unary(op, expr, w),
        Expression::Assign { target, value } => {
            emit_expression(target, w);
            w.write(" = ");
            emit_expression(value, w);
        }
        Expression::Let { name, ty, value } => emit_let(name, ty, value, w),
        Expression::Call {
            recipient,
            function,
            args,
        } => emit_call(recipient, function, args, w),
        Expression::FieldAccess { base, field } => {
            emit_expression(base, w);
            w.write(".");
            w.write(field);
        }
        Expression::TupleAccess { base, index } => {
            emit_expression(base, w);
            w.write(&format!("[{index}]"));
        }
        Expression::Index { base, index } => {
            emit_expression(base, w);
            w.write("[");
            emit_expression(index, w);
            w.write("]");
        }
        Expression::Array(elements) => {
            w.write("[");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                emit_expression(elem, w);
            }
            w.write("]");
        }
        Expression::Tuple(elements) => {
            w.write("[");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                emit_expression(elem, w);
            }
            w.write("]");
        }
        Expression::Struct { name, fields } => emit_struct_literal(name, fields, w),
        Expression::If {
            condition,
            then_branch,
            else_branch,
        } => emit_if(condition, then_branch, else_branch, w),
        Expression::While { condition, body } => {
            w.write("while (");
            emit_expression(condition, w);
            w.write(") ");
            emit_block_expression(body, w);
        }
        Expression::For {
            pattern,
            iterable,
            body,
        } => {
            w.write("for (const ");
            emit_expression(pattern, w);
            w.write(" of ");
            emit_expression(iterable, w);
            w.write(") ");
            emit_block_expression(body, w);
        }
        Expression::Block(stmts) => emit_block(stmts, w),
        Expression::Break => w.write("break"),
        Expression::Continue => w.write("continue"),
        Expression::Return(value) => {
            w.write("return");
            if let Some(expr) = value {
                w.write(" ");
                emit_expression(expr, w);
            }
        }
        Expression::Await(expr) => {
            w.write("await ");
            emit_expression(expr, w);
        }
        Expression::Reference(expr) => emit_expression(expr, w),
        Expression::Try(expr) => emit_expression(expr, w),
        Expression::Closure(func) => emit_closure(func, w),
        Expression::Format {
            format_string,
            args,
        } => emit_format(format_string, args, w),
        Expression::Path {
            ident,
            parent,
            generics: _,
        } => {
            if let Some(parent_expr) = parent {
                emit_expression(parent_expr, w);
                w.write(".");
            }
            w.write(ident);
        }
        Expression::Range {
            start,
            end,
            inclusive,
        } => emit_range(start, end, *inclusive, w),
        Expression::StructPattern { name, fields } => {
            emit_expression(name, w);
            w.write(" { ");
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                emit_expression(field, w);
            }
            w.write(" }");
        }
        Expression::TupleStruct { name, expressions } => {
            emit_expression(name, w);
            w.write("(");
            for (i, expr) in expressions.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                emit_expression(expr, w);
            }
            w.write(")");
        }
    }
}

pub fn emit_statement(expr: &Expression, w: &mut IndentWriter) {
    match expr {
        Expression::If { .. } | Expression::While { .. } | Expression::For { .. } => {
            emit_expression(expr, w);
            w.newline();
        }
        Expression::Block(stmts) => {
            emit_block(stmts, w);
            w.newline();
        }
        Expression::Let { .. } => {
            emit_expression(expr, w);
            w.write(";");
            w.newline();
        }
        Expression::Return(_) => {
            emit_expression(expr, w);
            w.write(";");
            w.newline();
        }
        Expression::Break | Expression::Continue => {
            emit_expression(expr, w);
            w.write(";");
            w.newline();
        }
        Expression::Assign { .. } => {
            emit_expression(expr, w);
            w.write(";");
            w.newline();
        }
        Expression::Binary { op, .. } if is_assign_op(op) => {
            emit_expression(expr, w);
            w.write(";");
            w.newline();
        }
        Expression::Call { .. } => {
            emit_expression(expr, w);
            w.write(";");
            w.newline();
        }
        Expression::Await(_) => {
            emit_expression(expr, w);
            w.write(";");
            w.newline();
        }
        Expression::Try(_) => {
            emit_expression(expr, w);
            w.write(";");
            w.newline();
        }
        _ => {
            emit_expression(expr, w);
            w.write(";");
            w.newline();
        }
    }
}

pub fn emit_body(body: &[Expression], w: &mut IndentWriter) {
    for expr in body {
        emit_statement(expr, w);
    }
}

fn emit_literal(lit: &LiteralValue, w: &mut IndentWriter) {
    match lit {
        LiteralValue::Bool(v) => w.write(if *v { "true" } else { "false" }),
        LiteralValue::I8(v) => w.write(&v.to_string()),
        LiteralValue::I16(v) => w.write(&v.to_string()),
        LiteralValue::I32(v) => w.write(&v.to_string()),
        LiteralValue::I64(v) => w.write(&v.to_string()),
        LiteralValue::U8(v) => w.write(&v.to_string()),
        LiteralValue::U16(v) => w.write(&v.to_string()),
        LiteralValue::U32(v) => w.write(&v.to_string()),
        LiteralValue::U64(v) => w.write(&v.to_string()),
        LiteralValue::Usize(v) => w.write(&v.to_string()),
        LiteralValue::F32(v) => w.write(&format_float(*v as f64)),
        LiteralValue::F64(v) => w.write(&format_float(*v)),
        LiteralValue::Str(s) => w.write(&format!("\"{}\"", escape_string(s))),
        LiteralValue::String(s) => w.write(&format!("\"{}\"", escape_string(s))),
        LiteralValue::Unit => w.write("undefined"),
    }
}

fn format_float(v: f64) -> String {
    let s = v.to_string();
    if s.contains('.') { s } else { format!("{s}.0") }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn emit_binary(op: &Op, left: &Expression, right: &Expression, w: &mut IndentWriter) {
    emit_expression(left, w);
    w.write(" ");
    w.write(op_str(op));
    w.write(" ");
    emit_expression(right, w);
}

fn op_str(op: &Op) -> &'static str {
    match op {
        Op::Add => "+",
        Op::AddAssign => "+=",
        Op::Sub => "-",
        Op::SubAssign => "-=",
        Op::Mul => "*",
        Op::MulAssign => "*=",
        Op::Div => "/",
        Op::DivAssign => "/=",
        Op::Rem => "%",
        Op::RemAssign => "%=",
        Op::And => "&&",
        Op::Or => "||",
        Op::BitXor => "^",
        Op::BitXorAssign => "^=",
        Op::BitAnd => "&",
        Op::BitAndAssign => "&=",
        Op::BitOr => "|",
        Op::BitOrAssign => "|=",
        Op::Shl => "<<",
        Op::ShlAssign => "<<=",
        Op::Shr => ">>",
        Op::ShrAssign => ">>=",
        Op::Eq => "===",
        Op::Ne => "!==",
        Op::Lt => "<",
        Op::Le => "<=",
        Op::Gt => ">",
        Op::Ge => ">=",
        Op::Not => "!",
        Op::Neg => "-",
        Op::Deref => "",
    }
}

fn is_assign_op(op: &Op) -> bool {
    matches!(
        op,
        Op::AddAssign
            | Op::SubAssign
            | Op::MulAssign
            | Op::DivAssign
            | Op::RemAssign
            | Op::BitXorAssign
            | Op::BitAndAssign
            | Op::BitOrAssign
            | Op::ShlAssign
            | Op::ShrAssign
    )
}

fn emit_unary(op: &Op, expr: &Expression, w: &mut IndentWriter) {
    match op {
        Op::Deref => emit_expression(expr, w),
        _ => {
            w.write(op_str(op));
            emit_expression(expr, w);
        }
    }
}

fn emit_let(
    name: &Expression,
    ty: &Option<fn() -> agdb::type_def::Type>,
    value: &Option<&'static Expression>,
    w: &mut IndentWriter,
) {
    w.write("let ");
    emit_expression(name, w);
    if let Some(ty_fn) = ty {
        w.write(": ");
        w.write(&type_annotation(&ty_fn()));
    }
    if let Some(val) = value {
        w.write(" = ");
        emit_expression(val, w);
    }
}

fn emit_call(
    recipient: &Option<&'static Expression>,
    function: &Expression,
    args: &[Expression],
    w: &mut IndentWriter,
) {
    if let Some(recv) = recipient {
        emit_expression(recv, w);
        w.write(".");
    }
    emit_fn_name(function, w);
    w.write("(");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        emit_expression(arg, w);
    }
    w.write(")");
}

fn emit_fn_name(function: &Expression, w: &mut IndentWriter) {
    match function {
        Expression::Path {
            ident,
            parent: None,
            ..
        } => w.write(ident),
        Expression::Path {
            ident,
            parent: Some(parent),
            ..
        } => {
            emit_expression(parent, w);
            w.write(".");
            w.write(ident);
        }
        Expression::Ident(name) => w.write(name),
        _ => emit_expression(function, w),
    }
}

fn emit_struct_literal(
    name: &Expression,
    fields: &[(&'static str, Expression)],
    w: &mut IndentWriter,
) {
    w.write("new ");
    emit_expression(name, w);
    w.write("({ ");
    for (i, (field_name, value)) in fields.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        w.write(field_name);
        w.write(": ");
        emit_expression(value, w);
    }
    w.write(" })");
}

fn emit_if(
    condition: &Expression,
    then_branch: &Expression,
    else_branch: &Option<&'static Expression>,
    w: &mut IndentWriter,
) {
    w.write("if (");
    emit_expression(condition, w);
    w.write(") ");
    emit_block_expression(then_branch, w);
    if let Some(else_expr) = else_branch {
        match else_expr {
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                w.write(" else ");
                emit_if(condition, then_branch, else_branch, w);
            }
            _ => {
                w.write(" else ");
                emit_block_expression(else_expr, w);
            }
        }
    }
}

fn emit_block_expression(expr: &Expression, w: &mut IndentWriter) {
    match expr {
        Expression::Block(stmts) => emit_block(stmts, w),
        _ => {
            w.write("{");
            w.newline();
            w.indent();
            emit_statement(expr, w);
            w.dedent();
            w.write("}");
        }
    }
}

fn emit_block(stmts: &[Expression], w: &mut IndentWriter) {
    w.write("{");
    w.newline();
    w.indent();
    emit_body(stmts, w);
    w.dedent();
    w.write("}");
}

fn emit_closure(func: &agdb::type_def::Function, w: &mut IndentWriter) {
    w.write("(");
    for (i, arg) in func.args.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        w.write(arg.name);
        if let Some(ty_fn) = arg.ty {
            w.write(": ");
            w.write(&type_annotation(&ty_fn()));
        }
    }
    w.write(")");

    let ret_ty = (func.ret)();
    if !matches!(
        ret_ty,
        agdb::type_def::Type::Literal(agdb::type_def::Literal::Unit)
    ) {
        w.write(": ");
        w.write(&type_annotation(&ret_ty));
    }

    w.write(" => ");

    if func.body.len() == 1
        && let Expression::Return(Some(expr)) = &func.body[0]
    {
        emit_expression(expr, w);
        return;
    }
    emit_block(func.body, w);
}

fn emit_format(format_string: &str, args: &[Expression], w: &mut IndentWriter) {
    w.write("`");
    let mut arg_iter = args.iter();
    let mut chars = format_string.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'}') {
                chars.next();
                w.write("${");
                if let Some(arg) = arg_iter.next() {
                    emit_expression(arg, w);
                }
                w.write("}");
            } else {
                let mut name = String::new();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    name.push(c);
                }
                if name.starts_with(':') {
                    w.write("${");
                    if let Some(arg) = arg_iter.next() {
                        emit_expression(arg, w);
                    }
                    w.write("}");
                } else {
                    w.write(&format!("${{{name}}}"));
                }
            }
        } else if ch == '`' {
            w.write("\\`");
        } else if ch == '$' {
            w.write("\\$");
        } else {
            w.write(&ch.to_string());
        }
    }
    w.write("`");
}

fn emit_range(
    start: &Option<&'static Expression>,
    end: &Option<&'static Expression>,
    inclusive: bool,
    w: &mut IndentWriter,
) {
    w.write("range(");
    if let Some(s) = start {
        emit_expression(s, w);
    } else {
        w.write("undefined");
    }
    w.write(", ");
    if let Some(e) = end {
        emit_expression(e, w);
    } else {
        w.write("undefined");
    }
    if inclusive {
        w.write(", true");
    }
    w.write(")");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::typescript::transpiler::format::IndentWriter;
    use agdb::type_def::Type;

    fn stmt_to_string(expr: &Expression) -> String {
        let mut w = IndentWriter::new("    ");
        emit_statement(expr, &mut w);
        w.into_string()
    }

    fn body_to_string(body: &[Expression]) -> String {
        let mut w = IndentWriter::new("    ");
        emit_body(body, &mut w);
        w.into_string()
    }

    fn get_body(name: &str) -> &'static [Expression] {
        let func_type = match name {
            "literal_i32" => __literal_i32_type_def(),
            "literal_bool_true" => __literal_bool_true_type_def(),
            "literal_bool_false" => __literal_bool_false_type_def(),
            "literal_string" => __literal_string_type_def(),
            "literal_float" => __literal_float_type_def(),
            "literal_suffixed" => __literal_suffixed_type_def(),
            "binary_arithmetic" => __binary_arithmetic_type_def(),
            "binary_comparison" => __binary_comparison_type_def(),
            "binary_logical" => __binary_logical_type_def(),
            "binary_assign_ops" => __binary_assign_ops_type_def(),
            "unary_neg" => __unary_neg_type_def(),
            "unary_not" => __unary_not_type_def(),
            "assign_var" => __assign_var_type_def(),
            "let_simple" => __let_simple_type_def(),
            "let_typed" => __let_typed_type_def(),
            "let_no_init" => __let_no_init_type_def(),
            "call_free" => __call_free_type_def(),
            "call_method" => __call_method_type_def(),
            "field_access" => __field_access_type_def(),
            "tuple_access" => __tuple_access_type_def(),
            "index_access" => __index_access_type_def(),
            "array_literal" => __array_literal_type_def(),
            "tuple_literal" => __tuple_literal_type_def(),
            "struct_literal" => __struct_literal_type_def(),
            "if_simple" => __if_simple_type_def(),
            "if_else" => __if_else_type_def(),
            "if_else_if" => __if_else_if_type_def(),
            "while_loop" => __while_loop_type_def(),
            "loop_infinite" => __loop_infinite_type_def(),
            "for_loop" => __for_loop_type_def(),
            "for_destructure" => __for_destructure_type_def(),
            "break_continue" => __break_continue_type_def(),
            "return_value" => __return_value_type_def(),
            "return_void" => __return_void_type_def(),
            "implicit_return" => __implicit_return_type_def(),
            "closure_simple" => __closure_simple_type_def(),
            "closure_typed" => __closure_typed_type_def(),
            "format_basic" => __format_basic_type_def(),
            "format_multi" => __format_multi_type_def(),
            "reference_stripped" => __reference_stripped_type_def(),
            "try_transparent" => __try_transparent_type_def(),
            "await_expr" => __await_expr_type_def(),
            "block_expr" => __block_expr_type_def(),
            "range_half_open" => __range_half_open_type_def(),
            "range_inclusive" => __range_inclusive_type_def(),
            "wild_pattern" => __wild_pattern_type_def(),
            "tuple_destructure" => __tuple_destructure_type_def(),
            "match_to_if" => __match_to_if_type_def(),
            "path_expr" => __path_expr_type_def(),
            _ => panic!("Unknown test function: {name}"),
        };
        let Type::Function(def) = func_type else {
            panic!("Expected function type definition");
        };
        def.body
    }

    // --- Literals ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_i32() -> i32 {
        42
    }

    #[test]
    fn test_literal_i32() {
        let body = get_body("literal_i32");
        assert_eq!(stmt_to_string(&body[0]), "return 42;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_bool_true() -> bool {
        true
    }

    #[test]
    fn test_literal_bool_true() {
        let body = get_body("literal_bool_true");
        assert_eq!(stmt_to_string(&body[0]), "return true;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_bool_false() -> bool {
        false
    }

    #[test]
    fn test_literal_bool_false() {
        let body = get_body("literal_bool_false");
        assert_eq!(stmt_to_string(&body[0]), "return false;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_string() -> &'static str {
        "hello world"
    }

    #[test]
    fn test_literal_string() {
        let body = get_body("literal_string");
        assert_eq!(stmt_to_string(&body[0]), "return \"hello world\";\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_float() -> f64 {
        7.12
    }

    #[test]
    fn test_literal_float() {
        let body = get_body("literal_float");
        assert_eq!(stmt_to_string(&body[0]), "return 7.12;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_suffixed() {
        let _a = 1u8;
        let _b = 2u16;
        let _c = 3u32;
        let _d = 4u64;
        let _e = 5i8;
        let _f = 6i16;
        let _g = 7i32;
        let _h = 8usize;
    }

    #[test]
    fn test_literal_suffixed_all_numeric() {
        let body = get_body("literal_suffixed");
        assert_eq!(stmt_to_string(&body[0]), "let _a = 1;\n");
        assert_eq!(stmt_to_string(&body[1]), "let _b = 2;\n");
        assert_eq!(stmt_to_string(&body[2]), "let _c = 3;\n");
        assert_eq!(stmt_to_string(&body[3]), "let _d = 4;\n");
        assert_eq!(stmt_to_string(&body[4]), "let _e = 5;\n");
        assert_eq!(stmt_to_string(&body[5]), "let _f = 6;\n");
        assert_eq!(stmt_to_string(&body[6]), "let _g = 7;\n");
        assert_eq!(stmt_to_string(&body[7]), "let _h = 8;\n");
    }

    // --- Binary operators ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn binary_arithmetic() {
        let _a = 1 + 2;
        let _b = 3 - 4;
        let _c = 5 * 6;
        let _d = 7 / 8;
        let _e = 9 % 3;
    }

    #[test]
    fn test_binary_arithmetic() {
        let body = get_body("binary_arithmetic");
        assert_eq!(stmt_to_string(&body[0]), "let _a = 1 + 2;\n");
        assert_eq!(stmt_to_string(&body[1]), "let _b = 3 - 4;\n");
        assert_eq!(stmt_to_string(&body[2]), "let _c = 5 * 6;\n");
        assert_eq!(stmt_to_string(&body[3]), "let _d = 7 / 8;\n");
        assert_eq!(stmt_to_string(&body[4]), "let _e = 9 % 3;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn binary_comparison() {
        let _a = 1 == 2;
        let _b = 1 != 2;
        let _c = 1 < 2;
        let _d = 1 <= 2;
        let _e = 1 > 2;
        let _f = 1 >= 2;
    }

    #[test]
    fn test_binary_comparison_strict_equality() {
        let body = get_body("binary_comparison");
        assert_eq!(stmt_to_string(&body[0]), "let _a = 1 === 2;\n");
        assert_eq!(stmt_to_string(&body[1]), "let _b = 1 !== 2;\n");
        assert_eq!(stmt_to_string(&body[2]), "let _c = 1 < 2;\n");
        assert_eq!(stmt_to_string(&body[3]), "let _d = 1 <= 2;\n");
        assert_eq!(stmt_to_string(&body[4]), "let _e = 1 > 2;\n");
        assert_eq!(stmt_to_string(&body[5]), "let _f = 1 >= 2;\n");
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::nonminimal_bool)]
    fn binary_logical() {
        let _a = true && false;
        let _b = true || false;
    }

    #[test]
    fn test_binary_logical() {
        let body = get_body("binary_logical");
        assert_eq!(stmt_to_string(&body[0]), "let _a = true && false;\n");
        assert_eq!(stmt_to_string(&body[1]), "let _b = true || false;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn binary_assign_ops() {
        let mut x = 0;
        x += 1;
        x -= 2;
        x *= 3;
        x /= 4;
        x %= 5;
    }

    #[test]
    fn test_binary_assign_ops() {
        let body = get_body("binary_assign_ops");
        assert_eq!(stmt_to_string(&body[1]), "x += 1;\n");
        assert_eq!(stmt_to_string(&body[2]), "x -= 2;\n");
        assert_eq!(stmt_to_string(&body[3]), "x *= 3;\n");
        assert_eq!(stmt_to_string(&body[4]), "x /= 4;\n");
        assert_eq!(stmt_to_string(&body[5]), "x %= 5;\n");
    }

    // --- Unary operators ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn unary_neg() -> i32 {
        -5
    }

    #[test]
    fn test_unary_neg() {
        let body = get_body("unary_neg");
        assert_eq!(stmt_to_string(&body[0]), "return -5;\n");
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::nonminimal_bool)]
    fn unary_not() -> bool {
        !true
    }

    #[test]
    fn test_unary_not() {
        let body = get_body("unary_not");
        assert_eq!(stmt_to_string(&body[0]), "return !true;\n");
    }

    // --- Assignment ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn assign_var() {
        let mut x = 1;
        x = 5;
    }

    #[test]
    fn test_assign() {
        let body = get_body("assign_var");
        assert_eq!(stmt_to_string(&body[0]), "let x = 1;\n");
        assert_eq!(stmt_to_string(&body[1]), "x = 5;\n");
    }

    // --- Let bindings ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn let_simple() {
        let _x = 42;
    }

    #[test]
    fn test_let_simple() {
        let body = get_body("let_simple");
        assert_eq!(stmt_to_string(&body[0]), "let _x = 42;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn let_typed() {
        let _x: i32 = 42;
    }

    #[test]
    fn test_let_typed() {
        let body = get_body("let_typed");
        assert_eq!(stmt_to_string(&body[0]), "let _x: number = 42;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn let_no_init() {
        let _x: i32;
    }

    #[test]
    fn test_let_no_init() {
        let body = get_body("let_no_init");
        assert_eq!(stmt_to_string(&body[0]), "let _x: number;\n");
    }

    // --- Function and method calls ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn call_free() {
        fn helper(_a: i32, _b: i32) -> i32 {
            0
        }
        let _r = helper(1, 2);
    }

    #[test]
    fn test_call_free_function() {
        let body = get_body("call_free");
        assert_eq!(stmt_to_string(&body[1]), "let _r = helper(1, 2);\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn call_method() {
        let v = [1, 2, 3];
        let _len = v.len();
    }

    #[test]
    fn test_call_method() {
        let body = get_body("call_method");
        assert_eq!(stmt_to_string(&body[1]), "let _len = v.len();\n");
    }

    // --- Field and index access ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn field_access() {
        struct S {
            value: i32,
        }
        let s = S { value: 10 };
        let _v = s.value;
    }

    #[test]
    fn test_field_access() {
        let body = get_body("field_access");
        assert_eq!(stmt_to_string(&body[2]), "let _v = s.value;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn tuple_access() {
        let t = (1, 2);
        let _first = t.0;
    }

    #[test]
    fn test_tuple_access() {
        let body = get_body("tuple_access");
        assert_eq!(stmt_to_string(&body[1]), "let _first = t[0];\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn index_access() {
        let arr = [10, 20, 30];
        let _v = arr[1];
    }

    #[test]
    fn test_index_access() {
        let body = get_body("index_access");
        assert_eq!(stmt_to_string(&body[0]), "let arr = [10, 20, 30];\n");
        assert_eq!(stmt_to_string(&body[1]), "let _v = arr[1];\n");
    }

    // --- Compound literals ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn array_literal() {
        let _arr = [1, 2, 3];
    }

    #[test]
    fn test_array_literal() {
        let body = get_body("array_literal");
        assert_eq!(stmt_to_string(&body[0]), "let _arr = [1, 2, 3];\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn tuple_literal() {
        let _t = (1, 2, 3);
    }

    #[test]
    fn test_tuple_literal() {
        let body = get_body("tuple_literal");
        assert_eq!(stmt_to_string(&body[0]), "let _t = [1, 2, 3];\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn struct_literal() {
        struct Point {
            x: i32,
            y: i32,
        }
        let _p = Point { x: 1, y: 2 };
    }

    #[test]
    fn test_struct_literal() {
        let body = get_body("struct_literal");
        assert_eq!(
            stmt_to_string(&body[1]),
            "let _p = new Point({ x: 1, y: 2 });\n"
        );
    }

    // --- Control flow: if/else ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn if_simple() {
        if true {
            let _x = 1;
        }
    }

    #[test]
    fn test_if_simple() {
        let body = get_body("if_simple");
        let output = stmt_to_string(&body[0]);
        assert_eq!(output, "if (true) {\n    let _x = 1;\n}\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn if_else() {
        if true {
            let _x = 1;
        } else {
            let _y = 2;
        }
    }

    #[test]
    fn test_if_else() {
        let body = get_body("if_else");
        let output = stmt_to_string(&body[0]);
        assert!(output.contains("if (true) {"), "Got: {output}");
        assert!(output.contains("} else {"), "Got: {output}");
        assert!(output.contains("let _x = 1;"), "Got: {output}");
        assert!(output.contains("let _y = 2;"), "Got: {output}");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn if_else_if() {
        if true {
            let _x = 1;
        } else if false {
            let _y = 2;
        } else {
            let _z = 3;
        }
    }

    #[test]
    fn test_if_else_if_chain() {
        let body = get_body("if_else_if");
        let output = stmt_to_string(&body[0]);
        assert!(output.contains("if (true) {"), "Got: {output}");
        assert!(output.contains("} else if (false) {"), "Got: {output}");
        assert!(output.contains("} else {"), "Got: {output}");
    }

    // --- Control flow: loops ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn while_loop() {
        let mut i = 0;
        while i < 10 {
            i += 1;
        }
    }

    #[test]
    fn test_while_loop() {
        let body = get_body("while_loop");
        let output = stmt_to_string(&body[1]);
        assert!(output.contains("while (i < 10) {"), "Got: {output}");
        assert!(output.contains("i += 1;"), "Got: {output}");
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::never_loop)]
    fn loop_infinite() {
        loop {
            break;
        }
    }

    #[test]
    fn test_loop_becomes_while_true() {
        let body = get_body("loop_infinite");
        let output = stmt_to_string(&body[0]);
        assert!(output.contains("while (true) {"), "Got: {output}");
        assert!(output.contains("break;"), "Got: {output}");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn for_loop() {
        let items = [1, 2, 3];
        for _item in items {
            let _x = 1;
        }
    }

    #[test]
    fn test_for_of_loop() {
        let body = get_body("for_loop");
        let output = stmt_to_string(&body[1]);
        assert!(
            output.contains("for (const _item of items) {"),
            "Got: {output}"
        );
        assert!(output.contains("let _x = 1;"), "Got: {output}");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn for_destructure() {
        let items = [(1, 2), (3, 4)];
        for (a, _b) in items {
            let _ = a;
        }
    }

    #[test]
    fn test_for_with_tuple_destructure() {
        let body = get_body("for_destructure");
        let output = stmt_to_string(&body[1]);
        assert!(
            output.contains("for (const [a, _b] of items) {"),
            "Got: {output}"
        );
    }

    // --- Break and continue ---

    #[agdb::fn_def]
    #[allow(unused, clippy::never_loop)]
    fn break_continue() {
        loop {
            break;
        }
        loop {
            continue;
        }
    }

    #[test]
    fn test_break_and_continue() {
        let body = get_body("break_continue");
        let output = body_to_string(body);
        assert!(output.contains("break;"), "Got: {output}");
        assert!(output.contains("continue;"), "Got: {output}");
    }

    // --- Return ---

    #[agdb::fn_def]
    #[allow(unused, clippy::needless_return)]
    fn return_value() -> i32 {
        return 42;
    }

    #[test]
    fn test_return_value() {
        let body = get_body("return_value");
        assert_eq!(stmt_to_string(&body[0]), "return 42;\n");
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::needless_return)]
    fn return_void() {
        return;
    }

    #[test]
    fn test_return_void() {
        let body = get_body("return_void");
        assert_eq!(stmt_to_string(&body[0]), "return;\n");
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::let_and_return)]
    fn implicit_return() -> i32 {
        let x = 5;
        x
    }

    #[test]
    fn test_implicit_return() {
        let body = get_body("implicit_return");
        assert_eq!(stmt_to_string(&body[1]), "return x;\n");
    }

    // --- Closures ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn closure_simple() {
        let _f = |x: i32| x;
    }

    #[test]
    fn test_closure_arrow_function() {
        let body = get_body("closure_simple");
        let output = stmt_to_string(&body[0]);
        assert!(output.contains("(x: number) => x"), "Got: {output}");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn closure_typed() {
        let _f = |x: i32| -> i32 { x + 1 };
    }

    #[test]
    fn test_closure_typed_with_body() {
        let body = get_body("closure_typed");
        let output = stmt_to_string(&body[0]);
        assert!(output.contains("(x: number): number =>"), "Got: {output}");
    }

    // --- Format strings ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn format_basic() {
        let x = 42;
        let _s = format!("{}", x);
    }

    #[test]
    fn test_format_to_template_literal() {
        let body = get_body("format_basic");
        let output = stmt_to_string(&body[1]);
        assert_eq!(output, "let _s = `${x}`;\n");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn format_multi() {
        let a = 1;
        let b = 2;
        let _s = format!("{} + {} = {}", a, b, a);
    }

    #[test]
    fn test_format_multiple_args() {
        let body = get_body("format_multi");
        let output = stmt_to_string(&body[2]);
        assert_eq!(output, "let _s = `${a} + ${b} = ${a}`;\n");
    }

    // --- Reference stripping ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn reference_stripped() {
        let x = 42;
        let _r = &x;
    }

    #[test]
    fn test_reference_is_stripped() {
        let body = get_body("reference_stripped");
        assert_eq!(stmt_to_string(&body[1]), "let _r = x;\n");
    }

    // --- Try operator (transparent) ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn try_transparent() -> Result<i32, String> {
        let r: Result<i32, String> = Ok(1);
        r?;
        Ok(0)
    }

    #[test]
    fn test_try_is_transparent() {
        let body = get_body("try_transparent");
        assert_eq!(stmt_to_string(&body[1]), "r;\n");
    }

    // --- Await ---

    #[agdb::fn_def]
    #[allow(unused)]
    async fn await_expr() {
        async fn fetch() -> i32 {
            1
        }
        let _v = fetch().await;
    }

    #[test]
    fn test_await_expression() {
        let body = get_body("await_expr");
        let output = stmt_to_string(&body[1]);
        assert!(output.contains("await"), "Got: {output}");
    }

    // --- Block expressions ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn block_expr() {
        {
            let _x = 1;
        };
    }

    #[test]
    fn test_block_expression() {
        let body = get_body("block_expr");
        let output = stmt_to_string(&body[0]);
        assert!(output.contains("{\n"), "Got: {output}");
        assert!(output.contains("let _x = 1;"), "Got: {output}");
        assert!(output.contains("}"), "Got: {output}");
    }

    // --- Range expressions ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn range_half_open() {
        let _r = 0..10;
    }

    #[test]
    fn test_range_half_open() {
        let body = get_body("range_half_open");
        let output = stmt_to_string(&body[0]);
        assert!(output.contains("range(0, 10)"), "Got: {output}");
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn range_inclusive() {
        let _r = 1..=5;
    }

    #[test]
    fn test_range_inclusive() {
        let body = get_body("range_inclusive");
        let output = stmt_to_string(&body[0]);
        assert!(output.contains("range(1, 5, true)"), "Got: {output}");
    }

    // --- Wild pattern ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn wild_pattern() {
        let _ = 42;
    }

    #[test]
    fn test_wild_pattern() {
        let body = get_body("wild_pattern");
        assert_eq!(stmt_to_string(&body[0]), "let _ = 42;\n");
    }

    // --- Tuple destructuring ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn tuple_destructure() {
        let (a, b) = (1, 2);
        let _ = a + b;
    }

    #[test]
    fn test_tuple_destructure() {
        let body = get_body("tuple_destructure");
        assert_eq!(stmt_to_string(&body[0]), "let [a, b] = [1, 2];\n");
    }

    // --- Match (desugared to if/else chain) ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn match_to_if() -> i32 {
        let x = 1;
        match x {
            1 => 10,
            2 => 20,
            _ => 0,
        }
    }

    #[test]
    fn test_match_desugared_to_if_chain() {
        let body = get_body("match_to_if");
        let output = body_to_string(body);
        assert!(output.contains("if (x === 1) {"), "Got: {output}");
        assert!(output.contains("} else if (x === 2) {"), "Got: {output}");
        assert!(output.contains("} else {"), "Got: {output}");
        assert!(output.contains("10;"), "Got: {output}");
        assert!(output.contains("20;"), "Got: {output}");
    }

    // --- Path expressions ---

    #[agdb::fn_def]
    #[allow(unused)]
    fn path_expr() {
        let _v: Option<i32> = None;
    }

    #[test]
    fn test_path_expr() {
        let body = get_body("path_expr");
        let output = stmt_to_string(&body[0]);
        assert!(output.contains("let _v"), "Got: {output}");
        assert!(output.contains("None"), "Got: {output}");
    }

    // --- Full body integration ---

    #[test]
    fn test_full_body_multiple_statements() {
        let body = get_body("binary_assign_ops");
        let output = body_to_string(body);
        assert_eq!(
            output,
            "let x = 0;\nx += 1;\nx -= 2;\nx *= 3;\nx /= 4;\nx %= 5;\n"
        );
    }
}
