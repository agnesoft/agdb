use agdb::QueryBuilder;

#[test]
fn select_ids_from() {
    let _query = QueryBuilder::select().ids().from(1.into()).query();
}

#[test]
fn select_ids_from_offset() {
    let _query = QueryBuilder::select()
        .ids()
        .from(1.into())
        .offset(10)
        .query();
}

#[test]
fn select_ids_from_limit() {
    let _query = QueryBuilder::select()
        .ids()
        .from(1.into())
        .limit(10)
        .query();
}

#[test]
fn select_ids_from_offset_limit() {
    let _query = QueryBuilder::select()
        .ids()
        .from(1.into())
        .offset(10)
        .limit(10)
        .query();
}

#[test]
fn select_ids_from_to() {
    let _query = QueryBuilder::select()
        .ids()
        .from(1.into())
        .to(2.into())
        .query();
}

#[test]
fn select_ids_from_to_offset() {
    let _query = QueryBuilder::select()
        .ids()
        .from(1.into())
        .to(2.into())
        .offset(10)
        .query();
}

#[test]
fn select_ids_from_to_limit() {
    let _query = QueryBuilder::select()
        .ids()
        .from(1.into())
        .to(2.into())
        .limit(10)
        .query();
}

#[test]
fn select_ids_from_to_offset_limit() {
    let _query = QueryBuilder::select()
        .ids()
        .from(1.into())
        .to(2.into())
        .offset(10)
        .limit(10)
        .query();
}

#[test]
fn select_ids_to() {
    let _query = QueryBuilder::select().ids().to(1.into()).query();
}

#[test]
fn select_ids_to_offset() {
    let _query = QueryBuilder::select().ids().to(1.into()).offset(10).query();
}

#[test]
fn select_ids_to_limit() {
    let _query = QueryBuilder::select().ids().to(1.into()).limit(10).query();
}

#[test]
fn select_ids_to_offset_limit() {
    let _query = QueryBuilder::select()
        .ids()
        .to(1.into())
        .offset(10)
        .limit(10)
        .query();
}
