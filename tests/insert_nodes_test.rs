mod framework;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryError;
use framework::TestDb;

#[test]
fn insert_nodes_aliases_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |transaction| -> Result<(), QueryError> {
            transaction.exec_mut(
                &QueryBuilder::insert()
                    .nodes()
                    .aliases(&["alias".into()])
                    .query(),
            )?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_error(
        QueryBuilder::select().id("alias").query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn insert_node_existing_alias() {
    let mut db = TestDb::new();
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .query(),
        &[1],
    );
    db.exec_mut_error(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .query(),
        "Alias 'alias' already exists (1)",
    )
}

#[test]
fn insert_nodes_aliases() {
    let mut db = TestDb::new();
    db.exec_mut_ids(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".to_string(), "alias2".to_string()])
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
            .aliases(&["alias1".to_string(), "alias2".to_string()])
            .values(&[
                &[("key", "value").into(), ("key2", "value2").into()],
                &[("key", "value3").into()],
            ])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&["alias1".into(), "alias2".into()])
            .query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
            DbElement {
                index: DbId(2),
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
                        .aliases(&["alias1".to_string(), "alias2".to_string()])
                        .values(&[
                            &[("key", "value").into(), ("key2", "value2").into()],
                            &[("key", "value3").into()],
                        ])
                        .query(),
                )
                .unwrap();
            assert_eq!(
                transaction
                    .exec(
                        &QueryBuilder::select()
                            .ids(&["alias1".into(), "alias2".into()])
                            .query()
                    )
                    .unwrap()
                    .elements,
                &[
                    DbElement {
                        index: DbId(1),
                        values: vec![("key", "value").into(), ("key2", "value2").into()],
                    },
                    DbElement {
                        index: DbId(2),
                        values: vec![("key", "value3").into()],
                    },
                ],
            );
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_error(
        QueryBuilder::select().id("alias1").query(),
        "Alias 'alias1' not found",
    );
    db.exec_error(
        QueryBuilder::select().id("alias2").query(),
        "Alias 'alias2' not found",
    );
}

#[test]
fn insert_nodes_aliases_values_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".to_string(), "alias2".to_string()])
            .values_uniform(&[("key", "value").into(), ("key2", "value2").into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&["alias1".into(), "alias2".into()])
            .query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
            DbElement {
                index: DbId(2),
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
            .values_uniform(&[("key", "value").into(), ("key2", "value2").into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(&[1.into(), 2.into()]).query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
            DbElement {
                index: DbId(2),
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
            .values(&[
                &[("key", "value").into(), ("key2", "value2").into()],
                &[("key", "value3").into()],
            ])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(&[1.into(), 2.into()]).query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
            DbElement {
                index: DbId(2),
                values: vec![("key", "value3").into()],
            },
        ],
    );
}

#[test]
fn insert_nodes_values_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values_uniform(&[("key", "value").into(), ("key2", "value2").into()])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().id(1).query(),
        &[DbElement {
            index: DbId(1),
            values: vec![("key", "value").into(), ("key2", "value2").into()],
        }],
    );
}
