use crate::action::ClusterAction;
use crate::config::Config;
use crate::db_pool::DbName;
use crate::password::Password;
use crate::raft::Log;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::AgdbSerialize;
use agdb::Comparison;
use agdb::CountComparison;
use agdb::Db;
use agdb::DbId;
use agdb::DbType;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::QueryResult;
use agdb::SearchQuery;
use agdb::StorageData;
use agdb::Transaction;
use agdb_api::DbType as ApiDbType;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use agdb_api::UserStatus;
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(DbType)]
pub(crate) struct ServerUser {
    pub(crate) db_id: Option<DbId>,
    pub(crate) username: String,
    pub(crate) password: Vec<u8>,
    pub(crate) salt: Vec<u8>,
    pub(crate) token: String,
}

#[derive(Default, DbType)]
pub(crate) struct Database {
    pub(crate) db_id: Option<DbId>,
    pub(crate) db: String,
    pub(crate) owner: String,
    pub(crate) db_type: ApiDbType,
    pub(crate) backup: u64,
}

impl Database {
    pub(crate) fn name(&self) -> DbName {
        DbName {
            owner: self.owner.clone(),
            db: self.db.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ServerDb(pub(crate) Arc<RwLock<Db>>);

const ADMIN: &str = "admin";
const CLUSTER_LOG: &str = "cluster_log";
const COMMITTED: &str = "committed";
const DB: &str = "db";
const DBS: &str = "dbs";
const EXECUTED: &str = "executed";
const NAME: &str = "name";
const OWNER: &str = "owner";
const ROLE: &str = "role";
const TOKEN: &str = "token";
const USERS: &str = "users";
const USERNAME: &str = "username";
const SERVER_DB_FILE: &str = "agdb_server.agdb";

pub(crate) async fn new(config: &Config) -> ServerResult<ServerDb> {
    std::fs::create_dir_all(&config.data_dir)?;
    let db_name = format!("{}/{}", config.data_dir, SERVER_DB_FILE);
    let db = ServerDb::new(&db_name)?;

    let admin = if let Some(admin_id) = db.find_user_id(&config.admin).await? {
        admin_id
    } else {
        let admin_password = Password::create(&config.admin, &config.admin);
        let admin = ServerUser {
            db_id: None,
            username: config.admin.clone(),
            password: admin_password.password.to_vec(),
            salt: admin_password.user_salt.to_vec(),
            token: String::new(),
        };
        db.insert_user(admin).await?
    };

    db.0.write()
        .await
        .exec_mut(QueryBuilder::insert().aliases(ADMIN).ids(admin).query())?;

    Ok(db)
}

impl ServerDb {
    fn new(name: &str) -> ServerResult<Self> {
        let mut db = Db::new(name)?;

        db.transaction_mut(|t| -> ServerResult<()> {
            let indexes: Vec<String> = t.exec(QueryBuilder::select().indexes().query())?.elements
                [0]
            .values
            .iter()
            .map(|kv| kv.key.to_string())
            .collect();

            if !indexes.iter().any(|i| i == USERNAME) {
                t.exec_mut(QueryBuilder::insert().index(USERNAME).query())?;
            }

            if !indexes.iter().any(|i| i == TOKEN) {
                t.exec_mut(QueryBuilder::insert().index(TOKEN).query())?;
            }

            if !indexes.iter().any(|i| i == EXECUTED) {
                t.exec_mut(QueryBuilder::insert().index(EXECUTED).query())?;
            }

            if !indexes.iter().any(|i| i == COMMITTED) {
                t.exec_mut(QueryBuilder::insert().index(COMMITTED).query())?;
            }

            if t.exec(QueryBuilder::select().ids(USERS).query()).is_err() {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(USERS).query())?;
            }

            if t.exec(QueryBuilder::select().ids(DBS).query()).is_err() {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(DBS).query())?;
            }

            if t.exec(QueryBuilder::select().ids(CLUSTER_LOG).query())
                .is_err()
            {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(CLUSTER_LOG).query())?;
            }

            // Migration to new Database struct introduced in 0.10.0. Remove in 0.12.0.
            let dbs: Vec<(DbId, String, String)> = t
                .exec(
                    QueryBuilder::select()
                        .values(NAME)
                        .search()
                        .from(DBS)
                        .where_()
                        .distance(CountComparison::Equal(2))
                        .and()
                        .keys(NAME)
                        .query(),
                )?
                .elements
                .iter()
                .filter_map(|e| {
                    e.values[0]
                        .value
                        .to_string()
                        .split_once('/')
                        .map(|(owner, db)| (e.id, owner.to_string(), db.to_string()))
                })
                .collect::<Vec<(DbId, String, String)>>();

            for (db_id, owner, db) in dbs {
                t.exec_mut(
                    QueryBuilder::insert()
                        .values([[(OWNER, owner).into(), (DB, db).into()]])
                        .ids(db_id)
                        .query(),
                )?;
                t.exec_mut(QueryBuilder::remove().values("name").ids(db_id).query())?;
            }

            Ok(())
        })?;

        Ok(Self(Arc::new(RwLock::new(db))))
    }

    pub(crate) async fn cluster_log(&self) -> ServerResult<(u64, u64, u64)> {
        self.0.write().await.transaction_mut(|t| {
            if let Some(e) = t
                .exec(
                    QueryBuilder::select()
                        .values(["index", "term"])
                        .search()
                        .depth_first()
                        .from(CLUSTER_LOG)
                        .limit(1)
                        .where_()
                        .distance(CountComparison::Equal(2))
                        .query(),
                )?
                .elements
                .first()
            {
                let commit = if let Some(c) = t
                    .exec(
                        QueryBuilder::select()
                            .values("index")
                            .search()
                            .depth_first()
                            .from(CLUSTER_LOG)
                            .limit(1)
                            .where_()
                            .distance(CountComparison::Equal(2))
                            .and()
                            .not()
                            .keys(COMMITTED)
                            .query(),
                    )?
                    .elements
                    .first()
                {
                    c.values[0].value.to_u64()?
                } else {
                    0
                };

                return Ok((
                    e.values[0].value.to_u64()?,
                    e.values[1].value.to_u64()?,
                    commit,
                ));
            }

            Ok((0, 0, 0))
        })
    }

    pub(crate) async fn log_committed(&self, log_id: DbId) -> ServerResult<()> {
        self.0
            .write()
            .await
            .exec_mut(QueryBuilder::remove().values(COMMITTED).ids(log_id).query())?;
        Ok(())
    }

    pub(crate) async fn log_executed(&self, log_id: DbId) -> ServerResult<()> {
        self.0
            .write()
            .await
            .exec_mut(QueryBuilder::remove().values(EXECUTED).ids(log_id).query())?;
        Ok(())
    }

    pub(crate) async fn db_count(&self) -> ServerResult<u64> {
        Ok(self
            .0
            .read()
            .await
            .exec(QueryBuilder::select().edge_count_from().ids(DBS).query())?
            .elements[0]
            .values[0]
            .value
            .to_u64()?)
    }

    pub(crate) async fn db_users(&self, db: DbId) -> ServerResult<Vec<DbUser>> {
        let mut users = vec![];

        self.0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .search()
                    .depth_first()
                    .to(db)
                    .where_()
                    .distance(CountComparison::LessThanOrEqual(2))
                    .and()
                    .where_()
                    .keys("role")
                    .or()
                    .keys("password")
                    .query(),
            )?
            .elements
            .into_iter()
            .for_each(|e| {
                if e.id.0 < 0 {
                    users.push(DbUser {
                        username: String::new(),
                        role: (&e.values[0].value).into(),
                    });
                } else {
                    users.last_mut().unwrap().username = e.values[0].value.to_string();
                }
            });

        Ok(users)
    }

    pub(crate) async fn dbs(&self) -> ServerResult<Vec<Database>> {
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<Database>()
                    .search()
                    .from(DBS)
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .query(),
            )?
            .try_into()?)
    }

    pub(crate) async fn find_user_db_id(
        &self,
        user: DbId,
        owner: &str,
        db: &str,
    ) -> ServerResult<Option<DbId>> {
        Ok(self
            .0
            .read()
            .await
            .exec(find_user_db_query(user, owner, db))?
            .elements
            .first()
            .map(|e| e.id))
    }

    pub(crate) async fn find_user_id(&self, username: &str) -> ServerResult<Option<DbId>> {
        Ok(self
            .0
            .read()
            .await
            .exec(find_user_query(username))?
            .elements
            .first()
            .map(|e| e.id))
    }

    pub(crate) async fn insert_db(&self, owner: DbId, db: Database) -> ServerResult<DbId> {
        self.0.write().await.transaction_mut(|t| {
            let id = t
                .exec_mut(QueryBuilder::insert().element(&db).query())?
                .elements[0]
                .id;
            t.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from([QueryId::from(owner), DBS.into()])
                    .to(id)
                    .values([vec![(ROLE, DbUserRole::Admin).into()], vec![]])
                    .query(),
            )?;
            Ok(id)
        })
    }

    pub(crate) async fn insert_db_user(
        &self,
        db: DbId,
        user: DbId,
        role: DbUserRole,
    ) -> ServerResult<()> {
        self.0.write().await.transaction_mut(|t| {
            let existing_role = t.exec(
                QueryBuilder::search()
                    .from(user)
                    .to(db)
                    .limit(1)
                    .where_()
                    .keys(ROLE)
                    .query(),
            )?;

            if existing_role.result == 1 {
                t.exec_mut(
                    QueryBuilder::insert()
                        .values([[(ROLE, role).into()]])
                        .ids(existing_role)
                        .query(),
                )?;
            } else {
                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from(user)
                        .to(db)
                        .values_uniform([(ROLE, role).into()])
                        .query(),
                )?;
            }
            Ok(())
        })
    }

    pub(crate) async fn insert_user(&self, user: ServerUser) -> ServerResult<DbId> {
        self.0.write().await.transaction_mut(|t| {
            let id = t
                .exec_mut(QueryBuilder::insert().element(&user).query())?
                .elements[0]
                .id;
            t.exec_mut(QueryBuilder::insert().edges().from(USERS).to(id).query())?;
            Ok(id)
        })
    }

    pub(crate) async fn is_admin(&self, token: &str) -> ServerResult<bool> {
        Ok(self
            .0
            .read()
            .await
            .exec(QueryBuilder::select().values(TOKEN).ids(ADMIN).query())?
            .elements[0]
            .values[0]
            .value
            .string()?
            == token)
    }

    pub(crate) async fn is_db_admin(&self, user: DbId, db: DbId) -> ServerResult<bool> {
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::search()
                    .from(user)
                    .to(db)
                    .limit(1)
                    .where_()
                    .distance(CountComparison::LessThanOrEqual(2))
                    .and()
                    .key(ROLE)
                    .value(Comparison::Equal(DbUserRole::Admin.into()))
                    .query(),
            )?
            .result
            == 1)
    }

    pub(crate) async fn append_log(&self, log: &Log<ClusterAction>) -> ServerResult<DbId> {
        self.0.write().await.transaction_mut(
            |t: &mut agdb::TransactionMut<'_, agdb::FileStorageMemoryMapped>| {
                let log_id = t
                    .exec_mut(QueryBuilder::insert().element(log).query())?
                    .elements[0]
                    .id;
                t.exec_mut(
                    QueryBuilder::insert()
                        .values([[(COMMITTED, false).into(), (EXECUTED, false).into()]])
                        .ids(log_id)
                        .query(),
                )?;
                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from(CLUSTER_LOG)
                        .to(log_id)
                        .query(),
                )?;
                Ok(log_id)
            },
        )
    }

    pub(crate) async fn logs_unexecuted(
        &self,
        index: u64,
    ) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.logs_until(index, EXECUTED).await
    }

    pub(crate) async fn logs_uncommitted(
        &self,
        index: u64,
    ) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.logs_until(index, COMMITTED).await
    }

    async fn logs_until(&self, index: u64, label: &str) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.0.read().await.transaction(|t| {
            let mut log_ids: Vec<(u64, DbId)> = t
                .exec(
                    QueryBuilder::select()
                        .values("index")
                        .search()
                        .index(label)
                        .value(false)
                        .query(),
                )?
                .elements
                .into_iter()
                .filter_map(|e| {
                    let log_index = e.values[0].value.to_u64().unwrap_or_default();

                    if log_index <= index {
                        Some((log_index, e.id))
                    } else {
                        None
                    }
                })
                .collect();
            log_ids.sort_by_key(|l| l.0);
            logs(t, log_ids.into_iter().map(|l| l.1).collect())
        })
    }

    pub(crate) async fn remove_uncommitted_logs(&self, from_index: u64) -> ServerResult<()> {
        self.0.write().await.transaction_mut(|t| {
            let logs: Vec<DbId> = t
                .exec(
                    QueryBuilder::select()
                        .values("index")
                        .search()
                        .index(COMMITTED)
                        .value(false)
                        .query(),
                )?
                .elements
                .into_iter()
                .filter_map(|e| {
                    let index = e.values[0].value.to_u64().unwrap_or_default();

                    if index >= from_index {
                        Some(e.id)
                    } else {
                        None
                    }
                })
                .collect();

            t.exec_mut(QueryBuilder::remove().ids(logs).query())
        })?;

        Ok(())
    }

    pub(crate) async fn logs_since(
        &self,
        from_index: u64,
    ) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.0.read().await.transaction(|t| {
            let log_count = t
                .exec(
                    QueryBuilder::select()
                        .edge_count_from()
                        .ids(CLUSTER_LOG)
                        .query(),
                )?
                .elements[0]
                .values[0]
                .value
                .to_u64()?;
            let mut log_ids = t
                .exec(
                    QueryBuilder::search()
                        .depth_first()
                        .from(CLUSTER_LOG)
                        .limit(log_count - from_index)
                        .where_()
                        .distance(CountComparison::Equal(2))
                        .query(),
                )?
                .ids();

            log_ids.reverse();
            logs(t, log_ids)
        })
    }

    pub(crate) async fn remove_db(&self, user: DbId, owner: &str, db: &str) -> ServerResult<()> {
        self.0.write().await.transaction_mut(|t| {
            let db_id = t
                .exec(find_user_db_query(user, owner, db))?
                .elements
                .first()
                .ok_or(db_not_found(db))?
                .id;

            t.exec_mut(QueryBuilder::remove().ids(db_id).query())?;

            Ok(())
        })
    }

    pub(crate) async fn remove_db_user(&self, db: DbId, user: DbId) -> ServerResult<()> {
        self.0.write().await.exec_mut(
            QueryBuilder::remove()
                .search()
                .from(user)
                .to(db)
                .limit(1)
                .where_()
                .keys("role")
                .query(),
        )?;
        Ok(())
    }

    pub(crate) async fn remove_user(&self, username: &str) -> ServerResult<Vec<DbName>> {
        let user = self.user_id(username).await?;
        let mut ids = vec![user];
        let mut dbs = vec![];

        self.user_dbs(user)
            .await?
            .into_iter()
            .for_each(|(_role, db)| {
                if db.owner == username {
                    ids.push(db.db_id.unwrap());
                    dbs.push(DbName {
                        owner: db.owner,
                        db: db.db,
                    });
                }
            });

        self.0
            .write()
            .await
            .exec_mut(QueryBuilder::remove().ids(ids).query())?;

        Ok(dbs)
    }

    pub(crate) async fn reset_tokens(&self) -> ServerResult<()> {
        self.0.write().await.exec_mut(
            QueryBuilder::insert()
                .values_uniform([(TOKEN, String::new()).into()])
                .search()
                .from(USERS)
                .where_()
                .distance(CountComparison::Equal(2))
                .and()
                .key(TOKEN)
                .value(Comparison::NotEqual(String::new().into()))
                .and()
                .not()
                .ids(ADMIN)
                .query(),
        )?;
        Ok(())
    }

    pub(crate) async fn save_db(&self, db: &Database) -> ServerResult<()> {
        self.0
            .write()
            .await
            .exec_mut(QueryBuilder::insert().element(db).query())?;
        Ok(())
    }

    pub(crate) async fn save_token(&self, user: DbId, token: &str) -> ServerResult<()> {
        self.0.write().await.exec_mut(
            QueryBuilder::insert()
                .values([[(TOKEN, token).into()]])
                .ids(user)
                .query(),
        )?;
        Ok(())
    }

    pub(crate) async fn save_user(&self, user: ServerUser) -> ServerResult<()> {
        self.0
            .write()
            .await
            .exec_mut(QueryBuilder::insert().element(&user).query())?;
        Ok(())
    }

    pub(crate) async fn user(&self, username: &str) -> ServerResult<ServerUser> {
        self.0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<ServerUser>()
                    .ids(find_user_query(username))
                    .query(),
            )?
            .try_into()
            .map_err(|_| user_not_found(username))
    }

    pub(crate) async fn user_name(&self, id: DbId) -> ServerResult<String> {
        Ok(self
            .0
            .read()
            .await
            .exec(QueryBuilder::select().values(USERNAME).ids(id).query())?
            .elements[0]
            .values[0]
            .value
            .to_string())
    }

    pub(crate) async fn user_by_id(&self, id: DbId) -> ServerResult<ServerUser> {
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<ServerUser>()
                    .ids(id)
                    .query(),
            )?
            .try_into()?)
    }

    pub(crate) async fn user_db(
        &self,
        user: DbId,
        owner: &str,
        db: &str,
    ) -> ServerResult<Database> {
        self.0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<Database>()
                    .ids(find_user_db_query(user, owner, db))
                    .query(),
            )?
            .try_into()
            .map_err(|_| db_not_found(db))
    }

    pub(crate) async fn user_db_id(&self, user: DbId, owner: &str, db: &str) -> ServerResult<DbId> {
        self.find_user_db_id(user, owner, db)
            .await?
            .ok_or(db_not_found(db))
    }

    pub(crate) async fn user_db_role(
        &self,
        user: DbId,
        owner: &str,
        db: &str,
    ) -> ServerResult<DbUserRole> {
        Ok((&self
            .0
            .read()
            .await
            .transaction(|t| -> Result<QueryResult, ServerError> {
                let db_id = t
                    .exec(find_user_db_query(user, owner, db))?
                    .elements
                    .first()
                    .ok_or(db_not_found(db))?
                    .id;
                Ok(t.exec(
                    QueryBuilder::select()
                        .search()
                        .depth_first()
                        .from(user)
                        .to(db_id)
                        .limit(1)
                        .where_()
                        .distance(CountComparison::LessThanOrEqual(2))
                        .and()
                        .keys("role")
                        .query(),
                )?)
            })?
            .elements[0]
            .values[0]
            .value)
            .into())
    }

    pub(crate) async fn user_dbs(&self, user: DbId) -> ServerResult<Vec<(DbUserRole, Database)>> {
        let mut dbs = vec![];

        let elements = self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .depth_first()
                            .from(user)
                            .where_()
                            .distance(CountComparison::Equal(1))
                            .or()
                            .distance(CountComparison::Equal(2))
                            .query(),
                    )
                    .query(),
            )?
            .elements;

        for e in elements {
            if e.id.0 < 0 {
                dbs.push(((&e.values[0].value).into(), Database::default()));
            } else {
                dbs.last_mut().unwrap().1 = Database::from_db_element(&e)?;
            }
        }

        Ok(dbs)
    }

    pub(crate) async fn user_count(&self) -> ServerResult<u64> {
        Ok(self
            .0
            .read()
            .await
            .exec(QueryBuilder::select().edge_count_from().ids(USERS).query())?
            .elements[0]
            .values[0]
            .value
            .to_u64()?)
    }

    pub(crate) async fn user_id(&self, username: &str) -> ServerResult<DbId> {
        self.find_user_id(username)
            .await?
            .ok_or(user_not_found(username))
    }

    pub(crate) async fn user_token(&self, user: DbId) -> ServerResult<String> {
        Ok(self
            .0
            .read()
            .await
            .exec(QueryBuilder::select().values(TOKEN).ids(user).query())?
            .elements[0]
            .values[0]
            .value
            .to_string())
    }

    pub(crate) async fn user_token_count(&self) -> ServerResult<u64> {
        self.0.read().await.transaction(|t| -> ServerResult<u64> {
            let empty_tokens = if t
                .exec(QueryBuilder::search().index("token").value("").query())?
                .result
                == 0
            {
                0
            } else {
                1
            };
            let tokens = t.exec(QueryBuilder::select().indexes().query())?.elements[0].values[1]
                .value
                .to_u64()?;
            Ok(tokens - empty_tokens)
        })
    }

    pub(crate) async fn user_token_id(&self, token: &str) -> ServerResult<DbId> {
        Ok(self
            .0
            .read()
            .await
            .exec(QueryBuilder::search().index(TOKEN).value(token).query())?
            .elements
            .first()
            .ok_or(token_not_found(token))?
            .id)
    }

    pub(crate) async fn user_statuses(&self) -> ServerResult<Vec<UserStatus>> {
        let admin_id = self
            .0
            .read()
            .await
            .exec(QueryBuilder::select().aliases().ids(ADMIN).query())?
            .elements[0]
            .id;

        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .values([USERNAME, TOKEN])
                    .search()
                    .from(USERS)
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .query(),
            )?
            .elements
            .into_iter()
            .map(|e| UserStatus {
                username: e.values[0].value.to_string(),
                login: !e.values[1].value.to_string().is_empty(),
                admin: e.id == admin_id,
            })
            .collect())
    }
}

fn db_not_found(name: &str) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("db not found: {name}"))
}

fn find_user_db_query(user: DbId, owner: &str, db: &str) -> SearchQuery {
    QueryBuilder::search()
        .depth_first()
        .from(user)
        .limit(1)
        .where_()
        .distance(CountComparison::Equal(2))
        .and()
        .key(OWNER)
        .value(Comparison::Equal(owner.into()))
        .and()
        .key(DB)
        .value(Comparison::Equal(db.into()))
        .query()
}

fn find_user_query(username: &str) -> SearchQuery {
    QueryBuilder::search()
        .index(USERNAME)
        .value(username)
        .query()
}

fn token_not_found(token: &str) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("token not found: {token}"))
}

fn user_not_found(name: &str) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("user not found: {name}"))
}

fn logs<T: StorageData>(
    t: &Transaction<T>,
    log_ids: Vec<DbId>,
) -> ServerResult<Vec<Log<ClusterAction>>> {
    Ok(t.exec(
        QueryBuilder::select()
            .elements::<Log<ClusterAction>>()
            .ids(log_ids)
            .query(),
    )?
    .try_into()?)
}

impl DbType for Log<ClusterAction> {
    type ValueType = Log<ClusterAction>;

    fn db_id(&self) -> Option<QueryId> {
        self.db_id.map(|id| id.into())
    }

    fn db_keys() -> Vec<agdb::DbValue> {
        vec!["index".into(), "term".into(), "data".into()]
    }

    fn from_db_element(element: &agdb::DbElement) -> Result<Self::ValueType, agdb::DbError> {
        let index = element.values[0].value.to_u64()?;
        let term = element.values[1].value.to_u64()?;
        let data = element.values[2].value.bytes()?;

        Ok(Log {
            db_id: Some(element.id),
            index,
            term,
            data: <ClusterAction as AgdbSerialize>::deserialize(data)?,
        })
    }

    fn to_db_values(&self) -> Vec<agdb::DbKeyValue> {
        vec![
            ("index", self.index).into(),
            ("term", self.term).into(),
            ("data", self.data.serialize()).into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFile {
        filename: &'static str,
    }

    impl TestFile {
        fn new(filename: &'static str) -> Self {
            let _ = std::fs::remove_file(filename);
            Self { filename }
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(self.filename);
        }
    }

    #[tokio::test]
    async fn db_upgrade() -> ServerResult {
        let file = TestFile::new("test_db.db");
        let _dot_file = TestFile::new(".test_db.db");
        let mut db = Db::new(file.filename)?;
        db.transaction_mut(|t| -> ServerResult {
            t.exec_mut(QueryBuilder::insert().nodes().aliases(DBS).query())?;
            let user_db = t.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .values([
                        vec![
                            ("name", "user/db1").into(),
                            ("db_type", DbType::Memory).into(),
                            ("backup", 0).into(),
                        ],
                        vec![
                            ("owner", "user").into(),
                            ("db", "db2").into(),
                            ("db_type", DbType::Memory).into(),
                            ("backup", 0).into(),
                        ],
                    ])
                    .query(),
            )?;
            t.exec_mut(QueryBuilder::insert().edges().from(DBS).to(user_db).query())?;
            Ok(())
        })?;

        let db = ServerDb::new("test_db.db")?;
        let dbs = db.dbs().await?;
        assert_eq!(dbs.len(), 2);
        assert_eq!(dbs[0].db, "db2");
        assert_eq!(dbs[0].owner, "user");
        assert_eq!(dbs[0].db_type, DbType::Memory);
        assert_eq!(dbs[0].backup, 0);
        assert_eq!(dbs[1].db, "db1");
        assert_eq!(dbs[1].owner, "user");
        assert_eq!(dbs[1].db_type, DbType::Memory);
        assert_eq!(dbs[1].backup, 0);

        Ok(())
    }
}
