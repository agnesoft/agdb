mod framework;

use agdb::QueryBuilder;
use agdb::QueryError;
use framework::TestDb;

#[test]
fn insert_node() {
    let mut db = TestDb::new();
    db.exec_mut_ids(QueryBuilder::insert().node().query(), &[1]);
}

#[test]
fn insert_node_alias() {
    let mut db = TestDb::new();
    db.exec_mut_ids(QueryBuilder::insert().node().alias("alias").query(), &[1]);
}

#[test]
fn insert_node_alias_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |transaction| -> Result<(), QueryError> {
            transaction.exec_mut(&QueryBuilder::insert().node().alias("alias").query())?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_error(
        QueryBuilder::select().id("alias").query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn insert_node_existing_alias() {
    let mut db = TestDb::new();
    db.exec_mut_ids(QueryBuilder::insert().node().alias("alias").query(), &[1]);
    db.exec_mut_error(
        QueryBuilder::insert().node().alias("alias").query(),
        "Alias 'alias' already exists",
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
