#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::DbElement;
use agdb::QueryBuilder;
use test_file::TestFile;

#[test]
fn insert_edge_from_to() {
    let test_file = TestFile::new();

    let db = Db::new(test_file.file_name()).unwrap();
    db.exec(&QueryBuilder::insert().node().alias("alias1").query())
        .unwrap();
    db.exec(&QueryBuilder::insert().node().query()).unwrap();

    let query = QueryBuilder::insert()
        .edge()
        .from("alias1".into())
        .to(2.into())
        .query();
    let result = db.exec(&query).unwrap();

    assert_eq!(result.result, 1);
    assert_eq!(
        result.elements,
        vec![DbElement {
            index: -1,
            values: vec![]
        }]
    )
}

#[test]
fn insert_edge_from_to_values() {
    let _query = QueryBuilder::insert()
        .edge()
        .from("alias1".into())
        .to("alias2".into())
        .values(&[("key", "value").into()])
        .query();
}

#[test]
fn insert_edge_from_to_values_id() {
    let _query = QueryBuilder::insert()
        .edge()
        .from("alias1".into())
        .to("alias2".into())
        .values_id("alias".into())
        .query();
}

#[test]
fn insert_edges_from_to() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into()])
        .query();
}

#[test]
fn insert_edges_from_to_each() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into(), "alias4".into()])
        .each()
        .query();
}

#[test]
fn insert_edges_from_to_each_values() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into(), "alias4".into()])
        .each()
        .values(&[&[("key", "value").into()], &[("key", "value2").into()]])
        .query();
}

#[test]
fn insert_edges_from_to_each_values_id() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into(), "alias4".into()])
        .each()
        .values_id("alias".into())
        .query();
}

#[test]
fn insert_edges_from_to_each_values_ids() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into()])
        .to(&["alias2".into()])
        .each()
        .values_ids(&["alias".into(), "alias3".into()])
        .query();
}

#[test]
fn insert_edges_from_to_each_values_uniform() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into()])
        .each()
        .values_uniform(&[("key", "value").into(), ("key", "value2").into()])
        .query();
}

#[test]
fn insert_edges_from_to_values() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into()])
        .values(&[&[("key", "value").into()], &[("key", "value2").into()]])
        .query();
}

#[test]
fn insert_edges_from_to_values_id() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias2".into()])
        .values_id("alias".into())
        .query();
}

#[test]
fn insert_edges_from_to_values_ids() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias2".into()])
        .values_ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn insert_edges_from_to_values_uniform() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into()])
        .values_uniform(&[("key", "value").into(), ("key", "value2").into()])
        .query();
}

#[test]
fn insert_edges_from_query_to() {
    let _query = QueryBuilder::insert()
        .edges()
        .from_search(QueryBuilder::search().from(1.into()).query())
        .to(&["alias".into()])
        .query();
}

#[test]
fn insert_edges_from_to_search() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias".into()])
        .to_search(QueryBuilder::search().from(2.into()).query())
        .query();
}
