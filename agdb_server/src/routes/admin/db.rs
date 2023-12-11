use crate::db::DbPool;
use crate::routes::db::ServerDatabase;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(get,
    path = "/api/v1/admin/db/list",
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = Vec<ServerDatabase>),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn list(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
) -> ServerResponse<(StatusCode, Json<Vec<ServerDatabase>>)> {
    let dbs = db_pool
        .find_dbs()?
        .into_iter()
        .map(|db| db.into())
        .collect();
    Ok((StatusCode::OK, Json(dbs)))
}
