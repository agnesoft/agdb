use agdb::QueryBuilder;

#[test]
fn select_key_count_id() {
    let _query = QueryBuilder::select()
        .key_count()
        .id("alias".into())
        .query();
}

#[test]
fn select_key_count_ids() {
    let _query = QueryBuilder::select()
        .key_count()
        .ids(&["alias".into()])
        .query();
}

#[test]
fn select_key_count_search() {
    let _query = QueryBuilder::select()
        .key_count()
        .search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
