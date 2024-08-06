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
        routes::admin::user::remove,
        routes::admin::shutdown,
        routes::user::login,
        routes::user::logout,
        routes::user::change_password,
        routes::db::add,
        routes::db::audit,
        routes::db::backup,
        routes::db::clear,
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
    ),
    components(schemas(
        routes::db::DbTypeParam,
        routes::db::ServerDatabaseRename,
        routes::db::ServerDatabaseResource,
        routes::db::user::DbUserRoleParam,
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
        agdb_api::StatusParams,
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
