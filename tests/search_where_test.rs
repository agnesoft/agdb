mod test_db;

use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbKeyOrder;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn search_from_where_keys_and_distance() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(5)
            .values_uniform(&[("key", "value").into()])
            .query(),
        5,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[1.into(), 2.into(), 3.into(), 4.into()])
            .to(&[2.into(), 3.into(), 4.into(), 5.into()])
            .query(),
        4,
    );

    db.exec_ids(
        QueryBuilder::search()
            .from(1)
            .where_()
            .keys(&["key".into()])
            .and()
            .distance(CountComparison::LessThan(5))
            .query(),
        &[1, 2, 3],
    );
}

#[test]
fn search_from_where_distance_less_than() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(5)
            .values_uniform(&[("key", "value").into()])
            .query(),
        5,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[1.into(), 2.into(), 3.into(), 4.into()])
            .to(&[2.into(), 3.into(), 4.into(), 5.into()])
            .query(),
        4,
    );

    db.exec_ids(
        QueryBuilder::search()
            .from(1)
            .where_()
            .distance(CountComparison::LessThan(2))
            .query(),
        &[1, -6],
    );
}

#[test]
fn search_from_where_edge_count_test() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .edge_count(CountComparison::GreaterThan(2))
        .query();
}

#[test]
fn search_from_where_edge_from_count_test() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .edge_count_from(CountComparison::GreaterThan(2))
        .query();
}

#[test]
fn search_from_where_edge_to_count_test() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .edge_count_to(CountComparison::GreaterThan(2))
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
        .distance(CountComparison::LessThan(2))
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
        .distance(CountComparison::LessThan(2))
        .query();
}

#[test]
fn search_from_ordered_by_where_key_value() {
    let _query = QueryBuilder::search()
        .from(1)
        .order_by(&[DbKeyOrder::Asc("key".into())])
        .where_()
        .key("key".into())
        .value(Comparison::LessThan(10.into()))
        .query();
}
