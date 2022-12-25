use agdb::QueryBuilder;

#[test]
fn insert_node() {
    let _query = QueryBuilder::insert().node().query();
}

#[test]
fn insert_node_alias() {
    let _query = QueryBuilder::insert().node().alias("alias").query();
}

#[test]
fn insert_node_values() {
    let _query = QueryBuilder::insert()
        .node()
        .values(&[("key", "value").into()])
        .query();
}

#[test]
fn insert_node_alias_values() {
    let _query = QueryBuilder::insert()
        .node()
        .alias("alias")
        .values(&[("key", "value").into()])
        .query();
}
