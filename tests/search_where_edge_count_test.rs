use agdb::Comparison;
use agdb::QueryBuilder;

#[test]
fn search_from_where_edge_count_test() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .edge_count(Comparison::GreaterThan(2.into()))
        .query();
}

#[test]
fn search_from_where_edge_from_count_test() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .edge_count_from(Comparison::GreaterThan(2.into()))
        .query();
}

#[test]
fn search_from_where_edge_to_count_test() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .edge_count_to(Comparison::GreaterThan(2.into()))
        .query();
}
