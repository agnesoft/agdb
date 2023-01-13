use agdb::QueryBuilder;

#[test]
fn search_from() {
    let _query = QueryBuilder::search().from(1.into()).query();
}

#[test]
fn search_from_limit() {
    let _query = QueryBuilder::search().from(1.into()).limit(10).query();
}

#[test]
fn search_from_offset() {
    let _query = QueryBuilder::search().from(1.into()).offset(10).query();
}

#[test]
fn search_from_offset_limit() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .offset(10)
        .limit(10)
        .query();
}

#[test]
fn search_from_to() {
    let _query = QueryBuilder::search().from(1.into()).to(2.into()).query();
}

#[test]
fn search_from_to_limit() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .to(2.into())
        .limit(10)
        .query();
}

#[test]
fn search_from_to_offset() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .to(2.into())
        .offset(10)
        .query();
}

#[test]
fn search_from_to_offset_limit() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .to(2.into())
        .offset(10)
        .limit(10)
        .query();
}

#[test]
fn search_to() {
    let _query = QueryBuilder::search().to(1.into()).query();
}

#[test]
fn search_to_limit() {
    let _query = QueryBuilder::search().to(1.into()).limit(10).query();
}

#[test]
fn search_to_offset() {
    let _query = QueryBuilder::search().to(1.into()).offset(10).query();
}

#[test]
fn search_to_offset_limit() {
    let _query = QueryBuilder::search()
        .to(1.into())
        .offset(10)
        .limit(10)
        .query();
}
