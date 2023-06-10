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
                id: DbId(1),
                values: vec![("alias", "alias1").into()],
            },
            DbElement {
                id: DbId(2),
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
                id: DbId(1),
                values: vec![("alias", "alias1").into()],
            },
            DbElement {
                id: DbId(2),
                values: vec![("alias", "alias2").into()],
            },
        ],
    );
}

#[test]
fn select_aliases_search() {
    let mut db = TestDb::new();

    let values = [
        ("key1", 1).into(),
        ("key2", 10).into(),
        ("key3", 100).into(),
    ];

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
            .from(&[1.into(), 3.into()])
            .to(&[3.into(), 5.into()])
            .values_uniform(&values)
            .query(),
        2,
    );

    db.exec_elements(
        QueryBuilder::select()
            .aliases()
            .search(QueryBuilder::search().from(3.into()).query())
            .query(),
        &[
            DbElement {
                id: DbId(3),
                values: vec![("alias", "alias3").into()],
            },
            DbElement {
                id: DbId(5),
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
            .aliases(&["alias1".into(), "alias2".into(), "alias3".into()])
            .query(),
        3,
    );
    db.exec_elements(
        QueryBuilder::select().aliases().query(),
        &[
            DbElement {
                id: DbId(1),
                values: vec![("alias", "alias1").into()],
            },
            DbElement {
                id: DbId(2),
                values: vec![("alias", "alias2").into()],
            },
            DbElement {
                id: DbId(3),
                values: vec![("alias", "alias3").into()],
            },
        ],
    );
}
