#[cfg(feature = "api")]
mod api;
mod api_error;
mod api_result;
mod api_types;
mod client;
mod http_client;

pub use api_error::AgdbApiError;
pub use api_result::AgdbApiResult;
pub use api_types::AdminStatus;
pub use api_types::ChangePassword;
pub use api_types::ClusterStatus;
pub use api_types::DbAudit;
pub use api_types::DbKind;
pub use api_types::DbResource;
pub use api_types::DbUser;
pub use api_types::DbUserRole;
pub use api_types::Queries;
pub use api_types::QueriesResults;
pub use api_types::QueryAudit;
pub use api_types::ServerDatabase;
pub use api_types::UserCredentials;
pub use api_types::UserLogin;
pub use api_types::UserStatus;
pub use client::AgdbApi;
pub use http_client::HttpClient;
pub use http_client::ReqwestClient;
