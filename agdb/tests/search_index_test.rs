mod test_db;

use agdb::DbId;
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
    };

    db.exec_error(query, "Index condition missing");
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
    };

    db.exec_error(query, "Index condition must be key value");
}
