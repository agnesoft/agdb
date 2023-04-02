#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::QueryBuilder;
use test_file::TestFile;

#[test]
pub fn remove_node() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().query()).unwrap();

    let query = QueryBuilder::remove().id(1.into()).query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, -1);
    assert_eq!(result.elements, vec![]);
}

#[test]
pub fn remove_nodes() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(
        &QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".to_string(), "alias2".to_string()])
            .query(),
    )
    .unwrap();

    let query = QueryBuilder::remove()
        .ids(&["alias".into(), "alias2".into()])
        .query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, -2);
    assert_eq!(result.elements, vec![]);
}

#[test]
pub fn remove_edge() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().alias("alias1").query())
        .unwrap();
    db.exec_mut(&QueryBuilder::insert().node().query()).unwrap();
    db.exec_mut(
        &QueryBuilder::insert()
            .edge()
            .from("alias1".into())
            .to(2.into())
            .query(),
    )
    .unwrap();

    let query = QueryBuilder::remove().id((-3).into()).query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, -1);
    assert_eq!(result.elements, vec![]);
}

#[test]
pub fn remove_edges() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().alias("alias1").query())
        .unwrap();
    db.exec_mut(&QueryBuilder::insert().node().query()).unwrap();
    db.exec_mut(
        &QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), 2.into()])
            .to(&[2.into(), "alias1".into()])
            .query(),
    )
    .unwrap();

    let query = QueryBuilder::remove()
        .ids(&[(-3).into(), (-4).into()])
        .query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, -2);
    assert_eq!(result.elements, vec![]);
}

#[test]
pub fn remove_search() {
    let _query = QueryBuilder::remove()
        .search(QueryBuilder::search().from("origin".into()).query())
        .query();
}
