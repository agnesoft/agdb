mod test_db;

use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_ids_aliases() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .query(),
        1,
    );
    db.exec_ids(QueryBuilder::select().ids(&["alias".into()]).query(), &[1]);
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
fn select_missing_aliases() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select().ids(&["alias".into()]).query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn select_missing_ids() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select().ids(&[1.into()]).query(),
        "Id '1' not found",
    );
}

#[test]
fn select_invalid_ids() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select().ids(&[0.into()]).query(),
        "Id '0' not found",
    );
}

#[test]
fn select_from_search() {
    let mut db = TestDb::new();

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&[
                "alias1".into(),
                "alias2".into(),
                "alias3".into(),
                "alias4".into(),
                "alias5".into(),
            ])
            .query(),
        5,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), "alias3".into()])
            .to(&["alias3".into(), "alias5".into()])
            .query(),
        2,
    );

    db.exec_ids(
        QueryBuilder::select()
            .search(QueryBuilder::search().from("alias1").query())
            .query(),
        &[1, -6, 3, -7, 5],
    );
}
