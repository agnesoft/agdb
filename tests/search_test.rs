mod test_db;

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
    db.exec_ids(
        QueryBuilder::search().from(1.into()).query(),
        &[1, -11, 3, -12, 5, -13, 7, -14, 9],
    );
}

#[test]
fn search_from_multiple_edges() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[
                1.into(),
                2.into(),
                3.into(),
                4.into(),
                1.into(),
                2.into(),
                3.into(),
                4.into(),
            ])
            .to(&[
                2.into(),
                3.into(),
                4.into(),
                5.into(),
                2.into(),
                3.into(),
                4.into(),
                5.into(),
            ])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search().from(1.into()).query(),
        &[1, -10, -6, 2, -11, -7, 3, -12, -8, 4, -13, -9, 5],
    );
}

#[test]
fn search_from_circular() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query(), 3);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[1.into(), 2.into(), 3.into()])
            .to(&[2.into(), 3.into(), 1.into()])
            .query(),
        3,
    );
    db.exec_ids(
        QueryBuilder::search().from(1.into()).query(),
        &[1, -4, 2, -5, 3, -6],
    );
}

#[test]
fn search_from_self_referential() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[1.into(), 1.into()])
            .to(&[1.into(), 1.into()])
            .query(),
        2,
    );
    db.exec_ids(QueryBuilder::search().from(1.into()).query(), &[1, -3, -2]);
}

#[test]
fn search_from_limit() {
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
    db.exec_ids(
        QueryBuilder::search().from(1.into()).limit(5).query(),
        &[1, -11, 3, -12, 5],
    );
}

#[test]
fn search_from_offset() {
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
    db.exec_ids(
        QueryBuilder::search().from(1.into()).offset(4).query(),
        &[5, -13, 7, -14, 9],
    );
}

#[test]
fn search_from_offset_limit() {
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
    db.exec_ids(
        QueryBuilder::search()
            .from(1.into())
            .offset(4)
            .limit(2)
            .query(),
        &[5, -13],
    );
}

#[test]
fn search_from_to() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[
                1.into(),
                2.into(),
                3.into(),
                4.into(),
                1.into(),
                2.into(),
                3.into(),
                4.into(),
            ])
            .to(&[
                2.into(),
                3.into(),
                4.into(),
                5.into(),
                2.into(),
                3.into(),
                4.into(),
                5.into(),
            ])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search().from(1.into()).to(4.into()).query(),
        &[1, -6, 2, -7, 3, -8, 4],
    );
}

#[test]
fn search_from_to_shortcut() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[1.into(), 2.into(), 3.into(), 4.into(), 1.into()])
            .to(&[2.into(), 3.into(), 4.into(), 5.into(), 5.into()])
            .query(),
        5,
    );
    db.exec_ids(
        QueryBuilder::search().from(1.into()).to(5.into()).query(),
        &[1, -10, 5],
    );
}

#[test]
fn search_from_to_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[
                1.into(),
                2.into(),
                3.into(),
                4.into(),
                1.into(),
                2.into(),
                3.into(),
                4.into(),
            ])
            .to(&[
                2.into(),
                3.into(),
                4.into(),
                5.into(),
                2.into(),
                3.into(),
                4.into(),
                5.into(),
            ])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search()
            .from(1.into())
            .to(4.into())
            .limit(4)
            .query(),
        &[1, -6, 2, -7],
    );
}

#[test]
fn search_from_to_offset() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[
                1.into(),
                2.into(),
                3.into(),
                4.into(),
                1.into(),
                2.into(),
                3.into(),
                4.into(),
            ])
            .to(&[
                2.into(),
                3.into(),
                4.into(),
                5.into(),
                2.into(),
                3.into(),
                4.into(),
                5.into(),
            ])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search()
            .from(1.into())
            .to(4.into())
            .offset(3)
            .query(),
        &[-7, 3, -8, 4],
    );
}

#[test]
fn search_from_to_offset_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[
                1.into(),
                2.into(),
                3.into(),
                4.into(),
                1.into(),
                2.into(),
                3.into(),
                4.into(),
            ])
            .to(&[
                2.into(),
                3.into(),
                4.into(),
                5.into(),
                2.into(),
                3.into(),
                4.into(),
                5.into(),
            ])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search()
            .from(1.into())
            .to(4.into())
            .offset(3)
            .limit(2)
            .query(),
        &[-7, 3],
    );
}

#[test]
fn search_to() {
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
    db.exec_ids(
        QueryBuilder::search().to(9.into()).query(),
        &[9, -14, 7, -13, 5, -12, 3, -11, 1],
    );
}

#[test]
fn search_to_limit() {
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
    db.exec_ids(
        QueryBuilder::search().to(9.into()).limit(3).query(),
        &[9, -14, 7],
    );
}

#[test]
fn search_to_offset() {
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
    db.exec_ids(
        QueryBuilder::search().to(9.into()).offset(2).query(),
        &[7, -13, 5, -12, 3, -11, 1],
    );
}

#[test]
fn search_to_offset_limit() {
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
    db.exec_ids(
        QueryBuilder::search()
            .to(9.into())
            .offset(2)
            .limit(4)
            .query(),
        &[7, -13, 5, -12],
    );
}
