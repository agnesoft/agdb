use agdb::QueryBuilder;

#[test]
fn insert_values_into() {
    let _query = QueryBuilder::insert()
        .values(&[("key", "value").into()])
        .into("alias".into())
        .query();
}

#[test]
fn insert_values_into_ids() {
    let _query = QueryBuilder::insert()
        .values(&[("key", "value").into()])
        .into_ids(&["alias".into()])
        .query();
}

#[test]
fn insert_values_into_query() {
    let _query = QueryBuilder::insert()
        .values(&[("key", "value").into()])
        .into_query(QueryBuilder::select().from(1.into()).query())
        .query();
}

#[test]
fn insert_values_multi_into_ids() {
    let _query = QueryBuilder::insert()
        .values_multi(&[&[("key", "value").into()]])
        .into_ids(&["alias".into()])
        .query();
}

#[test]
fn insert_values_multi_into_query() {
    let _query = QueryBuilder::insert()
        .values_multi(&[&[("key", "value").into()]])
        .into_query(QueryBuilder::select().from(1.into()).query())
        .query();
}
