use agdb::QueryBuilder;

#[test]
fn search_from_where_node() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .node()
        .query();
}
