use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbKeyOrder;
use agdb::DbType;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::QueryType;
use agdb::type_def::TypeDefinition;

#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct QueryCase {
    pub id: String,
    pub query: QueryType,
}

#[derive(Default, DbType)]
struct T {
    db_id: Option<QueryId>,
    value1: String,
    value2: i64,
}

fn case(id: &str, query: QueryType) -> QueryCase {
    QueryCase {
        id: id.to_string(),
        query,
    }
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub fn query_cases() -> Vec<QueryCase> {
    vec![
        case("q001", (QueryBuilder::insert().aliases("a").ids(1).query()).into()),
        case("q002", (QueryBuilder::insert().aliases("a").ids("b").query()).into()),
        case("q003", (QueryBuilder::insert().aliases(["a", "b"]).ids([1, 2]).query()).into()),
        case("q004", (QueryBuilder::insert().edges().from(1).to(2).query()).into()),
        case("q005", (QueryBuilder::insert().edges().from("a").to("b").query()).into()),
        case("q006", (QueryBuilder::insert().edges().from("a").to([1, 2]).query()).into()),
        case("q007", (QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).query()).into()),
        case("q008", (QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).each().query()).into()),
        case("q009", (QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).each().values([[("k", 1).into()], [("k", 2).into()]]).query()).into()),
        case("q010", (QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).each().values_uniform([("k", 1).into(), (1, 10).into()]).query()).into()),
        case("q011", (QueryBuilder::insert().edges().from("a").to([1, 2]).values([[("k", 1).into()], [("k", 2).into()]]).query()).into()),
        case("q012", (QueryBuilder::insert().edges().from("a").to([1, 2]).values_uniform([("k", "v").into(), (1, 10).into()]).query()).into()),
        case("q013", (QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).query()).into()),
        case("q014", (QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values([[("k", 1).into()],[("k", 2).into()]]).query()).into()),
        case("q015", (QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values_uniform([("k", "v").into(), (1, 10).into()]).query()).into()),
        case("q016", (QueryBuilder::insert().edges().ids(-3).from(1).to(2).query()).into()),
        case("q017", (QueryBuilder::insert().edges().ids([-3, -4]).from(1).to(2).query()).into()),
        case("q018", (QueryBuilder::insert().edges().ids(QueryBuilder::search().from(1).where_().edge().query()).from(1).to(2).query()).into()),
        case("q019", (QueryBuilder::insert().index("key").query()).into()),
        case("q020", (QueryBuilder::insert().nodes().count(2).query()).into()),
        case("q021", (QueryBuilder::insert().nodes().count(2).values_uniform([("k", "v").into(), (1, 10).into()]).query()).into()),
        case("q022", (QueryBuilder::insert().nodes().aliases(["a", "b"]).query()).into()),
        case("q023", (QueryBuilder::insert().nodes().aliases(["a", "b"]).values([[("k", 1).into()], [("k", 2).into()]]).query()).into()),
        case("q024", (QueryBuilder::insert().nodes().aliases(["a", "b"]).values_uniform([("k", "v").into(), (1, 10).into()]).query()).into()),
        case("q025", (QueryBuilder::insert().nodes().values([[("k", 1).into()], [("k", 2).into()]]).query()).into()),
        case("q026", (QueryBuilder::insert().nodes().ids(1).count(1).query()).into()),
        case("q027", (QueryBuilder::insert().nodes().ids([1, 2]).count(1).query()).into()),
        case("q028", (QueryBuilder::insert().nodes().ids("a").count(1).query()).into()),
        case("q029", (QueryBuilder::insert().nodes().ids("a").aliases("a").query()).into()),
        case("q030", (QueryBuilder::insert().nodes().ids(["a", "b"]).count(1).query()).into()),
        case("q031", (QueryBuilder::insert().nodes().ids([1, 2]).values([[("k", "v").into()], [(1, 10).into()]]).query()).into()),
        case("q032", (QueryBuilder::insert().nodes().ids([1, 2]).values_uniform([("k", "v").into(), (1, 10).into()]).query()).into()),
        case("q033", (QueryBuilder::insert().nodes().ids(QueryBuilder::search().from(1).query()).count(1).query()).into()),
        case("q034", (QueryBuilder::insert().element(T::default()).query()).into()),
        case("q035", (QueryBuilder::insert().elements(&[T::default(), T::default()]).query()).into()),
        case("q036", (QueryBuilder::insert().values([vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids([1, 2]).query()).into()),
        case("q037", (QueryBuilder::insert().values([vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids(QueryBuilder::search().from("a").query()).query()).into()),
        case("q038", (QueryBuilder::insert().values([vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).search().from("a").query()).into()),
        case("q039", (QueryBuilder::insert().values_uniform([("k", "v").into(), (1, 10).into()]).ids([1, 2]).query()).into()),
        case("q040", (QueryBuilder::insert().values_uniform([("k", "v").into(), (1, 10).into()]).ids(QueryBuilder::search().from("a").query()).query()).into()),
        case("q041", (QueryBuilder::insert().values_uniform([("k", "v").into(), (1, 10).into()]).search().from("a").query()).into()),
        case("q042", (QueryBuilder::remove().aliases("a").query()).into()),
        case("q043", (QueryBuilder::remove().aliases(["a", "b"]).query()).into()),
        case("q044", (QueryBuilder::remove().ids(1).query()).into()),
        case("q045", (QueryBuilder::remove().ids("a").query()).into()),
        case("q046", (QueryBuilder::remove().ids([1, 2]).query()).into()),
        case("q047", (QueryBuilder::remove().ids(["a", "b"]).query()).into()),
        case("q048", (QueryBuilder::remove().ids(QueryBuilder::search().from("a").query()).query()).into()),
        case("q049", (QueryBuilder::remove().search().from("a").query()).into()),
        case("q050", (QueryBuilder::remove().index("key").query()).into()),
        case("q051", (QueryBuilder::remove().values(["k1", "k2"]).ids([1, 2]).query()).into()),
        case("q052", (QueryBuilder::remove().values(["k1", "k2"]).ids(QueryBuilder::search().from("a").query()).query()).into()),
        case("q053", (QueryBuilder::remove().values(["k1", "k2"]).search().from("a").query()).into()),
        case("q054", (QueryBuilder::select().aliases().ids([1, 2]).query()).into()),
        case("q055", (QueryBuilder::select().aliases().ids(QueryBuilder::search().from(1).query()).query()).into()),
        case("q056", (QueryBuilder::select().aliases().search().from(1).query()).into()),
        case("q057", (QueryBuilder::select().aliases().query()).into()),
        case("q058", (QueryBuilder::select().edge_count().ids([1, 2]).query()).into()),
        case("q059", (QueryBuilder::select().edge_count_from().ids([1, 2]).query()).into()),
        case("q060", (QueryBuilder::select().edge_count_to().ids([1, 2]).query()).into()),
        case("q061", (QueryBuilder::select().edge_count().search().from(1).query()).into()),
        case("q062", (QueryBuilder::select().ids("a").query()).into()),
        case("q063", (QueryBuilder::select().ids([1, 2]).query()).into()),
        case("q064", (QueryBuilder::select().ids(QueryBuilder::search().from(1).query()).query()).into()),
        case("q065", (QueryBuilder::select().search().from(1).query()).into()),
        case("q066", (QueryBuilder::select().search().to(1).query()).into()),
        case("q067", (QueryBuilder::select().search().index("age").value(20).query()).into()),
        case("q068", (QueryBuilder::select().search().from("a").limit(10).query()).into()),
        case("q069", (QueryBuilder::select().search().from("a").offset(10).query()).into()),
        case("q070", (QueryBuilder::select().search().from("a").order_by(DbKeyOrder::Desc("age".into())).query()).into()),
        case("q071", (QueryBuilder::select().search().from("a").where_().node().query()).into()),
        case("q072", (QueryBuilder::select().indexes().query()).into()),
        case("q073", (QueryBuilder::select().keys().ids("a").query()).into()),
        case("q074", (QueryBuilder::select().keys().ids([1, 2]).query()).into()),
        case("q075", (QueryBuilder::select().keys().ids(QueryBuilder::search().from(1).query()).query()).into()),
        case("q076", (QueryBuilder::select().keys().search().from(1).query()).into()),
        case("q077", (QueryBuilder::select().key_count().ids("a").query()).into()),
        case("q078", (QueryBuilder::select().key_count().ids([1, 2]).query()).into()),
        case("q079", (QueryBuilder::select().key_count().ids(QueryBuilder::search().from(1).query()).query()).into()),
        case("q080", (QueryBuilder::select().key_count().search().from(1).query()).into()),
        case("q081", (QueryBuilder::select().node_count().query()).into()),
        case("q082", (QueryBuilder::select().values(["k", "k2"]).ids("a").query()).into()),
        case("q083", (QueryBuilder::select().values(["k", "k2"]).ids([1, 2]).query()).into()),
        case("q084", (QueryBuilder::select().values(["k", "k2"]).ids(QueryBuilder::search().from(1).query()).query()).into()),
        case("q085", (QueryBuilder::select().values(["k", "k2"]).search().from(1).query()).into()),
        case("q086", (QueryBuilder::search().from("a").query()).into()),
        case("q087", (QueryBuilder::search().to(1).query()).into()),
        case("q088", (QueryBuilder::search().from("a").to("b").query()).into()),
        case("q089", (QueryBuilder::search().breadth_first().from("a").query()).into()),
        case("q090", (QueryBuilder::search().depth_first().to(1).query()).into()),
        case("q091", (QueryBuilder::search().depth_first().from("a").query()).into()),
        case("q092", (QueryBuilder::search().elements().query()).into()),
        case("q093", (QueryBuilder::search().index("age").value(20).query()).into()),
        case("q094", (QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("age".into()), DbKeyOrder::Asc("name".into())]).query()).into()),
        case("q095", (QueryBuilder::search().from(1).offset(10).query()).into()),
        case("q096", (QueryBuilder::search().from(1).limit(5).query()).into()),
        case("q097", (QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).offset(10).query()).into()),
        case("q098", (QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).limit(5).query()).into()),
        case("q099", (QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).offset(10).limit(5).query()).into()),
        case("q100", (QueryBuilder::search().from(1).offset(10).limit(5).query()).into()),
        case("q101", (QueryBuilder::search().from(1).where_().distance(CountComparison::LessThan(3)).query()).into()),
        case("q102", (QueryBuilder::search().from(1).where_().neighbor().query()).into()),
        case("q103", (QueryBuilder::search().from(1).where_().edge().query()).into()),
        case("q104", (QueryBuilder::search().from(1).where_().edge_count(CountComparison::GreaterThan(2)).query()).into()),
        case("q105", (QueryBuilder::search().from(1).where_().edge_count_from(1).query()).into()),
        case("q106", (QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::NotEqual(1)).query()).into()),
        case("q107", (QueryBuilder::search().from(1).where_().node().query()).into()),
        case("q108", (QueryBuilder::search().from(1).where_().key("k").value(1).query()).into()),
        case("q109", (QueryBuilder::search().from(1).where_().keys(["k1", "k2"]).query()).into()),
        case("q110", (QueryBuilder::search().from(1).where_().not().keys(["k1", "k2"]).query()).into()),
        case("q111", (QueryBuilder::search().from(1).where_().ids([1, 2]).query()).into()),
        case("q112", (QueryBuilder::search().from(1).where_().beyond().keys(["k"]).query()).into()),
        case("q113", (QueryBuilder::search().from(1).where_().not().ids([1, 2]).query()).into()),
        case("q114", (QueryBuilder::search().from(1).where_().not_beyond().ids("a").query()).into()),
        case("q115", (QueryBuilder::search().from(1).where_().node().or().edge().query()).into()),
        case("q116", (QueryBuilder::search().from(1).where_().node().and().distance(CountComparison::GreaterThanOrEqual(3)).query()).into()),
        case("q117", (QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(1).end_where().query()).into()),
        case("q118", (QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(1.into())).end_where().query()).into()),
        case("q119", (QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(([1, 2]).into())).end_where().query()).into()),
        case("q120", (QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::StartsWith(1.into())).end_where().query()).into()),
        case("q121", (QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::EndsWith(([1, 2]).into())).end_where().query()).into()),
        case("q122", (QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).where_().node().query()).into()),
        case("q123", (QueryBuilder::search().from(1).limit(1).where_().node().query()).into()),
        case("q124", (QueryBuilder::search().from(1).offset(1).where_().node().query()).into()),
        case("q125", (QueryBuilder::search().to(1).offset(1).query()).into()),
        case("q126", (QueryBuilder::search().to(1).limit(1).query()).into()),
        case("q127", (QueryBuilder::search().to(1).where_().node().query()).into()),
        case("q128", (QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).where_().node().query()).into()),
    ]
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub fn assert_query_cases(expected: Vec<QueryCase>) {
    assert_eq!(query_cases(), expected);
}

#[test]
fn rust_query_cases_are_stable() {
    let cases = query_cases();
    assert!(!cases.is_empty());
    assert_query_cases(cases.clone());

    let ids: std::collections::HashSet<_> = cases.iter().map(|q| q.id.as_str()).collect();
    assert_eq!(ids.len(), cases.len());
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        QueryCase::type_def(),
        __query_cases_type_def(),
        __assert_query_cases_type_def(),
    ]
}
