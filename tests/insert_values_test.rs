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
fn insert_values_into_search() {
    let _query = QueryBuilder::insert()
        .values(&[("key", "value").into()])
        .into_search(QueryBuilder::search().from(1.into()).query())
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
fn insert_values_multi_into_search() {
    let _query = QueryBuilder::insert()
        .values_multi(&[&[("key", "value").into()]])
        .into_search(QueryBuilder::search().from(1.into()).query())
        .query();
}
