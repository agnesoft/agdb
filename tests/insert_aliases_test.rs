#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::QueryBuilder;
use agdb::QueryError;
use test_file::TestFile;

#[test]
fn insert_alias_id() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().query()).unwrap();
    let query = QueryBuilder::insert().alias("alias").id(1.into()).query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, 0);
    assert_eq!(result.elements, vec![]);
}

#[test]
fn insert_aliases_ids() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().nodes().count(2).query())
        .unwrap();
    let query = QueryBuilder::insert()
        .aliases(&["alias".into(), "alias2".into()])
        .ids(&[1.into(), 2.into()])
        .query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, 0);
    assert_eq!(result.elements, vec![]);
}

#[test]
fn insert_aliases_alias() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().alias("alias").query())
        .unwrap();
    db.exec_mut(&QueryBuilder::insert().node().query()).unwrap();
    let query = QueryBuilder::insert()
        .aliases(&["alias1".into(), "alias2".into()])
        .ids(&["alias".into(), 2.into()])
        .query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, 0);
    assert_eq!(result.elements, vec![]);
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
                .ids(&["alias".into(), 2.into()])
                .query();
            let _ = transaction.exec_mut(&query)?;
            let _ = transaction.exec(&QueryBuilder::select().id("alias1".into()).query())?;
            let _ = transaction.exec(&QueryBuilder::select().id("alias2".into()).query())?;

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
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().query()).unwrap();
    let query = QueryBuilder::insert().alias("").id(1.into()).query();
    let error = db.exec_mut(&query).unwrap_err();

    assert_eq!(error.description, "Empty alias is not allowed");
}
