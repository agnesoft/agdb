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
pub fn remove_query() {
    let _query = QueryBuilder::remove()
        .query(QueryBuilder::select().from("origin".into()).query())
        .query();
}
