use utoipa::openapi::security::Http;
use utoipa::openapi::security::HttpAuthScheme;
use utoipa::openapi::security::SecurityScheme;
use utoipa::Modify;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::admin::shutdown,
        crate::routes::user::create,
        crate::routes::user::login,
        crate::routes::user::change_password,
        crate::routes::db::add,
        crate::routes::db::delete,
        crate::routes::db::list,
        crate::routes::db::remove,
    ),
    components(schemas(
        crate::routes::user::ChangePassword,
        crate::routes::user::UserCredentials,
        crate::routes::db::DbType,
        crate::routes::db::ServerDatabase,
        crate::routes::db::ServerDatabaseName,
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
