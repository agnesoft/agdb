mod test_db;

use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbKeyOrder;
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
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .edge_count(CountComparison::GreaterThan(2))
        .query();
}

#[test]
fn search_from_where_edge_from_count_test() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .edge_count_from(CountComparison::GreaterThan(2))
        .query();
}

#[test]
fn search_from_where_edge_to_count_test() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .edge_count_to(CountComparison::GreaterThan(2))
        .query();
}

#[test]
fn search_from_where_edge() {
    let _query = QueryBuilder::search().from(1).where_().edge().query();
}

#[test]
fn search_from_where_ids() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .ids(&["alias".into(), "alias2".into()])
        .query();
}

#[test]
fn search_from_where_node() {
    let _query = QueryBuilder::search().from(1).where_().node().query();
}

#[test]
fn search_from_where_not_beyond_keys() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .not_beyond()
        .keys(&["key".into()])
        .query();
}

#[test]
fn search_from_where_not_key() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .not()
        .keys(&["key".into()])
        .query();
}

#[test]
fn search_from_where_keys_or_distance() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .keys(&["key".into()])
        .or()
        .distance(CountComparison::LessThan(2))
        .query();
}

#[test]
fn search_from_where_key_value() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .key("key".into())
        .value(Comparison::LessThan(10.into()))
        .query();
}

#[test]
fn search_from_where_where_key_and_key_end_where_and_distance() {
    let _query = QueryBuilder::search()
        .from(1)
        .where_()
        .where_()
        .keys(&["key".into()])
        .or()
        .keys(&["key2".into()])
        .end_where()
        .and()
        .distance(CountComparison::LessThan(2))
        .query();
}

#[test]
fn search_from_ordered_by_where_key_value() {
    let _query = QueryBuilder::search()
        .from(1)
        .order_by(&[DbKeyOrder::Asc("key".into())])
        .where_()
        .key("key".into())
        .value(Comparison::LessThan(10.into()))
        .query();
}
