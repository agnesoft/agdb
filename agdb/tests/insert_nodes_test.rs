mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryError;
use test_db::TestDb;

#[test]
fn insert_nodes_aliases_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |transaction| -> Result<(), QueryError> {
            transaction.exec_mut(QueryBuilder::insert().nodes().aliases("alias").query())?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_error(
        QueryBuilder::select().ids("alias").query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn insert_node_existing_alias() {
    let mut db = TestDb::new();
    db.exec_mut_ids(
        QueryBuilder::insert().nodes().aliases("alias").query(),
        &[1],
    );
    db.exec_mut_ids(
        QueryBuilder::insert().nodes().aliases("alias").query(),
        &[1],
    );
}

#[test]
fn insert_nodes_aliases() {
    let mut db = TestDb::new();
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2"])
            .query(),
        &[1, 2],
    );
}

#[test]
fn insert_nodes_count() {
    let mut db = TestDb::new();
    db.exec_mut_ids(QueryBuilder::insert().nodes().count(2).query(), &[1, 2]);
}

#[test]
fn insert_nodes_aliases_values() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2"])
            .values(vec![
                vec![("key", "value").into(), ("key2", "value2").into()],
                vec![("key", "value3").into()],
            ])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(vec!["alias1", "alias2"]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("key", "value3").into()],
            },
        ],
    );
}

#[test]
fn insert_nodes_aliases_values_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |transaction| -> Result<(), QueryError> {
            transaction
                .exec_mut(
                    QueryBuilder::insert()
                        .nodes()
                        .aliases(["alias1", "alias2"])
                        .values(vec![
                            vec![("key", "value").into(), ("key2", "value2").into()],
                            vec![("key", "value3").into()],
                        ])
                        .query(),
                )
                .unwrap();
            assert_eq!(
                transaction
                    .exec(QueryBuilder::select().ids(vec!["alias1", "alias2"]).query())
                    .unwrap()
                    .elements,
                &[
                    DbElement {
                        id: DbId(1),
                        from: None,
                        to: None,
                        values: vec![("key", "value").into(), ("key2", "value2").into()],
                    },
                    DbElement {
                        id: DbId(2),
                        from: None,
                        to: None,
                        values: vec![("key", "value3").into()],
                    },
                ],
            );
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_error(
        QueryBuilder::select().ids("alias1").query(),
        "Alias 'alias1' not found",
    );
    db.exec_error(
        QueryBuilder::select().ids("alias2").query(),
        "Alias 'alias2' not found",
    );
}

#[test]
fn insert_nodes_aliases_values_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2"])
            .values_uniform(vec![("key", "value").into(), ("key2", "value2").into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(vec!["alias1", "alias2"]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_nodes_count_values_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(2)
            .values_uniform(vec![("key", "value").into(), ("key2", "value2").into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(vec![1, 2]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_nodes_values() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("key", "value").into(), ("key2", "value2").into()],
                vec![("key", "value3").into()],
            ])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(vec![1, 2]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("key", "value3").into()],
            },
        ],
    );
}

#[test]
fn insert_nodes_existing_aliases_values() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases("alias")
            .values(vec![vec![("key", 1).into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["new_alias", "alias", "alias3"])
            .values(vec![
                vec![("some_key", "value").into()],
                vec![("key", 10).into(), ("new_key", 100).into()],
                vec![],
            ])
            .query(),
        3,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(vec!["alias", "new_alias"])
            .query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("key", 10).into(), ("new_key", 100).into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("some_key", "value").into()],
            },
        ],
    );
}

#[test]
fn insert_nodes_aliases_values_mismatched_length() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias", "alias2"])
            .values(vec![vec![("key", 1).into()]])
            .query(),
        "Aliases (2) and values (1) must have compatible lenghts (2 <= 1)",
    );
}

#[test]
fn insert_or_replace_insert_new() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("key").query(), 0);
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .ids(QueryBuilder::search().index("key").value(1).query())
            .values(vec![vec![("key", 1).into()]])
            .query(),
        &[1],
    );
}

#[test]
fn insert_or_replace_insert_count() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("key").query(), 0);
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .ids(QueryBuilder::search().index("key").value(1).query())
            .count(3)
            .query(),
        &[1, 2, 3],
    );
}

#[test]
fn insert_or_replace_existing() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .ids(QueryBuilder::search().from(1).query())
            .values(vec![vec![("key", 1).into()]])
            .query(),
        &[1],
    );
}

#[test]
fn insert_or_replace_with_new_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .ids(QueryBuilder::search().from(1).query())
            .aliases("my_alias")
            .values(vec![vec![("key", 1).into()]])
            .query(),
        &[1],
    );
    db.exec_ids(QueryBuilder::select().ids("my_alias").query(), &[1]);
}

#[test]
fn insert_or_replace_update_alias() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert().nodes().aliases("my_alias").query(),
        1,
    );
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .ids(QueryBuilder::search().from(1).query())
            .aliases("my_alias2")
            .query(),
        &[1],
    );
    db.exec_error(
        QueryBuilder::select().ids("my_alias").query(),
        "Alias 'my_alias' not found",
    );
    db.exec_ids(QueryBuilder::select().ids("my_alias2").query(), &[1]);
}

#[test]
fn insert_or_replace_mismatch_length() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut_error(
        QueryBuilder::insert()
            .nodes()
            .ids(vec![1, 2])
            .values(vec![vec![]])
            .query(),
        "Values (1) and ids (2) must have the same length",
    );
}

#[test]
fn insert_or_update_edge_id() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(2).query(), 1);
    db.exec_mut_error(
        QueryBuilder::insert().nodes().ids(-3).count(1).query(),
        "The ids for insert or update must all refer to nodes - edge id '-3' found",
    );
}

#[test]
fn insert_aliases_and_normal_nodes() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases("users")
            .values(vec![
                vec![],
                vec![("name", "alice").into()],
                vec![("name", "bob").into()],
            ])
            .query(),
        3,
    );
}

#[test]
fn insert_nodes_ids_values_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .ids(vec![1, 2])
            .values_uniform(vec![("key", "value").into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(vec![1, 2]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("key", "value").into()],
            },
        ],
    );
}
