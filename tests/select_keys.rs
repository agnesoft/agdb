mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::DbValue;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_keys_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .values(&[&[
                ("key", 100).into(),
                (1, "value").into(),
                (vec![1.1_f64], 1).into(),
            ]])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().keys().ids(&["alias".into()]).query(),
        &[DbElement {
            index: DbId(1),
            values: vec![
                ("key", DbValue::default()).into(),
                (1, DbValue::default()).into(),
                (vec![1.1_f64], DbValue::default()).into(),
            ],
        }],
    );
}

#[test]
fn select_keys_no_keys() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().keys().ids(&["alias".into()]).query(),
        &[DbElement {
            index: DbId(1),
            values: vec![],
        }],
    );
}

#[test]
fn select_keys_search() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select()
            .keys()
            .search(QueryBuilder::search().from("alias".into()).query())
            .query(),
        "Invalid select keys query",
    );
}
