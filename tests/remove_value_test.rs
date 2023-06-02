mod framework;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use framework::TestDb;

#[test]
fn remove_value_id() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .values(&[&[("key1", "value1").into()]])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().id("alias").query(),
        &[DbElement {
            index: DbId(1),
            values: vec![("key1", "value1").into()],
        }],
    );
    db.exec_mut(QueryBuilder::remove().value("key1").id("alias").query(), -1);
    db.exec_elements(
        QueryBuilder::select().id("alias").query(),
        &[DbElement {
            index: DbId(1),
            values: vec![],
        }],
    );
}

#[test]
fn remove_value_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into(), "alias2".into()])
            .values_uniform(&[("key1", "value1").into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("key1", "value1").into()],
            },
            DbElement {
                index: DbId(2),
                values: vec![("key1", "value1").into()],
            },
        ],
    );
    db.exec_mut(
        QueryBuilder::remove()
            .value("key1")
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        -2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![],
            },
            DbElement {
                index: DbId(2),
                values: vec![],
            },
        ],
    );
}

#[test]
fn remove_value_search() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::remove()
            .value("key1")
            .search(QueryBuilder::search().from("alias1".into()).query())
            .query(),
        "Invalid remove query",
    );
}

#[test]
fn remove_values_id() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .values(&[&[("key1", "value1").into(), ("key2", 100).into()]])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().id("alias").query(),
        &[DbElement {
            index: DbId(1),
            values: vec![("key1", "value1").into(), ("key2", 100).into()],
        }],
    );
    db.exec_mut(
        QueryBuilder::remove()
            .values(&["key1".into(), "key2".into()])
            .id("alias")
            .query(),
        -2,
    );
    db.exec_elements(
        QueryBuilder::select().id("alias").query(),
        &[DbElement {
            index: DbId(1),
            values: vec![],
        }],
    );
}

#[test]
fn remove_values_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into(), "alias2".into()])
            .values_uniform(&[("key1", "value1").into(), ("key2", 100).into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("key1", "value1").into(), ("key2", 100).into()],
            },
            DbElement {
                index: DbId(2),
                values: vec![("key1", "value1").into(), ("key2", 100).into()],
            },
        ],
    );
    db.exec_mut(
        QueryBuilder::remove()
            .values(&["key1".into(), "key2".into()])
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        -4,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![],
            },
            DbElement {
                index: DbId(2),
                values: vec![],
            },
        ],
    );
}

#[test]
fn remove_values_search() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::remove()
            .values(&["key1".into(), "key2".into()])
            .search(QueryBuilder::search().from("alias1".into()).query())
            .query(),
        "Invalid remove query",
    );
}

#[test]
fn remove_missing_key() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into(), "alias2".into()])
            .values_uniform(&[("key1", "value1").into(), ("key2", 100).into()])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .value("key3")
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        0,
    );
    db.exec_elements(
        QueryBuilder::select()
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("key1", "value1").into(), ("key2", 100).into()],
            },
            DbElement {
                index: DbId(2),
                values: vec![("key1", "value1").into(), ("key2", 100).into()],
            },
        ],
    );
}
