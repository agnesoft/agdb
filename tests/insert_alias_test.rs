use agdb::QueryBuilder;

#[test]
fn insert_alias_of() {
    let _query = QueryBuilder::insert().alias("alias").of(1.into()).query();
}

#[test]
fn insert_aliases_of() {
    let _query = QueryBuilder::insert()
        .aliases(&["alias".into(), "alias2".into()])
        .of(&[1.into(), 2.into()])
        .query();
}
