mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn search_from() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(10).query(), 10);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[1.into(), 3.into(), 5.into(), 7.into()])
            .to(&[3.into(), 5.into(), 7.into(), 9.into()])
            .query(),
        4,
    );
    db.exec_elements(
        QueryBuilder::search().from(1.into()).query(),
        &[
            DbElement {
                id: DbId(1),
                values: vec![],
            },
            DbElement {
                id: DbId(-11),
                values: vec![],
            },
            DbElement {
                id: DbId(3),
                values: vec![],
            },
            DbElement {
                id: DbId(-12),
                values: vec![],
            },
            DbElement {
                id: DbId(5),
                values: vec![],
            },
            DbElement {
                id: DbId(-13),
                values: vec![],
            },
            DbElement {
                id: DbId(7),
                values: vec![],
            },
            DbElement {
                id: DbId(-14),
                values: vec![],
            },
            DbElement {
                id: DbId(9),
                values: vec![],
            },
        ],
    );
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
