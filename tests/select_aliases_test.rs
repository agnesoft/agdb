mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_aliases_missing_id() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select().aliases().ids(&[1.into()]).query(),
        "Id '1' not found",
    );
}

#[test]
fn select_aliases_missing_alias() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select()
            .aliases()
            .ids(&["alias".into()])
            .query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn select_aliases_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into(), "alias2".into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .aliases()
            .ids(&[1.into(), 2.into()])
            .query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("alias", "alias1").into()],
            },
            DbElement {
                index: DbId(2),
                values: vec![("alias", "alias2").into()],
            },
        ],
    );
}

#[test]
fn select_aliases_aliases() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into(), "alias2".into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .aliases()
            .ids(&["alias1".into(), "alias2".into()])
            .query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("alias", "alias1").into()],
            },
            DbElement {
                index: DbId(2),
                values: vec![("alias", "alias2").into()],
            },
        ],
    );
}

#[test]
fn select_aliases_search() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select()
            .aliases()
            .search(QueryBuilder::search().from(1.into()).query())
            .query(),
        "Invalid select aliases query",
    );
}

#[test]
fn select_all_aliases_empty() {
    let db = TestDb::new();
    db.exec(QueryBuilder::select().aliases().query(), 0);
}

#[test]
fn select_all_aliases() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into(), "alias2".into(), "alias3".into()])
            .query(),
        3,
    );
    db.exec_elements(
        QueryBuilder::select().aliases().query(),
        &[
            DbElement {
                index: DbId(1),
                values: vec![("alias", "alias1").into()],
            },
            DbElement {
                index: DbId(2),
                values: vec![("alias", "alias2").into()],
            },
            DbElement {
                index: DbId(3),
                values: vec![("alias", "alias3").into()],
            },
        ],
    );
}
