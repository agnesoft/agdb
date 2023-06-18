mod test_db;

use agdb::Comparison;
use agdb::CountComparison;
use agdb::QueryBuilder;
use test_db::TestDb;

#[track_caller]
fn create_db() -> TestDb {
    let mut db = TestDb::new();
    //1, 2, 3
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["root".into(), "users".into(), "docs".into()])
            .query(),
        3,
    );
    //-4, -5
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["root".into()])
            .to(&["users".into(), "docs".into()])
            .query(),
        2,
    );
    //6, 7, 8
    let docs = db.exec_mut_result(
        QueryBuilder::insert()
            .nodes()
            .values(&[
                &[
                    ("name", "notes").into(),
                    ("content", vec!["abc", "def", "ghi"]).into(),
                ],
                &[
                    ("name", "book").into(),
                    (
                        "content",
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit",
                    )
                        .into(),
                ],
                &[
                    ("name", "shopping list").into(),
                    ("content", vec!["apples", "oranges"]).into(),
                ],
            ])
            .query(),
    );
    //-9, -10, -11
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["docs".into()])
            .to(&docs.ids())
            .query(),
        3,
    );
    //12, 13, 14, 15, 16
    let users = db.exec_mut_result(
        QueryBuilder::insert()
            .nodes()
            .values(&[
                &[
                    ("id", 1).into(),
                    ("username", "user_1").into(),
                    ("active", 1).into(),
                    ("registered", 10).into(),
                ],
                &[
                    ("id", 2).into(),
                    ("username", "user_2").into(),
                    ("active", 0).into(),
                    ("registered", 20).into(),
                ],
                &[
                    ("id", 3).into(),
                    ("username", "user_3").into(),
                    ("active", 1).into(),
                    ("registered", 30).into(),
                ],
                &[
                    ("id", 4).into(),
                    ("username", "user_4").into(),
                    ("active", 1).into(),
                    ("registered", 40).into(),
                ],
                &[
                    ("id", 5).into(),
                    ("username", "user_5").into(),
                    ("active", 0).into(),
                    ("registered", 50).into(),
                ],
            ])
            .query(),
    );
    //-17, -18, -19, -20, -21
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["users".into()])
            .to(&users.ids())
            .query(),
        5,
    );
    //-22, -23, -24
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&[
                users.elements[0].id.into(),
                users.elements[2].id.into(),
                users.elements[3].id.into(),
            ])
            .to(&[
                docs.elements[1].id.into(),
                docs.elements[0].id.into(),
                docs.elements[2].id.into(),
            ])
            .values(&[
                &[("type", "writes").into()],
                &[("type", "owns").into()],
                &[("type", "owns").into()],
            ])
            .query(),
        3,
    );

    db
}

#[test]
fn search_from_where_keys() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("users")
            .where_()
            .keys(&["username".into(), "id".into()])
            .query(),
        &[16, 15, 14, 13, 12],
    );
}

#[test]
fn search_from_where_distance() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .distance(CountComparison::LessThan(3))
            .query(),
        &[1, -5, -4, 3, 2],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .distance(CountComparison::LessThanOrEqual(2))
            .query(),
        &[1, -5, -4, 3, 2],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .distance(CountComparison::Equal(2))
            .query(),
        &[3, 2],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .distance(CountComparison::GreaterThan(1))
            .and()
            .distance(CountComparison::LessThan(3))
            .query(),
        &[3, 2],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .distance(CountComparison::GreaterThanOrEqual(2))
            .and()
            .distance(CountComparison::LessThan(3))
            .query(),
        &[3, 2],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .distance(CountComparison::NotEqual(1))
            .and()
            .distance(CountComparison::LessThan(3))
            .query(),
        &[1, 3, 2],
    );
}

#[test]
fn search_from_where_edge_count_test() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .edge_count(CountComparison::GreaterThan(2))
            .query(),
        &[3, 2],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .edge_count_from(CountComparison::GreaterThan(1))
            .query(),
        &[1, 3, 2],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .edge_count_to(CountComparison::GreaterThan(1))
            .query(),
        &[8, 7, 6],
    );
}

#[test]
fn search_from_where_node_edge() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("docs")
            .where_()
            .node()
            .and()
            .not()
            .ids(&["docs".into()])
            .query(),
        &[8, 7, 6],
    );
    db.exec_ids(
        QueryBuilder::search().from("docs").where_().edge().query(),
        &[-11, -10, -9],
    );
}

#[test]
fn search_from_where_ids() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from(1)
            .where_()
            .ids(&["docs".into(), "users".into()])
            .and()
            .not_beyond()
            .ids(&["docs".into(), "users".into()])
            .query(),
        &[3, 2],
    );
}

#[test]
fn search_from_where_key_value() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("users")
            .where_()
            .key("active")
            .value(Comparison::Equal(1.into()))
            .query(),
        &[15, 14, 12],
    );
}

#[test]
fn search_from_where_where_key_and_key_end_where_and_distance() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .where_()
            .key("active")
            .value(Comparison::Equal(1.into()))
            .or()
            .key("active")
            .value(Comparison::Equal(0.into()))
            .end_where()
            .or()
            .where_()
            .edge()
            .and()
            .key("type")
            .value(Comparison::Equal("writes".into()))
            .query(),
        &[16, 15, 14, 13, 12, -22],
    );
}
