use agdb::QueryBuilder;

#[test]
fn select_values_from() {
    let _query = QueryBuilder::select()
        .values(&["key1".into(), "key2".into()])
        .from("alias".into())
        .query();
}

#[test]
fn select_values_from_ids() {
    let _query = QueryBuilder::select()
        .values(&["key1".into(), "key2".into()])
        .from_ids(&["alias".into()])
        .query();
}

#[test]
fn select_values_from_search() {
    let _query = QueryBuilder::select()
        .values(&["key1".into(), "key2".into()])
        .from_search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
