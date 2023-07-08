mod test_db;

use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_ids_aliases() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias").query(), 1);
    db.exec_ids(QueryBuilder::select().ids("alias").query(), &[1]);
}

#[test]
fn select_from_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["alias", "alias2"])
            .query(),
        2,
    );
    db.exec_ids(
        QueryBuilder::select().ids(vec!["alias", "alias2"]).query(),
        &[1, 2],
    );
}

#[test]
fn select_missing_aliases() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select().ids("alias").query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn select_missing_ids() {
    let db = TestDb::new();
    db.exec_error(QueryBuilder::select().ids(1).query(), "Id '1' not found");
}

#[test]
fn select_invalid_ids() {
    let db = TestDb::new();
    db.exec_error(QueryBuilder::select().ids(0).query(), "Id '0' not found");
}

#[test]
fn select_from_search() {
    let mut db = TestDb::new();

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["alias1", "alias2", "alias3", "alias4", "alias5"])
            .query(),
        5,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec!["alias1", "alias3"])
            .to(vec!["alias3", "alias5"])
            .query(),
        2,
    );

    db.exec_ids(
        QueryBuilder::select()
            .ids(QueryBuilder::search().from("alias1").query())
            .query(),
        &[1, -6, 3, -7, 5],
    );
}
