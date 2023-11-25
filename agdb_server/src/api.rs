use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(crate::app::create_user),
    components(schemas(crate::app::CreateUser))
)]
pub(crate) struct Api;
