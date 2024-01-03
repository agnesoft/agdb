mod test_db;

use agdb::QueryBuilder;
use agdb::QueryError;
use test_db::TestDb;

#[test]
fn insert_empty_index() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
}

#[test]
fn insert_index_with_existing_data() {
    let mut db = TestDb::new();
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

    db.exec_mut(QueryBuilder::insert().index("username").query(), 3);
}

#[test]
fn insert_existing_index() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut_error(
        QueryBuilder::insert().index("username").query(),
        "Index 'username' already exists",
    );
}

#[test]
fn insert_index_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |t| {
            t.exec_mut(&QueryBuilder::insert().index("username").query())?;
            t.exec_mut(
                &QueryBuilder::insert()
                    .nodes()
                    .values(vec![
                        vec![("username", "user1").into()],
                        vec![("username", "user2").into()],
                        vec![("username", "user3").into()],
                    ])
                    .query(),
            )?;
            t.exec_mut(&QueryBuilder::insert().index("username").query())?;
            Ok(())
        },
        QueryError::from("Index 'username' already exists"),
    );
    db.exec(QueryBuilder::select().indexes().query(), 0);
}
