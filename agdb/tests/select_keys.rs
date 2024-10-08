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
            .aliases("alias")
            .values([[
                ("key", 100).into(),
                (1, "value").into(),
                (vec![1.1_f64], 1).into(),
            ]])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().keys().ids("alias").query(),
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
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
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias").query(), 1);
    db.exec_elements(
        QueryBuilder::select().keys().ids("alias").query(),
        &[DbElement {
            id: DbId(1),
            from: None,
            to: None,
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
            .values_uniform([
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
            .from([1, 3])
            .to([3, 5])
            .query(),
        2,
    );

    db.exec_elements(
        QueryBuilder::select()
            .keys()
            .ids(QueryBuilder::search().from(3).query())
            .query(),
        &[
            DbElement {
                id: DbId(3),
                from: None,
                to: None,
                values: vec![
                    ("key1", DbValue::default()).into(),
                    ("key2", DbValue::default()).into(),
                    ("key3", DbValue::default()).into(),
                ],
            },
            DbElement {
                id: DbId(-7),
                from: Some(DbId(3)),
                to: Some(DbId(5)),
                values: vec![],
            },
            DbElement {
                id: DbId(5),
                from: None,
                to: None,
                values: vec![
                    ("key1", DbValue::default()).into(),
                    ("key2", DbValue::default()).into(),
                    ("key3", DbValue::default()).into(),
                ],
            },
        ],
    );
}

#[test]
fn select_keys_search_alt() {
    let mut db = TestDb::new();

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(5)
            .values_uniform([
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
            .from([1, 3])
            .to([3, 5])
            .query(),
        2,
    );

    db.exec_elements(
        QueryBuilder::select().keys().search().from(3).query(),
        &[
            DbElement {
                id: DbId(3),
                from: None,
                to: None,
                values: vec![
                    ("key1", DbValue::default()).into(),
                    ("key2", DbValue::default()).into(),
                    ("key3", DbValue::default()).into(),
                ],
            },
            DbElement {
                id: DbId(-7),
                from: Some(DbId(3)),
                to: Some(DbId(5)),
                values: vec![],
            },
            DbElement {
                id: DbId(5),
                from: None,
                to: None,
                values: vec![
                    ("key1", DbValue::default()).into(),
                    ("key2", DbValue::default()).into(),
                    ("key3", DbValue::default()).into(),
                ],
            },
        ],
    );
}
