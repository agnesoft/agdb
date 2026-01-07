use crate::api_def::Type;
use crate::api_def::function_def::Function;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[derive(Debug)]
pub enum Expression {
    // [1, 2, 3]
    Array(&'static [Expression]),

    // target = value
    Assign {
        target: &'static Expression,
        value: &'static Expression,
    },

    // expr.await
    Await(&'static Expression),

    // 1 < 2
    Binary {
        op: Op,
        left: &'static Expression,
        right: &'static Expression,
    },

    // { ... }
    Block(&'static [Expression]),

    // break;
    Break,

    // func(args)
    // obj.method(args)
    Call {
        recipient: Option<&'static Expression>,
        function: &'static Expression,
        args: &'static [Expression],
    },

    // |args| -> ret { ... }
    // |args| { ... }
    Closure(Function),

    // continue;
    Continue,

    // obj.field
    FieldAccess {
        base: &'static Expression,
        field: &'static str,
    },

    // for pattern in iterable { body }
    For {
        pattern: &'static Expression,
        iterable: &'static Expression,
        body: &'static Expression,
    },

    // format!("{}", args)
    Format {
        format_string: &'static str,
        args: &'static [Expression],
    },

    // varname
    Ident(&'static str),

    // if condition { then_branch } else { else_branch }
    // if condition { then_branch }
    If {
        condition: &'static Expression,
        then_branch: &'static Expression,
        else_branch: Option<&'static Expression>,
    },

    // array[index]
    Index {
        base: &'static Expression,
        index: &'static Expression,
    },

    // let name: Type = value;
    // let name = value;
    // let name;
    Let {
        name: &'static Expression,
        ty: Option<fn() -> Type>,
        value: Option<&'static Expression>,
    },

    // "literal"
    // 42
    // true
    Literal(Literal),

    // Some::Type<T>::foo()
    Path {
        ident: &'static str,
        parent: Option<&'static Expression>,
        generics: &'static [fn() -> Type],
    },

    // &expr
    Reference(&'static Expression),

    // return expr;
    Return(Option<&'static Expression>),

    //  Struct { field1: value1, field2: value2 }
    Struct {
        name: &'static Expression,
        fields: &'static [(&'static str, Expression)],
    },

    // Struct { field1, field2 }
    StructPattern {
        name: &'static Expression,
        fields: &'static [Expression],
    },

    // call()?
    Try(&'static Expression),

    // (x, y)
    Tuple(&'static [Expression]),

    // Tuple(expr, expr2, ...)
    TupleStruct {
        name: &'static Expression,
        expressions: &'static [Expression],
    },

    // base.index
    TupleAccess {
        base: &'static Expression,
        index: u32,
    },

    // -expr
    // !expr
    Unary {
        op: Op,
        expr: &'static Expression,
    },

    // while condition { body }
    While {
        condition: &'static Expression,
        body: &'static Expression,
    },

    // _
    Wild,
}

#[derive(Debug)]
pub enum Literal {
    I64(i64),
    F64(f64),
    String(&'static str),
    Bool(bool),
}

#[derive(Debug)]
pub enum Op {
    // +
    Add,
    // +=
    AddAssign,
    // -
    Sub,
    // -=
    SubAssign,
    // *
    Mul,
    // *=
    MulAssign,
    // /
    Div,
    // /=
    DivAssign,
    // %
    Rem,
    // %=
    RemAssign,
    // &&
    And,
    // ||
    Or,
    // ^
    BitXor,
    // ^=
    BitXorAssign,
    // &
    BitAnd,
    // &=
    BitAndAssign,
    // |
    BitOr,
    // |=
    BitOrAssign,
    // <<
    Shl,
    // <<=
    ShlAssign,
    // >>
    Shr,
    // >>=
    ShrAssign,
    // ==
    Eq,
    // <
    Lt,
    // <=
    Le,
    // !=
    Ne,
    // >=
    Ge,
    // >
    Gt,
    // !
    Not,
    // -
    Neg,
    // *
    Deref,
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let op_str = match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Rem => "%",
            Op::And => "&&",
            Op::Or => "||",
            Op::BitXor => "^",
            Op::BitAnd => "&",
            Op::BitOr => "|",
            Op::Shl => "<<",
            Op::Shr => ">>",
            Op::Eq => "==",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Ne => "!=",
            Op::Ge => ">=",
            Op::Gt => ">",
            Op::Not => "!",
            Op::Neg => "-",
            Op::AddAssign => "+=",
            Op::SubAssign => "-=",
            Op::MulAssign => "*=",
            Op::DivAssign => "/=",
            Op::RemAssign => "%=",
            Op::BitXorAssign => "^=",
            Op::BitAndAssign => "&=",
            Op::BitOrAssign => "|=",
            Op::ShlAssign => "<<=",
            Op::ShrAssign => ">>=",
            Op::Deref => "*",
        };
        write!(f, "{}", op_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_def::TypeDefinition;

    fn write_expression(expr: &Expression) -> String {
        match expr {
            Expression::Array(elements) => format!(
                "[{}]",
                elements
                    .iter()
                    .map(write_expression)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::Assign { target, value } => {
                format!("{} = {}", write_expression(target), write_expression(value))
            }
            Expression::Await(expression) => format!("{}.await", write_expression(expression)),
            Expression::Binary { op, left, right } => format!(
                "{} {op} {}",
                write_expression(left),
                write_expression(right)
            ),
            Expression::Block(expressions) => format!(
                "{{\n{}\n}}",
                expressions
                    .iter()
                    .map(write_expression)
                    .collect::<Vec<_>>()
                    .join(";\n")
            ),
            Expression::Break => "break".to_string(),
            Expression::Call {
                recipient,
                function,
                args,
            } => format!(
                "{}{}({})",
                match recipient {
                    Some(rec) => format!("{}.", write_expression(rec)),
                    None => "".to_string(),
                },
                write_expression(function),
                args.iter()
                    .map(write_expression)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::Closure(function) => format!(
                "|{}|{} {{\n{}\n}}",
                function
                    .args
                    .iter()
                    .map(|arg| arg.name)
                    .collect::<Vec<_>>()
                    .join(", "),
                match function.ret {
                    Some(ret_fn) => format!(" -> {}", (ret_fn)().name()),
                    None => "".to_string(),
                },
                function
                    .expressions
                    .iter()
                    .map(write_expression)
                    .collect::<Vec<_>>()
                    .join(";\n")
            ),
            Expression::Continue => "continue".to_string(),
            Expression::FieldAccess { base, field } => {
                format!("{}.{field}", write_expression(base))
            }
            Expression::For {
                pattern,
                iterable,
                body,
            } => format!(
                "for {} in {} {}",
                write_expression(pattern),
                write_expression(iterable),
                write_expression(body)
            ),
            Expression::Format {
                format_string,
                args,
            } => {
                let args_str = args
                    .iter()
                    .map(write_expression)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("format!({}, {})", format_string, args_str)
            }
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => format!(
                "if {} {}{}",
                write_expression(condition),
                write_expression(then_branch),
                match else_branch {
                    Some(else_br) => format!(" else {}", write_expression(else_br)),
                    None => "".to_string(),
                }
            ),
            Expression::Index { base, index } => {
                format!("{}[{}]", write_expression(base), write_expression(index))
            }
            Expression::Let { name, ty, value } => format!(
                "let {}{}{}",
                write_expression(name),
                match ty {
                    Some(ty_fn) => format!(": {}", (ty_fn)().name()),
                    None => "".to_string(),
                },
                match value {
                    Some(val) => format!(" = {}", write_expression(val)),
                    None => "".to_string(),
                }
            ),
            Expression::Literal(literal_value) => match literal_value {
                Literal::I64(val) => val.to_string(),
                Literal::F64(val) => val.to_string(),
                Literal::String(val) => val.to_string(),
                Literal::Bool(val) => val.to_string(),
            },
            Expression::Path {
                ident,
                parent,
                generics,
            } => {
                let parent_str = match parent {
                    Some(p) => format!("{}::", write_expression(p)),
                    None => "".to_string(),
                };
                let generics_str = if generics.is_empty() {
                    "".to_string()
                } else {
                    let gens = generics
                        .iter()
                        .map(|gen_fn| (gen_fn)().name())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("<{}>", gens)
                };
                format!("{}{}{}", parent_str, ident, generics_str)
            }
            Expression::Reference(expr) => format!("&{}", write_expression(expr)),
            Expression::Return(expression) => match expression {
                Some(expr) => format!("return {}", write_expression(expr)),
                None => "return".to_string(),
            },
            Expression::Struct { name, fields } => {
                let fields_str = fields
                    .iter()
                    .map(|(field_name, expr)| format!("{}: {}", field_name, write_expression(expr)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", write_expression(name), fields_str)
            }
            Expression::StructPattern { name, fields } => {
                let fields_str = fields
                    .iter()
                    .map(write_expression)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", write_expression(name), fields_str)
            }
            Expression::Try(expression) => {
                format!("{}?", write_expression(expression))
            }
            Expression::Tuple(expressions) => {
                format!(
                    "({})",
                    expressions
                        .iter()
                        .map(write_expression)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expression::TupleAccess { base, index } => {
                format!("{}.{index}", write_expression(base))
            }
            Expression::TupleStruct { name, expressions } => format!(
                "{}({})",
                write_expression(name),
                expressions
                    .iter()
                    .map(write_expression)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::Unary { op, expr } => format!("{}{}", op, write_expression(expr)),
            Expression::Ident(v) => v.to_string(),
            Expression::While { condition, body } => format!(
                "while {} {}",
                write_expression(condition),
                write_expression(body)
            ),
            Expression::Wild => "_".to_string(),
        }
    }

    #[test]
    fn array() {
        #[derive(agdb::TypeDef)]
        struct StructWithArrayExpr;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithArrayExpr {
            fn get_array() {
                let _ = [1, 2, 3];
                let _arr = &[1];
                let _arr_of_arrs = [[1, 2], [3, 4]];
            }
        }

        let e = write_expression(&StructWithArrayExpr::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _ = [1, 2, 3]");

        let e = write_expression(&StructWithArrayExpr::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "let _arr = &[1]");

        let e = write_expression(&StructWithArrayExpr::type_def().functions()[0].expressions[2]);
        assert_eq!(e, "let _arr_of_arrs = [[1, 2], [3, 4]]");
    }

    #[test]
    fn assign() {
        #[derive(agdb::TypeDef)]
        struct StructWithAssignExpr;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithAssignExpr {
            fn assign_example() {
                let mut _x: i32 = 5;
                _x = 10;
                _x += 2;
            }
        }

        let e = write_expression(&StructWithAssignExpr::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _x: i32 = 5");

        let e = write_expression(&StructWithAssignExpr::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "_x = 10");

        let e = write_expression(&StructWithAssignExpr::type_def().functions()[0].expressions[2]);
        assert_eq!(e, "_x += 2");
    }

    #[test]
    fn async_block() {
        #[derive(agdb::TypeDef)]
        struct StructWithAsyncBlock;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithAsyncBlock {
            async fn async_example() {
                let _ = async { 42 }.await;
            }
        }

        let e = write_expression(&StructWithAsyncBlock::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _ = {\nreturn 42\n}.await");
    }

    #[test]
    fn block() {
        #[derive(agdb::TypeDef)]
        struct StructWithBlock;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithBlock {
            fn block_example() {
                let _ = {
                    let x = 10;
                    x + 5
                };
            }
        }

        let e = write_expression(&StructWithBlock::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _ = {\nlet x = 10;\nreturn x + 5\n}");
    }

    #[test]
    fn for_loop() {
        #[derive(agdb::TypeDef)]
        struct StructWithForLoop;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithForLoop {
            fn for_loop_example() {
                let ar = [0, 1, 2];

                for _i in ar {}

                for i in ar {
                    if i == 1 {
                        break;
                    }
                }

                for _i in ar {
                    continue;
                }
            }
        }

        let e = write_expression(&StructWithForLoop::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "for _i in ar {\n\n}");

        let e = write_expression(&StructWithForLoop::type_def().functions()[0].expressions[2]);
        assert_eq!(e, "for i in ar {\nif i == 1 {\nbreak\n}\n}");

        let e = write_expression(&StructWithForLoop::type_def().functions()[0].expressions[3]);
        assert_eq!(e, "for _i in ar {\ncontinue\n}");
    }

    #[test]
    fn call() {
        #[derive(agdb::TypeDef)]
        struct StructWithCall;

        fn bar(_i: i32, _s: &str) {}

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithCall {
            fn foo() {
                bar(32, "hello");
            }
        }

        let e = write_expression(&StructWithCall::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "bar(32, \"hello\")");
    }

    #[test]
    fn cast() {
        #[derive(agdb::TypeDef)]
        struct StructWithCast;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithCast {
            fn cast_example() -> f64 {
                let x = 42;
                x as f64
            }
        }

        let e = write_expression(&StructWithCast::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "x");
    }

    #[test]
    fn closure() {
        #[derive(agdb::TypeDef)]
        struct StructWithClosure;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithClosure {
            fn closure_example() {
                let _add = |a: i32, b: i32| -> i32 { a + b };
                let sub = |x| x;
                let _with_body = || {
                    let result = 5;
                    if result == 5 {
                        return result;
                    }
                    result
                };
                sub(5);
            }
        }

        let e = write_expression(&StructWithClosure::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _add = |a, b| -> i32 {\nreturn a + b\n}");
        let e = write_expression(&StructWithClosure::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "let sub = |x| {\nreturn x\n}");
        let e = write_expression(&StructWithClosure::type_def().functions()[0].expressions[2]);
        assert_eq!(
            e,
            "let _with_body = || {\nlet result = 5;\nif result == 5 {\nreturn result\n};\nreturn result\n}"
        );
    }

    #[test]
    fn const_block() {
        #[derive(agdb::TypeDef)]
        struct StructWithConstBlock;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithConstBlock {
            fn const_block_example() {
                let _ = const { 42 };
            }
        }

        let e = write_expression(&StructWithConstBlock::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _ = {\nreturn 42\n}");
    }

    #[test]
    fn field_access() {
        #[derive(agdb::TypeDef)]
        struct StructWithFieldAccess;

        #[derive(agdb::TypeDefImpl)]
        struct SomeStructWithField {
            field1: i32,
        }

        #[derive(agdb::TypeDefImpl)]
        struct SomeTupleWithField(i32);

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithFieldAccess {
            fn field_access_example(s: SomeStructWithField, s2: SomeTupleWithField) {
                let _ = s.field1;
                let _ = s2.0;
            }
        }

        let e = write_expression(&StructWithFieldAccess::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _ = s.field1");
        let e = write_expression(&StructWithFieldAccess::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "let _ = s2.0");
    }

    #[test]
    fn index() {
        #[derive(agdb::TypeDef)]
        struct StructWithIndex;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithIndex {
            fn index_example() {
                let arr = [10, 20, 30];
                let _ = arr[1];
            }
        }

        let e = write_expression(&StructWithIndex::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "let _ = arr[1]");
    }

    #[test]
    fn loop_expr() {
        #[derive(agdb::TypeDef)]
        struct StructWithLoop;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithLoop {
            fn loop_example() {
                let mut i = 0;
                loop {
                    if i >= 5 {
                        break;
                    }
                    i += 1;
                }
            }
        }

        let e = write_expression(&StructWithLoop::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "while true {\nif i >= 5 {\nbreak\n};\ni += 1\n}");
    }

    #[test]
    fn macro_expr() {
        #[derive(agdb::TypeDef)]
        struct StructWithMacro;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithMacro {
            fn macro_example() {
                let _ = vec![42];
                let _ = format!("Hello, {}", 42);
                let a = 42;
                let _ = format!("Value: {a}");
                let _ = format!("{a} {} {a}", 42);
            }
        }

        // Currently, macro parsing is not implemented, so this test is a placeholder.
        let e = write_expression(&StructWithMacro::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _ = [42]");
        let e = write_expression(&StructWithMacro::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "let _ = format!(\"Hello, {}\", 42)");
        let e = write_expression(&StructWithMacro::type_def().functions()[0].expressions[3]);
        assert_eq!(e, "let _ = format!(\"Value: {}\", a)");
        let e = write_expression(&StructWithMacro::type_def().functions()[0].expressions[4]);
        assert_eq!(e, "let _ = format!(\"{} {} {}\", a, 42, a)");
    }

    #[test]
    fn match_expr() {
        #[derive(agdb::TypeDef)]
        struct StructWithMatch;

        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E {
            A,
            B,
            C(i32),
            D { x: i32 },
        }

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithMatch {
            fn match_example(x: i32) {
                match x {
                    0 => (),
                    1 | 2 => (),
                    _ => (),
                }
            }
            fn match_enum(e: E) -> i32 {
                let x = 2;
                match e {
                    E::A => 1,
                    E::B if x == 2 => 2,
                    E::C(i) => i,
                    E::D { x } => x,
                    _ => 4,
                }
            }
        }

        // Currently, match parsing is not implemented, so this test is a placeholder.
        let e = write_expression(&StructWithMatch::type_def().functions()[0].expressions[0]);
        assert_eq!(
            e,
            "if x == 0 {\n\n} else if x == 1 || x == 2 {\n\n} else {\n\n}"
        );
        let e = write_expression(&StructWithMatch::type_def().functions()[1].expressions[1]);
        assert_eq!(
            e,
            "if e == E::A {\n1\n} else if e == E::B && x == 2 {\n2\n} else if e == E::C(i) {\ni\n} else if e == E::D { x } {\nx\n} else {\n4\n}"
        );
    }

    #[test]
    fn method_call() {
        #[derive(agdb::TypeDef)]
        struct StructWithMethodCall;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithMethodCall {
            fn foo(&self, x: i32) -> i32 {
                x
            }

            fn standalone<T>(x: T) -> T {
                x
            }

            fn bar(&self) -> i32 {
                self.foo(42);
                StructWithMethodCall::standalone::<i32>(42)
            }
        }

        let e = write_expression(&StructWithMethodCall::type_def().functions()[2].expressions[0]);
        assert_eq!(e, "self.foo(42)");

        let e = write_expression(&StructWithMethodCall::type_def().functions()[2].expressions[1]);
        assert_eq!(e, "StructWithMethodCall::standalone<i32>(42)");
    }

    #[test]
    fn struct_expr() {
        #[derive(agdb::TypeDef)]
        struct StructWithStructExpr;

        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct Point {
            x: i32,
            y: i32,
        }

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithStructExpr {
            fn create_point() {
                let _ = Point { x: 10, y: 20 };
            }
        }

        let e = write_expression(&StructWithStructExpr::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _ = Point { x: 10, y: 20 }");
    }

    #[test]
    fn try_expr() {
        #[derive(agdb::TypeDef)]
        struct StructWithTryExpr;

        fn foo() -> Result<i32, String> {
            Ok(42)
        }

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithTryExpr {
            fn try_example() -> Result<i32, String> {
                let v = foo()?;
                Ok(v)
            }
        }

        let e = write_expression(&StructWithTryExpr::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let v = foo()?");
    }

    #[test]
    fn tuple() {
        #[derive(agdb::TypeDef)]
        struct StructWithTuple;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithTuple {
            fn tuple_example() {
                let t = (1, "hello", true);
                let (_a, _b, _c) = t;
            }
        }

        let e = write_expression(&StructWithTuple::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let t = (1, \"hello\", true)");

        let e = write_expression(&StructWithTuple::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "let (_a, _b, _c) = t");
    }

    #[test]
    fn unary() {
        #[derive(agdb::TypeDef)]
        struct StructWithUnary;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithUnary {
            fn unary_example(v: bool) {
                let _ = -42;
                let _ = !v;
                let x = 10;
                let y = &x;
                let _ = *y;
            }
        }

        let e = write_expression(&StructWithUnary::type_def().functions()[0].expressions[0]);
        assert_eq!(e, "let _ = -42");
        let e = write_expression(&StructWithUnary::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "let _ = !v");
        let e = write_expression(&StructWithUnary::type_def().functions()[0].expressions[4]);
        assert_eq!(e, "let _ = *y");
    }

    #[test]
    fn while_loop() {
        #[derive(agdb::TypeDef)]
        struct StructWithWhileLoop;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithWhileLoop {
            fn while_example() {
                let mut i = 0;
                while i < 5 {
                    i += 1;
                }
            }
        }

        let e = write_expression(&StructWithWhileLoop::type_def().functions()[0].expressions[1]);
        assert_eq!(e, "while i < 5 {\ni += 1\n}");
    }
}
