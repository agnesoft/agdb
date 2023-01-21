use agdb::Comparison;
use agdb::QueryBuilder;

#[test]
fn search_from_where_distance_less_than() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .distance(Comparison::LessThan(2.into()))
        .query();
}
