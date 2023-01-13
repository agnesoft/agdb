use agdb::QueryBuilder;

#[test]
fn select_key_count_from() {
    let _query = QueryBuilder::select()
        .key_count()
        .from("alias".into())
        .query();
}

#[test]
fn select_key_count_from_ids() {
    let _query = QueryBuilder::select()
        .key_count()
        .from_ids(&["alias".into()])
        .query();
}

#[test]
fn select_key_count_from_search() {
    let _query = QueryBuilder::select()
        .key_count()
        .from_search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
