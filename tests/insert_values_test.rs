use agdb::QueryBuilder;

#[test]
fn insert_values_id() {
    let _query = QueryBuilder::insert()
        .values(&[("key", "value").into()])
        .id("alias".into())
        .query();
}

#[test]
fn insert_values_ids() {
    let _query = QueryBuilder::insert()
        .values(&[("key", "value").into()])
        .ids(&["alias".into()])
        .query();
}

#[test]
fn insert_values_search() {
    let _query = QueryBuilder::insert()
        .values(&[("key", "value").into()])
        .search(QueryBuilder::search().from(1.into()).query())
        .query();
}

#[test]
fn insert_values_multi_ids() {
    let _query = QueryBuilder::insert()
        .values_multi(&[&[("key", "value").into()]])
        .ids(&["alias".into()])
        .query();
}

#[test]
fn insert_values_multi_search() {
    let _query = QueryBuilder::insert()
        .values_multi(&[&[("key", "value").into()]])
        .search(QueryBuilder::search().from(1.into()).query())
        .query();
}
