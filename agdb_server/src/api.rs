use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::app::add_db,
        crate::app::create_user,
        crate::app::delete_db,
        crate::app::list,
        crate::app::login,
        crate::app::remove_db
    ),
    components(schemas(
        crate::app::ServerDatabase,
        crate::app::DbType,
        crate::app::ServerDatabaseName,
        crate::app::UserCredentials,
        crate::app::UserToken
    ))
)]
pub(crate) struct Api;
