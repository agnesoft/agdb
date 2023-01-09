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
fn select_key_count_from_query() {
    let _query = QueryBuilder::select()
        .key_count()
        .from_query(QueryBuilder::select().id().from("alias".into()).query())
        .query();
}
