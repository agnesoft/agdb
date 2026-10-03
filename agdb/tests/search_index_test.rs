mod test_db;

use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::QueryBuilder;
use agdb::QueryCondition;
use agdb::SearchQuery;
use test_db::TestDb;

#[test]
fn search_indexes() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![("username", "user2").into(), ("age", 20).into()],
                vec![("username", "user3").into()],
                vec![("username", "user4").into(), ("age", 33).into()],
                vec![("username", "user5").into()],
            ])
            .query(),
        5,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("username")
            .value("user3")
            .query(),
        &[3],
    );
}

#[test]
fn search_index_multiple_values() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("age").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![("username", "user2").into(), ("age", 20).into()],
                vec![("username", "user3").into()],
                vec![("username", "user4").into(), ("age", 33).into()],
                vec![("username", "user5").into()],
            ])
            .query(),
        5,
    );

    db.exec_ids(
        QueryBuilder::search().index("age").value(20).query(),
        &[1, 2],
    );
}

#[test]
fn search_index_missing_value() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("age").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![("username", "user2").into(), ("age", 20).into()],
                vec![("username", "user3").into()],
                vec![("username", "user4").into(), ("age", 33).into()],
                vec![("username", "user5").into()],
            ])
            .query(),
        5,
    );

    db.exec_ids(QueryBuilder::search().index("age").value(50).query(), &[]);
}

#[test]
fn missing_index() {
    let db = TestDb::new();

    db.exec_error(
        QueryBuilder::search()
            .index("missing")
            .value("anything")
            .query(),
        "Index 'missing' not found",
    );
}

#[test]
fn removed_index() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(QueryBuilder::remove().index("username").query(), 0);

    db.exec_error(
        QueryBuilder::search()
            .index("username")
            .value("anything")
            .query(),
        "Index 'username' not found",
    );
}

#[test]
fn missing_condition() {
    let db = TestDb::new();
    let query = SearchQuery {
        algorithm: agdb::SearchQueryAlgorithm::Index,
        origin: agdb::QueryId::Id(DbId(0)),
        destination: agdb::QueryId::Id(DbId(0)),
        limit: 0,
        offset: 0,
        order_by: vec![],
        conditions: vec![],
        reverse: false,
    };

    db.exec_error(query, "Index condition is required for index search");
}

#[test]
fn wrong_condition() {
    let db = TestDb::new();
    let query = SearchQuery {
        algorithm: agdb::SearchQueryAlgorithm::Index,
        origin: agdb::QueryId::Id(DbId(0)),
        destination: agdb::QueryId::Id(DbId(0)),
        limit: 0,
        offset: 0,
        order_by: vec![],
        conditions: vec![QueryCondition {
            logic: agdb::QueryConditionLogic::And,
            modifier: agdb::QueryConditionModifier::None,
            data: agdb::QueryConditionData::Node,
        }],
        reverse: false,
    };

    db.exec_error(query, "Index condition must be key value");
}

#[test]
fn search_index_where_key_value() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("age").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![("username", "user2").into(), ("age", 20).into()],
                vec![("username", "user3").into(), ("age", 20).into()],
            ])
            .query(),
        3,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("age")
            .value(20)
            .where_()
            .key("username")
            .value("user2")
            .query(),
        &[2],
    );
}

#[test]
fn search_index_where_multiple_conditions() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("role").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![
                    ("role", "admin").into(),
                    ("active", 1).into(),
                    ("name", "alice").into(),
                ],
                vec![
                    ("role", "admin").into(),
                    ("active", 0).into(),
                    ("name", "bob").into(),
                ],
                vec![
                    ("role", "admin").into(),
                    ("active", 1).into(),
                    ("name", "carol").into(),
                ],
            ])
            .query(),
        3,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("role")
            .value("admin")
            .where_()
            .key("active")
            .value(1)
            .query(),
        &[1, 3],
    );
}

#[test]
fn search_index_where_or() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("role").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("role", "admin").into(), ("name", "alice").into()],
                vec![("role", "admin").into(), ("name", "bob").into()],
                vec![("role", "admin").into(), ("name", "carol").into()],
            ])
            .query(),
        3,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("role")
            .value("admin")
            .where_()
            .key("name")
            .value("alice")
            .or()
            .key("name")
            .value("carol")
            .query(),
        &[1, 3],
    );
}

#[test]
fn search_index_where_not() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("age").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![
                    ("username", "user2").into(),
                    ("age", 20).into(),
                    ("inactive", 1).into(),
                ],
                vec![("username", "user3").into(), ("age", 20).into()],
            ])
            .query(),
        3,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("age")
            .value(20)
            .where_()
            .not()
            .keys("inactive")
            .query(),
        &[1, 3],
    );
}

#[test]
fn search_index_where_node() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("tag").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([vec![("tag", "x").into()], vec![("tag", "x").into()]])
            .query(),
        2,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("tag")
            .value("x")
            .where_()
            .node()
            .query(),
        &[1, 2],
    );
}

#[test]
fn search_index_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("age").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("age", 20).into()],
                vec![("age", 20).into()],
                vec![("age", 20).into()],
                vec![("age", 20).into()],
            ])
            .query(),
        4,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("age")
            .value(20)
            .limit(2)
            .query(),
        &[1, 2],
    );
}

#[test]
fn search_index_offset() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("age").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("age", 20).into()],
                vec![("age", 20).into()],
                vec![("age", 20).into()],
                vec![("age", 20).into()],
            ])
            .query(),
        4,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("age")
            .value(20)
            .offset(2)
            .query(),
        &[3, 4],
    );
}

#[test]
fn search_index_offset_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("age").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("age", 20).into()],
                vec![("age", 20).into()],
                vec![("age", 20).into()],
                vec![("age", 20).into()],
            ])
            .query(),
        4,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("age")
            .value(20)
            .offset(1)
            .limit(2)
            .query(),
        &[2, 3],
    );
}

#[test]
fn search_index_order_by() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("role").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("role", "admin").into(), ("name", "carol").into()],
                vec![("role", "admin").into(), ("name", "alice").into()],
                vec![("role", "admin").into(), ("name", "bob").into()],
            ])
            .query(),
        3,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("role")
            .value("admin")
            .order_by([DbKeyOrder::Asc("name".into())])
            .query(),
        &[2, 3, 1],
    );
}

#[test]
fn search_index_order_by_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("role").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("role", "admin").into(), ("name", "carol").into()],
                vec![("role", "admin").into(), ("name", "alice").into()],
                vec![("role", "admin").into(), ("name", "bob").into()],
            ])
            .query(),
        3,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("role")
            .value("admin")
            .order_by([DbKeyOrder::Asc("name".into())])
            .limit(2)
            .query(),
        &[2, 3],
    );
}

#[test]
fn search_index_where_and_limit() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("role").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("role", "admin").into(), ("active", 1).into()],
                vec![("role", "admin").into(), ("active", 0).into()],
                vec![("role", "admin").into(), ("active", 1).into()],
                vec![("role", "admin").into(), ("active", 1).into()],
            ])
            .query(),
        4,
    );

    db.exec_ids(
        QueryBuilder::search()
            .index("role")
            .value("admin")
            .where_()
            .key("active")
            .value(1)
            .query(),
        &[1, 3, 4],
    );
}

#[test]
fn search_index_where_nested() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("role").query(), 0);

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![
                    ("role", "admin").into(),
                    ("active", 1).into(),
                    ("level", 1).into(),
                ],
                vec![
                    ("role", "admin").into(),
                    ("active", 1).into(),
                    ("level", 2).into(),
                ],
                vec![
                    ("role", "admin").into(),
                    ("active", 0).into(),
                    ("level", 3).into(),
                ],
            ])
            .query(),
        3,
    );

    // active AND (level == 1 OR level == 2)
    db.exec_ids(
        QueryBuilder::search()
            .index("role")
            .value("admin")
            .where_()
            .key("active")
            .value(1)
            .and()
            .where_()
            .key("level")
            .value(1)
            .or()
            .key("level")
            .value(2)
            .end_where()
            .query(),
        &[1, 2],
    );
}
