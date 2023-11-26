use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::app::create_db,
        crate::app::create_user,
        crate::app::delete_db,
        crate::app::list,
        crate::app::login
    ),
    components(schemas(
        crate::app::ServerDatabase,
        crate::app::DbType,
        crate::app::DeleteServerDatabase,
        crate::app::UserCredentials,
        crate::app::UserToken
    ))
)]
pub(crate) struct Api;
