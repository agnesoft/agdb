use agdb::QueryBuilder;

#[test]
fn select_from() {
    let _query = QueryBuilder::select().from(1.into()).query();
}
