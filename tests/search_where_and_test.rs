use agdb::Comparison;
use agdb::QueryBuilder;

#[test]
fn search_from_where_keys_and_distance() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .keys(&["key".into()])
        .and()
        .distance(Comparison::LessThan(2.into()))
        .query();
}

#[test]
fn search_from_where_key_and_distance() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .key("key".into())
        .and()
        .distance(Comparison::LessThan(2.into()))
        .query();
}
