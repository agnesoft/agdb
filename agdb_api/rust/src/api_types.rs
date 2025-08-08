use agdb::AgdbDeSerialize;
use agdb::DbError;
use agdb::DbValue;
use agdb::QueryResult;
use agdb::QueryType;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    AgdbDeSerialize,
)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
#[serde(rename_all = "snake_case")]
pub enum DbType {
    #[default]
    Memory,
    Mapped,
    File,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    AgdbDeSerialize,
)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
#[serde(rename_all = "snake_case")]
pub enum DbResource {
    #[default]
    All,
    Db,
    Audit,
    Backup,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct DbUser {
    pub username: String,
    pub role: DbUserRole,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    AgdbDeSerialize,
)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
#[serde(rename_all = "snake_case")]
pub enum DbUserRole {
    #[default]
    Admin,
    Write,
    Read,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct ChangePassword {
    pub password: String,
    pub new_password: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct ClusterStatus {
    pub address: String,
    pub status: bool,
    pub leader: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct AdminStatus {
    pub uptime: u64,
    pub dbs: u64,
    pub users: u64,
    pub logged_in_users: u64,
    pub size: u64,
}

#[derive(Clone, Deserialize, Serialize, ToSchema, AgdbDeSerialize)]
pub struct Queries(pub Vec<QueryType>);

#[derive(Serialize, ToSchema)]
pub struct QueriesResults(pub Vec<QueryResult>);

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct QueryAudit {
    pub timestamp: u64,
    pub username: String,
    pub query: QueryType,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct DbAudit(pub Vec<QueryAudit>);

#[derive(Debug, Default, Deserialize, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct ServerDatabase {
    pub db: String,
    pub owner: String,
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
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct UserStatus {
    pub username: String,
    pub login: bool,
    pub admin: bool,
}

impl From<&str> for DbType {
    fn from(value: &str) -> Self {
        match value {
            "mapped" => Self::Mapped,
            "file" => Self::File,
            _ => Self::Memory,
        }
    }
}

impl From<&str> for DbResource {
    fn from(value: &str) -> Self {
        match value {
            "db" => Self::Db,
            "audit" => Self::Audit,
            "backup" => Self::Backup,
            _ => Self::All,
        }
    }
}

impl From<&str> for DbUserRole {
    fn from(value: &str) -> Self {
        match value {
            "admin" => Self::Admin,
            "write" => Self::Write,
            _ => Self::Read,
        }
    }
}

impl TryFrom<DbValue> for DbType {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(Self::from(value.to_string().as_str()))
    }
}

impl TryFrom<DbValue> for DbResource {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(Self::from(value.to_string().as_str()))
    }
}

impl TryFrom<DbValue> for DbUserRole {
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

impl From<DbResource> for DbValue {
    fn from(value: DbResource) -> Self {
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
        Self::from(value.to_string().as_str())
    }
}

impl From<DbUserRole> for DbValue {
    fn from(value: DbUserRole) -> Self {
        value.to_string().into()
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
                username: "user".to_string(),
                role: DbUserRole::Admin
            }
        );
        let _ = format!(
            "{:?}",
            ServerDatabase {
                db: "db".to_string(),
                owner: "owner".to_string(),
                db_type: DbType::Memory,
                role: DbUserRole::Admin,
                size: 0,
                backup: 0
            }
        );
        let _ = format!(
            "{:?}",
            UserStatus {
                username: "user".to_string(),
                login: true,
                admin: false
            }
        );
        let _ = format!(
            "{:?}",
            QueryAudit {
                timestamp: 0,
                username: "user".to_string(),
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
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        let query_audit = QueryAudit {
            timestamp: 0,
            username: "user".to_string(),
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
            username: "user".to_string(),
            role: DbUserRole::Admin,
        };
        let other = DbUser {
            username: "user2".to_string(),
            role: DbUserRole::Admin,
        };
        assert!(user < other);
        let db = ServerDatabase {
            db: "db".to_string(),
            owner: "owner".to_string(),
            db_type: DbType::Memory,
            role: DbUserRole::Admin,
            size: 0,
            backup: 0,
        };
        let other = ServerDatabase {
            db: "db2".to_string(),
            owner: "owner".to_string(),
            db_type: DbType::Memory,
            role: DbUserRole::Admin,
            size: 0,
            backup: 0,
        };
        assert!(db < other);
        let status = UserStatus {
            username: "user".to_string(),
            login: true,
            admin: false,
        };
        let other = UserStatus {
            username: "user2".to_string(),
            login: true,
            admin: false,
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
            username: "user".to_string(),
            role: DbUserRole::Admin,
        };

        assert_eq!(user.cmp(&user), std::cmp::Ordering::Equal);

        let db = ServerDatabase {
            db: "db".to_string(),
            owner: "owner".to_string(),
            db_type: DbType::Memory,
            role: DbUserRole::Admin,
            size: 0,
            backup: 0,
        };

        assert_eq!(db.cmp(&db), std::cmp::Ordering::Equal);

        let status = UserStatus {
            username: "user".to_string(),
            login: false,
            admin: false,
        };

        assert_eq!(status.cmp(&status), std::cmp::Ordering::Equal);
    }

    #[test]
    fn derived_from_serde() {
        let cs1 = ClusterStatus {
            address: "localhost".to_string(),
            status: true,
            leader: false,
        };
        let data = serde_json::to_string(&cs1).unwrap();
        let cs2: ClusterStatus = serde_json::from_str(&data).unwrap();
        assert_eq!(cs1, cs2);
    }

    #[test]
    fn db_resource() {
        let db_resource = DbResource::from("db");
        assert_eq!(db_resource, DbResource::Db);
        let db_resource = DbResource::from("audit");
        assert_eq!(db_resource, DbResource::Audit);
        let db_resource = DbResource::from("backup");
        assert_eq!(db_resource, DbResource::Backup);
        let db_resource = DbResource::from("all");
        assert_eq!(db_resource, DbResource::All);

        let db_value = DbValue::from(DbResource::All);
        let db_resource: DbResource = db_value.try_into().unwrap();
        assert_eq!(db_resource, DbResource::All);
    }

    #[test]
    fn db_user_role() {
        let db_role = DbUserRole::from("admin");
        assert_eq!(db_role, DbUserRole::Admin);
        let db_role = DbUserRole::from("write");
        assert_eq!(db_role, DbUserRole::Write);
        let db_role = DbUserRole::from("read");
        assert_eq!(db_role, DbUserRole::Read);

        let db_value = DbValue::from(DbUserRole::Admin);
        let db_role: DbUserRole = db_value.try_into().unwrap();
        assert_eq!(db_role, DbUserRole::Admin);
    }

    #[test]
    fn db_type() {
        let db_type = DbType::from("mapped");
        assert_eq!(db_type, DbType::Mapped);
        let db_type = DbType::from("file");
        assert_eq!(db_type, DbType::File);
        let db_type = DbType::from("memory");
        assert_eq!(db_type, DbType::Memory);

        let db_value = DbValue::from(DbType::Memory);
        let db_type: DbType = db_value.try_into().unwrap();
        assert_eq!(db_type, DbType::Memory);
    }
}
