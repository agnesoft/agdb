use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(crate::app::create_db, crate::app::create_user, crate::app::login),
    components(schemas(
        crate::app::CreateDb,
        crate::app::DbType,
        crate::app::UserCredentials,
        crate::app::UserToken
    ))
)]
pub(crate) struct Api;
