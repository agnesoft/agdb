use agdb::QueryBuilder;

#[test]
fn select_id_from() {
    let _query = QueryBuilder::select().id().from(1.into()).query();
}

#[test]
fn select_id_from_offset() {
    let _query = QueryBuilder::select()
        .id()
        .from(1.into())
        .offset(10)
        .query();
}

#[test]
fn select_id_from_to() {
    let _query = QueryBuilder::select()
        .id()
        .from(1.into())
        .to(2.into())
        .query();
}

#[test]
fn select_id_from_to_offset() {
    let _query = QueryBuilder::select()
        .id()
        .from(1.into())
        .to(2.into())
        .offset(10)
        .query();
}

#[test]
fn select_id_to() {
    let _query = QueryBuilder::select().id().to(1.into()).query();
}

#[test]
fn select_id_to_offset() {
    let _query = QueryBuilder::select().id().to(1.into()).offset(10).query();
}
