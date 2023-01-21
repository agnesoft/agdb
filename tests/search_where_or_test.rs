use agdb::Comparison;
use agdb::QueryBuilder;

#[test]
fn search_from_where_keys_or_distance() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .keys(&["key".into()])
        .or()
        .distance(Comparison::LessThan(2.into()))
        .query();
}

#[test]
fn search_from_where_key_or_distance() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .key("key".into())
        .or()
        .distance(Comparison::LessThan(2.into()))
        .query();
}
