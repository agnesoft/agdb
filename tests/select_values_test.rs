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
            .aliases(&["alias1".to_string(), "alias2".to_string()])
            .values(&[
                &[
                    ("key", "value").into(),
                    ("key2", "value2").into(),
                    ("key3", "value3").into(),
                ],
                &[
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
            .values(&["key".into(), "key2".into()])
            .ids(&["alias1".into(), "alias2".into()])
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
            .aliases(&["alias1".to_string(), "alias2".to_string()])
            .values(&[
                &[
                    ("key", "value").into(),
                    ("key2", "value2").into(),
                    ("key3", "value3").into(),
                ],
                &[("key", "value4").into()],
            ])
            .query(),
        2,
    );
    db.exec_error(
        QueryBuilder::select()
            .values(&["key".into(), "key2".into()])
            .ids(&["alias1".into(), "alias2".into()])
            .query(),
        "Missing key 'key2' for id '2'",
    );
}

#[test]
fn select_values_search() {
    let db = TestDb::new();
    db.exec_error(
        QueryBuilder::select()
            .values(&["key1".into(), "key2".into()])
            .search(QueryBuilder::search().from("alias".into()).query())
            .query(),
        "Invalid select values query",
    );
}
