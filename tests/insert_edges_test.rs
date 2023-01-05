use agdb::QueryBuilder;

#[test]
fn insert_edges_from_to() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into()])
        .query();
}

#[test]
fn insert_edges_from_to_each() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into(), "alias4".into()])
        .each()
        .query();
}

#[test]
fn insert_edges_from_to_each_values() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into(), "alias4".into()])
        .each()
        .values(&[&[("key", "value").into()], &[("key", "value2").into()]])
        .query();
}

#[test]
fn insert_edges_from_to_each_values_id() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into(), "alias4".into()])
        .each()
        .values_id("alias".into())
        .query();
}

#[test]
fn insert_edges_from_to_each_values_ids() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into()])
        .to(&["alias2".into()])
        .each()
        .values_ids(&["alias".into(), "alias3".into()])
        .query();
}

#[test]
fn insert_edges_from_to_each_values_uniform() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into()])
        .each()
        .values_uniform(&[("key", "value").into(), ("key", "value2").into()])
        .query();
}

#[test]
fn insert_edges_from_to_values() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into()])
        .values(&[&[("key", "value").into()], &[("key", "value2").into()]])
        .query();
}

#[test]
fn insert_edges_from_to_values_id() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias2".into()])
        .values_id("alias".into())
        .query();
}

#[test]
fn insert_edges_from_to_values_ids() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias2".into()])
        .values_ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn insert_edges_from_to_values_uniform() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias1".into(), "alias2".into()])
        .to(&["alias3".into()])
        .values_uniform(&[("key", "value").into(), ("key", "value2").into()])
        .query();
}

#[test]
fn insert_edges_from_query_to() {
    let _query = QueryBuilder::insert()
        .edges()
        .from_query(QueryBuilder::select().ids().from(1.into()).query())
        .to(&["alias".into()])
        .query();
}

#[test]
fn insert_edges_from_to_query() {
    let _query = QueryBuilder::insert()
        .edges()
        .from(&["alias".into()])
        .to_query(QueryBuilder::select().ids().from(2.into()).query())
        .query();
}
