mod test_db;

use agdb::DbError;
use agdb::DbErrorType;
use agdb::QueryBuilder;
use agdb::QueryResult;
use test_db::TestDb;

#[test]
fn remove_aliases() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias", "alias2"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::remove().aliases(["alias", "alias2"]).query(),
        -2,
    );
}

#[test]
fn remove_aliases_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias", "alias2"])
            .query(),
        2,
    );

    db.transaction_mut_error(
        |t| -> Result<QueryResult, DbError> {
            t.exec_mut(QueryBuilder::remove().aliases(["alias", "alias2"]).query())?;
            t.exec(QueryBuilder::select().ids("alias2").query())
        },
        DbError::db(DbErrorType::NotFound, "Alias 'alias2' not found"),
    );

    db.exec(QueryBuilder::select().ids("alias2").query(), 1);
}

#[test]
fn remove_missing_aliases() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().aliases("alias").query(), 0);
}

#[test]
fn remove_missing_alias_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::remove().aliases("alias").query())?;
            Err(DbError::db(DbErrorType::NotAllowed, "error"))
        },
        DbError::db(DbErrorType::NotAllowed, "error"),
    );
}
