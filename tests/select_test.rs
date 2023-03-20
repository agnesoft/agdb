#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::DbElement;
use agdb::QueryBuilder;
use test_file::TestFile;

#[test]
fn select_id_alias() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(&QueryBuilder::insert().node().alias("alias").query())
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
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(
        &QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".into(), "alias2".into()])
            .query(),
    )
    .unwrap();
    let query = QueryBuilder::select()
        .ids(&["alias".into(), "alias2".into()])
        .query();
    let result = db.exec(&query).unwrap();

    assert_eq!(result.result, 2);
    assert_eq!(
        result.elements,
        vec![
            DbElement {
                index: 1,
                values: vec![]
            },
            DbElement {
                index: 2,
                values: vec![]
            }
        ]
    );
}

#[test]
fn select_missing_alias() {
    let test_file = TestFile::new();

    let db = Db::new(test_file.file_name()).unwrap();
    let query = QueryBuilder::select().id("alias".into()).query();
    let query_error = db.exec(&query).unwrap_err();

    assert_eq!(query_error.description, "Alias 'alias' not found");
}

#[test]
fn select_missing_id() {
    let test_file = TestFile::new();

    let db = Db::new(test_file.file_name()).unwrap();
    let query = QueryBuilder::select().id(1.into()).query();
    let query_error = db.exec(&query).unwrap_err();

    assert_eq!(query_error.description, "Id '1' not found");
}

#[test]
fn select_from_search() {
    let _query = QueryBuilder::select()
        .search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
