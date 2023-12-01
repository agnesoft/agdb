use crate::config::Config;
use crate::db::DbPool;
use crate::utilities;
use agdb::DbId;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::RequestPartsExt;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;

pub(crate) struct UserId(pub(crate) DbId);

pub(crate) struct AdminId(pub(crate) DbId);

#[axum::async_trait]
impl<S: Sync + Send> FromRequestParts<S> for UserId
where
    S: Send + Sync,
    DbPool: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let bearer: TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(unauthorized_error)?;
        let id = DbPool::from_ref(state)
            .find_user_id(utilities::unquote(bearer.token()))
            .map_err(unauthorized_error)?;
        Ok(Self(id))
    }
}

#[axum::async_trait]
impl<S: Sync + Send> FromRequestParts<S> for AdminId
where
    S: Send + Sync,
    DbPool: FromRef<S>,
    Config: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let admin_user = Config::from_ref(state).admin.clone();
        tracing::info!("ADMIN_USER: {admin_user}");
        let admin = DbPool::from_ref(state)
            .find_user(&admin_user)
            .map_err(unauthorized_error)?;
        tracing::info!("ADMIN: {:?}", admin);
        let bearer: TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(unauthorized_error)?;

        tracing::info!("TOKEN: {}", utilities::unquote(bearer.token()));

        if admin.token != utilities::unquote(bearer.token()) {
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(Self(admin.db_id.unwrap()))
    }
}

fn unauthorized_error<E>(_: E) -> StatusCode {
    StatusCode::UNAUTHORIZED
}
