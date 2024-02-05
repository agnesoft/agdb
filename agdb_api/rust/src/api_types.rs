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
    use agdb::SelectIndexesQuery;

    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", DbType::Memory);
        format!("{:?}", DbUserRole::Admin);
        format!(
            "{:?}",
            DbUser {
                user: "user".to_string(),
                role: DbUserRole::Admin
            }
        );
        format!(
            "{:?}",
            ServerDatabase {
                name: "db".to_string(),
                db_type: DbType::Memory,
                role: DbUserRole::Admin,
                size: 0,
                backup: 0
            }
        );
        format!(
            "{:?}",
            UserStatus {
                name: "user".to_string()
            }
        );
        format!(
            "{:?}",
            QueryAudit {
                timestamp: 0,
                user: "user".to_string(),
                query: QueryType::SelectIndexes(SelectIndexesQuery {})
            }
        );
        format!("{:?}", DbAudit(vec![]));
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
        };
        let other = UserStatus {
            name: "user2".to_string(),
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
        };

        assert_eq!(status.cmp(&status), std::cmp::Ordering::Equal);
    }
}
