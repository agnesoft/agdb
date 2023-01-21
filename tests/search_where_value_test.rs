use agdb::Comparison;
use agdb::QueryBuilder;

#[test]
fn search_from_where_key_value() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .key("key".into())
        .value(Comparison::LessThan(10.into()))
        .query();
}
