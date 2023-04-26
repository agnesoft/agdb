#[path = "../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryError;
use test_file::TestFile;

#[test]
fn insert_node() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    let query = QueryBuilder::insert().node().query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, 1);
    assert_eq!(
        result.elements,
        vec![DbElement {
            index: DbId(1),
            values: vec![]
        }]
    )
}

#[test]
fn insert_node_values() {
    let _query = QueryBuilder::insert()
        .node()
        .values(&[("key", "value").into()])
        .query();
}

#[test]
fn insert_node_values_id() {
    let _query = QueryBuilder::insert().node().values_id("alias").query();
}

#[test]
fn insert_node_alias() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    let query = QueryBuilder::insert().node().alias("alias").query();
    let result = db.exec_mut(&query).unwrap();

    assert_eq!(result.result, 1);
    assert_eq!(
        result.elements,
        vec![DbElement {
            index: DbId(1),
            values: vec![]
        }]
    );
}

#[test]
fn insert_node_alias_empty() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();
    let query = QueryBuilder::insert().node().alias("").query();
    let error = db.exec_mut(&query).unwrap_err();

    assert_eq!(error.description, "Empty alias is not allowed");
}

#[test]
fn insert_node_alias_rollback() {
    let test_file = TestFile::new();

    let mut db = Db::new(test_file.file_name()).unwrap();

    let error = db
        .transaction_mut(|transaction| -> Result<(), QueryError> {
            let result = transaction
                .exec_mut(&QueryBuilder::insert().node().alias("alias").query())
                .unwrap();

            assert_eq!(result.result, 1);
            assert_eq!(
                result.elements,
                vec![DbElement {
                    index: DbId(1),
                    values: vec![]
                }]
            );

            Err(QueryError::from("error"))
        })
        .unwrap_err();

    assert_eq!(error.description, "error");

    let error2 = db
        .exec(&QueryBuilder::select().id("alias".into()).query())
        .unwrap_err();

    assert_eq!(error2.description, "Alias 'alias' not found");
}

#[test]
fn insert_node_alias_values() {
    let _query = QueryBuilder::insert()
        .node()
        .alias("alias")
        .values(&[("key", "value").into()])
        .query();
}

#[test]
fn insert_node_alias_values_id() {
    let _query = QueryBuilder::insert()
        .node()
        .alias("alias1")
        .values_id("alias2")
        .query();
}

#[test]
fn insert_nodes_aliases() {
    let _query = QueryBuilder::insert()
        .nodes()
        .aliases(&["alias1".to_string(), "alias2".to_string()])
        .query();
}

#[test]
fn insert_nodes_aliases_values() {
    let _query = QueryBuilder::insert()
        .nodes()
        .aliases(&["alias1".to_string(), "alias2".to_string()])
        .values(&[
            &[("key", "value").into(), ("key2", "value2").into()],
            &[("key", "value3").into()],
        ])
        .query();
}

#[test]
fn insert_nodes_aliases_values_id() {
    let _query = QueryBuilder::insert()
        .nodes()
        .aliases(&["alias1".to_string(), "alias2".to_string()])
        .values_id("alias3")
        .query();
}

#[test]
fn insert_nodes_aliases_values_ids() {
    let _query = QueryBuilder::insert()
        .nodes()
        .aliases(&["alias1".to_string(), "alias2".to_string()])
        .values_ids(&["alias3".into(), 4.into()])
        .query();
}

#[test]
fn insert_nodes_aliases_values_uniform() {
    let _query = QueryBuilder::insert()
        .nodes()
        .aliases(&["alias1".to_string(), "alias2".to_string()])
        .values_uniform(&[("key", "value").into(), ("key2", "value2").into()])
        .query();
}

#[test]
fn insert_nodes_count() {
    let _query = QueryBuilder::insert().nodes().count(2).query();
}

#[test]
fn insert_nodes_count_values_id() {
    let _query = QueryBuilder::insert()
        .nodes()
        .count(2)
        .values_id("alias3")
        .query();
}

#[test]
fn insert_nodes_count_values_uniform() {
    let _query = QueryBuilder::insert()
        .nodes()
        .count(2)
        .values_uniform(&[("key", "value").into(), ("key2", "value2").into()])
        .query();
}

#[test]
fn insert_nodes_values() {
    let _query = QueryBuilder::insert()
        .nodes()
        .values(&[
            &[("key", "value").into(), ("key2", "value2").into()],
            &[("key", "value3").into()],
        ])
        .query();
}

#[test]
fn insert_nodes_values_id() {
    let _query = QueryBuilder::insert().nodes().values_id("alias3").query();
}

#[test]
fn insert_nodes_values_ids() {
    let _query = QueryBuilder::insert()
        .nodes()
        .values_ids(&["alias3".into(), 4.into()])
        .query();
}

#[test]
fn insert_nodes_values_uniform() {
    let _query = QueryBuilder::insert()
        .nodes()
        .values_uniform(&[("key", "value").into(), ("key2", "value2").into()])
        .query();
}
