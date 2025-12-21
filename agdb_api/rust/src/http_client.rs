use crate::AgdbApiError;
use crate::api_result::AgdbApiResult;
use crate::client::AgdbApiClient;
use agdb::api_def::ImplDefinition;
#[cfg(feature = "api")]
use agdb::api_def::{Type, TypeDefinition};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait HttpClient {
    fn delete(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<u16>> + Send;
    fn get<T: DeserializeOwned + Send>(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<(u16, T)>> + Send;
    fn post<T: Serialize + Send, R: DeserializeOwned + Send>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<(u16, R)>> + Send;
    fn put<T: Serialize + Send>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<u16>> + Send;
}

pub struct ReqwestClient {
    pub client: reqwest::Client,
}

impl AgdbApiClient for ReqwestClient {}

#[cfg(feature = "api")]
impl TypeDefinition for ReqwestClient {
    fn type_def() -> Type {
        todo!()
    }
}

impl ImplDefinition for ReqwestClient {
    fn functions() -> &'static [agdb::api_def::Function] {
        &[]
    }
}

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
        json: Option<T>,
        token: &Option<String>,
    ) -> AgdbApiResult<(u16, R)> {
        let mut request = self.client.post(uri);
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
        let mut request = self.client.put(uri);
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
}
