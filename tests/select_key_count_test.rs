mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_key_count_ids() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases("alias")
            .values(vec![vec![
                ("key", 100).into(),
                (1, "value").into(),
                (vec![1.1_f64], 1).into(),
            ]])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().key_count().ids("alias").query(),
        &[DbElement {
            id: DbId(1),
            values: vec![("key_count", 3_u64).into()],
        }],
    );
}

#[test]
fn select_keys_count_no_keys() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias").query(), 1);
    db.exec_elements(
        QueryBuilder::select().key_count().ids("alias").query(),
        &[DbElement {
            id: DbId(1),
            values: vec![("key_count", 0_u64).into()],
        }],
    );
}

#[test]
fn select_keys_search() {
    let mut db = TestDb::new();

    let values = vec![
        ("key1", 1).into(),
        ("key2", 10).into(),
        ("key3", 100).into(),
    ];

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(5)
            .values_uniform(values.clone())
            .query(),
        5,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec![1, 3])
            .to(vec![3, 5])
            .values_uniform(values)
            .query(),
        2,
    );

    db.exec_elements(
        QueryBuilder::select()
            .key_count()
            .search(QueryBuilder::search().from(3).query())
            .query(),
        &[
            DbElement {
                id: DbId(3),
                values: vec![("key_count", 3_u64).into()],
            },
            DbElement {
                id: DbId(-7),
                values: vec![("key_count", 3_u64).into()],
            },
            DbElement {
                id: DbId(5),
                values: vec![("key_count", 3_u64).into()],
            },
        ],
    );
}
