use crate::config::Config;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
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

pub(crate) struct AdminId();

pub(crate) struct ClusterId();

#[derive(Default)]
pub(crate) struct UserName(pub(crate) String);

impl<S: Sync + Send> FromRequestParts<S> for UserName
where
    S: Send + Sync,
    ServerDb: FromRef<S>,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(bearer) = parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            let db_pool = ServerDb::from_ref(state);
            let id = db_pool
                .user_token_id(utilities::unquote(bearer.token()))
                .await?;
            return Ok(UserName(db_pool.user_name(id).await?));
        }

        Ok(Self("".to_string()))
    }
}

impl<S: Sync + Send> FromRequestParts<S> for UserId
where
    S: Send + Sync,
    ServerDb: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let bearer: TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(unauthorized)?;
        let id = ServerDb::from_ref(state)
            .user_token_id(utilities::unquote(bearer.token()))
            .await
            .map_err(unauthorized)?;
        Ok(Self(id))
    }
}

impl<S: Sync + Send> FromRequestParts<S> for AdminId
where
    S: Send + Sync,
    ServerDb: FromRef<S>,
    Config: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let bearer: TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(unauthorized)?;

        if !ServerDb::from_ref(state)
            .is_admin(utilities::unquote(bearer.token()))
            .await
            .map_err(unauthorized)?
        {
            return Err(unauthorized(()));
        }

        Ok(Self())
    }
}

impl<S: Sync + Send> FromRequestParts<S> for ClusterId
where
    S: Send + Sync,
    Config: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let expected_token = &Config::from_ref(state).cluster_token;
        let bearer: TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(unauthorized)?;

        if expected_token != utilities::unquote(bearer.token()) {
            return Err(unauthorized(()));
        }

        Ok(Self())
    }
}

fn unauthorized<E>(_: E) -> StatusCode {
    StatusCode::UNAUTHORIZED
}
