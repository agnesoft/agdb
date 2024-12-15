use crate::routes;
use utoipa::openapi::security::Http;
use utoipa::openapi::security::HttpAuthScheme;
use utoipa::openapi::security::SecurityScheme;
use utoipa::Modify;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "http://localhost:3000", description = "Local server"),
    ),
    paths(
        routes::status,
        routes::admin::db::add,
        routes::admin::db::audit,
        routes::admin::db::backup,
        routes::admin::db::clear,
        routes::admin::db::convert,
        routes::admin::db::copy,
        routes::admin::db::delete,
        routes::admin::db::exec,
        routes::admin::db::list,
        routes::admin::db::optimize,
        routes::admin::db::rename,
        routes::admin::db::remove,
        routes::admin::db::restore,
        routes::admin::db::user::add,
        routes::admin::db::user::list,
        routes::admin::db::user::remove,
        routes::admin::user::change_password,
        routes::admin::user::add,
        routes::admin::user::list,
        routes::admin::user::logout,
        routes::admin::user::remove,
        routes::admin::shutdown,
        routes::admin::status,
        routes::user::login,
        routes::user::logout,
        routes::user::change_password,
        routes::user::status,
        routes::db::add,
        routes::db::audit,
        routes::db::backup,
        routes::db::clear,
        routes::db::convert,
        routes::db::copy,
        routes::db::delete,
        routes::db::exec,
        routes::db::list,
        routes::db::optimize,
        routes::db::remove,
        routes::db::rename,
        routes::db::restore,
        routes::db::user::add,
        routes::db::user::list,
        routes::db::user::remove,
        routes::cluster::login,
        routes::cluster::logout,
        routes::cluster::status,
    ),
    components(schemas(
        routes::db::DbTypeParam,
        routes::db::ServerDatabaseRename,
        routes::db::ServerDatabaseResource,
        routes::db::user::DbUserRoleParam,
        agdb_api::AdminStatus,
        agdb_api::DbAudit,
        agdb_api::DbType,
        agdb_api::DbUser,
        agdb_api::DbUserRole,
        agdb_api::DbResource,
        agdb_api::ChangePassword,
        agdb_api::ClusterStatus,
        agdb_api::Queries,
        agdb_api::QueriesResults,
        agdb_api::QueryAudit,
        agdb_api::ServerDatabase,
        agdb_api::UserCredentials,
        agdb_api::UserLogin,
        agdb_api::UserStatus,
        agdb::QueryResult,
        agdb::DbElement,
        agdb::DbId,
        agdb::DbKeyValue,
        agdb::DbKeyOrder,
        agdb::DbValue,
        agdb::DbF64,
        agdb::QueryType,
        agdb::InsertAliasesQuery,
        agdb::InsertEdgesQuery,
        agdb::InsertIndexQuery,
        agdb::InsertNodesQuery,
        agdb::InsertValuesQuery,
        agdb::Comparison,
        agdb::CountComparison,
        agdb::QueryCondition,
        agdb::QueryConditionData,
        agdb::QueryConditionLogic,
        agdb::QueryConditionModifier,
        agdb::QueryId,
        agdb::QueryIds,
        agdb::QueryResult,
        agdb::QueryValues,
        agdb::RemoveAliasesQuery,
        agdb::RemoveIndexQuery,
        agdb::RemoveQuery,
        agdb::RemoveValuesQuery,
        agdb::SearchQuery,
        agdb::SearchQueryAlgorithm,
        agdb::SelectAliasesQuery,
        agdb::SelectAllAliasesQuery,
        agdb::SelectEdgeCountQuery,
        agdb::SelectIndexesQuery,
        agdb::SelectKeyCountQuery,
        agdb::SelectKeysQuery,
        agdb::SelectNodeCountQuery,
        agdb::SelectValuesQuery,
    )),
    modifiers(&BearerToken),
)]
pub(crate) struct Api;

pub(crate) struct BearerToken;

impl Modify for BearerToken {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "Token",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::Comparison;
    use agdb::CountComparison;
    use agdb::DbKeyOrder;
    use agdb::QueryBuilder;
    use agdb::QueryId;
    use agdb::QueryType;
    use agdb::UserValue;
    use std::fs::File;
    use std::io::Write;

    macro_rules! queries {
        ($($x:expr),+ $(,)?) => {
            {
                let mut vec: Vec<(String, QueryType)> = Vec::new();
                $(
                    {
                        let mut as_string = stringify!($x).to_string();
                        as_string.retain(|c| !c.is_whitespace());
                        vec.push((as_string, $x.into()));
                    }
                )*
                vec
            }
        };
    }

    #[derive(Default, UserValue)]
    struct T {
        db_id: Option<QueryId>,
        value1: String,
        value2: i64,
    }

    #[test]
    fn openapi() {
        let schema = Api::openapi().to_pretty_json().unwrap();
        let mut file = File::create("openapi.json").unwrap();
        file.write_all(schema.as_bytes()).unwrap();
    }

    #[test]
    fn test_queries() {
        #[rustfmt::skip]
        let queries = queries![
QueryBuilder::insert().aliases("a").ids(1).query(),
QueryBuilder::insert().aliases("a").ids("b").query(),
QueryBuilder::insert().aliases(["a", "b"]).ids([1, 2]).query(),
QueryBuilder::insert().edges().from(1).to(2).query(),
QueryBuilder::insert().edges().from("a").to("b").query(),
QueryBuilder::insert().edges().from("a").to([1, 2]).query(),
QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).query(),
QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).each().query(),
QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).each().values([[("k", 1).into()], [("k", 2).into()]]).query(),
QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).each().values_uniform([("k", 1).into(), (1, 10).into()]).query(),
QueryBuilder::insert().edges().from("a").to([1, 2]).values([[("k", 1).into()], [("k", 2).into()]]).query(),
QueryBuilder::insert().edges().from("a").to([1, 2]).values_uniform([("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).query(),
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values([[("k", 1).into()],[("k", 2).into()]]).query(),
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values_uniform([("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().edges().ids(-3).from(1).to(2).query(),
QueryBuilder::insert().edges().ids([-3, -4]).from(1).to(2).query(),
QueryBuilder::insert().edges().ids(QueryBuilder::search().from(1).where_().edge().query()).from(1).to(2).query(),
QueryBuilder::insert().index("key").query(),
QueryBuilder::insert().nodes().count(2).query(),
QueryBuilder::insert().nodes().count(2).values_uniform([("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().nodes().aliases(["a", "b"]).query(),
QueryBuilder::insert().nodes().aliases(["a", "b"]).values([[("k", 1).into()], [("k", 2).into()]]).query(),
QueryBuilder::insert().nodes().aliases(["a", "b"]).values_uniform([("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().nodes().values([[("k", 1).into()], [("k", 2).into()]]).query(),
QueryBuilder::insert().nodes().ids(1).count(1).query(),
QueryBuilder::insert().nodes().ids([1, 2]).count(1).query(),
QueryBuilder::insert().nodes().ids("a").count(1).query(),
QueryBuilder::insert().nodes().ids("a").aliases("a").query(),
QueryBuilder::insert().nodes().ids(["a", "b"]).count(1).query(),
QueryBuilder::insert().nodes().ids([1, 2]).values([[("k", "v").into()], [(1, 10).into()]]).query(),
QueryBuilder::insert().nodes().ids([1, 2]).values_uniform([("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().nodes().ids(QueryBuilder::search().from(1).query()).count(1).query(),
QueryBuilder::insert().element(&T::default()).query(),
QueryBuilder::insert().elements(&[T::default(), T::default()]).query(),
QueryBuilder::insert().values([vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids([1, 2]).query(),
QueryBuilder::insert().values([vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids(QueryBuilder::search().from("a").query()).query(),
QueryBuilder::insert().values([vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).search().from("a").query(),
QueryBuilder::insert().values_uniform([("k", "v").into(), (1, 10).into()]).ids([1, 2]).query(),
QueryBuilder::insert().values_uniform([("k", "v").into(), (1, 10).into()]).ids(QueryBuilder::search().from("a").query()).query(),
QueryBuilder::insert().values_uniform([("k", "v").into(), (1, 10).into()]).search().from("a").query(),
QueryBuilder::remove().aliases("a").query(),
QueryBuilder::remove().aliases(["a", "b"]).query(),
QueryBuilder::remove().ids(1).query(),
QueryBuilder::remove().ids("a").query(),
QueryBuilder::remove().ids([1, 2]).query(),
QueryBuilder::remove().ids(["a", "b"]).query(),
QueryBuilder::remove().ids(QueryBuilder::search().from("a").query()).query(),
QueryBuilder::remove().search().from("a").query(),
QueryBuilder::remove().index("key").query(),
QueryBuilder::remove().values(["k1", "k2"]).ids([1, 2]).query(),
QueryBuilder::remove().values(["k1", "k2"]).ids(QueryBuilder::search().from("a").query()).query(),
QueryBuilder::remove().values(["k1", "k2"]).search().from("a").query(),
QueryBuilder::select().aliases().ids([1, 2]).query(),
QueryBuilder::select().aliases().ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::select().aliases().search().from(1).query(),
QueryBuilder::select().aliases().query(),
QueryBuilder::select().edge_count().ids([1, 2]).query(),
QueryBuilder::select().edge_count_from().ids([1, 2]).query(),
QueryBuilder::select().edge_count_to().ids([1, 2]).query(),
QueryBuilder::select().edge_count().search().from(1).query(),
QueryBuilder::select().ids("a").query(),
QueryBuilder::select().ids([1, 2]).query(),
QueryBuilder::select().ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::select().search().from(1).query(),
QueryBuilder::select().search().to(1).query(),
QueryBuilder::select().search().index("age").value(20).query(),
QueryBuilder::select().search().from("a").limit(10).query(),
QueryBuilder::select().search().from("a").offset(10).query(),
QueryBuilder::select().search().from("a").order_by(DbKeyOrder::Desc("age".into())).query(),
QueryBuilder::select().search().from("a").where_().node().query(),
QueryBuilder::select().indexes().query(),
QueryBuilder::select().keys().ids("a").query(),
QueryBuilder::select().keys().ids([1, 2]).query(),
QueryBuilder::select().keys().ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::select().keys().search().from(1).query(),
QueryBuilder::select().key_count().ids("a").query(),
QueryBuilder::select().key_count().ids([1, 2]).query(),
QueryBuilder::select().key_count().ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::select().key_count().search().from(1).query(),
QueryBuilder::select().node_count().query(),
QueryBuilder::select().values(["k", "k2"]).ids("a").query(),
QueryBuilder::select().values(["k", "k2"]).ids([1, 2]).query(),
QueryBuilder::select().values(["k", "k2"]).ids(QueryBuilder::search().from(1).query()).query(),
QueryBuilder::select().values(["k", "k2"]).search().from(1).query(),
QueryBuilder::search().from("a").query(),
QueryBuilder::search().to(1).query(), 
QueryBuilder::search().from("a").to("b").query(), 
QueryBuilder::search().breadth_first().from("a").query(), 
QueryBuilder::search().depth_first().to(1).query(),
QueryBuilder::search().depth_first().from("a").query(),
QueryBuilder::search().elements().query(),
QueryBuilder::search().index("age").value(20).query(), 
QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("age".into()), DbKeyOrder::Asc("name".into())]).query(),
QueryBuilder::search().from(1).offset(10).query(),
QueryBuilder::search().from(1).limit(5).query(),
QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).offset(10).query(),
QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).limit(5).query(),
QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).offset(10).limit(5).query(),
QueryBuilder::search().from(1).offset(10).limit(5).query(),
QueryBuilder::search().from(1).where_().distance(CountComparison::LessThan(3)).query(),
QueryBuilder::search().from(1).where_().edge().query(),
QueryBuilder::search().from(1).where_().edge_count(CountComparison::GreaterThan(2)).query(),
QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1)).query(),
QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::NotEqual(1)).query(),
QueryBuilder::search().from(1).where_().node().query(),
QueryBuilder::search().from(1).where_().key("k").value(Comparison::Equal(1.into())).query(),
QueryBuilder::search().from(1).where_().keys(["k1", "k2"]).query(),
QueryBuilder::search().from(1).where_().not().keys(["k1", "k2"]).query(),
QueryBuilder::search().from(1).where_().ids([1, 2]).query(),
QueryBuilder::search().from(1).where_().beyond().keys(["k"]).query(),
QueryBuilder::search().from(1).where_().not().ids([1, 2]).query(),
QueryBuilder::search().from(1).where_().not_beyond().ids("a").query(),
QueryBuilder::search().from(1).where_().node().or().edge().query(),
QueryBuilder::search().from(1).where_().node().and().distance(CountComparison::GreaterThanOrEqual(3)).query(),
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Equal(1.into())).end_where().query(),
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(1.into())).end_where().query(),
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(([1, 2]).into())).end_where().query(),
QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).where_().node().query(),
QueryBuilder::search().from(1).limit(1).where_().node().query(),
QueryBuilder::search().from(1).offset(1).where_().node().query(),
QueryBuilder::search().to(1).offset(1).query(),
QueryBuilder::search().to(1).limit(1).query(),
QueryBuilder::search().to(1).where_().node().query(),
QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).where_().node().query()
        ];

        serde_json::to_writer_pretty(File::create("test_queries.json").unwrap(), &queries).unwrap();
    }
}
