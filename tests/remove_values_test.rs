use agdb::QueryBuilder;

#[test]
fn remove_value_from() {
    let _query = QueryBuilder::remove()
        .value("key1".into())
        .from("alias".into())
        .query();
}

#[test]
fn remove_value_from_ids() {
    let _query = QueryBuilder::remove()
        .value("key1".into())
        .from_ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn remove_value_from_query() {
    let _query = QueryBuilder::remove()
        .value("key1".into())
        .from_query(QueryBuilder::select().from("alias1".into()).query())
        .query();
}

#[test]
fn remove_values_from() {
    let _query = QueryBuilder::remove()
        .values(&["key1".into(), "key2".into()])
        .from("alias".into())
        .query();
}

#[test]
fn remove_values_from_ids() {
    let _query = QueryBuilder::remove()
        .values(&["key1".into(), "key2".into()])
        .from_ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn remove_values_from_query() {
    let _query = QueryBuilder::remove()
        .values(&["key1".into(), "key2".into()])
        .from_query(QueryBuilder::select().from("alias1".into()).query())
        .query();
}
