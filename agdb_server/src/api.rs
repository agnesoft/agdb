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
        crate::routes::status,
        crate::routes::admin::db::add,
        crate::routes::admin::db::audit,
        crate::routes::admin::db::backup,
        crate::routes::admin::db::copy,
        crate::routes::admin::db::delete,
        crate::routes::admin::db::exec,
        crate::routes::admin::db::list,
        crate::routes::admin::db::optimize,
        crate::routes::admin::db::rename,
        crate::routes::admin::db::remove,
        crate::routes::admin::db::restore,
        crate::routes::admin::db::user::add,
        crate::routes::admin::db::user::list,
        crate::routes::admin::db::user::remove,
        crate::routes::admin::user::change_password,
        crate::routes::admin::user::add,
        crate::routes::admin::user::list,
        crate::routes::admin::user::remove,
        crate::routes::admin::shutdown,
        crate::routes::user::login,
        crate::routes::user::logout,
        crate::routes::user::change_password,
        crate::routes::db::add,
        crate::routes::db::audit,
        crate::routes::db::backup,
        crate::routes::db::copy,
        crate::routes::db::delete,
        crate::routes::db::exec,
        crate::routes::db::list,
        crate::routes::db::optimize,
        crate::routes::db::remove,
        crate::routes::db::rename,
        crate::routes::db::restore,
        crate::routes::db::user::add,
        crate::routes::db::user::list,
        crate::routes::db::user::remove,
    ),
    components(schemas(
        crate::routes::db::DbTypeParam,
        crate::routes::db::ServerDatabaseRename,
        crate::routes::db::user::DbUserRoleParam,
        agdb_api::DbAudit,
        agdb_api::DbType,
        agdb_api::DbUser,
        agdb_api::DbUserRole,
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
        agdb::SelectQuery,
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
