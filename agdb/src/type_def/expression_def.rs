use crate::type_def::Function;
use crate::type_def::Type;

#[derive(Debug, agdb::TypeDef)]
pub enum Expression {
    Array(&'static [Expression]),
    Assign {
        target: &'static Expression,
        value: &'static Expression,
    },
    Await(&'static Expression),
    Binary {
        op: Op,
        left: &'static Expression,
        right: &'static Expression,
    },
    Block(&'static [Expression]),
    Break,
    Call {
        recipient: Option<&'static Expression>,
        function: &'static Expression,
        args: &'static [Expression],
    },
    Closure(Function),
    Continue,
    FieldAccess {
        base: &'static Expression,
        field: &'static str,
    },
    For {
        pattern: &'static Expression,
        iterable: &'static Expression,
        body: &'static Expression,
    },
    Format {
        format_string: &'static str,
        args: &'static [Expression],
    },
    Ident(&'static str),
    If {
        condition: &'static Expression,
        then_branch: &'static Expression,
        else_branch: Option<&'static Expression>,
    },
    Index {
        base: &'static Expression,
        index: &'static Expression,
    },
    Let {
        name: &'static Expression,
        ty: Option<fn() -> Type>,
        value: Option<&'static Expression>,
    },
    Literal(LiteralValue),
    Path {
        ident: &'static str,
        parent: Option<&'static Expression>,
        generics: &'static [fn() -> Type],
    },
    Range {
        start: Option<&'static Expression>,
        end: Option<&'static Expression>,
        inclusive: bool,
    },
    Reference(&'static Expression),
    Return(Option<&'static Expression>),
    Struct {
        name: &'static Expression,
        fields: &'static [(&'static str, Expression)],
    },
    StructPattern {
        name: &'static Expression,
        fields: &'static [Expression],
    },
    Try(&'static Expression),
    Tuple(&'static [Expression]),
    TupleStruct {
        name: &'static Expression,
        expressions: &'static [Expression],
    },
    TupleAccess {
        base: &'static Expression,
        index: u32,
    },
    Unary {
        op: Op,
        expr: &'static Expression,
    },
    While {
        condition: &'static Expression,
        body: &'static Expression,
    },
    Match {
        scrutinee: &'static Expression,
        arms: &'static [MatchArm],
    },
    Wild,
}

#[derive(Debug, agdb::TypeDef)]
pub enum LiteralValue {
    Bool(bool),
    F32(f32),
    F64(f64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Str(&'static str),
    String(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Unit,
    Usize(usize),
}

#[derive(Copy, Clone, Debug, agdb::TypeDef)]
pub enum Op {
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    Rem,
    RemAssign,
    And,
    Or,
    BitXor,
    BitXorAssign,
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    Shl,
    ShlAssign,
    Shr,
    ShrAssign,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
    Not,
    Neg,
    Deref,
}

#[derive(Debug, agdb::TypeDef)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<&'static Expression>,
    pub body: &'static Expression,
}

#[derive(Debug, agdb::TypeDef)]
pub enum Pattern {
    Literal(LiteralValue),
    Ident(&'static str),
    Constructor {
        name: &'static str,
        fields: &'static [Pattern],
    },
    Tuple(&'static [Pattern]),
    Or(&'static [Pattern]),
    Struct {
        name: &'static str,
        fields: &'static [(&'static str, Pattern)],
    },
    Wild,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_def::Literal;
    use crate::type_def::Type;

    fn get_body(name: &str) -> &'static [Expression] {
        let func_type = match name {
            "array_expr" => __array_expr_type_def(),
            "assert_macros_expr" => __assert_macros_expr_type_def(),
            "assign_expr" => __assign_expr_type_def(),
            "binary_arithmetic" => __binary_arithmetic_type_def(),
            "binary_assign_ops" => __binary_assign_ops_type_def(),
            "binary_comparison" => __binary_comparison_type_def(),
            "binary_logical" => __binary_logical_type_def(),
            "block_expr" => __block_expr_type_def(),
            "break_continue" => __break_continue_type_def(),
            "call_function" => __call_function_type_def(),
            "call_method" => __call_method_type_def(),
            "closure_simple" => __closure_simple_type_def(),
            "closure_typed" => __closure_typed_type_def(),
            "field_access_expr" => __field_access_expr_type_def(),
            "for_loop_expr" => __for_loop_expr_type_def(),
            "for_pattern" => __for_pattern_type_def(),
            "format_expr" => __format_expr_type_def(),
            "ident_expr" => __ident_expr_type_def(),
            "if_else_expr" => __if_else_expr_type_def(),
            "if_else_if_expr" => __if_else_if_expr_type_def(),
            "if_expr" => __if_expr_type_def(),
            "implicit_return" => __implicit_return_type_def(),
            "index_expr" => __index_expr_type_def(),
            "let_no_init" => __let_no_init_type_def(),
            "let_pattern_tuple" => __let_pattern_tuple_type_def(),
            "let_simple" => __let_simple_type_def(),
            "let_typed" => __let_typed_type_def(),
            "literal_bool" => __literal_bool_type_def(),
            "literal_float" => __literal_float_type_def(),
            "literal_integer" => __literal_integer_type_def(),
            "literal_string" => __literal_string_type_def(),
            "literal_suffixed" => __literal_suffixed_type_def(),
            "loop_expr" => __loop_expr_type_def(),
            "match_expr" => __match_expr_type_def(),
            "path_expr" => __path_expr_type_def(),
            "range_from" => __range_from_type_def(),
            "range_full" => __range_full_type_def(),
            "range_half_open" => __range_half_open_type_def(),
            "range_inclusive" => __range_inclusive_type_def(),
            "range_to_inclusive" => __range_to_inclusive_type_def(),
            "range_to" => __range_to_type_def(),
            "reference_expr" => __reference_expr_type_def(),
            "return_none" => __return_none_type_def(),
            "return_value" => __return_value_type_def(),
            "struct_expr" => __struct_expr_type_def(),
            "try_expr" => __try_expr_type_def(),
            "tuple_access_expr" => __tuple_access_expr_type_def(),
            "tuple_expr" => __tuple_expr_type_def(),
            "unary_neg" => __unary_neg_type_def(),
            "unary_not" => __unary_not_type_def(),
            "vec_macro_expr" => __vec_macro_expr_type_def(),
            "while_loop_expr" => __while_loop_expr_type_def(),
            "wild_expr" => __wild_expr_type_def(),
            _ => panic!("Unknown test function: {name}"),
        };
        let Type::Function(def) = func_type else {
            panic!("Expected function type definition");
        };
        def.body
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_integer() -> i32 {
        42
    }

    #[test]
    fn test_literal_integer() {
        let body = get_body("literal_integer");
        assert_eq!(body.len(), 1);
        assert!(
            matches!(
                &body[0],
                Expression::Return(Some(Expression::Literal(LiteralValue::I32(42))))
            ),
            "Got: {:?}",
            body[0]
        );
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_bool() -> bool {
        true
    }

    #[test]
    fn test_literal_bool() {
        let body = get_body("literal_bool");
        assert_eq!(body.len(), 1);
        assert!(
            matches!(
                &body[0],
                Expression::Return(Some(Expression::Literal(LiteralValue::Bool(true))))
            ),
            "Got: {:?}",
            body[0]
        );
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_string() -> &'static str {
        "hello"
    }

    #[test]
    fn test_literal_string() {
        let body = get_body("literal_string");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Return(Some(Expression::Literal(LiteralValue::Str(s)))) => {
                assert_eq!(*s, "hello");
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn literal_float() -> f64 {
        3.3
    }

    #[test]
    fn test_literal_float() {
        let body = get_body("literal_float");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Return(Some(Expression::Literal(LiteralValue::F64(v)))) => {
                assert!((*v - 3.3).abs() < f64::EPSILON);
            }
            _ => panic!("Got: {:?}", body[0]),
        }
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
        let _i = 1.0f32;
    }

    #[test]
    fn test_literal_suffixed() {
        let body = get_body("literal_suffixed");
        assert_eq!(body.len(), 9);

        match &body[0] {
            Expression::Let {
                value: Some(Expression::Literal(LiteralValue::U8(1))),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[0]),
        }
        match &body[1] {
            Expression::Let {
                value: Some(Expression::Literal(LiteralValue::U16(2))),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[1]),
        }
        match &body[2] {
            Expression::Let {
                value: Some(Expression::Literal(LiteralValue::U32(3))),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[2]),
        }
        match &body[3] {
            Expression::Let {
                value: Some(Expression::Literal(LiteralValue::U64(4))),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[3]),
        }
        match &body[4] {
            Expression::Let {
                value: Some(Expression::Literal(LiteralValue::I8(5))),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[4]),
        }
        match &body[5] {
            Expression::Let {
                value: Some(Expression::Literal(LiteralValue::I16(6))),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[5]),
        }
        match &body[6] {
            Expression::Let {
                value: Some(Expression::Literal(LiteralValue::I32(7))),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[6]),
        }
        match &body[7] {
            Expression::Let {
                value: Some(Expression::Literal(LiteralValue::Usize(8))),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[7]),
        }
        match &body[8] {
            Expression::Let {
                value: Some(Expression::Literal(LiteralValue::F32(_))),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[8]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn array_expr() {
        let _arr = [1, 2, 3];
    }

    #[test]
    fn test_array() {
        let body = get_body("array_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value: Some(Expression::Array(elems)),
                ..
            } => {
                assert_eq!(elems.len(), 3);
                assert!(matches!(
                    elems[0],
                    Expression::Literal(LiteralValue::I32(1))
                ));
                assert!(matches!(
                    elems[1],
                    Expression::Literal(LiteralValue::I32(2))
                ));
                assert!(matches!(
                    elems[2],
                    Expression::Literal(LiteralValue::I32(3))
                ));
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn assign_expr() {
        let mut x = 1;
        x = 2;
    }

    #[test]
    fn test_assign() {
        let body = get_body("assign_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Assign { target, value } => {
                assert!(matches!(target, Expression::Ident("x")));
                assert!(matches!(value, Expression::Literal(LiteralValue::I32(2))));
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn binary_arithmetic() {
        let _a = 1 + 2;
        let _b = 3 - 4;
        let _c = 5 * 6;
        let _d = 7 / 8;
        let _e = 19 % 10;
    }

    #[test]
    fn test_binary_arithmetic() {
        let body = get_body("binary_arithmetic");
        assert_eq!(body.len(), 5);

        match &body[0] {
            Expression::Let {
                value: Some(Expression::Binary { op: Op::Add, .. }),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[0]),
        }
        match &body[1] {
            Expression::Let {
                value: Some(Expression::Binary { op: Op::Sub, .. }),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[1]),
        }
        match &body[2] {
            Expression::Let {
                value: Some(Expression::Binary { op: Op::Mul, .. }),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[2]),
        }
        match &body[3] {
            Expression::Let {
                value: Some(Expression::Binary { op: Op::Div, .. }),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[3]),
        }
        match &body[4] {
            Expression::Let {
                value: Some(Expression::Binary { op: Op::Rem, .. }),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[4]),
        }
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
    fn test_binary_comparison() {
        let body = get_body("binary_comparison");
        assert_eq!(body.len(), 6);

        let ops = [Op::Eq, Op::Ne, Op::Lt, Op::Le, Op::Gt, Op::Ge];
        for (i, expected_op) in ops.iter().enumerate() {
            match &body[i] {
                Expression::Let {
                    value: Some(Expression::Binary { op, .. }),
                    ..
                } => assert!(
                    std::mem::discriminant(op) == std::mem::discriminant(expected_op),
                    "body[{i}] expected {:?}, got {:?}",
                    expected_op,
                    op
                ),
                _ => panic!("body[{i}]: {:?}", body[i]),
            }
        }
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
        assert_eq!(body.len(), 2);

        match &body[0] {
            Expression::Let {
                value: Some(Expression::Binary { op: Op::And, .. }),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[0]),
        }
        match &body[1] {
            Expression::Let {
                value: Some(Expression::Binary { op: Op::Or, .. }),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn binary_assign_ops() {
        let mut x = 0;
        x += 1;
        x -= 1;
        x *= 2;
        x /= 2;
        x %= 3;
    }

    #[test]
    fn test_binary_assign_ops() {
        let body = get_body("binary_assign_ops");
        assert_eq!(body.len(), 6);

        let ops = [
            Op::AddAssign,
            Op::SubAssign,
            Op::MulAssign,
            Op::DivAssign,
            Op::RemAssign,
        ];
        for (i, expected_op) in ops.iter().enumerate() {
            match &body[i + 1] {
                Expression::Binary { op, .. } => assert!(
                    std::mem::discriminant(op) == std::mem::discriminant(expected_op),
                    "body[{}] expected {:?}, got {:?}",
                    i + 1,
                    expected_op,
                    op
                ),
                _ => panic!("body[{}]: {:?}", i + 1, body[i + 1]),
            }
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn unary_neg() -> i32 {
        -5
    }

    #[test]
    fn test_unary_neg() {
        let body = get_body("unary_neg");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Return(Some(Expression::Unary { op: Op::Neg, .. })) => {}
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::nonminimal_bool)]
    fn unary_not() -> bool {
        !true
    }

    #[test]
    fn test_unary_not() {
        let body = get_body("unary_not");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Return(Some(Expression::Unary { op: Op::Not, .. })) => {}
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn block_expr() {
        {
            let _x = 1;
        };
    }

    #[test]
    fn test_block() {
        let body = get_body("block_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                assert!(matches!(stmts[0], Expression::Let { .. }));
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

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
    fn test_break_continue() {
        let body = get_body("break_continue");
        assert_eq!(body.len(), 2);
        match &body[0] {
            Expression::While { body, .. } => match body {
                Expression::Block(stmts) => {
                    assert!(matches!(stmts[0], Expression::Break));
                }
                _ => panic!("Expected block, got {:?}", body),
            },
            _ => panic!("Got: {:?}", body[0]),
        }
        match &body[1] {
            Expression::While { body, .. } => match body {
                Expression::Block(stmts) => {
                    assert!(matches!(stmts[0], Expression::Continue));
                }
                _ => panic!("Expected block, got {:?}", body),
            },
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn call_function() {
        fn helper(_x: i32) -> i32 {
            0
        }
        let _r = helper(42);
    }

    #[test]
    fn test_call_function() {
        let body = get_body("call_function");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Let {
                value:
                    Some(Expression::Call {
                        recipient: None,
                        function,
                        args,
                    }),
                ..
            } => {
                assert!(
                    matches!(function, Expression::Ident("helper")),
                    "Got function: {:?}",
                    function
                );
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Got: {:?}", body[1]),
        }
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
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Let {
                value:
                    Some(Expression::Call {
                        recipient: Some(_),
                        function,
                        args,
                    }),
                ..
            } => {
                assert!(
                    matches!(function, Expression::Path { ident: "len", .. }),
                    "Got function: {:?}",
                    function
                );
                assert_eq!(args.len(), 0);
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn closure_simple() {
        let _f = |x: i32| x;
    }

    #[test]
    fn test_closure_simple() {
        let body = get_body("closure_simple");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value: Some(Expression::Closure(func)),
                ..
            } => {
                assert_eq!(func.name, "");
                assert_eq!(func.args.len(), 1);
                assert_eq!(func.args[0].name, "x");
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn closure_typed() {
        let _f = |x: i32| -> i32 { x + 1 };
    }

    #[test]
    fn test_closure_typed() {
        let body = get_body("closure_typed");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value: Some(Expression::Closure(func)),
                ..
            } => {
                assert_eq!(func.args.len(), 1);
                assert_eq!(func.args[0].name, "x");
                assert!(!func.body.is_empty());
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn field_access_expr() {
        struct S {
            field: i32,
        }
        let s = S { field: 1 };
        let _f = s.field;
    }

    #[test]
    fn test_field_access() {
        let body = get_body("field_access_expr");
        assert_eq!(body.len(), 3);
        match &body[2] {
            Expression::Let {
                value:
                    Some(Expression::FieldAccess {
                        base,
                        field: "field",
                    }),
                ..
            } => {
                assert!(
                    matches!(base, Expression::Ident("s")),
                    "Got base: {:?}",
                    base
                );
            }
            _ => panic!("Got: {:?}", body[2]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn tuple_access_expr() {
        let t = (1, 2);
        let _first = t.0;
    }

    #[test]
    fn test_tuple_access() {
        let body = get_body("tuple_access_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Let {
                value: Some(Expression::TupleAccess { base, index: 0 }),
                ..
            } => {
                assert!(
                    matches!(base, Expression::Ident("t")),
                    "Got base: {:?}",
                    base
                );
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn for_loop_expr() {
        let items = [1, 2, 3];
        for _item in items {
            let _x = 1;
        }
    }

    #[test]
    fn test_for_loop() {
        let body = get_body("for_loop_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::For {
                pattern,
                iterable,
                body,
            } => {
                assert!(
                    matches!(pattern, Expression::Ident("_item")),
                    "Got pattern: {:?}",
                    pattern
                );
                assert!(
                    matches!(iterable, Expression::Ident("items")),
                    "Got iterable: {:?}",
                    iterable
                );
                assert!(matches!(body, Expression::Block(_)));
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn while_loop_expr() {
        let mut i = 0;
        while i < 10 {
            i += 1;
        }
    }

    #[test]
    fn test_while_loop() {
        let body = get_body("while_loop_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::While { condition, body } => {
                assert!(
                    matches!(condition, Expression::Binary { op: Op::Lt, .. }),
                    "Got condition: {:?}",
                    condition
                );
                assert!(matches!(body, Expression::Block(_)));
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::never_loop)]
    fn loop_expr() {
        loop {
            break;
        }
    }

    #[test]
    fn test_loop_desugars_to_while_true() {
        let body = get_body("loop_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::While { condition, .. } => {
                assert!(
                    matches!(condition, Expression::Literal(LiteralValue::Bool(true))),
                    "Got condition: {:?}",
                    condition
                );
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn if_expr() {
        if true {
            let _x = 1;
        }
    }

    #[test]
    fn test_if() {
        let body = get_body("if_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::If {
                condition,
                then_branch,
                else_branch: None,
            } => {
                assert!(matches!(
                    condition,
                    Expression::Literal(LiteralValue::Bool(true))
                ));
                assert!(matches!(then_branch, Expression::Block(_)));
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn if_else_expr() {
        if true {
            let _x = 1;
        } else {
            let _y = 2;
        }
    }

    #[test]
    fn test_if_else() {
        let body = get_body("if_else_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::If {
                else_branch: Some(eb),
                ..
            } => {
                assert!(matches!(eb, Expression::Block(_)));
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn if_else_if_expr() {
        if true {
            let _x = 1;
        } else if false {
            let _y = 2;
        } else {
            let _z = 3;
        }
    }

    #[test]
    fn test_if_else_if() {
        let body = get_body("if_else_if_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::If {
                else_branch:
                    Some(Expression::If {
                        else_branch: Some(_),
                        ..
                    }),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn index_expr() {
        let arr = [1, 2, 3];
        let _v = arr[0];
    }

    #[test]
    fn test_index() {
        let body = get_body("index_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Let {
                value: Some(Expression::Index { base, index }),
                ..
            } => {
                assert!(
                    matches!(base, Expression::Ident("arr")),
                    "Got base: {:?}",
                    base
                );
                assert!(
                    matches!(index, Expression::Literal(LiteralValue::I32(0))),
                    "Got index: {:?}",
                    index
                );
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn let_simple() {
        let _x = 42;
    }

    #[test]
    fn test_let_simple() {
        let body = get_body("let_simple");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                name,
                ty: None,
                value: Some(Expression::Literal(LiteralValue::I32(42))),
            } => {
                assert!(
                    matches!(name, Expression::Ident("_x")),
                    "Got name: {:?}",
                    name
                );
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn let_typed() {
        let _x: i32 = 42;
    }

    #[test]
    fn test_let_typed() {
        let body = get_body("let_typed");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                name,
                ty: Some(ty_fn),
                value: Some(_),
            } => {
                assert!(matches!(name, Expression::Ident("_x")));
                let ty = ty_fn();
                assert!(
                    matches!(ty, Type::Literal(Literal::I32)),
                    "Got type: {:?}",
                    ty
                );
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn let_no_init() {
        let _x: i32;
    }

    #[test]
    fn test_let_no_init() {
        let body = get_body("let_no_init");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value: None,
                ty: Some(_),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn reference_expr() {
        let x = 42;
        let _r = &x;
    }

    #[test]
    fn test_reference() {
        let body = get_body("reference_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Let {
                value: Some(Expression::Reference(inner)),
                ..
            } => {
                assert!(matches!(inner, Expression::Ident("x")), "Got: {:?}", inner);
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::needless_return)]
    fn return_value() -> i32 {
        return 42;
    }

    #[test]
    fn test_return_value() {
        let body = get_body("return_value");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Return(Some(Expression::Literal(LiteralValue::I32(42)))) => {}
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::needless_return)]
    fn return_none() {
        return;
    }

    #[test]
    fn test_return_none() {
        let body = get_body("return_none");
        assert_eq!(body.len(), 1);
        assert!(
            matches!(&body[0], Expression::Return(None)),
            "Got: {:?}",
            body[0]
        );
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn try_expr() -> Result<i32, String> {
        let r: Result<i32, String> = Ok(1);
        r?;
        Ok(0)
    }

    #[test]
    fn test_try() {
        let body = get_body("try_expr");
        assert_eq!(body.len(), 3);
        match &body[1] {
            Expression::Try(inner) => {
                assert!(matches!(inner, Expression::Ident("r")), "Got: {:?}", inner);
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn tuple_expr() {
        let _t = (1, 2, 3);
    }

    #[test]
    fn test_tuple() {
        let body = get_body("tuple_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value: Some(Expression::Tuple(elems)),
                ..
            } => {
                assert_eq!(elems.len(), 3);
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn format_expr() {
        let x = 42;
        let _s = format!("{}", x);
    }

    #[test]
    fn test_format_macro() {
        let body = get_body("format_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Let {
                value:
                    Some(Expression::Format {
                        format_string,
                        args,
                    }),
                ..
            } => {
                assert_eq!(*format_string, "{}");
                assert_eq!(args.len(), 1);
                assert!(
                    matches!(args[0], Expression::Ident("x")),
                    "Got arg: {:?}",
                    args[0]
                );
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused, clippy::useless_vec)]
    fn vec_macro_expr() {
        let _v = vec![1, 2, 3];
    }

    #[test]
    fn test_vec_macro() {
        let body = get_body("vec_macro_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value: Some(Expression::Array(elems)),
                ..
            } => {
                assert_eq!(elems.len(), 3);
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn assert_macros_expr() {
        let x = 42;
        assert!(x > 0);
        assert_eq!(x, 42);
        let _matched = matches!(Some(x), Some(42));
    }

    #[test]
    fn test_assert_macros() {
        let body = get_body("assert_macros_expr");
        assert_eq!(body.len(), 4);

        assert!(
            matches!(
                &body[1],
                Expression::Call {
                    function: Expression::Path {
                        ident: "assert",
                        ..
                    },
                    ..
                }
            ),
            "Got: {:?}",
            body[1]
        );

        assert!(
            matches!(
                &body[2],
                Expression::Call {
                    function: Expression::Path {
                        ident: "assert_eq",
                        ..
                    },
                    ..
                }
            ),
            "Got: {:?}",
            body[2]
        );

        match &body[3] {
            Expression::Let {
                value: Some(Expression::Call { function, .. }),
                ..
            } => {
                assert!(
                    matches!(
                        function,
                        Expression::Path {
                            ident: "matches",
                            ..
                        }
                    ),
                    "Got: {:?}",
                    function
                );
            }
            _ => panic!("Got: {:?}", body[3]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn match_expr() -> i32 {
        let x = 1;
        match x {
            1 => 10,
            2 => 20,
            _ => 0,
        }
    }

    #[test]
    fn test_match() {
        let body = get_body("match_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Match { scrutinee, arms } => {
                assert!(
                    matches!(scrutinee, Expression::Ident("x")),
                    "Got scrutinee: {:?}",
                    scrutinee
                );
                assert_eq!(arms.len(), 3);
                assert!(
                    matches!(arms[0].pattern, Pattern::Literal(LiteralValue::I32(1))),
                    "Got arm[0] pattern: {:?}",
                    arms[0].pattern
                );
                assert!(arms[0].guard.is_none());
                assert!(
                    matches!(arms[1].pattern, Pattern::Literal(LiteralValue::I32(2))),
                    "Got arm[1] pattern: {:?}",
                    arms[1].pattern
                );
                assert!(
                    matches!(arms[2].pattern, Pattern::Wild),
                    "Got arm[2] pattern: {:?}",
                    arms[2].pattern
                );
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn struct_expr() {
        struct Point {
            x: i32,
            y: i32,
        }
        let _p = Point { x: 1, y: 2 };
    }

    #[test]
    fn test_struct_expr() {
        let body = get_body("struct_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Let {
                value: Some(Expression::Struct { name, fields }),
                ..
            } => {
                assert!(
                    matches!(name, Expression::Ident("Point")),
                    "Got name: {:?}",
                    name
                );
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert_eq!(fields[1].0, "y");
            }
            _ => panic!("Got: {:?}", body[1]),
        }
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
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Return(Some(Expression::Ident("x"))) => {}
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn ident_expr() {
        let x = 1;
        let _y = x;
    }

    #[test]
    fn test_ident() {
        let body = get_body("ident_expr");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Let {
                value: Some(Expression::Ident("x")),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn wild_expr() {
        let _ = 42;
    }

    #[test]
    fn test_wild() {
        let body = get_body("wild_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                name: Expression::Wild,
                ..
            } => {}
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn path_expr() {
        let _v: Option<i32> = None;
    }

    #[test]
    fn test_path() {
        let body = get_body("path_expr");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value: Some(Expression::Ident("None")),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn let_pattern_tuple() {
        let (a, b) = (1, 2);
        let _ = a + b;
    }

    #[test]
    fn test_let_pattern_tuple() {
        let body = get_body("let_pattern_tuple");
        assert_eq!(body.len(), 2);
        match &body[0] {
            Expression::Let {
                name: Expression::Tuple(elems),
                ..
            } => {
                assert_eq!(elems.len(), 2);
                assert!(matches!(elems[0], Expression::Ident("a")));
                assert!(matches!(elems[1], Expression::Ident("b")));
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn for_pattern() {
        let items = [(1, 2), (3, 4)];
        for (a, _b) in items {
            let _ = a;
        }
    }

    #[test]
    fn test_for_pattern() {
        let body = get_body("for_pattern");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::For { pattern, .. } => {
                assert!(
                    matches!(pattern, Expression::Tuple(_)),
                    "Got pattern: {:?}",
                    pattern
                );
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn range_half_open() {
        let _r = 0..10;
    }

    #[test]
    fn test_range_half_open() {
        let body = get_body("range_half_open");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value:
                    Some(Expression::Range {
                        start: Some(start),
                        end: Some(end),
                        inclusive: false,
                    }),
                ..
            } => {
                assert!(
                    matches!(start, Expression::Literal(LiteralValue::I32(0))),
                    "Got start: {:?}",
                    start
                );
                assert!(
                    matches!(end, Expression::Literal(LiteralValue::I32(10))),
                    "Got end: {:?}",
                    end
                );
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn range_inclusive() {
        let _r = 1..=5;
    }

    #[test]
    fn test_range_inclusive() {
        let body = get_body("range_inclusive");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value:
                    Some(Expression::Range {
                        start: Some(start),
                        end: Some(end),
                        inclusive: true,
                    }),
                ..
            } => {
                assert!(
                    matches!(start, Expression::Literal(LiteralValue::I32(1))),
                    "Got start: {:?}",
                    start
                );
                assert!(
                    matches!(end, Expression::Literal(LiteralValue::I32(5))),
                    "Got end: {:?}",
                    end
                );
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn range_from() {
        let start = 3;
        let _r = start..;
    }

    #[test]
    fn test_range_from() {
        let body = get_body("range_from");
        assert_eq!(body.len(), 2);
        match &body[1] {
            Expression::Let {
                value:
                    Some(Expression::Range {
                        start: Some(start),
                        end: None,
                        inclusive: false,
                    }),
                ..
            } => {
                assert!(
                    matches!(start, Expression::Ident("start")),
                    "Got start: {:?}",
                    start
                );
            }
            _ => panic!("Got: {:?}", body[1]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn range_to() {
        let _r = ..10;
    }

    #[test]
    fn test_range_to() {
        let body = get_body("range_to");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value:
                    Some(Expression::Range {
                        start: None,
                        end: Some(end),
                        inclusive: false,
                    }),
                ..
            } => {
                assert!(
                    matches!(end, Expression::Literal(LiteralValue::I32(10))),
                    "Got end: {:?}",
                    end
                );
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn range_to_inclusive() {
        let _r = ..=10;
    }

    #[test]
    fn test_range_to_inclusive() {
        let body = get_body("range_to_inclusive");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value:
                    Some(Expression::Range {
                        start: None,
                        end: Some(end),
                        inclusive: true,
                    }),
                ..
            } => {
                assert!(
                    matches!(end, Expression::Literal(LiteralValue::I32(10))),
                    "Got end: {:?}",
                    end
                );
            }
            _ => panic!("Got: {:?}", body[0]),
        }
    }

    #[agdb::fn_def]
    #[allow(unused)]
    fn range_full() {
        let _r = ..;
    }

    #[test]
    fn test_range_full() {
        let body = get_body("range_full");
        assert_eq!(body.len(), 1);
        match &body[0] {
            Expression::Let {
                value:
                    Some(Expression::Range {
                        start: None,
                        end: None,
                        inclusive: false,
                    }),
                ..
            } => {}
            _ => panic!("Got: {:?}", body[0]),
        }
    }
}
