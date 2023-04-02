#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::QueryBuilder;
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
