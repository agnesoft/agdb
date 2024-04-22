use agdb::DbElement;
use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::InsertNodesQuery;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb::QueryType;
use agdb::SelectQuery;

use crate::test_db::TestDb;

#[test]
fn serialize_deserialize() {
    let insert_query_json = serde_json::to_string(
        &QueryBuilder::insert()
            .nodes()
            .values(vec![vec![("key", 10).into()]])
            .query(),
    )
    .unwrap();
    let select_query_json = serde_json::to_string(&QueryBuilder::select().ids(1).query()).unwrap();

    let mut db = TestDb::new();
    let insert_query: InsertNodesQuery = serde_json::from_str(&insert_query_json).unwrap();
    let select_query: SelectQuery = serde_json::from_str(&select_query_json).unwrap();

    db.exec_mut(insert_query, 1);
    let result = db.exec_result(select_query);

    let result_string = serde_json::to_string(&result).unwrap();
    let result_back: QueryResult = serde_json::from_str(&result_string).unwrap();

    assert_eq!(result_back.result, 1);
    assert_eq!(
        result_back.elements,
        vec![DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("key", 10).into()]
        }]
    );
}

#[test]
fn query_type() {
    let queries: Vec<QueryType> = vec![
        QueryBuilder::insert().aliases("").ids(1).query().into(),
        QueryBuilder::insert().edges().from(1).to(1).query().into(),
        QueryBuilder::insert().index("").query().into(),
        QueryBuilder::insert().nodes().count(1).query().into(),
        QueryBuilder::insert().values(vec![]).ids(1).query().into(),
        QueryBuilder::remove().ids(1).query().into(),
        QueryBuilder::remove().aliases("").query().into(),
        QueryBuilder::remove().index("").query().into(),
        QueryBuilder::remove().values(vec![]).ids(1).query().into(),
        QueryBuilder::search().from(1).query().into(),
        QueryBuilder::select().ids(1).query().into(),
        QueryBuilder::select().aliases().ids(1).query().into(),
        QueryBuilder::select().aliases().query().into(),
        QueryBuilder::select().edge_count().ids(1).query().into(),
        QueryBuilder::select().keys().ids(1).query().into(),
        QueryBuilder::select().key_count().ids(1).query().into(),
        QueryBuilder::select().values(vec![]).ids(1).query().into(),
        QueryBuilder::select().indexes().query().into(),
    ];

    let as_str = serde_json::to_string(&queries).unwrap();
    let back: Vec<QueryType> = serde_json::from_str(&as_str).unwrap();

    assert_eq!(queries.len(), back.len());
}

#[test]
fn conditions() {
    let query = QueryType::Search(
        QueryBuilder::search()
            .depth_first()
            .from(1)
            .to(2)
            .order_by(vec![DbKeyOrder::Asc("key".into())])
            .offset(10)
            .limit(10)
            .where_()
            .distance(agdb::CountComparison::LessThan(10))
            .and()
            .where_()
            .keys(vec!["key".into()])
            .or()
            .key("key")
            .value(agdb::Comparison::Equal(1.1.into()))
            .end_where()
            .and()
            .edge_count(agdb::CountComparison::GreaterThan(1))
            .and()
            .not_beyond()
            .ids("alias")
            .and()
            .where_()
            .edge()
            .or()
            .node()
            .end_where()
            .and()
            .not()
            .edge_count_from(agdb::CountComparison::LessThan(1))
            .and()
            .not()
            .edge_count_to(agdb::CountComparison::GreaterThan(5))
            .and()
            .beyond()
            .key("key")
            .value(agdb::Comparison::Contains("something".into()))
            .query(),
    );

    let as_str = serde_json::to_string(&query).unwrap();
    let _back: QueryType = serde_json::from_str(&as_str).unwrap();
}
