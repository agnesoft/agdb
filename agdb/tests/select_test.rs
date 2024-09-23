mod test_db;

use agdb::DbElement;
use agdb::DbId;
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
            .aliases(["alias", "alias2"])
            .query(),
        2,
    );
    db.exec_ids(
        QueryBuilder::select().ids(["alias", "alias2"]).query(),
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
            .aliases(["alias1", "alias2", "alias3", "alias4", "alias5"])
            .query(),
        5,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias3"])
            .to(["alias3", "alias5"])
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

#[test]
fn select_embedded_search() {
    let mut db = TestDb::new();

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3", "alias4", "alias5"])
            .values([
                [("k", 1).into()],
                [("k", 2).into()],
                [("k", 3).into()],
                [("k", 4).into()],
                [("k", 5).into()],
            ])
            .query(),
        5,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias3"])
            .to(["alias3", "alias5"])
            .query(),
        2,
    );

    db.exec_elements(
        QueryBuilder::select()
            .search()
            .from("alias1")
            .where_()
            .node()
            .query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("k", 1).into()],
            },
            DbElement {
                id: DbId(3),
                from: None,
                to: None,
                values: vec![("k", 3).into()],
            },
            DbElement {
                id: DbId(5),
                from: None,
                to: None,
                values: vec![("k", 5).into()],
            },
        ],
    );
}
