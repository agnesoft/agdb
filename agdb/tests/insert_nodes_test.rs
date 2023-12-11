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
            transaction.exec_mut(&QueryBuilder::insert().nodes().aliases("alias").query())?;
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
    db.exec_mut_error(
        QueryBuilder::insert().nodes().aliases("alias").query(),
        "Alias 'alias' already exists (1)",
    )
}

#[test]
fn insert_nodes_aliases() {
    let mut db = TestDb::new();
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["alias1", "alias2"])
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
            .aliases(vec!["alias1", "alias2"])
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
                    &QueryBuilder::insert()
                        .nodes()
                        .aliases(vec!["alias1", "alias2"])
                        .values(vec![
                            vec![("key", "value").into(), ("key2", "value2").into()],
                            vec![("key", "value3").into()],
                        ])
                        .query(),
                )
                .unwrap();
            assert_eq!(
                transaction
                    .exec(&QueryBuilder::select().ids(vec!["alias1", "alias2"]).query())
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
            .aliases(vec!["alias1", "alias2"])
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
