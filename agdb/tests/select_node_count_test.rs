mod test_db;

use crate::test_db::TestDb;
use agdb::QueryBuilder;

#[test]
fn select_node_count_empty() {
    let db = TestDb::new();

    let result = db.exec_result(QueryBuilder::select().node_count().query());
    assert_eq!(result.result, 0);
    assert!(result.elements.is_empty());
}

#[test]
fn select_node_count() {
    let mut db = TestDb::new();

    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);

    let result = db.exec_result(QueryBuilder::select().node_count().query());
    assert_eq!(result.result, 5);
    assert!(result.elements.is_empty());
}

#[test]
fn select_node_count_after_removal() {
    let mut db = TestDb::new();

    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);
    db.exec_mut(QueryBuilder::remove().ids([2, 4]).query(), 2);

    let result = db.exec_result(QueryBuilder::select().node_count().query());
    assert_eq!(result.result, 3);
    assert!(result.elements.is_empty());
}
