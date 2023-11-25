use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(crate::app::create_db),
    components(schemas(crate::app::CreateUser))
)]
pub(crate) struct Api;
