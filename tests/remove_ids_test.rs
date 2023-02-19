#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::QueryBuilder;
use test_file::TestFile;

#[test]
pub fn remove_node() {
    let test_file = TestFile::new();

    let db = Db::new(test_file.file_name()).unwrap();
    let query = QueryBuilder::insert().node().query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::remove().id(1.into()).query();
    let result = db.exec(&query).unwrap();

    assert_eq!(result.result, 1);
    assert_eq!(result.elements, vec![]);
}

#[test]
pub fn remove_nodes() {
    let test_file = TestFile::new();

    let db = Db::new(test_file.file_name()).unwrap();
    let query = QueryBuilder::insert()
        .nodes()
        .aliases(&["alias".to_string(), "alias2".to_string()])
        .query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::remove()
        .ids(&["alias".into(), "alias2".into()])
        .query();
    let result = db.exec(&query).unwrap();

    assert_eq!(result.result, 2);
    assert_eq!(result.elements, vec![]);
}

#[test]
pub fn remove_search() {
    let _query = QueryBuilder::remove()
        .search(QueryBuilder::search().from("origin".into()).query())
        .query();
}
