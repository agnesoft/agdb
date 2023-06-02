use agdb::QueryBuilder;

#[test]
fn select_alias_id() {
    let _query = QueryBuilder::select().aliases().ids(&[1.into()]).query();
}

#[test]
fn select_aliases() {
    let _query = QueryBuilder::select().aliases().query();
}

#[test]
fn select_aliases_ids() {
    let _query = QueryBuilder::select().aliases().ids(&[1, 2]).query();
}

#[test]
fn select_aliases_search() {
    let _query = QueryBuilder::select()
        .aliases()
        .search(QueryBuilder::search().from(1.into()).query())
        .query();
}
