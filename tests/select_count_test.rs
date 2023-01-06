use agdb::QueryBuilder;

#[test]
fn select_count_from() {
    let _query = QueryBuilder::select().count().from(1.into()).query();
}

#[test]
fn select_count_from_offset() {
    let _query = QueryBuilder::select()
        .count()
        .from(1.into())
        .offset(10)
        .query();
}

#[test]
fn select_count_from_to() {
    let _query = QueryBuilder::select()
        .count()
        .from(1.into())
        .to(2.into())
        .query();
}

#[test]
fn select_count_from_to_offset() {
    let _query = QueryBuilder::select()
        .count()
        .from(1.into())
        .to(2.into())
        .offset(10)
        .query();
}

#[test]
fn select_count_to() {
    let _query = QueryBuilder::select().count().to(1.into()).query();
}

#[test]
fn select_count_to_offset() {
    let _query = QueryBuilder::select()
        .count()
        .to(1.into())
        .offset(10)
        .query();
}
