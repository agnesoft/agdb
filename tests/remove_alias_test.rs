mod framework;

use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryResult;
use framework::TestDb;

#[test]
fn remove_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias").query(), 1);
    db.exec_mut(QueryBuilder::remove().alias("alias").query(), -1);
    db.exec_error(
        QueryBuilder::select().id("alias").query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn remove_aliases() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into(), "alias2".into()])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .aliases(&["alias".into(), "alias2".into()])
            .query(),
        -2,
    );
}

#[test]
fn remove_aliases_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into(), "alias2".into()])
            .query(),
        2,
    );

    db.transaction_mut_error(
        |t| -> Result<QueryResult, QueryError> {
            t.exec_mut(
                &QueryBuilder::remove()
                    .aliases(&["alias".into(), "alias2".into()])
                    .query(),
            )?;
            t.exec(&QueryBuilder::select().id("alias2").query())
        },
        "Alias 'alias2' not found".into(),
    );

    db.exec(QueryBuilder::select().id("alias2").query(), 1);
}

#[test]
fn remove_missing_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().alias("alias").query(), 0);
}

#[test]
fn remove_missing_alias_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::remove().alias("alias").query())?;
            Err("error".into())
        },
        "error".into(),
    );
}
