use crate::api_result::AgdbApiResult;
#[cfg(feature = "reqwest")]
use crate::AgdbApiError;
use serde::de::DeserializeOwned;
use serde::Serialize;

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
    client: reqwest::Client,
}

#[cfg(feature = "reqwest")]
impl ReqwestClient {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "reqwest")]
impl HttpClient for ReqwestClient {
    async fn delete(&self, uri: &str, token: &Option<String>) -> AgdbApiResult<u16> {
        let mut request = self.client.delete(uri);
        if let Some(token) = token {
            request = request.header("Authorization", format!("Bearer {}", token));
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
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        let response = request.send().await?;
        let status = response.status().as_u16();
        if response.status().is_success() {
            let body = response.json::<T>().await?;
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
        let client = reqwest::Client::new();
        let mut request = client.post(uri);
        if let Some(token) = token {
            request = request.header("Authorization", format!("Bearer {}", token));
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
            request = request.header("Authorization", format!("Bearer {}", token));
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
