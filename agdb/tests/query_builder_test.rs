#![cfg(all(feature = "derive", feature = "serde"))]

use agdb::Amend;
use agdb::BitwiseOp;
use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::DbKeyValue;
use agdb::DbType;
use agdb::DbValue;
use agdb::InsertAliasesQuery;
use agdb::InsertEdgesQuery;
use agdb::InsertIndexQuery;
use agdb::InsertNodesQuery;
use agdb::InsertValuesQuery;
use agdb::KeyValueComparison;
use agdb::QueryBuilder;
use agdb::QueryCondition;
use agdb::QueryConditionData;
use agdb::QueryConditionLogic;
use agdb::QueryConditionModifier;
use agdb::QueryId;
use agdb::QueryIds;
use agdb::QueryType;
use agdb::QueryValues;
use agdb::RemoveAliasesQuery;
use agdb::RemoveIndexQuery;
use agdb::RemoveQuery;
use agdb::RemoveValuesQuery;
use agdb::SearchQuery;
use agdb::SearchQueryAlgorithm;
use agdb::SelectAliasesQuery;
use agdb::SelectAllAliasesQuery;
use agdb::SelectEdgeCountQuery;
use agdb::SelectIndexesQuery;
use agdb::SelectKeyCountQuery;
use agdb::SelectKeysQuery;
use agdb::SelectNodeCountQuery;
use agdb::SelectValuesQuery;

#[derive(Default, DbType)]
struct T {
    db_id: Option<QueryId>,
    value1: String,
    value2: i64,
}

#[test]
fn insert_aliases_ids_alias_id() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().aliases("a").ids(1).query()),
        QueryType::InsertAlias(InsertAliasesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1))]),
            aliases: vec!["a".to_string()]
        })
    );
}

#[test]
fn insert_aliases_ids_alias_alias() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().aliases("a").ids("b").query()),
        QueryType::InsertAlias(InsertAliasesQuery {
            ids: QueryIds::Ids(vec![QueryId::Alias("b".to_string())]),
            aliases: vec!["a".to_string()]
        })
    );
}

#[test]
fn insert_aliases_ids_aliases_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .aliases(["a", "b"])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertAlias(InsertAliasesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            aliases: vec!["a".to_string(), "b".to_string()]
        })
    );
}

#[test]
fn insert_aliases_ids_aliases_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .aliases(["a", "b"])
                .ids(QueryBuilder::search().from(1).where_().node().query())
                .query()
        ),
        QueryType::InsertAlias(InsertAliasesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(1)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }],
                reverse: false
            }),
            aliases: vec!["a".to_string(), "b".to_string()]
        })
    );
}

#[test]
fn insert_aliases_search_from_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .aliases(["a", "b"])
                .search()
                .from(1)
                .where_()
                .node()
                .query()
        ),
        QueryType::InsertAlias(InsertAliasesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(1)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }],
                reverse: false
            }),
            aliases: vec!["a".to_string(), "b".to_string()]
        })
    );
}

#[test]
fn insert_edges_from_to_id_id() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().edges().from(1).to(2).query()),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Id(DbId(1))]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(2))]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![]),
            each: false
        })
    );
}

#[test]
fn insert_edges_from_to_alias_alias() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().edges().from("a").to("b").query()),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Alias("a".to_string())]),
            to: QueryIds::Ids(vec![QueryId::Alias("b".to_string())]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![]),
            each: false
        })
    );
}

#[test]
fn insert_edges_from_to_alias_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().edges().from("a").to([1, 2]).query()),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Alias("a".to_string())]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![]),
            each: false
        })
    );
}

#[test]
fn insert_edges_from_to_ids_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .from([1, 2])
                .to([2, 3])
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(2)), QueryId::Id(DbId(3))]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![]),
            each: false
        })
    );
}

#[test]
fn insert_edges_from_to_each() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .from([1, 2])
                .to([2, 3])
                .each()
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(2)), QueryId::Id(DbId(3))]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![]),
            each: true
        })
    );
}

#[test]
fn insert_edges_from_to_each_values() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .from([1, 2])
                .to([2, 3])
                .each()
                .values([[("k", 1).into()], [("k", 2).into()]])
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(2)), QueryId::Id(DbId(3))]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            each: true
        })
    );
}

#[test]
fn insert_edges_from_to_each_values_uniform() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .from([1, 2])
                .to([2, 3])
                .each()
                .values_uniform([("k", 1).into(), (1, 10).into()])
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(2)), QueryId::Id(DbId(3))]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![
                DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                },
                DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }
            ]),
            each: true
        })
    );
}

#[test]
fn insert_edges_from_to_values_alias_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .from("a")
                .to([1, 2])
                .values([[("k", 1).into()], [("k", 2).into()]])
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Alias("a".to_string())]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            each: false
        })
    );
}

#[test]
fn insert_edges_from_to_values_uniform_alias_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .from("a")
                .to([1, 2])
                .values_uniform([("k", "v").into(), (1, 10).into()])
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Alias("a".to_string())]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![
                DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::String("v".to_string())
                },
                DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }
            ]),
            each: false
        })
    );
}

#[test]
fn insert_edges_from_to_search_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .from(QueryBuilder::search().from("a").where_().node().query())
                .to(QueryBuilder::search().from("b").where_().node().query())
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }],
                reverse: false
            }),
            to: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("b".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }],
                reverse: false
            }),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![]),
            each: false
        })
    );
}

#[test]
fn insert_edges_from_to_values_search_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .from(QueryBuilder::search().from("a").where_().node().query())
                .to(QueryBuilder::search().from("b").where_().node().query())
                .values([[("k", 1).into()], [("k", 2).into()]])
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }],
                reverse: false
            }),
            to: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("b".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }],
                reverse: false
            }),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            each: false
        })
    );
}

#[test]
fn insert_edges_from_to_values_uniform_search_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .from(QueryBuilder::search().from("a").where_().node().query())
                .to(QueryBuilder::search().from("b").where_().node().query())
                .values_uniform([("k", "v").into(), (1, 10).into()])
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }],
                reverse: false
            }),
            to: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("b".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }],
                reverse: false
            }),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![
                DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::String("v".to_string())
                },
                DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }
            ]),
            each: false
        })
    );
}

#[test]
fn insert_edges_ids_from_to_edge() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().edges().ids(-3).from(1).to(2).query()),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Id(DbId(1))]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(2))]),
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(-3))]),
            values: QueryValues::Single(vec![]),
            each: false
        })
    );
}

#[test]
fn insert_edges_ids_from_to_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .ids([-3, -4])
                .from(1)
                .to(2)
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Id(DbId(1))]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(2))]),
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(-3)), QueryId::Id(DbId(-4))]),
            values: QueryValues::Single(vec![]),
            each: false
        })
    );
}

#[test]
fn insert_edges_ids_from_to_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .edges()
                .ids(QueryBuilder::search().from(1).where_().edge().query())
                .from(1)
                .to(2)
                .query()
        ),
        QueryType::InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Id(DbId(1))]),
            to: QueryIds::Ids(vec![QueryId::Id(DbId(2))]),
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(1)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Edge
                }],
                reverse: false
            }),
            values: QueryValues::Single(vec![]),
            each: false
        })
    );
}

#[test]
fn insert_index() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().index("key").query()),
        QueryType::InsertIndex(InsertIndexQuery(DbValue::String("key".to_string())))
    );
}

#[test]
fn insert_nodes_count() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().nodes().count(2).query()),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 2,
            values: QueryValues::Single(vec![]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![])
        })
    );
}

#[test]
fn insert_nodes_count_values_uniform() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .nodes()
                .count(2)
                .values_uniform([("k", "v").into(), (1, 10).into()])
                .query()
        ),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 2,
            values: QueryValues::Single(vec![
                DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::String("v".to_string())
                },
                DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }
            ]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![])
        })
    );
}

#[test]
fn insert_nodes_aliases() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().nodes().aliases(["a", "b"]).query()),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Single(vec![]),
            aliases: vec!["a".to_string(), "b".to_string()],
            ids: QueryIds::Ids(vec![])
        })
    );
}

#[test]
fn insert_nodes_aliases_values() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .nodes()
                .aliases(["a", "b"])
                .values([[("k", 1).into()], [("k", 2).into()]])
                .query()
        ),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            aliases: vec!["a".to_string(), "b".to_string()],
            ids: QueryIds::Ids(vec![])
        })
    );
}

#[test]
fn insert_nodes_aliases_values_uniform() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .nodes()
                .aliases(["a", "b"])
                .values_uniform([("k", "v").into(), (1, 10).into()])
                .query()
        ),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Single(vec![
                DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::String("v".to_string())
                },
                DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }
            ]),
            aliases: vec!["a".to_string(), "b".to_string()],
            ids: QueryIds::Ids(vec![])
        })
    );
}

#[test]
fn insert_nodes_values() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .nodes()
                .values([[("k", 1).into()], [("k", 2).into()]])
                .query()
        ),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![])
        })
    );
}

#[test]
fn insert_nodes_ids_count_id() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().nodes().ids(1).count(1).query()),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 1,
            values: QueryValues::Single(vec![]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1))])
        })
    );
}

#[test]
fn insert_nodes_ids_count_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().nodes().ids([1, 2]).count(1).query()),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 1,
            values: QueryValues::Single(vec![]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))])
        })
    );
}

#[test]
fn insert_nodes_ids_count_alias() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().nodes().ids("a").count(1).query()),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 1,
            values: QueryValues::Single(vec![]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![QueryId::Alias("a".to_string())])
        })
    );
}

#[test]
fn insert_nodes_ids_aliases() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().nodes().ids("a").aliases("a").query()),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Single(vec![]),
            aliases: vec!["a".to_string()],
            ids: QueryIds::Ids(vec![QueryId::Alias("a".to_string())])
        })
    );
}

#[test]
fn insert_nodes_ids_count_aliases() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .nodes()
                .ids(["a", "b"])
                .count(1)
                .query()
        ),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 1,
            values: QueryValues::Single(vec![]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![
                QueryId::Alias("a".to_string()),
                QueryId::Alias("b".to_string())
            ])
        })
    );
}

#[test]
fn insert_nodes_ids_values() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .nodes()
                .ids([1, 2])
                .values([[("k", "v").into()], [(1, 10).into()]])
                .query()
        ),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::String("v".to_string())
                }],
                vec![DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }]
            ]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))])
        })
    );
}

#[test]
fn insert_nodes_ids_values_uniform() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .nodes()
                .ids([1, 2])
                .values_uniform([("k", "v").into(), (1, 10).into()])
                .query()
        ),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Single(vec![
                DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::String("v".to_string())
                },
                DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }
            ]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))])
        })
    );
}

#[test]
fn insert_nodes_ids_count_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .nodes()
                .ids(QueryBuilder::search().from(1).query())
                .count(1)
                .query()
        ),
        QueryType::InsertNodes(InsertNodesQuery {
            count: 1,
            values: QueryValues::Single(vec![]),
            aliases: vec![],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(1)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        })
    );
}

#[test]
fn insert_element() {
    assert_eq!(
        QueryType::from(QueryBuilder::insert().element(T::default()).query()),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(0))]),
            values: QueryValues::Multi(vec![vec![
                DbKeyValue {
                    key: DbValue::String("value1".to_string()),
                    value: DbValue::String("".to_string())
                },
                DbKeyValue {
                    key: DbValue::String("value2".to_string()),
                    value: DbValue::I64(0)
                }
            ]]),
            amend: Amend::None
        })
    );
}

#[test]
fn insert_elements() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .elements(&[T::default(), T::default()])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(0)), QueryId::Id(DbId(0))]),
            values: QueryValues::Multi(vec![
                vec![
                    DbKeyValue {
                        key: DbValue::String("value1".to_string()),
                        value: DbValue::String("".to_string())
                    },
                    DbKeyValue {
                        key: DbValue::String("value2".to_string()),
                        value: DbValue::I64(0)
                    }
                ],
                vec![
                    DbKeyValue {
                        key: DbValue::String("value1".to_string()),
                        value: DbValue::String("".to_string())
                    },
                    DbKeyValue {
                        key: DbValue::String("value2".to_string()),
                        value: DbValue::I64(0)
                    }
                ]
            ]),
            amend: Amend::None
        })
    );
}

#[test]
fn insert_values_ids_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .values([
                    vec![("k", "v").into(), (1, 10).into()],
                    vec![("k", 2).into()]
                ])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Multi(vec![
                vec![
                    DbKeyValue {
                        key: DbValue::String("k".to_string()),
                        value: DbValue::String("v".to_string())
                    },
                    DbKeyValue {
                        key: DbValue::I64(1),
                        value: DbValue::I64(10)
                    }
                ],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::None
        })
    );
}

#[test]
fn insert_values_ids_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .values([
                    vec![("k", "v").into(), (1, 10).into()],
                    vec![("k", 2).into()]
                ])
                .ids(QueryBuilder::search().from("a").query())
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            }),
            values: QueryValues::Multi(vec![
                vec![
                    DbKeyValue {
                        key: DbValue::String("k".to_string()),
                        value: DbValue::String("v".to_string())
                    },
                    DbKeyValue {
                        key: DbValue::I64(1),
                        value: DbValue::I64(10)
                    }
                ],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::None
        })
    );
}

#[test]
fn insert_values_search_from() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .values([
                    vec![("k", "v").into(), (1, 10).into()],
                    vec![("k", 2).into()]
                ])
                .search()
                .from("a")
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            }),
            values: QueryValues::Multi(vec![
                vec![
                    DbKeyValue {
                        key: DbValue::String("k".to_string()),
                        value: DbValue::String("v".to_string())
                    },
                    DbKeyValue {
                        key: DbValue::I64(1),
                        value: DbValue::I64(10)
                    }
                ],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::None
        })
    );
}

#[test]
fn insert_values_uniform_ids_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .values_uniform([("k", "v").into(), (1, 10).into()])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Single(vec![
                DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::String("v".to_string())
                },
                DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }
            ]),
            amend: Amend::None
        })
    );
}

#[test]
fn insert_values_uniform_ids_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .values_uniform([("k", "v").into(), (1, 10).into()])
                .ids(QueryBuilder::search().from("a").query())
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            }),
            values: QueryValues::Single(vec![
                DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::String("v".to_string())
                },
                DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }
            ]),
            amend: Amend::None
        })
    );
}

#[test]
fn insert_values_uniform_search_from() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .values_uniform([("k", "v").into(), (1, 10).into()])
                .search()
                .from("a")
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            }),
            values: QueryValues::Single(vec![
                DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::String("v".to_string())
                },
                DbKeyValue {
                    key: DbValue::I64(1),
                    value: DbValue::I64(10)
                }
            ]),
            amend: Amend::None
        })
    );
}

#[test]
fn insert_amend_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .amend([[("k", 1).into()], [("k", 2).into()]])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::Add
        })
    );
}

#[test]
fn insert_amend_uniform_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .amend_uniform([("k", 1).into()])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Single(vec![DbKeyValue {
                key: DbValue::String("k".to_string()),
                value: DbValue::I64(1)
            }]),
            amend: Amend::Add
        })
    );
}

#[test]
fn insert_amend_or_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .amend_or([[("k", 1).into()], [("k", 2).into()]])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::AddBitwise(BitwiseOp::Or)
        })
    );
}

#[test]
fn insert_amend_or_uniform_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .amend_or_uniform([("k", 1).into()])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Single(vec![DbKeyValue {
                key: DbValue::String("k".to_string()),
                value: DbValue::I64(1)
            }]),
            amend: Amend::AddBitwise(BitwiseOp::Or)
        })
    );
}

#[test]
fn insert_amend_and_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .amend_and([[("k", 1).into()], [("k", 2).into()]])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::AddBitwise(BitwiseOp::And)
        })
    );
}

#[test]
fn insert_amend_and_uniform_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .amend_and_uniform([("k", 1).into()])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Single(vec![DbKeyValue {
                key: DbValue::String("k".to_string()),
                value: DbValue::I64(1)
            }]),
            amend: Amend::AddBitwise(BitwiseOp::And)
        })
    );
}

#[test]
fn insert_amend_xor_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .amend_xor([[("k", 1).into()], [("k", 2).into()]])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::AddBitwise(BitwiseOp::Xor)
        })
    );
}

#[test]
fn insert_amend_xor_uniform_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::insert()
                .amend_xor_uniform([("k", 1).into()])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Single(vec![DbKeyValue {
                key: DbValue::String("k".to_string()),
                value: DbValue::I64(1)
            }]),
            amend: Amend::AddBitwise(BitwiseOp::Xor)
        })
    );
}

#[test]
fn remove_aliases_alias() {
    assert_eq!(
        QueryType::from(QueryBuilder::remove().aliases("a").query()),
        QueryType::RemoveAliases(RemoveAliasesQuery(vec!["a".to_string()]))
    );
}

#[test]
fn remove_aliases_aliases() {
    assert_eq!(
        QueryType::from(QueryBuilder::remove().aliases(["a", "b"]).query()),
        QueryType::RemoveAliases(RemoveAliasesQuery(vec!["a".to_string(), "b".to_string()]))
    );
}

#[test]
fn remove_ids_id() {
    assert_eq!(
        QueryType::from(QueryBuilder::remove().ids(1).query()),
        QueryType::Remove(RemoveQuery(QueryIds::Ids(vec![QueryId::Id(DbId(1))])))
    );
}

#[test]
fn remove_ids_alias() {
    assert_eq!(
        QueryType::from(QueryBuilder::remove().ids("a").query()),
        QueryType::Remove(RemoveQuery(QueryIds::Ids(vec![QueryId::Alias(
            "a".to_string()
        )])))
    );
}

#[test]
fn remove_ids_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::remove().ids([1, 2]).query()),
        QueryType::Remove(RemoveQuery(QueryIds::Ids(vec![
            QueryId::Id(DbId(1)),
            QueryId::Id(DbId(2))
        ])))
    );
}

#[test]
fn remove_ids_aliases() {
    assert_eq!(
        QueryType::from(QueryBuilder::remove().ids(["a", "b"]).query()),
        QueryType::Remove(RemoveQuery(QueryIds::Ids(vec![
            QueryId::Alias("a".to_string()),
            QueryId::Alias("b".to_string())
        ])))
    );
}

#[test]
fn remove_ids_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .ids(QueryBuilder::search().from("a").query())
                .query()
        ),
        QueryType::Remove(RemoveQuery(QueryIds::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Alias("a".to_string()),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })))
    );
}

#[test]
fn remove_search_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::remove().search().from("a").query()),
        QueryType::Remove(RemoveQuery(QueryIds::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Alias("a".to_string()),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })))
    );
}

#[test]
fn remove_index() {
    assert_eq!(
        QueryType::from(QueryBuilder::remove().index("key").query()),
        QueryType::RemoveIndex(RemoveIndexQuery(DbValue::String("key".to_string())))
    );
}

#[test]
fn remove_values_ids_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .values(["k1", "k2"])
                .ids([1, 2])
                .query()
        ),
        QueryType::RemoveValues(RemoveValuesQuery(SelectValuesQuery {
            keys: vec![
                DbValue::String("k1".to_string()),
                DbValue::String("k2".to_string())
            ],
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))])
        }))
    );
}

#[test]
fn remove_values_ids_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .values(["k1", "k2"])
                .ids(QueryBuilder::search().from("a").query())
                .query()
        ),
        QueryType::RemoveValues(RemoveValuesQuery(SelectValuesQuery {
            keys: vec![
                DbValue::String("k1".to_string()),
                DbValue::String("k2".to_string())
            ],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        }))
    );
}

#[test]
fn remove_values_search_from() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .values(["k1", "k2"])
                .search()
                .from("a")
                .query()
        ),
        QueryType::RemoveValues(RemoveValuesQuery(SelectValuesQuery {
            keys: vec![
                DbValue::String("k1".to_string()),
                DbValue::String("k2".to_string())
            ],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        }))
    );
}

#[test]
fn remove_amend_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .amend([[("k", 1).into()], [("k", 2).into()]])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::Remove
        })
    );
}

#[test]
fn remove_amend_uniform_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .amend_uniform([("k", 1).into()])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Single(vec![DbKeyValue {
                key: DbValue::String("k".to_string()),
                value: DbValue::I64(1)
            }]),
            amend: Amend::Remove
        })
    );
}

#[test]
fn remove_amend_uniform_search_from() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .amend_uniform([("k", 1).into()])
                .search()
                .from("a")
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            }),
            values: QueryValues::Single(vec![DbKeyValue {
                key: DbValue::String("k".to_string()),
                value: DbValue::I64(1)
            }]),
            amend: Amend::Remove
        })
    );
}

#[test]
fn remove_amend_or_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .amend_or([[("k", 1).into()], [("k", 2).into()]])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::RemoveBitwise(BitwiseOp::Or)
        })
    );
}

#[test]
fn remove_amend_or_uniform_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .amend_or_uniform([("k", 1).into()])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Single(vec![DbKeyValue {
                key: DbValue::String("k".to_string()),
                value: DbValue::I64(1)
            }]),
            amend: Amend::RemoveBitwise(BitwiseOp::Or)
        })
    );
}

#[test]
fn remove_amend_and_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .amend_and([[("k", 1).into()], [("k", 2).into()]])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::RemoveBitwise(BitwiseOp::And)
        })
    );
}

#[test]
fn remove_amend_and_uniform_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .amend_and_uniform([("k", 1).into()])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Single(vec![DbKeyValue {
                key: DbValue::String("k".to_string()),
                value: DbValue::I64(1)
            }]),
            amend: Amend::RemoveBitwise(BitwiseOp::And)
        })
    );
}

#[test]
fn remove_amend_xor_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .amend_xor([[("k", 1).into()], [("k", 2).into()]])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Multi(vec![
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(1)
                }],
                vec![DbKeyValue {
                    key: DbValue::String("k".to_string()),
                    value: DbValue::I64(2)
                }]
            ]),
            amend: Amend::RemoveBitwise(BitwiseOp::Xor)
        })
    );
}

#[test]
fn remove_amend_xor_uniform_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::remove()
                .amend_xor_uniform([("k", 1).into()])
                .ids([1, 2])
                .query()
        ),
        QueryType::InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            values: QueryValues::Single(vec![DbKeyValue {
                key: DbValue::String("k".to_string()),
                value: DbValue::I64(1)
            }]),
            amend: Amend::RemoveBitwise(BitwiseOp::Xor)
        })
    );
}

#[test]
fn select_aliases_ids_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().aliases().ids([1, 2]).query()),
        QueryType::SelectAliases(SelectAliasesQuery(QueryIds::Ids(vec![
            QueryId::Id(DbId(1)),
            QueryId::Id(DbId(2))
        ])))
    );
}

#[test]
fn select_aliases_ids_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .aliases()
                .ids(QueryBuilder::search().from(1).query())
                .query()
        ),
        QueryType::SelectAliases(SelectAliasesQuery(QueryIds::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })))
    );
}

#[test]
fn select_aliases_search_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().aliases().search().from(1).query()),
        QueryType::SelectAliases(SelectAliasesQuery(QueryIds::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })))
    );
}

#[test]
fn select_aliases() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().aliases().query()),
        QueryType::SelectAllAliases(SelectAllAliasesQuery {})
    );
}

#[test]
fn select_edge_count_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().edge_count().ids([1, 2]).query()),
        QueryType::SelectEdgeCount(SelectEdgeCountQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            from: true,
            to: true
        })
    );
}

#[test]
fn select_edge_count_from_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().edge_count_from().ids([1, 2]).query()),
        QueryType::SelectEdgeCount(SelectEdgeCountQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            from: true,
            to: false
        })
    );
}

#[test]
fn select_edge_count_to_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().edge_count_to().ids([1, 2]).query()),
        QueryType::SelectEdgeCount(SelectEdgeCountQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))]),
            from: false,
            to: true
        })
    );
}

#[test]
fn select_edge_count_search_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().edge_count().search().from(1).query()),
        QueryType::SelectEdgeCount(SelectEdgeCountQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(1)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            }),
            from: true,
            to: true
        })
    );
}

#[test]
fn select_ids_alias() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().ids("a").query()),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Ids(vec![QueryId::Alias("a".to_string())])
        })
    );
}

#[test]
fn select_ids_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().ids([1, 2]).query()),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))])
        })
    );
}

#[test]
fn select_ids_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .ids(QueryBuilder::search().from(1).query())
                .query()
        ),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(1)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        })
    );
}

#[test]
fn select_search_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().search().from(1).query()),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(1)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        })
    );
}

#[test]
fn select_search_to() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().search().to(1).query()),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(0)),
                destination: QueryId::Id(DbId(1)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        })
    );
}

#[test]
fn select_search_index_value() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .search()
                .index("age")
                .value(20)
                .query()
        ),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::Index,
                origin: QueryId::Id(DbId(0)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                }],
                reverse: false
            })
        })
    );
}

#[test]
fn select_search_from_limit() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().search().from("a").limit(10).query()),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 10,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        })
    );
}

#[test]
fn select_search_from_offset() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().search().from("a").offset(10).query()),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 10,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        })
    );
}

#[test]
fn select_search_from_order_by() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .search()
                .from("a")
                .order_by(DbKeyOrder::Desc("age".into()))
                .query()
        ),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![DbKeyOrder::Desc(DbValue::String("age".to_string()))],
                conditions: vec![],
                reverse: false
            })
        })
    );
}

#[test]
fn select_search_from_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .search()
                .from("a")
                .where_()
                .node()
                .query()
        ),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Alias("a".to_string()),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }],
                reverse: false
            })
        })
    );
}

#[test]
fn select_indexes() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().indexes().query()),
        QueryType::SelectIndexes(SelectIndexesQuery {})
    );
}

#[test]
fn select_keys_ids_alias() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().keys().ids("a").query()),
        QueryType::SelectKeys(SelectKeysQuery(QueryIds::Ids(vec![QueryId::Alias(
            "a".to_string()
        )])))
    );
}

#[test]
fn select_keys_ids_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().keys().ids([1, 2]).query()),
        QueryType::SelectKeys(SelectKeysQuery(QueryIds::Ids(vec![
            QueryId::Id(DbId(1)),
            QueryId::Id(DbId(2))
        ])))
    );
}

#[test]
fn select_keys_ids_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .keys()
                .ids(QueryBuilder::search().from(1).query())
                .query()
        ),
        QueryType::SelectKeys(SelectKeysQuery(QueryIds::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })))
    );
}

#[test]
fn select_keys_search_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().keys().search().from(1).query()),
        QueryType::SelectKeys(SelectKeysQuery(QueryIds::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })))
    );
}

#[test]
fn select_key_count_ids_alias() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().key_count().ids("a").query()),
        QueryType::SelectKeyCount(SelectKeyCountQuery(QueryIds::Ids(vec![QueryId::Alias(
            "a".to_string()
        )])))
    );
}

#[test]
fn select_key_count_ids_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().key_count().ids([1, 2]).query()),
        QueryType::SelectKeyCount(SelectKeyCountQuery(QueryIds::Ids(vec![
            QueryId::Id(DbId(1)),
            QueryId::Id(DbId(2))
        ])))
    );
}

#[test]
fn select_key_count_ids_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .key_count()
                .ids(QueryBuilder::search().from(1).query())
                .query()
        ),
        QueryType::SelectKeyCount(SelectKeyCountQuery(QueryIds::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })))
    );
}

#[test]
fn select_key_count_search_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().key_count().search().from(1).query()),
        QueryType::SelectKeyCount(SelectKeyCountQuery(QueryIds::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })))
    );
}

#[test]
fn select_node_count() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().node_count().query()),
        QueryType::SelectNodeCount(SelectNodeCountQuery {})
    );
}

#[test]
fn select_values_ids_alias() {
    assert_eq!(
        QueryType::from(QueryBuilder::select().values(["k", "k2"]).ids("a").query()),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![
                DbValue::String("k".to_string()),
                DbValue::String("k2".to_string())
            ],
            ids: QueryIds::Ids(vec![QueryId::Alias("a".to_string())])
        })
    );
}

#[test]
fn select_values_ids_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .values(["k", "k2"])
                .ids([1, 2])
                .query()
        ),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![
                DbValue::String("k".to_string()),
                DbValue::String("k2".to_string())
            ],
            ids: QueryIds::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))])
        })
    );
}

#[test]
fn select_values_ids_search() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .values(["k", "k2"])
                .ids(QueryBuilder::search().from(1).query())
                .query()
        ),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![
                DbValue::String("k".to_string()),
                DbValue::String("k2".to_string())
            ],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(1)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        })
    );
}

#[test]
fn select_values_search_from() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::select()
                .values(["k", "k2"])
                .search()
                .from(1)
                .query()
        ),
        QueryType::SelectValues(SelectValuesQuery {
            keys: vec![
                DbValue::String("k".to_string()),
                DbValue::String("k2".to_string())
            ],
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(1)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false
            })
        })
    );
}

#[test]
fn search_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().from("a").query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Alias("a".to_string()),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_to() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().to(1).query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(1)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_from_to() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().from("a").to("b").query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Alias("a".to_string()),
            destination: QueryId::Alias("b".to_string()),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_to_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().to("a").from("b").query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Alias("b".to_string()),
            destination: QueryId::Alias("a".to_string()),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: true
        })
    );
}

#[test]
fn search_breadth_first_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().breadth_first().from("a").query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Alias("a".to_string()),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_depth_first_to() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().depth_first().to(1).query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::DepthFirst,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(1)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_depth_first_from() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().depth_first().from("a").query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::DepthFirst,
            origin: QueryId::Alias("a".to_string()),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_elements() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().elements().query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Elements,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_index_value() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().index("age").value(20).query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("age".to_string()),
                    value: Comparison::Equal(DbValue::I64(20))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_limit() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .limit(5)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 5,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("age".to_string()),
                    value: Comparison::Equal(DbValue::I64(20))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_offset() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .offset(5)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 5,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("age".to_string()),
                    value: Comparison::Equal(DbValue::I64(20))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_offset_limit() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .offset(5)
                .limit(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 1,
            offset: 5,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("age".to_string()),
                    value: Comparison::Equal(DbValue::I64(20))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_offset_limit_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .offset(5)
                .limit(1)
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 1,
            offset: 5,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_limit_where_edge() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .limit(1)
                .where_()
                .edge()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 1,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Edge
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_offset_where_keys() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .offset(5)
                .where_()
                .keys(["name", "age"])
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 5,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Keys(vec![
                        DbValue::String("name".to_string()),
                        DbValue::String("age".to_string())
                    ])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .ids([1, 2])
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_not_keys() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .not()
                .keys("name")
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::Not,
                    data: QueryConditionData::Keys(vec![DbValue::String("name".to_string())])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_greater_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .greater_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::GreaterThan(DbValue::I64(1))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_greater_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .greater_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::GreaterThanOrEqual(DbValue::I64(1))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_less_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .less_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::LessThan(DbValue::I64(1))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_less_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .less_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::LessThanOrEqual(DbValue::I64(1))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_not_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .not_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::NotEqual(DbValue::I64(1))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_contains() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .contains(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::Contains(DbValue::I64(1))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_starts_with() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .starts_with(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::StartsWith(DbValue::I64(1))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_ends_with() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .ends_with(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::EndsWith(DbValue::I64(1))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_any() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .any(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::Any(DbValue::I64(1))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCount(CountComparison::Equal(2))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_from() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_from(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountFrom(CountComparison::Equal(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_to() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_to(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountTo(CountComparison::Equal(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_greater_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_greater_than(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCount(CountComparison::GreaterThan(2))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_greater_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_greater_than_or_equal(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCount(CountComparison::GreaterThanOrEqual(2))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_less_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_less_than(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCount(CountComparison::LessThan(2))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_less_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_less_than_or_equal(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCount(CountComparison::LessThanOrEqual(2))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_not_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_not_equal(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCount(CountComparison::NotEqual(2))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_from_greater_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_from_greater_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountFrom(CountComparison::GreaterThan(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_from_greater_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_from_greater_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountFrom(CountComparison::GreaterThanOrEqual(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_from_less_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_from_less_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountFrom(CountComparison::LessThan(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_from_less_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_from_less_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountFrom(CountComparison::LessThanOrEqual(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_from_not_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_from_not_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountFrom(CountComparison::NotEqual(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_to_greater_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_to_greater_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountTo(CountComparison::GreaterThan(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_to_greater_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_to_greater_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountTo(CountComparison::GreaterThanOrEqual(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_to_less_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_to_less_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountTo(CountComparison::LessThan(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_to_less_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_to_less_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountTo(CountComparison::LessThanOrEqual(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge_count_to_not_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge_count_to_not_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::EdgeCountTo(CountComparison::NotEqual(1))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_edge() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .edge()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Edge
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_node_and_where_key_value_or_key_value_end_where() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .node()
                .and()
                .where_()
                .key("k")
                .value(1)
                .or()
                .key("k")
                .value(2)
                .end_where()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Where(vec![
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::KeyValue(KeyValueComparison {
                                key: DbValue::String("k".to_string()),
                                value: Comparison::Equal(DbValue::I64(1))
                            })
                        },
                        QueryCondition {
                            logic: QueryConditionLogic::Or,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::KeyValue(KeyValueComparison {
                                key: DbValue::String("k".to_string()),
                                value: Comparison::Equal(DbValue::I64(2))
                            })
                        }
                    ])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_where_key_regex() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .where_()
                .key("k")
                .regex("pattern")
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("k".to_string()),
                        value: Comparison::Regex(DbValue::String("pattern".to_string()))
                    })
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_order_by() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .order_by([DbKeyOrder::Asc("k".into())])
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![DbKeyOrder::Asc(DbValue::String("k".to_string()))],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("age".to_string()),
                    value: Comparison::Equal(DbValue::I64(20))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_order_by_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .order_by([DbKeyOrder::Asc("k".into())])
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![DbKeyOrder::Asc(DbValue::String("k".to_string()))],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_order_by_offset_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .order_by([DbKeyOrder::Asc("k".into())])
                .offset(5)
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 5,
            order_by: vec![DbKeyOrder::Asc(DbValue::String("k".to_string()))],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_order_by_limit_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .order_by([DbKeyOrder::Asc("k".into())])
                .limit(5)
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 5,
            offset: 0,
            order_by: vec![DbKeyOrder::Asc(DbValue::String("k".to_string()))],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_index_value_order_by_offset_limit_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .index("age")
                .value(20)
                .order_by([DbKeyOrder::Asc("k".into())])
                .offset(5)
                .limit(5)
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::Index,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 5,
            offset: 5,
            order_by: vec![DbKeyOrder::Asc(DbValue::String("k".to_string()))],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::KeyValue(KeyValueComparison {
                        key: DbValue::String("age".to_string()),
                        value: Comparison::Equal(DbValue::I64(20))
                    })
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_from_order_by() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .order_by([
                    DbKeyOrder::Desc("age".into()),
                    DbKeyOrder::Asc("name".into())
                ])
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![
                DbKeyOrder::Desc(DbValue::String("age".to_string())),
                DbKeyOrder::Asc(DbValue::String("name".to_string()))
            ],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_from_offset() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().from(1).offset(10).query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 10,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_from_limit() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().from(1).limit(5).query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 5,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_from_order_by_offset() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .order_by([DbKeyOrder::Desc("k".into())])
                .offset(10)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 10,
            order_by: vec![DbKeyOrder::Desc(DbValue::String("k".to_string()))],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_from_order_by_limit() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .order_by([DbKeyOrder::Desc("k".into())])
                .limit(5)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 5,
            offset: 0,
            order_by: vec![DbKeyOrder::Desc(DbValue::String("k".to_string()))],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_from_order_by_offset_limit() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .order_by([DbKeyOrder::Desc("k".into())])
                .offset(10)
                .limit(5)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 5,
            offset: 10,
            order_by: vec![DbKeyOrder::Desc(DbValue::String("k".to_string()))],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_from_offset_limit() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().from(1).offset(10).limit(5).query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 5,
            offset: 10,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_distance() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .distance(CountComparison::LessThan(3))
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Distance(CountComparison::LessThan(3))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_neighbor() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().from(1).where_().neighbor().query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Distance(CountComparison::Equal(2))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().from(1).where_().edge().query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Edge
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count(CountComparison::GreaterThan(2))
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCount(CountComparison::GreaterThan(2))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_from() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_from(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountFrom(CountComparison::Equal(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_to() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_to(CountComparison::NotEqual(1))
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountTo(CountComparison::NotEqual(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_node() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().from(1).where_().node().query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Node
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_value() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .value(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::Equal(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_keys() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .keys(["k1", "k2"])
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Keys(vec![
                    DbValue::String("k1".to_string()),
                    DbValue::String("k2".to_string())
                ])
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_not_keys() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .not()
                .keys(["k1", "k2"])
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::Not,
                data: QueryConditionData::Keys(vec![
                    DbValue::String("k1".to_string()),
                    DbValue::String("k2".to_string())
                ])
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_ids() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().from(1).where_().ids([1, 2]).query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))])
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_beyond_keys() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .beyond()
                .keys(["k"])
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::Beyond,
                data: QueryConditionData::Keys(vec![DbValue::String("k".to_string())])
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_not_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .not()
                .ids([1, 2])
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::Not,
                data: QueryConditionData::Ids(vec![QueryId::Id(DbId(1)), QueryId::Id(DbId(2))])
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_not_beyond_ids() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .not_beyond()
                .ids("a")
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::NotBeyond,
                data: QueryConditionData::Ids(vec![QueryId::Alias("a".to_string())])
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_node_or_edge() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .node()
                .or()
                .edge()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                },
                QueryCondition {
                    logic: QueryConditionLogic::Or,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Edge
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_node_and_distance() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .node()
                .and()
                .distance(CountComparison::GreaterThanOrEqual(3))
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                },
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Distance(CountComparison::GreaterThanOrEqual(3))
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_node_or_where_edge_and_key_value_end_where_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .node()
                .or()
                .where_()
                .edge()
                .and()
                .key("k")
                .value(1)
                .end_where()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                },
                QueryCondition {
                    logic: QueryConditionLogic::Or,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Where(vec![
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::Edge
                        },
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::KeyValue(KeyValueComparison {
                                key: DbValue::String("k".to_string()),
                                value: Comparison::Equal(DbValue::I64(1))
                            })
                        }
                    ])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_node_or_where_edge_and_key_value_end_where_contains() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .node()
                .or()
                .where_()
                .edge()
                .and()
                .key("k")
                .value(Comparison::Contains(1.into()))
                .end_where()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                },
                QueryCondition {
                    logic: QueryConditionLogic::Or,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Where(vec![
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::Edge
                        },
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::KeyValue(KeyValueComparison {
                                key: DbValue::String("k".to_string()),
                                value: Comparison::Contains(DbValue::I64(1))
                            })
                        }
                    ])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_node_or_where_edge_and_key_value_end_where_contains_list() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .node()
                .or()
                .where_()
                .edge()
                .and()
                .key("k")
                .value(Comparison::Contains(([1, 2]).into()))
                .end_where()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                },
                QueryCondition {
                    logic: QueryConditionLogic::Or,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Where(vec![
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::Edge
                        },
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::KeyValue(KeyValueComparison {
                                key: DbValue::String("k".to_string()),
                                value: Comparison::Contains(DbValue::VecI64(vec![1, 2]))
                            })
                        }
                    ])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_node_or_where_edge_and_key_value_end_where_starts_with() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .node()
                .or()
                .where_()
                .edge()
                .and()
                .key("k")
                .value(Comparison::StartsWith(1.into()))
                .end_where()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                },
                QueryCondition {
                    logic: QueryConditionLogic::Or,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Where(vec![
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::Edge
                        },
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::KeyValue(KeyValueComparison {
                                key: DbValue::String("k".to_string()),
                                value: Comparison::StartsWith(DbValue::I64(1))
                            })
                        }
                    ])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_node_or_where_edge_and_key_value_end_where_ends_with_list() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .node()
                .or()
                .where_()
                .edge()
                .and()
                .key("k")
                .value(Comparison::EndsWith(([1, 2]).into()))
                .end_where()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                },
                QueryCondition {
                    logic: QueryConditionLogic::Or,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Where(vec![
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::Edge
                        },
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::KeyValue(KeyValueComparison {
                                key: DbValue::String("k".to_string()),
                                value: Comparison::EndsWith(DbValue::VecI64(vec![1, 2]))
                            })
                        }
                    ])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_node_or_where_edge_and_key_value_end_where_any_list() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .node()
                .or()
                .where_()
                .edge()
                .and()
                .key("k")
                .value(Comparison::Any(([1, 2]).into()))
                .end_where()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![
                QueryCondition {
                    logic: QueryConditionLogic::And,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Node
                },
                QueryCondition {
                    logic: QueryConditionLogic::Or,
                    modifier: QueryConditionModifier::None,
                    data: QueryConditionData::Where(vec![
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::Edge
                        },
                        QueryCondition {
                            logic: QueryConditionLogic::And,
                            modifier: QueryConditionModifier::None,
                            data: QueryConditionData::KeyValue(KeyValueComparison {
                                key: DbValue::String("k".to_string()),
                                value: Comparison::Any(DbValue::VecI64(vec![1, 2]))
                            })
                        }
                    ])
                }
            ],
            reverse: false
        })
    );
}

#[test]
fn search_from_order_by_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .order_by([DbKeyOrder::Asc("k".into())])
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![DbKeyOrder::Asc(DbValue::String("k".to_string()))],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Node
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_limit_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .limit(1)
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 1,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Node
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_offset_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .offset(1)
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 1,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Node
            }],
            reverse: false
        })
    );
}

#[test]
fn search_to_offset() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().to(1).offset(1).query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(1)),
            limit: 0,
            offset: 1,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_to_limit() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().to(1).limit(1).query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(1)),
            limit: 1,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
            reverse: false
        })
    );
}

#[test]
fn search_to_where_node() {
    assert_eq!(
        QueryType::from(QueryBuilder::search().to(1).where_().node().query()),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(1)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Node
            }],
            reverse: false
        })
    );
}

#[test]
fn search_to_order_by_where_node() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .to(1)
                .order_by([DbKeyOrder::Asc("k".into())])
                .where_()
                .node()
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(1)),
            limit: 0,
            offset: 0,
            order_by: vec![DbKeyOrder::Asc(DbValue::String("k".to_string()))],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Node
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_greater_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .greater_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::GreaterThan(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_greater_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .greater_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::GreaterThanOrEqual(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_less_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .less_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::LessThan(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_less_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .less_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::LessThanOrEqual(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_not_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .not_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::NotEqual(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_contains() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .contains(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::Contains(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_starts_with() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .starts_with(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::StartsWith(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_ends_with() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .ends_with(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::EndsWith(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_any() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .any(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::Any(DbValue::I64(1))
                })
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_distance_greater_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .distance_greater_than(3)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Distance(CountComparison::GreaterThan(3))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_distance_greater_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .distance_greater_than_or_equal(3)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Distance(CountComparison::GreaterThanOrEqual(3))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_distance_less_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .distance_less_than(3)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Distance(CountComparison::LessThan(3))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_distance_less_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .distance_less_than_or_equal(3)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Distance(CountComparison::LessThanOrEqual(3))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_distance_not_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .distance_not_equal(3)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Distance(CountComparison::NotEqual(3))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_greater_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_greater_than(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCount(CountComparison::GreaterThan(2))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_greater_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_greater_than_or_equal(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCount(CountComparison::GreaterThanOrEqual(2))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_less_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_less_than(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCount(CountComparison::LessThan(2))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_less_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_less_than_or_equal(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCount(CountComparison::LessThanOrEqual(2))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_not_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_not_equal(2)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCount(CountComparison::NotEqual(2))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_from_greater_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_from_greater_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountFrom(CountComparison::GreaterThan(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_from_greater_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_from_greater_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountFrom(CountComparison::GreaterThanOrEqual(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_from_less_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_from_less_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountFrom(CountComparison::LessThan(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_from_less_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_from_less_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountFrom(CountComparison::LessThanOrEqual(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_from_not_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_from_not_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountFrom(CountComparison::NotEqual(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_to_greater_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_to_greater_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountTo(CountComparison::GreaterThan(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_to_greater_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_to_greater_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountTo(CountComparison::GreaterThanOrEqual(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_to_less_than() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_to_less_than(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountTo(CountComparison::LessThan(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_to_less_than_or_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_to_less_than_or_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountTo(CountComparison::LessThanOrEqual(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_edge_count_to_not_equal() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .edge_count_to_not_equal(1)
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::EdgeCountTo(CountComparison::NotEqual(1))
            }],
            reverse: false
        })
    );
}

#[test]
fn search_from_where_key_regex() {
    assert_eq!(
        QueryType::from(
            QueryBuilder::search()
                .from(1)
                .where_()
                .key("k")
                .regex("pattern")
                .query()
        ),
        QueryType::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(1)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DbValue::String("k".to_string()),
                    value: Comparison::Regex(DbValue::String("pattern".to_string()))
                })
            }],
            reverse: false
        })
    );
}
