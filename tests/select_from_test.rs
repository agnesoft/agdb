use agdb::QueryBuilder;

#[test]
fn select_from() {
    let _query = QueryBuilder::select().from("alias".into()).query();
}

#[test]
fn select_from_ids() {
    let _query = QueryBuilder::select()
        .from_ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn select_from_search() {
    let _query = QueryBuilder::select()
        .from_search(QueryBuilder::search().from("alias".into()).query())
        .query();
}
