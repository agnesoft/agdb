mod framework;

use agdb::QueryBuilder;
use agdb::QueryError;
use framework::TestDb;

#[test]
fn insert_alias_of() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.exec_mut(QueryBuilder::insert().alias("alias").of(1).query(), 1);
}

#[test]
fn insert_aliases_of() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut(
        QueryBuilder::insert()
            .aliases(&["alias".into(), "alias2".into()])
            .of(&[1.into(), 2.into()])
            .query(),
        2,
    );
}

#[test]
fn insert_aliases_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias").query(), 1);
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .aliases(&["alias1".into(), "alias2".into()])
            .of(&["alias".into(), 2.into()])
            .query(),
        2,
    );
}

#[test]
fn insert_aliases_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias").query(), 1);
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(
                &QueryBuilder::insert()
                    .aliases(&["alias1".into(), "alias2".into()])
                    .of(&["alias".into(), 2.into()])
                    .query(),
            )?;

            // This fails and causes a rollback
            // since the alias was overwritten
            // in the transaction.
            t.exec(&QueryBuilder::select().id("alias").query())?;
            Ok(())
        },
        "Alias 'alias' not found".into(),
    );

    db.exec(QueryBuilder::select().id("alias").query(), 1);
}

#[test]
fn insert_alias_empty() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert().alias(String::new()).of(1).query(),
        "Empty alias is not allowed",
    );
}

#[test]
fn insert_alias_by_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("old_alias").query(), 1);
    db.exec_mut(QueryBuilder::insert().alias("alias").of(1).query(), 1);
}
