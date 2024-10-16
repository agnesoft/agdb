use std::fmt::Display;

use agdb::DbError;
use agdb::DbValue;
use agdb::QueryResult;
use agdb::QueryType;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(
    Copy, Clone, Debug, Default, Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum DbType {
    #[default]
    Memory,
    Mapped,
    File,
}

#[derive(
    Copy, Clone, Debug, Default, Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum DbResource {
    #[default]
    All,
    Db,
    Audit,
    Backup,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
pub struct DbUser {
    pub user: String,
    pub role: DbUserRole,
}

#[derive(
    Clone, Copy, Debug, Default, Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum DbUserRole {
    #[default]
    Admin,
    Write,
    Read,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ChangePassword {
    pub password: String,
    pub new_password: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ClusterStatus {
    pub address: String,
    pub status: bool,
    pub leader: bool,
    pub term: u64,
    pub commit: u64,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AdminStatus {
    pub uptime: u64,
    pub dbs: u64,
    pub users: u64,
    pub logged_in_users: u64,
    pub size: u64,
}

#[derive(Deserialize, ToSchema)]
pub struct Queries(pub Vec<QueryType>);

#[derive(Serialize, ToSchema)]
pub struct QueriesResults(pub Vec<QueryResult>);

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct QueryAudit {
    pub timestamp: u64,
    pub user: String,
    pub query: QueryType,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct DbAudit(pub Vec<QueryAudit>);

#[derive(Debug, Default, Deserialize, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
pub struct ServerDatabase {
    pub name: String,
    pub db_type: DbType,
    pub role: DbUserRole,
    pub size: u64,
    pub backup: u64,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UserCredentials {
    pub password: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserStatus {
    pub name: String,
    pub login: bool,
}

impl From<&str> for DbType {
    fn from(value: &str) -> Self {
        match value {
            "mapped" => DbType::Mapped,
            "file" => DbType::File,
            _ => DbType::Memory,
        }
    }
}

impl TryFrom<DbValue> for DbType {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(Self::from(value.to_string().as_str()))
    }
}

impl From<DbType> for DbValue {
    fn from(value: DbType) -> Self {
        value.to_string().into()
    }
}

impl Display for DbType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbType::File => f.write_str("file"),
            DbType::Mapped => f.write_str("mapped"),
            DbType::Memory => f.write_str("memory"),
        }
    }
}

impl Display for DbResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbResource::All => f.write_str("all"),
            DbResource::Db => f.write_str("db"),
            DbResource::Audit => f.write_str("audit"),
            DbResource::Backup => f.write_str("backup"),
        }
    }
}

impl From<&DbValue> for DbUserRole {
    fn from(value: &DbValue) -> Self {
        match value.to_u64().unwrap_or_default() {
            1 => Self::Admin,
            2 => Self::Write,
            _ => Self::Read,
        }
    }
}

impl From<DbUserRole> for DbValue {
    fn from(value: DbUserRole) -> Self {
        match value {
            DbUserRole::Admin => 1_u64.into(),
            DbUserRole::Write => 2_u64.into(),
            DbUserRole::Read => 3_u64.into(),
        }
    }
}

impl Display for DbUserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbUserRole::Admin => f.write_str("admin"),
            DbUserRole::Read => f.write_str("read"),
            DbUserRole::Write => f.write_str("write"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::SelectIndexesQuery;

    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", DbType::Memory);
        let _ = format!("{:?}", DbUserRole::Admin);
        let _ = format!(
            "{:?}",
            DbUser {
                user: "user".to_string(),
                role: DbUserRole::Admin
            }
        );
        let _ = format!(
            "{:?}",
            ServerDatabase {
                name: "db".to_string(),
                db_type: DbType::Memory,
                role: DbUserRole::Admin,
                size: 0,
                backup: 0
            }
        );
        let _ = format!(
            "{:?}",
            UserStatus {
                name: "user".to_string(),
                login: true
            }
        );
        let _ = format!(
            "{:?}",
            QueryAudit {
                timestamp: 0,
                user: "user".to_string(),
                query: QueryType::SelectIndexes(SelectIndexesQuery {})
            }
        );
        let _ = format!("{:?}", DbAudit(vec![]));
        let _ = format!(
            "{:?}",
            ClusterStatus {
                address: "localhost".to_string(),
                status: true,
                leader: false,
                term: 0,
                commit: 0,
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        let query_audit = QueryAudit {
            timestamp: 0,
            user: "user".to_string(),
            query: QueryType::SelectIndexes(SelectIndexesQuery {}),
        };
        let audit = DbAudit(vec![query_audit]);
        assert_eq!(audit, audit);
    }

    #[test]
    fn derived_from_parital_ord() {
        assert!(DbType::Memory < DbType::File);
        assert!(DbUserRole::Admin < DbUserRole::Write);
        let user = DbUser {
            user: "user".to_string(),
            role: DbUserRole::Admin,
        };
        let other = DbUser {
            user: "user2".to_string(),
            role: DbUserRole::Admin,
        };
        assert!(user < other);
        let db = ServerDatabase {
            name: "db".to_string(),
            db_type: DbType::Memory,
            role: DbUserRole::Admin,
            size: 0,
            backup: 0,
        };
        let other = ServerDatabase {
            name: "db2".to_string(),
            db_type: DbType::Memory,
            role: DbUserRole::Admin,
            size: 0,
            backup: 0,
        };
        assert!(db < other);
        let status = UserStatus {
            name: "user".to_string(),
            login: true,
        };
        let other = UserStatus {
            name: "user2".to_string(),
            login: true,
        };
        assert!(status < other);
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(
            DbType::Memory.cmp(&DbType::Memory),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            DbUserRole::Admin.cmp(&DbUserRole::Admin),
            std::cmp::Ordering::Equal
        );

        let user = DbUser {
            user: "user".to_string(),
            role: DbUserRole::Admin,
        };

        assert_eq!(user.cmp(&user), std::cmp::Ordering::Equal);

        let db = ServerDatabase {
            name: "db".to_string(),
            db_type: DbType::Memory,
            role: DbUserRole::Admin,
            size: 0,
            backup: 0,
        };

        assert_eq!(db.cmp(&db), std::cmp::Ordering::Equal);

        let status = UserStatus {
            name: "user".to_string(),
            login: false,
        };

        assert_eq!(status.cmp(&status), std::cmp::Ordering::Equal);
    }

    #[test]
    fn derived_from_serde() {
        let cs1 = ClusterStatus {
            address: "localhost".to_string(),
            status: true,
            leader: false,
            term: 0,
            commit: 0,
        };
        let data = serde_json::to_string(&cs1).unwrap();
        let cs2: ClusterStatus = serde_json::from_str(&data).unwrap();
        assert_eq!(cs1, cs2);
    }
}
