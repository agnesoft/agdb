mod framework;

use agdb::QueryBuilder;
use framework::TestDb;

#[test]
fn select_id_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias").query(), 1);
    db.exec_ids(QueryBuilder::select().id("alias").query(), &[1]);
}

#[test]
fn select_from_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into(), "alias2".into()])
            .query(),
        2,
    );
    db.exec_ids(
        QueryBuilder::select()
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        &[1, 2],
    );
}

#[test]
fn select_missing_alias() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select().id("alias").query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn select_missing_id() {
    let db = TestDb::new();
    db.exec_error(QueryBuilder::select().id(1).query(), "Id '1' not found");
}

#[test]
fn select_from_search() {
    let _query = QueryBuilder::select()
        .search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
