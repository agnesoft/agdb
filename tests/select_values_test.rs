mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_values_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["alias1", "alias2"])
            .values(vec![
                vec![
                    ("key", "value").into(),
                    ("key2", "value2").into(),
                    ("key3", "value3").into(),
                ],
                vec![
                    ("key", "value4").into(),
                    ("key2", "value5").into(),
                    ("key3", "value6").into(),
                ],
            ])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select()
            .values(vec!["key".into(), "key2".into()])
            .ids(vec!["alias1", "alias2"])
            .query(),
        &[
            DbElement {
                id: DbId(1),
                values: vec![("key", "value").into(), ("key2", "value2").into()],
            },
            DbElement {
                id: DbId(2),
                values: vec![("key", "value4").into(), ("key2", "value5").into()],
            },
        ],
    );
}

#[test]
fn select_values_ids_missing_key() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["alias1", "alias2"])
            .values(vec![
                vec![
                    ("key", "value").into(),
                    ("key2", "value2").into(),
                    ("key3", "value3").into(),
                ],
                vec![("key", "value4").into()],
            ])
            .query(),
        2,
    );
    db.exec_error(
        QueryBuilder::select()
            .values(vec!["key".into(), "key2".into()])
            .ids(vec!["alias1", "alias2"])
            .query(),
        "Missing key 'key2' for id '2'",
    );
}

#[test]
fn select_values_search() {
    let mut db = TestDb::new();

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(5)
            .values_uniform(vec![
                ("key1", 1).into(),
                ("key2", 10).into(),
                ("key3", 100).into(),
            ])
            .query(),
        5,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec![1, 3])
            .to(vec![3, 5])
            .query(),
        2,
    );

    db.exec_elements(
        QueryBuilder::select()
            .values(vec!["key2".into()])
            .search(QueryBuilder::search().from(3).query())
            .query(),
        &[
            DbElement {
                id: DbId(3),
                values: vec![("key2", 10).into()],
            },
            DbElement {
                id: DbId(-7),
                values: vec![],
            },
            DbElement {
                id: DbId(5),
                values: vec![("key2", 10).into()],
            },
        ],
    );
}
