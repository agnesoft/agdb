use agdb::QueryBuilder;

#[test]
fn select_keys_from() {
    let _query = QueryBuilder::select().keys().from("alias".into()).query();
}

#[test]
fn select_keys_from_ids() {
    let _query = QueryBuilder::select()
        .keys()
        .from_ids(&["alias".into()])
        .query();
}

#[test]
fn select_keys_from_search() {
    let _query = QueryBuilder::select()
        .keys()
        .from_search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
