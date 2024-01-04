use agdb::QueryResult;
use agdb::QueryType;
use serde::Deserialize;
use serde::Serialize;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Copy, Clone, Default, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DbType {
    #[default]
    Memory,
    Mapped,
    File,
}

#[derive(Default, Deserialize, Serialize, ToSchema)]
pub struct ServerDatabase {
    pub(crate) name: String,
    pub(crate) db_type: DbType,
    pub(crate) role: DbUserRole,
    pub(crate) size: u64,
    pub(crate) backup: u64,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DbUserRole {
    #[default]
    Admin,
    Write,
    Read,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub(crate) struct ServerDatabaseRename {
    pub(crate) new_name: String,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct Queries(pub(crate) Vec<QueryType>);

#[derive(Serialize, ToSchema)]
pub(crate) struct QueriesResults(pub(crate) Vec<QueryResult>);

impl From<&str> for DbType {
    fn from(value: &str) -> Self {
        match value {
            "mapped" => DbType::Mapped,
            "file" => DbType::File,
            _ => DbType::Memory,
        }
    }
}

impl std::fmt::Display for DbType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbType::File => f.write_str("file"),
            DbType::Mapped => f.write_str("mapped"),
            DbType::Memory => f.write_str("memory"),
        }
    }
}
