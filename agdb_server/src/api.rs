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
        crate::routes::user::UserToken,
        crate::routes::db::DbType,
        crate::routes::db::ServerDatabase,
        crate::routes::db::ServerDatabaseName,
    ))
)]
pub(crate) struct Api;
