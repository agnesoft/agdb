use crate::AgdbApiError;
use crate::api_result::AgdbApiResult;
use crate::client::AgdbApiClient;
use agdb::AgdbSerialize;
use agdb::QueryResult;
use agdb::QueryType;
#[cfg(feature = "api")]
use agdb::type_def::{Type, TypeDefinition};
use reqwest::header::CONTENT_TYPE;
use reqwest::header::USER_AGENT;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/// Content type of query batches and their results encoded with agdb's own
/// binary serialization instead of JSON. Servers that predate it answer 415.
/// The version is part of the content type so that a future format change
/// is negotiated the same way.
pub const BINARY_CONTENT_TYPE: &str = "application/vnd.agdb.api.v1";

/// Abstraction over HTTP verbs used by [`AgdbApi`](crate::AgdbApi).
/// Implement this trait to swap the transport layer.
#[cfg_attr(feature = "api", agdb::trait_def())]
#[allow(async_fn_in_trait)]
pub trait HttpClient {
    async fn delete(&self, uri: &str, token: &Option<String>) -> AgdbApiResult<u16>;
    async fn get<T: DeserializeOwned + Send>(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, T)>;
    async fn post<T: Serialize + Send, R: DeserializeOwned + Send>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, R)>;
    async fn put<T: Serialize + Send>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> AgdbApiResult<u16>;

    /// Posts a query batch to an `exec` or `exec_mut` endpoint. The default
    /// implementation sends JSON.
    async fn exec(
        &self,
        uri: &str,
        queries: &[QueryType],
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        self.post(uri, Some(queries), token).await
    }
}

pub struct ReqwestClientTypeDef(pub reqwest::Client);

impl std::ops::Deref for ReqwestClientTypeDef {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ReqwestClientTypeDef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "api")]
impl TypeDefinition for ReqwestClientTypeDef {
    fn type_def() -> Type {
        Type::Struct(agdb::type_def::Struct {
            name: "reqwest::Client".to_owned(),
            generics: vec![],
            fields: vec![],
            impl_defs: Vec::new,
        })
    }
}

/// Default [`HttpClient`] implementation backed by [`reqwest`].
///
/// Query batches are sent as [`BINARY_CONTENT_TYPE`]. If the server answers
/// 415 the client switches to JSON until the server becomes unreachable
/// (e.g. while it is restarted with a newer version).
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct ReqwestClient {
    pub client: ReqwestClientTypeDef,
    pub user_agent: String,
    json_only: AtomicBool,
}

impl AgdbApiClient for ReqwestClient {}

impl ReqwestClient {
    /// Creates a new client with default settings and the `"agdb_api"` user-agent.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::with_client(reqwest::Client::new())
    }

    /// Creates a client from an existing [`reqwest::Client`] with the default `"agdb_api"` user-agent.
    pub fn with_client(client: reqwest::Client) -> Self {
        Self::with_user_agent(client, "agdb_api")
    }

    /// Creates a client from an existing [`reqwest::Client`] with a custom user-agent string.
    pub fn with_user_agent(client: reqwest::Client, user_agent: impl Into<String>) -> Self {
        Self {
            client: ReqwestClientTypeDef(client),
            user_agent: user_agent.into(),
            json_only: AtomicBool::new(false),
        }
    }

    async fn exec_negotiated(
        &self,
        uri: &str,
        queries: &[QueryType],
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        if !self.json_only.load(Ordering::Relaxed) {
            match self.send_queries(uri, queries, token, true).await {
                Err(e) if e.status == 415 => self.json_only.store(true, Ordering::Relaxed),
                result => return result,
            }
        }

        self.send_queries(uri, queries, token, false).await
    }

    async fn send_queries(
        &self,
        uri: &str,
        queries: &[QueryType],
        token: &Option<String>,
        binary: bool,
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        let mut request = self.client.post(uri).header(USER_AGENT, &self.user_agent);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        request = if binary {
            request
                .header(CONTENT_TYPE, BINARY_CONTENT_TYPE)
                .body(encode_queries(queries))
        } else {
            request.json(queries)
        };
        let response = request.send().await.inspect_err(|_| {
            self.json_only.store(false, Ordering::Relaxed);
        })?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            if matches!(status, 502..=504) {
                self.json_only.store(false, Ordering::Relaxed);
            }

            return Err(AgdbApiError {
                status,
                description: response.text().await?,
            });
        }

        let binary_response = response
            .headers()
            .get(CONTENT_TYPE)
            .is_some_and(|value| value == BINARY_CONTENT_TYPE);
        let results = if binary_response {
            decode_results(&response.bytes().await?)?
        } else {
            response.json().await?
        };

        Ok((status, results))
    }
}

// Same layout as `Vec<QueryType>` without cloning the borrowed queries.
fn encode_queries(queries: &[QueryType]) -> Vec<u8> {
    let mut bytes = AgdbSerialize::serialize(&queries.len());

    for query in queries {
        bytes.extend(AgdbSerialize::serialize(query));
    }

    bytes
}

fn decode_results(bytes: &[u8]) -> AgdbApiResult<Vec<QueryResult>> {
    let results = <Vec<QueryResult> as AgdbSerialize>::deserialize(bytes)?;
    let size = AgdbSerialize::serialized_size(&results);

    if size != bytes.len() as u64 {
        return Err(AgdbApiError {
            status: 0,
            description: format!("query results have {} bytes, expected {size}", bytes.len()),
        });
    }

    Ok(results)
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl HttpClient for ReqwestClient {
    async fn delete(&self, uri: &str, token: &Option<String>) -> AgdbApiResult<u16> {
        let mut request = self.client.delete(uri).header(USER_AGENT, &self.user_agent);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await?;
        let status = response.status().as_u16();
        if response.status().is_success() {
            Ok(status)
        } else {
            Err(AgdbApiError {
                status,
                description: response.text().await?,
            })
        }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, T)> {
        let mut request = self.client.get(uri).header(USER_AGENT, &self.user_agent);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await?;
        let status = response.status().as_u16();
        if response.status().is_success() {
            let body = if response.content_length().unwrap_or(0) != 0 {
                response.json::<T>().await?
            } else {
                serde_json::from_value(serde_json::Value::default())?
            };
            Ok((status, body))
        } else {
            Err(AgdbApiError {
                status,
                description: response.text().await?,
            })
        }
    }

    async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, R)> {
        let mut request = self.client.post(uri).header(USER_AGENT, &self.user_agent);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        if let Some(json) = json {
            request = request.json(&json);
        }
        let response = request.send().await?;
        let status = response.status().as_u16();
        if response.status().is_success() {
            let body = if response.content_length().unwrap_or(0) != 0 {
                response.json::<R>().await?
            } else {
                serde_json::from_value(serde_json::Value::default())?
            };
            Ok((status, body))
        } else {
            Err(AgdbApiError {
                status,
                description: response.text().await?,
            })
        }
    }

    async fn put<T: Serialize + Send>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> AgdbApiResult<u16> {
        let mut request = self.client.put(uri).header(USER_AGENT, &self.user_agent);
        if let Some(json) = json {
            request = request.json(&json);
        }
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await?;
        let status = response.status().as_u16();
        if response.status().is_success() {
            Ok(status)
        } else {
            Err(AgdbApiError {
                status,
                description: response.text().await?,
            })
        }
    }

    async fn exec(
        &self,
        uri: &str,
        queries: &[QueryType],
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        self.exec_negotiated(uri, queries, token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::QueryBuilder;

    #[test]
    fn encoded_queries_match_vec_layout() {
        let queries: Vec<QueryType> = vec![
            QueryBuilder::insert().nodes().count(1).query().into(),
            QueryBuilder::select().node_count().query().into(),
        ];

        assert_eq!(encode_queries(&queries), AgdbSerialize::serialize(&queries));
    }

    #[test]
    fn decode_results_rejects_malformed_input() {
        let mut bytes = AgdbSerialize::serialize(&vec![QueryResult::default()]);
        assert_eq!(
            decode_results(&bytes).unwrap(),
            vec![QueryResult::default()]
        );

        bytes.push(0);
        assert!(decode_results(&bytes).is_err());
        assert!(decode_results(&bytes[..4]).is_err());
    }
}
