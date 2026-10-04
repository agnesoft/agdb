//! Typed API client for the agdb server.
//!
//! Use [`AgdbApi`] to connect to a running agdb server, authenticate users,
//! create and manage databases, and execute query batches over the `/api/v1`
//! HTTP surface. The crate also exposes the request/response model types
//! shared between the server and client tooling.
//!
//! # Features
//!
//! - `api`: enables type-definition metadata used by transpilers and language
//!   bindings.
//! - `tls`: enables native TLS for the HTTP client transport (via
//!   `reqwest/default-tls`).
//! - `test_server`: adds helpers used by agdb server integration tests.

#[cfg(feature = "api")]
pub mod api;
mod api_error;
mod api_result;
mod api_types;
mod client;
mod http_client;
#[cfg(feature = "test_server")]
pub mod test_server;
#[cfg(feature = "test_server")]
pub mod tests;
#[cfg(feature = "api")]
pub mod transpiler;

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
pub use api_types::LogLevelFilter;
pub use api_types::Queries;
pub use api_types::QueriesResults;
pub use api_types::QueryAudit;
pub use api_types::ServerDatabase;
pub use api_types::UserCredentials;
pub use api_types::UserLogin;
pub use api_types::UserSession;
pub use api_types::UserStatus;
pub use api_types::config_impl;
pub use client::AgdbApi;
pub use http_client::HttpClient;
pub use http_client::ReqwestClient;
#[cfg(feature = "api")]
pub use {client::AgdbApiClientDef, http_client::HttpClientDef};
