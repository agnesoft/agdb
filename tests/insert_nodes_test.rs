use agdb::QueryBuilder;

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
        .values_id("alias3".into())
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
        .values_id("alias3".into())
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
    let _query = QueryBuilder::insert()
        .nodes()
        .values_id("alias3".into())
        .query();
}

#[test]
fn insert_nodes_values_ids() {
    let _query = QueryBuilder::insert()
        .nodes()
        .values_ids(&["alias3".into(), 4.into()])
        .query();
}

#[test]
fn insert_nodes_values_single() {
    let _query = QueryBuilder::insert()
        .nodes()
        .values_single(&[("key", "value").into(), ("key2", "value2").into()])
        .query();
}
