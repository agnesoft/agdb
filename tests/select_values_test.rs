use agdb::QueryBuilder;

#[test]
fn select_values_ids() {
    let _query = QueryBuilder::select()
        .values(&["key1".into(), "key2".into()])
        .ids(&["alias".into()])
        .query();
}

#[test]
fn select_values_search() {
    let _query = QueryBuilder::select()
        .values(&["key1".into(), "key2".into()])
        .search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
