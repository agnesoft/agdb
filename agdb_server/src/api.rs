use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(crate::app::create_db),
    components(schemas(crate::app::CreateDb, crate::app::CreateDbType))
)]
pub(crate) struct Api;
