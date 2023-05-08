mod framework;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use framework::TestDb;

#[test]
fn insert_values_id() {
    let mut db = TestDb::new();
    db.exec_mut_ids(QueryBuilder::insert().node().alias("alias").query(), &[1]);
    db.exec_mut(
        QueryBuilder::insert()
            .values(&[("key", "value").into()])
            .id("alias")
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().id("alias").query(),
        &[DbElement {
            index: DbId(1),
            values: vec![("key", "value").into()],
        }],
    );
}

#[test]
fn insert_values_ids() {
    let _query = QueryBuilder::insert()
        .values(&[("key", "value").into()])
        .ids(&["alias".into()])
        .query();
}

#[test]
fn insert_values_search() {
    let _query = QueryBuilder::insert()
        .values(&[("key", "value").into()])
        .search(QueryBuilder::search().from(1.into()).query())
        .query();
}

#[test]
fn insert_values_multi_ids() {
    let _query = QueryBuilder::insert()
        .values_multi(&[&[("key", "value").into()]])
        .ids(&["alias".into()])
        .query();
}

#[test]
fn insert_values_multi_search() {
    let _query = QueryBuilder::insert()
        .values_multi(&[&[("key", "value").into()]])
        .search(QueryBuilder::search().from(1.into()).query())
        .query();
}
