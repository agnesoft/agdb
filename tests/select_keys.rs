use agdb::QueryBuilder;

#[test]
fn select_keys_id() {
    let _query = QueryBuilder::select().keys().id("alias".into()).query();
}

#[test]
fn select_keys_ids() {
    let _query = QueryBuilder::select().keys().ids(&["alias".into()]).query();
}

#[test]
fn select_keys_search() {
    let _query = QueryBuilder::select()
        .keys()
        .search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
