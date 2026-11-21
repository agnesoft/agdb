use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::test_error::TestError;

async fn exec_negotiates_encoding() -> Result<(), TestError> {
    use crate::test_server::ADMIN;
    use crate::test_server::TestServer;
    use crate::test_server::next_db_name;
    use crate::test_server::next_user_name;
    use agdb::AgdbSerialize;
    use agdb::QueryBuilder;
    use agdb::QueryResult;
    use agdb::QueryType;
    use agdb_api::BINARY_CONTENT_TYPE;
    use agdb_api::DbKind;

    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Memory).await?;
    let token = server.api.token.clone().unwrap_or_default();
    let url = format!("{}/db/{owner}/{db}/exec_mut", server.api.base_url());
    let queries: Vec<QueryType> = vec![QueryBuilder::insert().nodes().count(1).query().into()];
    let client = reqwest::Client::new();

    let json = client
        .post(&url)
        .bearer_auth(&token)
        .json(&queries)
        .send()
        .await?;
    assert_eq!(json.status(), 200);
    assert_eq!(json.headers()["content-type"], "application/json");
    assert_eq!(json.json::<Vec<QueryResult>>().await?[0].result, 1);

    let binary_request = AgdbSerialize::serialize(&queries);
    let binary = client
        .post(&url)
        .bearer_auth(&token)
        .header("content-type", BINARY_CONTENT_TYPE)
        .body(binary_request.clone())
        .send()
        .await?;
    assert_eq!(binary.status(), 200);
    assert_eq!(binary.headers()["content-type"], BINARY_CONTENT_TYPE);
    let results = <Vec<QueryResult> as AgdbSerialize>::deserialize(&binary.bytes().await?)?;
    assert_eq!(results[0].ids(), vec![agdb::DbId(2)]);

    let truncated = client
        .post(&url)
        .bearer_auth(&token)
        .header("content-type", BINARY_CONTENT_TYPE)
        .body(binary_request[..binary_request.len() - 1].to_vec())
        .send()
        .await?;
    assert_eq!(truncated.status(), 400);

    Ok(())
}

async fn exec_falls_back_to_json_for_older_servers() -> Result<(), TestError> {
    use agdb::QueryBuilder;
    use agdb::QueryResult;
    use agdb::QueryType;
    use agdb_api::AgdbApi;
    use agdb_api::BINARY_CONTENT_TYPE;
    use agdb_api::ReqwestClient;
    use axum::http::HeaderMap;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    #[derive(Default)]
    struct OldServer {
        binary_attempts: AtomicUsize,
        down: AtomicBool,
    }

    // Like servers that predate binary support: JSON only, 415 otherwise.
    async fn exec(
        axum::extract::State(state): axum::extract::State<Arc<OldServer>>,
        headers: HeaderMap,
        body: axum::body::Bytes,
    ) -> axum::response::Response {
        if state.down.swap(false, Ordering::Relaxed) {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }

        if headers["content-type"] == BINARY_CONTENT_TYPE {
            state.binary_attempts.fetch_add(1, Ordering::Relaxed);
            return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
        }

        let queries: Vec<QueryType> = serde_json::from_slice(&body).unwrap_or_default();
        axum::Json(vec![QueryResult::default(); queries.len()]).into_response()
    }

    let state = Arc::new(OldServer::default());
    let app = axum::Router::new()
        .route("/api/v1/db/{owner}/{db}/exec", axum::routing::post(exec))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = format!("http://{}", listener.local_addr()?);
    tokio::spawn(async move { axum::serve(listener, app).await });
    let api = AgdbApi::new(ReqwestClient::new(), &address);
    let queries: Vec<QueryType> = vec![QueryBuilder::select().node_count().query().into()];

    assert_eq!(api.db_exec("o", "d", &queries).await?.1.len(), 1);
    assert_eq!(api.db_exec("o", "d", &queries).await?.1.len(), 1);
    assert_eq!(state.binary_attempts.load(Ordering::Relaxed), 1);

    // An outage may be an upgrade, so binary is tried again afterwards.
    state.down.store(true, Ordering::Relaxed);
    assert_eq!(
        api.db_exec("o", "d", &queries).await.unwrap_err().status,
        503
    );
    assert_eq!(api.db_exec("o", "d", &queries).await?.1.len(), 1);
    assert_eq!(state.binary_attempts.load(Ordering::Relaxed), 2);

    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    exec_negotiates_encoding,
    exec_falls_back_to_json_for_older_servers,
];
