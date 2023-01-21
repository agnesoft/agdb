use agdb::QueryBuilder;

#[test]
fn search_from_where_edge() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .edge()
        .query();
}
