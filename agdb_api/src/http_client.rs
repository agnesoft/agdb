use crate::AgdbApiError;
use crate::api_result::AgdbApiResult;
use crate::client::AgdbApiClient;
#[cfg(feature = "api")]
use agdb::type_def::{Type, TypeDefinition};
use reqwest::header::USER_AGENT;
use serde::Serialize;
use serde::de::DeserializeOwned;

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
}

pub struct ReqwestClientTypeDef(pub reqwest::Client);

impl From<reqwest::Client> for ReqwestClientTypeDef {
    fn from(client: reqwest::Client) -> Self {
        ReqwestClientTypeDef(client)
    }
}

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
            name: "reqwest::Client",
            generics: &[],
            fields: &[],
            impl_defs: Vec::new,
        })
    }
}

#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[cfg_attr(feature = "api", type_def(inherent, HttpClient))]
pub struct ReqwestClient {
    pub client: ReqwestClientTypeDef,
    pub user_agent: String,
}

impl AgdbApiClient for ReqwestClient {}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl ReqwestClient {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::with_client(ReqwestClientTypeDef(reqwest::Client::new()))
    }

    pub fn with_client<C: Into<ReqwestClientTypeDef>>(client: C) -> Self {
        Self::with_user_agent(client, "agdb_api".to_string())
    }

    pub fn with_user_agent<C: Into<ReqwestClientTypeDef>, S: Into<String>>(client: C, user_agent: S) -> Self {
        Self {
            client: client.into(),
            user_agent: user_agent.into(),
        }
    }
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
}
