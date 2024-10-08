mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_aliases_missing_id() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select().aliases().ids(1).query(),
        "Id '1' not found",
    );
}

#[test]
fn select_aliases_missing_alias() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select().aliases().ids("alias").query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn select_aliases_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2"])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().aliases().ids([1, 2]).query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("alias", "alias1").into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
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
            .aliases(["alias1", "alias2"])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .aliases()
            .ids(["alias1", "alias2"])
            .query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("alias", "alias1").into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("alias", "alias2").into()],
            },
        ],
    );
}

#[test]
fn select_aliases_search() {
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
            .from([1, 3])
            .to([3, 5])
            .values_uniform([
                ("key1", 1).into(),
                ("key2", 10).into(),
                ("key3", 100).into(),
            ])
            .query(),
        2,
    );

    db.exec_elements(
        QueryBuilder::select()
            .aliases()
            .ids(QueryBuilder::search().from(3).query())
            .query(),
        &[
            DbElement {
                id: DbId(3),
                from: None,
                to: None,
                values: vec![("alias", "alias3").into()],
            },
            DbElement {
                id: DbId(5),
                from: None,
                to: None,
                values: vec![("alias", "alias5").into()],
            },
        ],
    );
}

#[test]
fn select_aliases_search_alt() {
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
            .from([1, 3])
            .to([3, 5])
            .values_uniform([
                ("key1", 1).into(),
                ("key2", 10).into(),
                ("key3", 100).into(),
            ])
            .query(),
        2,
    );

    db.exec_elements(
        QueryBuilder::select().aliases().search().from(3).query(),
        &[
            DbElement {
                id: DbId(3),
                from: None,
                to: None,
                values: vec![("alias", "alias3").into()],
            },
            DbElement {
                id: DbId(5),
                from: None,
                to: None,
                values: vec![("alias", "alias5").into()],
            },
        ],
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
            .aliases(["alias1", "alias2", "alias3"])
            .query(),
        3,
    );
    db.exec_elements(
        QueryBuilder::select().aliases().query(),
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("alias", "alias1").into()],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("alias", "alias2").into()],
            },
            DbElement {
                id: DbId(3),
                from: None,
                to: None,
                values: vec![("alias", "alias3").into()],
            },
        ],
    );
}
