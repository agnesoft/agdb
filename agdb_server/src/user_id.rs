use crate::config::Config;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use crate::utilities;
use agdb::DbId;
use axum::RequestPartsExt;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;

pub(crate) struct UserId(pub(crate) DbId);

#[derive(Default)]
pub(crate) struct UserToken(pub(crate) String);

pub(crate) struct AdminId();

pub(crate) struct ClusterId();

#[derive(Default)]
pub(crate) struct UserName(pub(crate) String);

#[derive(Default)]
pub(crate) struct UserAgent(pub(crate) String);

impl<S: Sync + Send> FromRequestParts<S> for UserToken
where
    S: Send + Sync,
    ServerDb: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let bearer: TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(unauthorized)?;
        let token = utilities::unquote(bearer.token());
        ServerDb::from_ref(state)
            .user_id_from_token(token)
            .await
            .map_err(unauthorized)?;
        Ok(Self(token.to_string()))
    }
}

impl<S: Sync + Send> FromRequestParts<S> for UserName
where
    S: Send + Sync,
    ServerDb: FromRef<S>,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(bearer) = parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            let server_db = ServerDb::from_ref(state);
            let id = server_db
                .user_id_from_token(utilities::unquote(bearer.token()))
                .await?;
            return Ok(UserName(server_db.user_name(id).await?));
        }

        Ok(Self("".to_string()))
    }
}

impl<S: Sync + Send> FromRequestParts<S> for UserAgent
where
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|header| header.to_str().ok())
            .unwrap_or("");

        Ok(Self(value.to_string()))
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
            .user_id_from_token(utilities::unquote(bearer.token()))
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
