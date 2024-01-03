mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn select_indexes_empty() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
}

#[test]
fn select_indexes() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(QueryBuilder::insert().index("age").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![("username", "user2").into(), ("age", 20).into()],
                vec![("username", "user3").into()],
            ])
            .query(),
        3,
    );

    let result = db.exec_result(QueryBuilder::select().indexes().query());
    assert_eq!(result.result, 2);
    assert_eq!(
        result.elements,
        vec![DbElement {
            id: DbId(0),
            from: None,
            to: None,
            values: vec![("username", 3_u64).into(), ("age", 2_u64).into()],
        }]
    );
}
