use agdb::QueryBuilder;

#[test]
fn remove_value_id() {
    let _query = QueryBuilder::remove()
        .value("key1".into())
        .id("alias".into())
        .query();
}

#[test]
fn remove_value_ids() {
    let _query = QueryBuilder::remove()
        .value("key1".into())
        .ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn remove_value_search() {
    let _query = QueryBuilder::remove()
        .value("key1".into())
        .search(QueryBuilder::search().from("alias1".into()).query())
        .query();
}

#[test]
fn remove_values_id() {
    let _query = QueryBuilder::remove()
        .values(&["key1".into(), "key2".into()])
        .id("alias".into())
        .query();
}

#[test]
fn remove_values_ids() {
    let _query = QueryBuilder::remove()
        .values(&["key1".into(), "key2".into()])
        .ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn remove_values_search() {
    let _query = QueryBuilder::remove()
        .values(&["key1".into(), "key2".into()])
        .search(QueryBuilder::search().from("alias1".into()).query())
        .query();
}
