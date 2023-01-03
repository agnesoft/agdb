use agdb::QueryBuilder;

#[test]
fn select_alias_of() {
    let _query = QueryBuilder::select().alias().of(1).query();
}

#[test]
fn select_aliases() {
    let _query = QueryBuilder::select().aliases().query();
}

#[test]
fn select_aliases_of() {
    let _query = QueryBuilder::select().aliases().of(&[1, 2]).query();
}

#[test]
fn select_aliases_of_query() {
    let _query = QueryBuilder::select()
        .aliases()
        .of_query(QueryBuilder::select().from(1.into()).query())
        .query();
}
