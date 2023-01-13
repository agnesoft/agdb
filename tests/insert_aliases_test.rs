use agdb::QueryBuilder;

#[test]
fn insert_alias_id() {
    let _query = QueryBuilder::insert().alias("alias").id(1.into()).query();
}

#[test]
fn insert_aliases_ids() {
    let _query = QueryBuilder::insert()
        .aliases(&["alias".into(), "alias2".into()])
        .of(&[1.into(), 2.into()])
        .query();
}
