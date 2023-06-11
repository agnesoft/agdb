mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn remove_values_ids() {
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
                id: DbId(1),
                values: vec![("key1", "value1").into()],
            },
            DbElement {
                id: DbId(2),
                values: vec![("key1", "value1").into()],
            },
        ],
    );
    db.exec_mut(
        QueryBuilder::remove()
            .values(&["key1".into()])
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
                id: DbId(1),
                values: vec![],
            },
            DbElement {
                id: DbId(2),
                values: vec![],
            },
        ],
    );
}

#[test]
fn remove_values_search() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(&[&[("key", 1).into()], &[("key", 2).into()]])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[1.into()])
            .to(&[2.into()])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .values(&["key".into()])
            .search(QueryBuilder::search().from(1).query())
            .query(),
        -2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(&[1.into(), 2.into()]).query(),
        &[
            DbElement {
                id: DbId(1),
                values: vec![],
            },
            DbElement {
                id: DbId(2),
                values: vec![],
            },
        ],
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
            .values(&["key3".into()])
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
                id: DbId(1),
                values: vec![("key1", "value1").into(), ("key2", 100).into()],
            },
            DbElement {
                id: DbId(2),
                values: vec![("key1", "value1").into(), ("key2", 100).into()],
            },
        ],
    );
}
