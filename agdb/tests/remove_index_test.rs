mod test_db;

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
            t.exec_mut(&QueryBuilder::remove().index("username").query())?;
            Err(QueryError::from("error"))
        },
        QueryError::from("error"),
    );
    db.exec(QueryBuilder::select().indexes().query(), 1);
}
