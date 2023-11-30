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

#[axum::async_trait]
impl<S: Sync + Send> FromRequestParts<S> for UserId
where
    S: Send + Sync,
    DbPool: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, db_pool: &S) -> Result<Self, Self::Rejection> {
        let bearer: TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(unauthorized_error)?;
        let id = DbPool::from_ref(db_pool)
            .find_user_id(utilities::unquote(bearer.token()))
            .map_err(unauthorized_error)?;
        Ok(Self(id))
    }
}

fn unauthorized_error<E>(_: E) -> StatusCode {
    StatusCode::UNAUTHORIZED
}
