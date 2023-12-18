use utoipa::openapi::security::Http;
use utoipa::openapi::security::HttpAuthScheme;
use utoipa::openapi::security::SecurityScheme;
use utoipa::Modify;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::status,
        crate::routes::admin::db::add,
        crate::routes::admin::db::delete,
        crate::routes::admin::db::list,
        crate::routes::admin::db::remove,
        crate::routes::admin::db::user::add,
        crate::routes::admin::db::user::list,
        crate::routes::admin::db::user::remove,
        crate::routes::admin::user::change_password,
        crate::routes::admin::user::create,
        crate::routes::admin::user::list,
        crate::routes::admin::shutdown,
        crate::routes::user::login,
        crate::routes::user::change_password,
        crate::routes::db::add,
        crate::routes::db::delete,
        crate::routes::db::exec,
        crate::routes::db::list,
        crate::routes::db::remove,
        crate::routes::db::rename,
        crate::routes::db::user::add,
        crate::routes::db::user::list,
        crate::routes::db::user::remove,
    ),
    components(schemas(
        crate::routes::admin::user::UserStatus,
        crate::routes::db::DbType,
        crate::routes::db::Queries,
        crate::routes::db::QueriesResults,
        crate::routes::db::ServerDatabase,
        crate::routes::db::ServerDatabaseWithRole,
        crate::routes::db::ServerDatabaseName,
        crate::routes::db::ServerDatabaseRename,
        crate::routes::db::user::DbUser,
        crate::routes::db::user::RemoveDbUser,
        crate::routes::db::user::DbUserRole,
        crate::routes::user::ChangePassword,
        crate::routes::user::UserCredentials,
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
