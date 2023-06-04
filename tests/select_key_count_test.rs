mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_key_count_ids() {
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
        QueryBuilder::select()
            .key_count()
            .ids(&["alias".into()])
            .query(),
        &[DbElement {
            index: DbId(1),
            values: vec![("key_count", 3_u64).into()],
        }],
    );
}

#[test]
fn select_keys_count_no_keys() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select()
            .key_count()
            .ids(&["alias".into()])
            .query(),
        &[DbElement {
            index: DbId(1),
            values: vec![("key_count", 0_u64).into()],
        }],
    );
}

#[test]
fn select_keys_search() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select()
            .key_count()
            .search(QueryBuilder::search().from("alias".into()).query())
            .query(),
        "Invalid select key count query",
    );
}
