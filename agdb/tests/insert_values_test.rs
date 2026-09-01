mod test_db;

use agdb::DbElement;
use agdb::DbError;
use agdb::DbErrorType;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn insert_values_ids_rollback() {
    let mut db = TestDb::new();
    db.exec_mut_ids(QueryBuilder::insert().nodes().count(1).query(), &[1]);
    db.transaction_mut_error(
        |t| -> Result<(), DbError> {
            assert_eq!(
                t.exec_mut(
                    QueryBuilder::insert()
                        .values([[
                            ("key", vec![1.1, 2.1]).into(),
                            (vec!["a".to_string(), "b".to_string()], vec![1, 2]).into(),
                            ("numbers", vec![1_u64, 2_u64, 3_u64]).into(),
                            (1_u64, 10_u64).into(),
                            ("bytes", vec![0_u8, 1_u8, 2_u8, 3_u8, 4_u8]).into(),
                            ("really large bytes", vec![1_u8; 32]).into()
                        ]])
                        .ids(1)
                        .query()
                )?
                .result,
                6
            );
            assert_eq!(
                t.exec(QueryBuilder::select().ids(1).query())?.elements,
                vec![DbElement {
                    id: DbId(1),
                    from: DbId::default(),
                    to: DbId::default(),
                    values: vec![
                        ("key", vec![1.1, 2.1]).into(),
                        (vec!["a".to_string(), "b".to_string()], vec![1, 2]).into(),
                        ("numbers", vec![1_u64, 2_u64, 3_u64]).into(),
                        (1_u64, 10_u64).into(),
                        ("bytes", vec![0_u8, 1_u8, 2_u8, 3_u8, 4_u8]).into(),
                        ("really large bytes", vec![1_u8; 32]).into()
                    ],
                }]
            );
            Err(DbError::db(DbErrorType::NotAllowed, "error"))
        },
        DbError::db(DbErrorType::NotAllowed, "error"),
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![],
        }],
    );
}

#[test]
fn insert_values_ids() {
    let mut db = TestDb::new();
    db.exec_mut_ids(QueryBuilder::insert().nodes().count(2).query(), &[1, 2]);
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("some really long key", 1000).into()], [(10, 1.1).into()]])
            .ids([1, 2])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids([1, 2]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![("some really long key", 1000).into()],
            },
            DbElement {
                id: DbId(2),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![(10, 1.1).into()],
            },
        ],
    );
}

#[test]
fn insert_values_invalid_length() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert()
            .values([[("key", "value").into()]])
            .ids([1, 2])
            .query(),
        "Ids (2) must match values (1)",
    )
}

#[test]
fn insert_values_uniform_ids() {
    let mut db = TestDb::new();
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias", "alias2"])
            .query(),
        &[1, 2],
    );
    db.exec_mut(
        QueryBuilder::insert()
            .values_uniform([("key", "value").into()])
            .ids(["alias", "alias2"])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(["alias", "alias2"]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(2),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![("key", "value").into()],
            },
        ],
    );
}

#[test]
fn insert_values_uniform_search() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from([1, 2])
            .to([2, 3])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .values_uniform([("key", "value").into()])
            .ids(QueryBuilder::search().from(1).query())
            .query(),
        5,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(QueryBuilder::search().from(1).query())
            .query(),
        &[
            DbElement {
                id: DbId(1),
                from: DbId(-4),
                to: DbId::default(),
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(-4),
                from: DbId(1),
                to: DbId(2),
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(2),
                from: DbId(-5),
                to: DbId(-4),
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(-5),
                from: DbId(2),
                to: DbId(3),
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(3),
                from: DbId::default(),
                to: DbId(-5),
                values: vec![("key", "value").into()],
            },
        ],
    );
}

#[test]
fn insert_values_search() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .values([
                [("key1", "value1").into()],
                [("key2", "value2").into()],
                [("key3", "value3").into()],
            ])
            .ids(QueryBuilder::search().from(1).query())
            .query(),
        3,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(QueryBuilder::search().from(1).query())
            .query(),
        &[
            DbElement {
                id: DbId(1),
                from: DbId(-4),
                to: DbId::default(),
                values: vec![("key1", "value1").into()],
            },
            DbElement {
                id: DbId(-4),
                from: DbId(1),
                to: DbId(3),
                values: vec![("key2", "value2").into()],
            },
            DbElement {
                id: DbId(3),
                from: DbId::default(),
                to: DbId(-4),
                values: vec![("key3", "value3").into()],
            },
        ],
    );
}

#[test]
fn insert_values_search_alt() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .values([
                [("key1", "value1").into()],
                [("key2", "value2").into()],
                [("key3", "value3").into()],
            ])
            .search()
            .from(1)
            .query(),
        3,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(QueryBuilder::search().from(1).query())
            .query(),
        &[
            DbElement {
                id: DbId(1),
                from: DbId(-4),
                to: DbId::default(),
                values: vec![("key1", "value1").into()],
            },
            DbElement {
                id: DbId(-4),
                from: DbId(1),
                to: DbId(3),
                values: vec![("key2", "value2").into()],
            },
            DbElement {
                id: DbId(3),
                from: DbId::default(),
                to: DbId(-4),
                values: vec![("key3", "value3").into()],
            },
        ],
    );
}

#[test]
fn insert_values_search_invalid_length() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query(), 1);
    db.exec_mut_error(
        QueryBuilder::insert()
            .values([[("key1", "value1").into()], [("key2", "value2").into()]])
            .ids(QueryBuilder::search().from(1).query())
            .query(),
        "Ids (3) must match values (2)",
    );
}

#[test]
fn insert_values_overwrite() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("key", 10).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .values_uniform([("key", 20).into(), ("key2", 30).into()])
            .ids(1)
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("key", 20).into(), ("key2", 30).into()],
        }],
    )
}

#[test]
fn insert_values_overwrite_transaction() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("key", 10).into()]])
            .query(),
        1,
    );

    db.transaction_mut_error(
        |t| -> Result<(), DbError> {
            t.exec_mut(
                QueryBuilder::insert()
                    .values_uniform([("key", 20).into(), ("key2", 30).into()])
                    .ids(1)
                    .query(),
            )?;
            Err(DbError::db(DbErrorType::NotAllowed, "error"))
        },
        DbError::db(DbErrorType::NotAllowed, "error"),
    );

    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("key", 10).into()],
        }],
    )
}

#[test]
fn overwrite_empty_value() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("v", "").into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("v", "a").into()]])
            .ids(1)
            .query(),
        1,
    );
}

// ============================================================
// Amend tests
// ============================================================

#[test]
fn amend_add_i64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("counter", 10_i64).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("counter", 5_i64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("counter", 15_i64).into()],
        }],
    );
}

#[test]
fn amend_add_i64_negative() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("counter", 10_i64).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("counter", -3_i64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("counter", 7_i64).into()],
        }],
    );
}

#[test]
fn amend_add_u64_saturating() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("counter", u64::MAX - 5).into()]])
            .query(),
        1,
    );
    // Should saturate at u64::MAX, not overflow
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("counter", 10_u64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("counter", u64::MAX).into()],
        }],
    );
}

#[test]
fn amend_add_f64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("score", 1.5_f64).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("score", 2.5_f64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("score", 4.0_f64).into()],
        }],
    );
}

#[test]
fn amend_add_string() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("name", "hello").into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("name", " world").into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("name", "hello world").into()],
        }],
    );
}

#[test]
fn amend_add_vec_i64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("list", vec![1_i64, 2]).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("list", vec![3_i64, 4]).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("list", vec![1_i64, 2, 3, 4]).into()],
        }],
    );
}

#[test]
fn amend_add_vec_u64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("list", vec![1_u64, 2]).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("list", vec![3_u64]).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("list", vec![1_u64, 2, 3]).into()],
        }],
    );
}

#[test]
fn amend_add_vec_f64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("list", vec![1.0_f64]).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("list", vec![2.0_f64]).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("list", vec![1.0_f64, 2.0]).into()],
        }],
    );
}

#[test]
fn amend_add_vec_string() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("tags", vec!["a".to_string()]).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("tags", vec!["b".to_string(), "c".to_string()]).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![
                (
                    "tags",
                    vec!["a".to_string(), "b".to_string(), "c".to_string()],
                )
                    .into(),
            ],
        }],
    );
}

#[test]
fn amend_add_bytes() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("data", vec![1_u8, 2]).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("data", vec![3_u8, 4]).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("data", vec![1_u8, 2, 3, 4]).into()],
        }],
    );
}

#[test]
fn amend_add_missing_key_inserts() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("counter", 5_i64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("counter", 5_i64).into()],
        }],
    );
}

#[test]
fn amend_add_new_element_creates() {
    let mut db = TestDb::new();
    db.exec_mut_ids(
        QueryBuilder::insert()
            .amend([[("counter", 5_i64).into()]])
            .ids(0)
            .query(),
        &[1],
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("counter", 5_i64).into()],
        }],
    );
}

#[test]
fn amend_add_type_mismatch_error() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("key", 10_i64).into()]])
            .query(),
        1,
    );
    db.exec_mut_error(
        QueryBuilder::insert()
            .amend([[("key", "string_value").into()]])
            .ids(1)
            .query(),
        "Cannot amend 'i64' with 'string'.",
    );
}

#[test]
fn amend_add_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("counter", 10_i64).into()], [("counter", 20_i64).into()]])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend_uniform([("counter", 5_i64).into()])
            .ids([1, 2])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids([1, 2]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![("counter", 15_i64).into()],
            },
            DbElement {
                id: DbId(2),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![("counter", 25_i64).into()],
            },
        ],
    );
}

#[test]
fn amend_add_with_search() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["node1", "node2"])
            .values([[("counter", 10_i64).into()], [("counter", 20_i64).into()]])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend_uniform([("counter", 100_i64).into()])
            .ids(QueryBuilder::search().from("node1").query())
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids("node1").query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("counter", 110_i64).into()],
        }],
    );
}

#[test]
fn amend_add_multiple_times() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("counter", 0_i64).into()]])
            .query(),
        1,
    );
    for _ in 0..5 {
        db.exec_mut(
            QueryBuilder::insert()
                .amend([[("counter", 2_i64).into()]])
                .ids(1)
                .query(),
            1,
        );
    }
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("counter", 10_i64).into()],
        }],
    );
}

// ============================================================
// Amend::Remove tests
// ============================================================

#[test]
fn amend_remove_i64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("counter", 10_i64).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("counter", 3_i64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("counter", 7_i64).into()],
        }],
    );
}

#[test]
fn amend_remove_u64_saturating() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("counter", 3_u64).into()]])
            .query(),
        1,
    );
    // Subtracting more than the value should saturate at 0
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("counter", 10_u64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("counter", 0_u64).into()],
        }],
    );
}

#[test]
fn amend_remove_f64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("score", 10.5_f64).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("score", 3.5_f64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("score", 7.0_f64).into()],
        }],
    );
}

#[test]
fn amend_remove_string() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("text", "hello world hello").into()]])
            .query(),
        1,
    );
    // Removes ALL occurrences
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("text", "hello").into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("text", " world ").into()],
        }],
    );
}

#[test]
fn amend_remove_vec_i64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("list", vec![1_i64, 2, 3, 2, 4]).into()]])
            .query(),
        1,
    );
    // Remove first occurrence of 2
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("list", vec![2_i64]).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("list", vec![1_i64, 3, 2, 4]).into()],
        }],
    );
}

#[test]
fn amend_remove_vec_string() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[(
                "tags",
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
            )
                .into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("tags", vec!["b".to_string()]).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("tags", vec!["a".to_string(), "c".to_string()]).into()],
        }],
    );
}

#[test]
fn amend_remove_bytes_error() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("data", vec![1_u8, 2, 3]).into()]])
            .query(),
        1,
    );
    db.exec_mut_error(
        QueryBuilder::remove()
            .amend([[("data", vec![1_u8]).into()]])
            .ids(1)
            .query(),
        "Amend remove is not supported for bytes.",
    );
}

#[test]
fn amend_remove_missing_key_noop() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    // Remove from nonexistent key: should be a no-op (result 0)
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("nonexistent", 5_i64).into()]])
            .ids(1)
            .query(),
        0,
    );
}

#[test]
fn amend_remove_new_element_error() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::remove()
            .amend([[("counter", 5_i64).into()]])
            .ids(0)
            .query(),
        "Cannot amend-remove on a new element.",
    );
}

#[test]
fn amend_remove_type_mismatch_error() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("key", 10_i64).into()]])
            .query(),
        1,
    );
    db.exec_mut_error(
        QueryBuilder::remove()
            .amend([[("key", "string_value").into()]])
            .ids(1)
            .query(),
        "Cannot amend 'i64' with 'string'.",
    );
}

#[test]
fn amend_remove_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("counter", 100_i64).into()], [("counter", 200_i64).into()]])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .amend_uniform([("counter", 50_i64).into()])
            .ids([1, 2])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids([1, 2]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![("counter", 50_i64).into()],
            },
            DbElement {
                id: DbId(2),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![("counter", 150_i64).into()],
            },
        ],
    );
}

#[test]
fn amend_add_scalar_to_vec_i64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("list", vec![1_i64, 2]).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("list", 3_i64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("list", vec![1_i64, 2, 3]).into()],
        }],
    );
}

#[test]
fn amend_add_scalar_to_vec_string() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("tags", vec!["a".to_string(), "b".to_string()]).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("tags", "c").into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![
                (
                    "tags",
                    vec!["a".to_string(), "b".to_string(), "c".to_string()],
                )
                    .into(),
            ],
        }],
    );
}

#[test]
fn amend_remove_scalar_from_vec_i64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("list", vec![1_i64, 2, 3, 2]).into()]])
            .query(),
        1,
    );
    // Remove first occurrence of 2
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("list", 2_i64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("list", vec![1_i64, 3, 2]).into()],
        }],
    );
}

#[test]
fn amend_remove_scalar_from_vec_string() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[(
                "tags",
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
            )
                .into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("tags", "b").into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("tags", vec!["a".to_string(), "c".to_string()]).into()],
        }],
    );
}

#[test]
fn amend_add_f64_with_i64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("score", 2.5_f64).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("score", 3_i64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("score", 5.5_f64).into()],
        }],
    );
}

#[test]
fn amend_add_f64_with_u64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("score", 1.25_f64).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .amend([[("score", 2_u64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("score", 3.25_f64).into()],
        }],
    );
}

#[test]
fn amend_remove_f64_with_i64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("score", 10.5_f64).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("score", 3_i64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("score", 7.5_f64).into()],
        }],
    );
}

#[test]
fn amend_remove_f64_with_u64() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("score", 10.5_f64).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .amend([[("score", 4_u64).into()]])
            .ids(1)
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![("score", 6.5_f64).into()],
        }],
    );
}
