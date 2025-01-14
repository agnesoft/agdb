use crate::routes::ServerResult;
use crate::server_error::ServerError;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum_extra::headers::ContentType;
use include_dir::include_dir;
use include_dir::Dir;
use reqwest::StatusCode;

static AGDB_STUDIO: Dir = include_dir!("agdb_studio/dist");

pub(crate) async fn studio_root() -> ServerResult<impl IntoResponse> {
    studio(Path("index.html".to_string())).await
}

pub(crate) async fn studio(Path(file): Path<String>) -> ServerResult<impl IntoResponse> {
    let f = if let Some(f) = AGDB_STUDIO.get_file(&file) {
        f
    } else {
        AGDB_STUDIO.get_file("index.html").ok_or(ServerError::new(
            StatusCode::NOT_FOUND,
            "index.html not found",
        ))?
    };

    let content_type = if file.ends_with(".js") {
        "application/javascript".to_string()
    } else if file.ends_with(".css") {
        "text/css".to_string()
    } else if file.ends_with(".svg") {
        "image/svg+xml".to_string()
    } else {
        ContentType::html().to_string()
    };

    Ok((
        StatusCode::CREATED,
        [("Content-Type", content_type)],
        f.contents_utf8().ok_or(ServerError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to read content of '{file}'"),
        ))?,
    ))
}
