#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::DbElement;
use agdb::QueryBuilder;
use test_file::TestFile;

#[test]
fn select_id_alias() {
    let test_file = TestFile::new();

    let db = Db::new(test_file.file_name()).unwrap();
    db.exec(&QueryBuilder::insert().node().alias("alias").query())
        .unwrap();
    let query = QueryBuilder::select().id("alias".into()).query();
    let result = db.exec(&query).unwrap();

    assert_eq!(result.result, 1);
    assert_eq!(
        result.elements,
        vec![DbElement {
            index: 1,
            values: vec![]
        }]
    );
}

#[test]
fn select_from_ids() {
    let _query = QueryBuilder::select()
        .ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn select_from_search() {
    let _query = QueryBuilder::select()
        .search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
