#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::QueryBuilder;
use test_file::TestFile;

#[test]
fn remove_alias() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().alias("alias").query())
        .unwrap();
    let query = QueryBuilder::remove().alias("alias").query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, 0);
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

    assert_eq!(result.result, 0);
    assert_eq!(result.elements, vec![]);
}
