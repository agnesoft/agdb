use crate::api_result::AgdbApiResult;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait HttpClient {
    fn delete(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<u16>>;
    fn get<T: DeserializeOwned>(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<(u16, T)>>;
    fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        uri: &str,
        json: &Option<T>,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<(u16, R)>>;
    fn put<T: Serialize>(
        &self,
        uri: &str,
        json: &Option<T>,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<u16>>;
}
