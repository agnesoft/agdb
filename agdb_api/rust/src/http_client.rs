#[cfg(feature = "reqwest")]
use crate::AgdbApiError;
use crate::api::AgdbApiClient;
use crate::api_result::AgdbApiResult;
use agdb::api::ApiDefinition;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[allow(async_fn_in_trait)]
pub trait HttpClient {
    async fn delete(&self, uri: &str, token: &Option<String>) -> AgdbApiResult<u16>;
    async fn get<T: DeserializeOwned + Send>(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, T)>;
    async fn post<T: Serialize, R: DeserializeOwned + Send>(
        &self,
        uri: &str,
        json: &Option<T>,
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, R)>;
    async fn put<T: Serialize>(
        &self,
        uri: &str,
        json: &Option<T>,
        token: &Option<String>,
    ) -> AgdbApiResult<u16>;
}

#[cfg(feature = "reqwest")]
pub struct ReqwestClient {
    pub client: reqwest::Client,
}

impl AgdbApiClient for ReqwestClient {}

#[cfg(feature = "api")]
impl ApiDefinition for ReqwestClient {
    fn def() -> agdb::api::Type {
        use agdb::api::NamedType;

        static STRUCT: std::sync::OnceLock<agdb::api::Struct> = std::sync::OnceLock::new();

        agdb::api::Type::Struct(STRUCT.get_or_init(|| agdb::api::Struct {
            name: "ReqwestClient",
            fields: vec![NamedType {
                name: "client",
                ty: || agdb::api::Type::User,
            }],
        }))
    }
}

#[cfg(feature = "reqwest")]
impl ReqwestClient {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "reqwest")]
impl HttpClient for ReqwestClient {
    async fn delete(&self, uri: &str, token: &Option<String>) -> AgdbApiResult<u16> {
        let mut request = self.client.delete(uri);
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
        let mut request = self.client.get(uri);
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
        json: &Option<T>,
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, R)> {
        let mut request = self.client.post(uri);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        if let Some(json) = json {
            request = request.json(json);
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

    async fn put<T: Serialize>(
        &self,
        uri: &str,
        json: &Option<T>,
        token: &Option<String>,
    ) -> AgdbApiResult<u16> {
        let mut request = self.client.put(uri);
        if let Some(json) = json {
            request = request.json(json);
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
}
