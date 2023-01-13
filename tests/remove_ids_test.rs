use agdb::QueryBuilder;

#[test]
pub fn remove_id() {
    let _query = QueryBuilder::remove().id("alias".into()).query();
}

#[test]
pub fn remove_ids() {
    let _query = QueryBuilder::remove()
        .ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
pub fn remove_search() {
    let _query = QueryBuilder::remove()
        .search(QueryBuilder::search().from("origin".into()).query())
        .query();
}
