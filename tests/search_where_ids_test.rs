use agdb::QueryBuilder;

#[test]
fn search_from_where_id() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .id("alias".into())
        .query();
}

#[test]
fn search_from_where_ids() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .ids(&["alias".into(), "alias2".into()])
        .query();
}
