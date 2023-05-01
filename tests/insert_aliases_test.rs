#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::QueryBuilder;
use agdb::QueryError;
use test_file::TestFile;

mod framework;
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
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().alias("alias").query())
        .unwrap();
    db.exec_mut(&QueryBuilder::insert().node().query()).unwrap();

    let error = db
        .transaction_mut(|transaction| -> Result<(), QueryError> {
            let query = QueryBuilder::insert()
                .aliases(&["alias1".into(), "alias2".into()])
                .of(&["alias".into(), 2.into()])
                .query();
            let _ = transaction.exec_mut(&query).unwrap();
            let _ = transaction
                .exec(&QueryBuilder::select().id("alias1".into()).query())
                .unwrap();
            let _ = transaction
                .exec(&QueryBuilder::select().id("alias2".into()).query())
                .unwrap();

            // This fails and causes a rollback
            // since the alias was overwritten
            // in the transaction.
            transaction.exec(&QueryBuilder::select().id("alias".into()).query())?;

            Ok(())
        })
        .unwrap_err();

    let _ = db
        .exec(&QueryBuilder::select().id("alias".into()).query())
        .unwrap();
    assert_eq!(error.description, "Alias 'alias' not found");
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
