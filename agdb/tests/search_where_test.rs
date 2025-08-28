mod test_db;

use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbKeyOrder;
use agdb::KeyValueComparison;
use agdb::QueryBuilder;
use agdb::QueryConditionData;
use test_db::TestDb;

#[track_caller]
fn create_db() -> TestDb {
    let mut db = TestDb::new();
    //1, 2, 3
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["root", "users", "docs"])
            .query(),
        3,
    );
    //-4, -5
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from("root")
            .to(["users", "docs"])
            .query(),
        2,
    );
    //6, 7, 8
    let docs = db.exec_mut_result(
        QueryBuilder::insert()
            .nodes()
            .values([
                [
                    ("name", "notes").into(),
                    ("content", vec!["abc", "def", "ghi"]).into(),
                ],
                [
                    ("name", "book").into(),
                    (
                        "content",
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit",
                    )
                        .into(),
                ],
                [
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
            .from("docs")
            .to(&docs)
            .query(),
        3,
    );
    //12, 13, 14, 15, 16
    let users = db.exec_mut_result(
        QueryBuilder::insert()
            .nodes()
            .values([
                [
                    ("id", 1).into(),
                    ("username", "user_1").into(),
                    ("active", 1).into(),
                    ("registered", 10).into(),
                ],
                [
                    ("id", 2).into(),
                    ("username", "user_2").into(),
                    ("active", 0).into(),
                    ("registered", 20).into(),
                ],
                [
                    ("id", 3).into(),
                    ("username", "user_3").into(),
                    ("active", 1).into(),
                    ("registered", 30).into(),
                ],
                [
                    ("id", 4).into(),
                    ("username", "user_4").into(),
                    ("active", 1).into(),
                    ("registered", 40).into(),
                ],
                [
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
            .from("users")
            .to(&users)
            .query(),
        5,
    );
    //-22, -23, -24
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from([
                users.elements[0].id,
                users.elements[2].id,
                users.elements[3].id,
            ])
            .to([
                docs.elements[1].id,
                docs.elements[0].id,
                docs.elements[2].id,
            ])
            .values([
                [("type", "writes").into()],
                [("type", "owns").into()],
                [("type", "owns").into()],
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
            .keys(["username", "id"])
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
            .distance(2)
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
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .edge_count(CountComparison::GreaterThanOrEqual(2))
            .query(),
        &[1, 3, 2, 8, 7, 6, 15, 14, 12],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .edge_count(CountComparison::LessThanOrEqual(2))
            .query(),
        &[1, 8, 7, 6, 16, 15, 14, 13, 12],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .edge_count(CountComparison::LessThan(2))
            .query(),
        &[16, 13],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .edge_count(2)
            .query(),
        &[1, 8, 7, 6, 15, 14, 12],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .edge_count(CountComparison::NotEqual(2))
            .query(),
        &[3, 2, 16, 13],
    );
}

#[test]
fn search_from_where_node_edge() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .node()
            .and()
            .not()
            .ids([1, 2, 3])
            .and()
            .not_beyond()
            .ids("users")
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
            .ids(["docs", "users"])
            .and()
            .not_beyond()
            .ids(["docs", "users"])
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
            .order_by([DbKeyOrder::Asc("id".into())])
            .where_()
            .key("active")
            .value(1)
            .query(),
        &[12, 14, 15],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("users")
            .order_by([DbKeyOrder::Asc("id".into())])
            .where_()
            .key("active")
            .value(Comparison::NotEqual(1.into()))
            .query(),
        &[13, 16],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("users")
            .order_by([DbKeyOrder::Asc("id".into())])
            .where_()
            .key("active")
            .value(Comparison::LessThan(1.into()))
            .query(),
        &[13, 16],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("users")
            .order_by([DbKeyOrder::Asc("id".into())])
            .where_()
            .key("active")
            .value(Comparison::LessThanOrEqual(1.into()))
            .query(),
        &[12, 13, 14, 15, 16],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("users")
            .order_by([DbKeyOrder::Desc("id".into())])
            .where_()
            .key("active")
            .value(Comparison::GreaterThan(0.into()))
            .query(),
        &[15, 14, 12],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("users")
            .order_by([DbKeyOrder::Desc("id".into())])
            .where_()
            .key("active")
            .value(Comparison::GreaterThanOrEqual(0.into()))
            .query(),
        &[16, 15, 14, 13, 12],
    );
}

#[test]
fn search_from_where_where() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .where_()
            .where_()
            .key("active")
            .value(1)
            .or()
            .key("active")
            .value(0)
            .end_where()
            .or()
            .where_()
            .edge()
            .and()
            .key("type")
            .value("writes")
            .query(),
        &[16, 15, 14, 13, 12, -22],
    );
}

#[test]
fn search_from_limit_offset_where() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .limit(2)
            .where_()
            .node()
            .and()
            .not()
            .ids([1, 2, 3])
            .and()
            .not_beyond()
            .ids("users")
            .query(),
        &[8, 7],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .offset(1)
            .where_()
            .node()
            .and()
            .not()
            .ids([1, 2, 3])
            .and()
            .not_beyond()
            .ids("users")
            .query(),
        &[7, 6],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .offset(1)
            .limit(1)
            .where_()
            .node()
            .and()
            .not()
            .ids([1, 2, 3])
            .and()
            .not_beyond()
            .ids("users")
            .query(),
        &[7],
    );
}

#[test]
fn search_from_to_where() {
    let db = create_db();

    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .to(7)
            .where_()
            .not_beyond()
            .ids("docs")
            .and()
            .keys("id")
            .query(),
        &[12.into()],
    )
}

#[test]
fn search_from_to_where_filter() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["start", "end"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["start", "start"])
            .to(["end", "end"])
            .values([vec![], vec![("key", 1).into()]])
            .query(),
        2,
    );

    db.exec_ids(
        QueryBuilder::search().from("start").to("end").query(),
        &[1, -3, 2],
    );

    db.exec_ids(
        QueryBuilder::search()
            .from("start")
            .to("end")
            .where_()
            .keys("key")
            .query(),
        &[-4],
    );
}

#[test]
fn search_from_where_key_value_contains() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("docs")
            .where_()
            .key("content")
            .value(Comparison::Contains("apples".into()))
            .query(),
        &[8],
    );
}

#[test]
fn search_path_with_distance() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("root")
            .to("users")
            .where_()
            .distance(CountComparison::LessThanOrEqual(2))
            .query(),
        &[1, -4, 2],
    );
}

#[test]
fn search_where_distance_defaults_to_equals() {
    let query = QueryBuilder::search()
        .from("root")
        .where_()
        .distance(2)
        .query();

    assert_eq!(
        query.conditions[0].data,
        QueryConditionData::Distance(2.into())
    );
}

#[test]
fn search_where_edge_count_defaults_to_equals() {
    let query = QueryBuilder::search()
        .from("root")
        .where_()
        .edge_count(1)
        .and()
        .edge_count_from(2)
        .and()
        .edge_count_to(3)
        .query();

    assert_eq!(
        query.conditions[0].data,
        QueryConditionData::EdgeCount(1.into())
    );

    assert_eq!(
        query.conditions[1].data,
        QueryConditionData::EdgeCountFrom(2.into())
    );

    assert_eq!(
        query.conditions[2].data,
        QueryConditionData::EdgeCountTo(3.into())
    );
}

#[test]
fn search_where_key_value_defaults_to_equals() {
    let query = QueryBuilder::search()
        .from("root")
        .where_()
        .key("active")
        .value(1)
        .query();

    assert_eq!(
        query.conditions[0].data,
        QueryConditionData::KeyValue(KeyValueComparison {
            key: "active".into(),
            value: Comparison::Equal(1.into())
        })
    );
}

#[test]
fn search_where_starts_with() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("docs")
            .where_()
            .key("content")
            .value(Comparison::StartsWith("Lorem".into()))
            .query(),
        &[7],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("docs")
            .where_()
            .key("content")
            .value(Comparison::StartsWith("ipsum".into()))
            .query(),
        &[],
    );
}

#[test]
fn search_where_ends_with() {
    let db = create_db();
    db.exec_ids(
        QueryBuilder::search()
            .from("docs")
            .where_()
            .key("content")
            .value(Comparison::EndsWith(vec!["adipiscing ", "elit"].into()))
            .query(),
        &[7],
    );
    db.exec_ids(
        QueryBuilder::search()
            .from("docs")
            .where_()
            .key("content")
            .value(Comparison::EndsWith("adipiscing".into()))
            .query(),
        &[],
    );
}

#[test]
fn search_neighbor() {
    let neighbor_query = QueryBuilder::search()
        .from("root")
        .where_()
        .neighbor()
        .query();
    let distance_same = QueryBuilder::search()
        .from("root")
        .where_()
        .distance(2)
        .query();

    assert_eq!(neighbor_query, distance_same);
}
