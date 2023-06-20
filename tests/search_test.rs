mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::QueryBuilder;
use test_db::TestDb;

#[test]
fn search_from() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(10).query(), 10);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec![1, 3, 5, 7])
            .to(vec![3, 5, 7, 9])
            .query(),
        4,
    );
    db.exec_ids(
        QueryBuilder::search().from(1).query(),
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
            .from(vec![1, 2, 3, 4, 1, 2, 3, 4])
            .to(vec![2, 3, 4, 5, 2, 3, 4, 5])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search().from(1).query(),
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
            .from(vec![1, 2, 3])
            .to(vec![2, 3, 1])
            .query(),
        3,
    );
    db.exec_ids(
        QueryBuilder::search().from(1).query(),
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
            .from(vec![1, 1])
            .to(vec![1, 1])
            .query(),
        2,
    );
    db.exec_ids(QueryBuilder::search().from(1).query(), &[1, -3, -2]);
}

#[test]
fn search_from_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(10).query(), 10);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec![1, 3, 5, 7])
            .to(vec![3, 5, 7, 9])
            .query(),
        4,
    );
    db.exec_ids(
        QueryBuilder::search().from(1).limit(5).query(),
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
            .from(vec![1, 3, 5, 7])
            .to(vec![3, 5, 7, 9])
            .query(),
        4,
    );
    db.exec_ids(
        QueryBuilder::search().from(1).offset(4).query(),
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
            .from(vec![1, 3, 5, 7])
            .to(vec![3, 5, 7, 9])
            .query(),
        4,
    );
    db.exec_ids(
        QueryBuilder::search().from(1).offset(4).limit(2).query(),
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
            .from(vec![1, 2, 3, 4, 1, 2, 3, 4])
            .to(vec![2, 3, 4, 5, 2, 3, 4, 5])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search().from(1).to(4).query(),
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
            .from(vec![1, 2, 3, 4, 1])
            .to(vec![2, 3, 4, 5, 5])
            .query(),
        5,
    );
    db.exec_ids(QueryBuilder::search().from(1).to(5).query(), &[1, -10, 5]);
}

#[test]
fn search_from_to_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query(), 5);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec![1, 2, 3, 4, 1, 2, 3, 4])
            .to(vec![2, 3, 4, 5, 2, 3, 4, 5])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search().from(1).to(4).limit(4).query(),
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
            .from(vec![1, 2, 3, 4, 1, 2, 3, 4])
            .to(vec![2, 3, 4, 5, 2, 3, 4, 5])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search().from(1).to(4).offset(3).query(),
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
            .from(vec![1, 2, 3, 4, 1, 2, 3, 4])
            .to(vec![2, 3, 4, 5, 2, 3, 4, 5])
            .query(),
        8,
    );
    db.exec_ids(
        QueryBuilder::search()
            .from(1)
            .to(4)
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
            .from(vec![1, 3, 5, 7])
            .to(vec![3, 5, 7, 9])
            .query(),
        4,
    );
    db.exec_ids(
        QueryBuilder::search().to(9).query(),
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
            .from(vec![1, 3, 5, 7])
            .to(vec![3, 5, 7, 9])
            .query(),
        4,
    );
    db.exec_ids(QueryBuilder::search().to(9).limit(3).query(), &[9, -14, 7]);
}

#[test]
fn search_to_offset() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(10).query(), 10);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec![1, 3, 5, 7])
            .to(vec![3, 5, 7, 9])
            .query(),
        4,
    );
    db.exec_ids(
        QueryBuilder::search().to(9).offset(2).query(),
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
            .from(vec![1, 3, 5, 7])
            .to(vec![3, 5, 7, 9])
            .query(),
        4,
    );
    db.exec_ids(
        QueryBuilder::search().to(9).offset(2).limit(4).query(),
        &[7, -13, 5, -12],
    );
}

#[test]
fn search_from_ordered_by() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("users").query(), 1);

    let users = db.exec_mut_result(
        QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("name", "z").into(), ("age", 31).into(), ("id", 1).into()],
                vec![("name", "x").into(), ("age", 12).into(), ("id", 2).into()],
                vec![("name", "y").into(), ("age", 57).into(), ("id", 3).into()],
                vec![("name", "a").into(), ("age", 60).into(), ("id", 4).into()],
                vec![("name", "f").into(), ("age", 4).into(), ("id", 5).into()],
                vec![("name", "s").into(), ("age", 18).into(), ("id", 6).into()],
                vec![("name", "y").into(), ("age", 28).into(), ("id", 7).into()],
                vec![("name", "k").into(), ("age", 9).into(), ("id", 8).into()],
                vec![("name", "w").into(), ("age", 6).into(), ("id", 9).into()],
                vec![("name", "c").into(), ("age", 5).into(), ("id", 10).into()],
            ])
            .query(),
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from("users")
            .to(users.ids())
            .query(),
        10,
    );

    db.exec_elements(
        QueryBuilder::select()
            .search(
                QueryBuilder::search()
                    .from("users")
                    .order_by(&[
                        DbKeyOrder::Desc("age".into()),
                        DbKeyOrder::Asc("name".into()),
                    ])
                    .offset(3)
                    .limit(3)
                    .query(),
            )
            .query(),
        &[
            DbElement {
                id: DbId(8),
                values: vec![("name", "y").into(), ("age", 28).into(), ("id", 7).into()],
            },
            DbElement {
                id: DbId(7),
                values: vec![("name", "s").into(), ("age", 18).into(), ("id", 6).into()],
            },
            DbElement {
                id: DbId(3),
                values: vec![("name", "x").into(), ("age", 12).into(), ("id", 2).into()],
            },
        ],
    );
}

#[test]
fn search_to_ordered_by() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("users").query(), 1);

    let users = db.exec_mut_result(
        QueryBuilder::insert()
            .nodes()
            .values(vec![
                vec![("name", "z").into(), ("age", 31).into(), ("id", 1).into()],
                vec![("name", "x").into(), ("id", 2).into()],
                vec![("name", "y").into(), ("age", 57).into(), ("id", 3).into()],
                vec![("name", "a").into(), ("age", 60).into(), ("id", 4).into()],
                vec![("name", "f").into(), ("age", 4).into(), ("id", 5).into()],
                vec![("name", "s").into(), ("age", 18).into(), ("id", 6).into()],
                vec![("name", "y").into(), ("age", 28).into(), ("id", 7).into()],
                vec![("name", "k").into(), ("id", 8).into()],
                vec![("name", "w").into(), ("age", 6).into(), ("id", 9).into()],
                vec![("name", "c").into(), ("age", 5).into(), ("id", 10).into()],
            ])
            .query(),
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(users.ids())
            .to("users")
            .query(),
        10,
    );

    db.exec_elements(
        QueryBuilder::select()
            .search(
                QueryBuilder::search()
                    .to("users")
                    .order_by(&[
                        DbKeyOrder::Asc("age".into()),
                        DbKeyOrder::Desc("name".into()),
                    ])
                    .limit(3)
                    .query(),
            )
            .query(),
        &[
            DbElement {
                id: DbId(6),
                values: vec![("name", "f").into(), ("age", 4).into(), ("id", 5).into()],
            },
            DbElement {
                id: DbId(11),
                values: vec![("name", "c").into(), ("age", 5).into(), ("id", 10).into()],
            },
            DbElement {
                id: DbId(10),
                values: vec![("name", "w").into(), ("age", 6).into(), ("id", 9).into()],
            },
        ],
    );
}
