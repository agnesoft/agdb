//! What the lowering does not read from the agdb source yet, written in the lowered IR as the
//! source has it:
//!
//! - the `From` impls behind the `Into<X>` bounds of the builder API;
//! - the `DbType` trait, which the user types implement with its derive;
//! - the hand-written `impl Default`s;
//! - the `#[serde(default, skip_serializing_if = ..)]` fields.

use agsource::codegen::lowered::*;

fn v() -> Expr {
    Expr::new(K::Local("v".into()))
}

fn var(en: &str, var: &str, payload: Expr) -> Expr {
    Expr::variant(en, var, Some(payload))
}

fn into(e: Expr, to: &str) -> Expr {
    Expr::new(K::Convert {
        value: e.boxed(),
        to: to.into(),
    })
}

fn newtype(class: &str, e: Expr) -> Expr {
    Expr::new(K::Construct {
        class: class.into(),
        fields: vec![("0".into(), e)],
    })
}

fn list(items: Vec<Expr>) -> Expr {
    Expr::new(K::Array(items))
}

fn map(e: Expr, body: Expr) -> Expr {
    Expr::new(K::Call {
        callee: Callee::Intrinsic(Intrinsic::Map),
        recv: Some(e.boxed()),
        args: vec![Expr::new(K::Closure {
            params: vec!["x".into()],
            body: vec![Expr::new(K::Return(Some(body.boxed())))],
        })],
    })
}

fn x() -> Expr {
    Expr::new(K::Local("x".into()))
}

fn method(recv: Expr, m: &str) -> Expr {
    Expr::new(K::Call {
        callee: Callee::Method(m.into()),
        recv: Some(recv.boxed()),
        args: vec![],
    })
}

fn int(n: &'static str) -> Ty {
    Ty::Int(n)
}

fn l(t: Ty) -> Ty {
    Ty::List(Box::new(t))
}

fn into_ty(n: &str) -> Ty {
    Ty::Into(n.into())
}

const DB_TYPE: &str = "DbType";

fn db_type() -> Ty {
    Ty::Param(DB_TYPE.into())
}

pub(crate) fn conversions() -> Vec<Conversion> {
    let c = |to: &str, from: Ty, body: Expr| Conversion {
        to: to.into(),
        from,
        body,
    };
    vec![
        // db_value.rs
        c("DbValue", int("i32"), var("DbValue", "I64", v())),
        c("DbValue", int("i64"), var("DbValue", "I64", v())),
        c("DbValue", int("u32"), var("DbValue", "U64", v())),
        c("DbValue", int("u64"), var("DbValue", "U64", v())),
        c("DbValue", int("usize"), var("DbValue", "U64", v())),
        c(
            "DbValue",
            Ty::Float,
            var("DbValue", "F64", newtype("DbF64", v())),
        ),
        c("DbValue", Ty::Str, var("DbValue", "String", v())),
        c(
            "DbValue",
            Ty::Bool,
            var(
                "DbValue",
                "U64",
                Expr::new(K::If {
                    cond: v().boxed(),
                    then: vec![Expr::new(K::Return(Some(
                        Expr::new(K::Lit(Lit::Int(1))).boxed(),
                    )))],
                    els: Some(vec![Expr::new(K::Return(Some(
                        Expr::new(K::Lit(Lit::Int(0))).boxed(),
                    )))]),
                }),
            ),
        ),
        c("DbValue", l(int("u8")), var("DbValue", "Bytes", v())),
        c("DbValue", l(int("i32")), var("DbValue", "VecI64", v())),
        c("DbValue", l(int("i64")), var("DbValue", "VecI64", v())),
        c("DbValue", l(int("u64")), var("DbValue", "VecU64", v())),
        c(
            "DbValue",
            l(Ty::Float),
            var("DbValue", "VecF64", map(v(), newtype("DbF64", x()))),
        ),
        c("DbValue", l(Ty::Str), var("DbValue", "VecString", v())),
        // `impl<T: Into<DbValue>> From<Vec<T>> for DbValues` + scalar impls.
        c(
            "DbValues",
            l(into_ty("DbValue")),
            newtype("DbValues", map(v(), into(x(), "DbValue"))),
        ),
        c(
            "DbValues",
            into_ty("DbValue"),
            newtype("DbValues", list(vec![into(v(), "DbValue")])),
        ),
        // query_condition.rs
        c(
            "CountComparison",
            int("u64"),
            var("CountComparison", "Equal", v()),
        ),
        c(
            "Comparison",
            into_ty("DbValue"),
            var("Comparison", "Equal", into(v(), "DbValue")),
        ),
        // query_id.rs
        c("QueryId", Ty::Str, var("QueryId", "Alias", v())),
        c(
            "QueryId",
            int("i64"),
            var("QueryId", "Id", newtype("DbId", v())),
        ),
        c("QueryId", Ty::named("DbId"), var("QueryId", "Id", v())),
        // query_ids.rs
        c(
            "QueryIds",
            l(into_ty("QueryId")),
            var("QueryIds", "Ids", map(v(), into(x(), "QueryId"))),
        ),
        c(
            "QueryIds",
            into_ty("QueryId"),
            var("QueryIds", "Ids", list(vec![into(v(), "QueryId")])),
        ),
        c(
            "QueryIds",
            Ty::named("QueryResult"),
            var(
                "QueryIds",
                "Ids",
                map(
                    Expr::new(K::Field {
                        base: v().boxed(),
                        name: "elements".into(),
                    }),
                    into(
                        Expr::new(K::Field {
                            base: x().boxed(),
                            name: "id".into(),
                        }),
                        "QueryId",
                    ),
                ),
            ),
        ),
        c(
            "QueryIds",
            Ty::named("SearchQuery"),
            var("QueryIds", "Search", v()),
        ),
        // query_aliases.rs
        c("QueryAliases", l(Ty::Str), newtype("QueryAliases", v())),
        c(
            "QueryAliases",
            Ty::Str,
            newtype("QueryAliases", list(vec![v()])),
        ),
        // db_key_order.rs
        c(
            "DbKeyOrders",
            l(Ty::named("DbKeyOrder")),
            newtype("DbKeyOrders", v()),
        ),
        c(
            "DbKeyOrders",
            Ty::named("DbKeyOrder"),
            newtype("DbKeyOrders", list(vec![v()])),
        ),
        // db_key_value.rs: `impl<K: Into<DbValue>, T: Into<DbValue>> From<(K, T)>`
        c(
            "DbKeyValue",
            Ty::Tuple(vec![into_ty("DbValue"), into_ty("DbValue")]),
            Expr::new(K::Construct {
                class: "DbKeyValue".into(),
                fields: vec![
                    (
                        "key".into(),
                        into(
                            Expr::new(K::Field {
                                base: v().boxed(),
                                name: "0".into(),
                            }),
                            "DbValue",
                        ),
                    ),
                    (
                        "value".into(),
                        into(
                            Expr::new(K::Field {
                                base: v().boxed(),
                                name: "1".into(),
                            }),
                            "DbValue",
                        ),
                    ),
                ],
            }),
        ),
        // query_values.rs
        c(
            "SingleValues",
            l(Ty::named("DbKeyValue")),
            newtype("SingleValues", v()),
        ),
        c(
            "SingleValues",
            db_type(),
            newtype("SingleValues", method(v(), "to_db_values")),
        ),
        c(
            "MultiValues",
            l(l(Ty::named("DbKeyValue"))),
            newtype("MultiValues", v()),
        ),
        c(
            "MultiValues",
            l(db_type()),
            newtype("MultiValues", map(v(), method(x(), "to_db_values"))),
        ),
        c(
            "MultiValues",
            db_type(),
            newtype("MultiValues", list(vec![method(v(), "to_db_values")])),
        ),
    ]
}

/// `pub trait DbType` (agdb/src/db/db_type.rs), signatures only.
pub(crate) fn db_type_trait() -> Vec<Func> {
    let f = |name: &str, recv: Option<Recv>, ret: Ty| Func {
        recv,
        ..Func::new(name, Vec::new(), ret, Vec::new())
    };
    vec![
        f(
            "db_id",
            Some(Recv::Ref),
            Ty::Option(Box::new(Ty::named("QueryId"))),
        ),
        f("db_keys", None, l(Ty::named("DbValue"))),
        f(
            "db_element_id",
            None,
            Ty::Option(Box::new(Ty::named("DbValue"))),
        ),
        f("to_db_values", Some(Recv::Ref), l(Ty::named("DbKeyValue"))),
    ]
}

pub(crate) fn defaults() -> Vec<(String, Expr)> {
    vec![(
        "QueryId".into(),
        Expr::typed(
            K::Variant {
                en: "QueryId".into(),
                var: "Id".into(),
                payload: Some(
                    newtype("DbId", Expr::typed(K::Lit(Lit::Int(0)), Ty::Int("u64"))).boxed(),
                ),
            },
            Ty::named("QueryId"),
        ),
    )]
}

/// (struct, field, value omitted from the JSON).
pub(crate) fn skipped_defaults() -> Vec<(String, String, Expr)> {
    vec![(
        "InsertValuesQuery".into(),
        "amend".into(),
        Expr::variant("Amend", "None", None),
    )]
}
