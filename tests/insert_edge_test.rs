use agdb::QueryBuilder;

#[test]
fn insert_edge_from_to() {
    let _query = QueryBuilder::insert()
        .edge()
        .from("alias1".into())
        .to("alias2".into())
        .query();
}

#[test]
fn insert_edge_from_to_values() {
    let _query = QueryBuilder::insert()
        .edge()
        .from("alias1".into())
        .to("alias2".into())
        .values(&[("key", "value").into()])
        .query();
}

#[test]
fn insert_edge_from_to_values_id() {
    let _query = QueryBuilder::insert()
        .edge()
        .from("alias1".into())
        .to("alias2".into())
        .values_id("alias".into())
        .query();
}
