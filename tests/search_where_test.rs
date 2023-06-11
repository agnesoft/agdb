use agdb::Comparison;
use agdb::QueryBuilder;

#[test]
fn search_from_where_keys_and_distance() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .keys(&["key".into()])
        .and()
        .distance(Comparison::LessThan(2.into()))
        .query();
}

#[test]
fn search_from_where_distance_less_than() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .distance(Comparison::LessThan(2.into()))
        .query();
}

#[test]
fn search_from_where_edge_count_test() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .edge_count(Comparison::GreaterThan(2.into()))
        .query();
}

#[test]
fn search_from_where_edge_from_count_test() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .edge_count_from(Comparison::GreaterThan(2.into()))
        .query();
}

#[test]
fn search_from_where_edge_to_count_test() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .edge_count_to(Comparison::GreaterThan(2.into()))
        .query();
}

#[test]
fn search_from_where_edge() {
    let _query = QueryBuilder::search().from(1).where_().edge().query();
}

#[test]
fn search_from_where_ids() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn search_from_where_keys() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .keys(&["key".into(), "key2".into()])
        .query();
}

#[test]
fn search_from_where_node() {
    let _query = QueryBuilder::search().from(1).where_().node().query();
}

#[test]
fn search_from_where_not_beyond_keys() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .not_beyond()
        .keys(&["key".into()])
        .query();
}

#[test]
fn search_from_where_not_key() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .not()
        .keys(&["key".into()])
        .query();
}

#[test]
fn search_from_where_keys_or_distance() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .keys(&["key".into()])
        .or()
        .distance(Comparison::LessThan(2.into()))
        .query();
}

#[test]
fn search_from_where_key_value() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .key("key".into())
        .value(Comparison::LessThan(10.into()))
        .query();
}

#[test]
fn search_from_where_where_key_and_key_end_where_and_distance() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .where_()
        .keys(&["key".into()])
        .or()
        .keys(&["key2".into()])
        .end_where()
        .and()
        .distance(Comparison::LessThan(2.into()))
        .query();
}
