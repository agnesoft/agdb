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
fn insert_nodes_count() {
    let _query = QueryBuilder::insert().nodes().count(2).query();
}

#[test]
fn insert_nodes_count_aliases() {
    let _query = QueryBuilder::insert()
        .nodes()
        .count(2)
        .aliases(&["alias1".to_string(), "alias2".to_string()])
        .query();
}

#[test]
fn insert_nodes_count_aliases_values() {
    let _query = QueryBuilder::insert()
        .nodes()
        .count(2)
        .aliases(&["alias1".to_string(), "alias2".to_string()])
        .values(&[
            &[("key", "value").into(), ("key2", "value2").into()],
            &[("key", "value3").into()],
        ])
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
