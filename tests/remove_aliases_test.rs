use agdb::QueryBuilder;

#[test]
fn remove_alias() {
    let _query = QueryBuilder::remove().alias("alias").query();
}

#[test]
fn remove_aliases() {
    let _query = QueryBuilder::remove()
        .aliases(&["alias".into(), "alias2".into()])
        .query();
}
