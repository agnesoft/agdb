mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::DbValue;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_keys_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .values(&[&[
                ("key", 100).into(),
                (1, "value").into(),
                (vec![1.1_f64], 1).into(),
            ]])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().keys().ids(&["alias".into()]).query(),
        &[DbElement {
            id: DbId(1),
            values: vec![
                ("key", DbValue::default()).into(),
                (1, DbValue::default()).into(),
                (vec![1.1_f64], DbValue::default()).into(),
            ],
        }],
    );
}

#[test]
fn select_keys_no_keys() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into()])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().keys().ids(&["alias".into()]).query(),
        &[DbElement {
            id: DbId(1),
            values: vec![],
        }],
    );
}

#[test]
fn select_keys_search() {
    let mut db = TestDb::new();

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(5)
            .values_uniform(&[
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
            .from(&[1.into(), 3.into()])
            .to(&[3.into(), 5.into()])
            .query(),
        2,
    );

    db.exec_elements(
        QueryBuilder::select()
            .keys()
            .search(QueryBuilder::search().from(3.into()).query())
            .query(),
        &[
            DbElement {
                id: DbId(3),
                values: vec![
                    ("key1", DbValue::default()).into(),
                    ("key2", DbValue::default()).into(),
                    ("key3", DbValue::default()).into(),
                ],
            },
            DbElement {
                id: DbId(-7),
                values: vec![],
            },
            DbElement {
                id: DbId(5),
                values: vec![
                    ("key1", DbValue::default()).into(),
                    ("key2", DbValue::default()).into(),
                    ("key3", DbValue::default()).into(),
                ],
            },
        ],
    );
}
