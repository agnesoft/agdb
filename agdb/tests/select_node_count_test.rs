mod test_db;

use crate::test_db::TestDb;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;

#[test]
fn select_node_count_empty() {
    let db = TestDb::new();

    db.exec_elements(
        QueryBuilder::select().node_count().query(),
        &[DbElement {
            id: DbId(0),
            from: None,
            to: None,
            values: vec![("node_count", 0_u64).into()],
        }],
    );
}

#[test]
fn select_node_count() {
    let mut db = TestDb::new();

    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);

    db.exec_elements(
        QueryBuilder::select().node_count().query(),
        &[DbElement {
            id: DbId(0),
            from: None,
            to: None,
            values: vec![("node_count", 5_u64).into()],
        }],
    );
}

#[test]
fn select_node_count_after_removal() {
    let mut db = TestDb::new();

    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);
    db.exec_mut(QueryBuilder::remove().ids(vec![2, 4]).query(), -2);

    db.exec_elements(
        QueryBuilder::select().node_count().query(),
        &[DbElement {
            id: DbId(0),
            from: None,
            to: None,
            values: vec![("node_count", 3_u64).into()],
        }],
    );
}
