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
            .aliases(vec!["alias", "alias2"])
            .values_uniform(vec![("key1", "value1").into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(vec!["alias", "alias2"]).query(),
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
            .values(vec!["key1".into()])
            .ids(vec!["alias", "alias2"])
            .query(),
        -2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(vec!["alias", "alias2"]).query(),
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
            .values(vec![vec![("key", 1).into()], vec![("key", 2).into()]])
            .query(),
        2,
    );
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(2).query(), 1);
    db.exec_mut(
        QueryBuilder::remove()
            .values(vec!["key".into()])
            .ids(QueryBuilder::search().from(1).query())
            .query(),
        -2,
    );
    db.exec_elements(
        QueryBuilder::select().ids(vec![1, 2]).query(),
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
            .aliases(vec!["alias", "alias2"])
            .values_uniform(vec![("key1", "value1").into(), ("key2", 100).into()])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .values(vec!["key3".into()])
            .ids(vec!["alias", "alias2"])
            .query(),
        0,
    );
    db.exec_elements(
        QueryBuilder::select().ids(vec!["alias", "alias2"]).query(),
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
