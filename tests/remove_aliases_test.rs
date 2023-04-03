#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryResult;
use test_file::TestFile;

#[test]
fn remove_alias() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().alias("alias").query())
        .unwrap();
    let query = QueryBuilder::remove().alias("alias").query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, -1);
    assert_eq!(result.elements, vec![]);
}

#[test]
fn remove_aliases() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(
        &QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into(), "alias2".into()])
            .query(),
    )
    .unwrap();
    let query = QueryBuilder::remove()
        .aliases(&["alias".into(), "alias2".into()])
        .query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, -2);
    assert_eq!(result.elements, vec![]);
}

#[test]
fn remove_aliases_rollback() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(
        &QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into(), "alias2".into()])
            .query(),
    )
    .unwrap();

    let error = db
        .transaction_mut(|transaction| -> Result<QueryResult, QueryError> {
            let query = QueryBuilder::remove()
                .aliases(&["alias".into(), "alias2".into()])
                .query();
            let result = transaction.exec_mut(&query).unwrap();

            assert_eq!(result.result, -2);
            assert_eq!(result.elements, vec![]);

            transaction.exec(&QueryBuilder::select().id("alias2".into()).query())
        })
        .unwrap_err();

    assert_eq!(error.description, "Alias 'alias2' not found");

    let result = db
        .exec(&QueryBuilder::select().id("alias2".into()).query())
        .unwrap();

    assert_eq!(result.result, 1);
}

#[test]
fn remove_missing_alias() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    let query = QueryBuilder::remove().alias("alias").query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, 0);
    assert_eq!(result.elements, vec![]);
}

#[test]
fn remove_missing_alias_rollback() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();

    let error = db
        .transaction_mut(|transaction| -> Result<(), QueryError> {
            let query = QueryBuilder::remove().alias("alias").query();
            let result = transaction.exec_mut(&query).unwrap();

            assert_eq!(result.result, 0);

            Err("error".into())
        })
        .unwrap_err();

    assert_eq!(error.description, "error");
}
