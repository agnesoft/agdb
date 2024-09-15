mod test_db;

use agdb::DbValue;
use agdb::QueryBuilder;
use agdb::QueryError;
use test_db::TestDb;

#[test]
fn remove_index_with_data() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("username", "user1").into()],
                vec![("username", "user2").into()],
                vec![("username", "user3").into()],
            ])
            .query(),
        3,
    );
    db.exec_mut(QueryBuilder::remove().index("username").query(), -3);
    db.exec(QueryBuilder::select().indexes().query(), 0);
}

#[test]
fn rmove_missing_index() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().index("username").query(), 0);
}

#[test]
fn remove_index_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("username", "user1").into()],
                vec![("username", "user2").into()],
                vec![("username", "user3").into()],
            ])
            .query(),
        3,
    );
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(QueryBuilder::remove().index("username").query())?;
            Err(QueryError::from("error"))
        },
        QueryError::from("error"),
    );
    db.exec(QueryBuilder::select().indexes().query(), 1);
}

#[test]
fn remove_node_with_indexed_values() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![("username", "user2").into(), ("age", 33).into()],
                vec![("username", "user3").into()],
            ])
            .query(),
        3,
    );
    db.exec_mut(QueryBuilder::remove().ids(2).query(), -1);
    let result = db.exec_result(QueryBuilder::select().indexes().query());
    assert_eq!(result.elements[0].values[0].value, DbValue::from(2_u64));
}

#[test]
fn remove_indexed_key() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![("username", "user2").into(), ("age", 33).into()],
                vec![("username", "user3").into()],
            ])
            .query(),
        3,
    );
    db.exec_mut(QueryBuilder::remove().values("username").ids(2).query(), -1);
    let result = db.exec_result(QueryBuilder::select().indexes().query());
    assert_eq!(result.elements[0].values[0].value, DbValue::from(2_u64));
}

#[test]
fn remove_indexed_key_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![("username", "user2").into(), ("age", 33).into()],
                vec![("username", "user3").into()],
            ])
            .query(),
        3,
    );
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(QueryBuilder::remove().values("username").ids(2).query())?;
            Err(QueryError::from("error"))
        },
        QueryError::from("error"),
    );

    let result = db.exec_result(QueryBuilder::select().indexes().query());
    assert_eq!(result.elements[0].values[0].value, DbValue::from(3_u64));
}
