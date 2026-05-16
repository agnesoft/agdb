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
use agdb::SelectValuesQuery;
use agdb_api::DbKind;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use agdb_api::UserSession;
use agdb_api::UserStatus;
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

#[derive(DbType)]
pub(crate) struct ServerUser {
    pub(crate) db_id: Option<DbId>,
    pub(crate) username: String,
    pub(crate) password: Vec<u8>,
    pub(crate) salt: Vec<u8>,
}

#[derive(DbType)]
pub(crate) struct UserToken {
    pub(crate) db_id: Option<DbId>,
    pub(crate) token: String,
    pub(crate) created: u64,
    pub(crate) agent: String,
}

#[derive(Default, DbType)]
pub(crate) struct Database {
    pub(crate) db_id: Option<DbId>,
    pub(crate) db: String,
    pub(crate) owner: String,
    pub(crate) db_type: DbKind,
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
pub(crate) struct ServerDb {
    pub(crate) db: Arc<RwLock<Db>>,
    pub(crate) token_expiry_seconds: u64,
}

const ADMIN: &str = "admin";
const AGENT: &str = "agent";
const CREATED: &str = "created";
const DB: &str = "db";
const DBS: &str = "dbs";
const OWNER: &str = "owner";
const ROLE: &str = "role";
const TOKEN: &str = "token";
const USERS: &str = "users";
const USERNAME: &str = "username";
const SERVER_DB_FILE: &str = "agdb_server.agdb";

pub(crate) async fn new(
    config: &Config,
    mut shutdown_receiver: broadcast::Receiver<()>,
) -> ServerResult<ServerDb> {
    std::fs::create_dir_all(&config.data_dir)?;
    let db_name = format!("{}/{}", config.data_dir, SERVER_DB_FILE);
    let db = ServerDb::new(&db_name, config.token_expiry_seconds)?;

    let admin = if let Some(admin_id) = db.find_user_id(&config.admin).await? {
        admin_id
    } else {
        let admin_password = Password::create(&config.admin, &config.admin);
        let admin = ServerUser {
            db_id: None,
            username: config.admin.clone(),
            password: admin_password.password.to_vec(),
            salt: admin_password.user_salt.to_vec(),
        };
        db.insert_user(admin).await?
    };

    db.db
        .write()
        .await
        .exec_mut(QueryBuilder::insert().aliases(ADMIN).ids(admin).query())?;

    let expiry = config.token_expiry_seconds;

    if let Err(e) = db.remove_expired_tokens().await {
        tracing::warn!("Token cleanup on startup failed: {e:?}");
    }

    let cleanup_db = db.clone();
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_secs(expiry));
        timer.tick().await;
        loop {
            tokio::select! {
                _ = timer.tick() => {
                    if let Err(e) = cleanup_db
                        .remove_expired_tokens()
                        .await
                    {
                        tracing::warn!("Token cleanup failed: {e:?}");
                    }
                }
                _ = shutdown_receiver.recv() => {
                    break;
                }
            }
        }
    });

    Ok(db)
}

impl ServerDb {
    fn new(name: &str, token_expiry_seconds: u64) -> ServerResult<Self> {
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

            if t.exec(QueryBuilder::select().ids(USERS).query()).is_err() {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(USERS).query())?;
            }

            if t.exec(QueryBuilder::select().ids(DBS).query()).is_err() {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(DBS).query())?;
            }

            // Remove all old user tokens still on users from previous db version
            if t.exec(QueryBuilder::select().values(TOKEN).ids(ADMIN).query())
                .is_ok()
            {
                t.exec_mut(QueryBuilder::remove().index(TOKEN).query())?;
                t.exec_mut(
                    QueryBuilder::remove()
                        .values(TOKEN)
                        .search()
                        .from(USERS)
                        .where_()
                        .neighbor()
                        .query(),
                )?;
                t.exec_mut(QueryBuilder::insert().index(TOKEN).query())?;
            }

            Ok(())
        })?;

        Ok(Self {
            db: Arc::new(RwLock::new(db)),
            token_expiry_seconds,
        })
    }

    pub(crate) async fn db_count(&self) -> ServerResult<u64> {
        Ok(self
            .db
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

        self.db
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
                    users
                        .last_mut()
                        .expect("Expected at least one user")
                        .username = e.values[0].value.to_string();
                }
            });

        Ok(users)
    }

    pub(crate) async fn dbs(&self) -> ServerResult<Vec<Database>> {
        Ok(self
            .db
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<Database>()
                    .search()
                    .from(DBS)
                    .where_()
                    .neighbor()
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
            .db
            .read()
            .await
            .exec(find_user_db_query(user, owner, db))?
            .elements
            .first()
            .map(|e| e.id))
    }

    pub(crate) async fn find_user_id(&self, username: &str) -> ServerResult<Option<DbId>> {
        Ok(self
            .db
            .read()
            .await
            .exec(
                QueryBuilder::search()
                    .index(USERNAME)
                    .value(username)
                    .query(),
            )?
            .elements
            .first()
            .map(|e| e.id))
    }

    pub(crate) async fn insert_db(&self, owner: DbId, db: Database) -> ServerResult<DbId> {
        self.db.write().await.transaction_mut(|t| {
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
        self.db.write().await.transaction_mut(|t| {
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
        self.db.write().await.transaction_mut(|t| {
            let id = t
                .exec_mut(QueryBuilder::insert().element(&user).query())?
                .elements[0]
                .id;
            t.exec_mut(QueryBuilder::insert().edges().from(USERS).to(id).query())?;
            Ok(id)
        })
    }

    pub(crate) async fn is_admin(&self, token: &str) -> ServerResult<bool> {
        self.db.read().await.transaction(|t| {
            let (id, created) = t
                .exec(
                    QueryBuilder::select()
                        .values(CREATED)
                        .search()
                        .index(TOKEN)
                        .value(token)
                        .query(),
                )?
                .elements
                .first()
                .map(|e| (e.id, e.values[0].value.to_u64().unwrap_or_default()))
                .ok_or_else(|| token_not_found(token))?;

            if created < token_expiry_limit(self.token_expiry_seconds) {
                return Ok(false);
            }

            Ok(t.exec(QueryBuilder::search().from(id).to(ADMIN).query())?
                .result
                != 0)
        })
    }

    pub(crate) async fn is_db_admin(&self, user: DbId, db: DbId) -> ServerResult<bool> {
        Ok(self
            .db
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
                    .value(DbUserRole::Admin)
                    .query(),
            )?
            .result
            == 1)
    }

    pub(crate) async fn remove_db(&self, user: DbId, owner: &str, db: &str) -> ServerResult<()> {
        self.db.write().await.transaction_mut(|t| {
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
        self.db.write().await.exec_mut(
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

        self.db
            .write()
            .await
            .exec_mut(QueryBuilder::remove().ids(ids).query())?;

        Ok(dbs)
    }

    pub(crate) async fn remove_all_tokens(&self) -> ServerResult<()> {
        self.db.write().await.transaction_mut(|t| {
            let users = t
                .exec(
                    QueryBuilder::search()
                        .from(USERS)
                        .where_()
                        .neighbor()
                        .and()
                        .not()
                        .ids(ADMIN)
                        .query(),
                )?
                .ids();

            for user in users {
                t.exec_mut(
                    QueryBuilder::remove()
                        .search()
                        .to(user)
                        .where_()
                        .neighbor()
                        .and()
                        .not()
                        .ids(USERS)
                        .query(),
                )?;
            }

            Ok(())
        })
    }

    pub(crate) async fn remove_tokens(&self, user: DbId) -> ServerResult<()> {
        self.db.write().await.transaction_mut(|t| {
            t.exec_mut(
                QueryBuilder::remove()
                    .search()
                    .to(user)
                    .where_()
                    .neighbor()
                    .and()
                    .not()
                    .ids(USERS)
                    .query(),
            )?;
            Ok(())
        })
    }

    pub(crate) async fn save_db(&self, db: &Database) -> ServerResult<()> {
        self.db
            .write()
            .await
            .exec_mut(QueryBuilder::insert().element(db).query())?;
        Ok(())
    }

    pub(crate) async fn save_token(
        &self,
        user: DbId,
        token: &str,
        agent: &str,
    ) -> ServerResult<DbId> {
        self.db
            .write()
            .await
            .transaction_mut(|t| -> ServerResult<DbId> {
                let token_id = t
                    .exec_mut(
                        QueryBuilder::insert()
                            .element(&UserToken {
                                db_id: None,
                                token: token.to_string(),
                                created: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs(),
                                agent: agent.to_string(),
                            })
                            .query(),
                    )?
                    .elements[0]
                    .id;

                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from(token_id)
                        .to(user)
                        .query(),
                )?;

                Ok(token_id)
            })
    }

    pub(crate) async fn save_user(&self, user: ServerUser) -> ServerResult<()> {
        self.db
            .write()
            .await
            .exec_mut(QueryBuilder::insert().element(&user).query())?;
        Ok(())
    }

    pub(crate) async fn user(&self, username: &str) -> ServerResult<ServerUser> {
        self.db
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<ServerUser>()
                    .search()
                    .index(USERNAME)
                    .value(username)
                    .query(),
            )?
            .try_into()
            .map_err(|_| user_not_found(username))
    }

    pub(crate) async fn user_name(&self, id: DbId) -> ServerResult<String> {
        Ok(self
            .db
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
            .db
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
        self.db
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
            .db
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
            .db
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .search()
                    .depth_first()
                    .from(user)
                    .where_()
                    .distance(1)
                    .or()
                    .neighbor()
                    .query(),
            )?
            .elements;

        for e in elements {
            if e.id.0 < 0 {
                dbs.push(((&e.values[0].value).into(), Database::default()));
            } else {
                dbs.last_mut().expect("Expected at least one database").1 =
                    Database::from_db_element(&e)?;
            }
        }

        Ok(dbs)
    }

    pub(crate) async fn user_count(&self) -> ServerResult<u64> {
        Ok(self
            .db
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

    pub(crate) async fn users_with_token(&self) -> ServerResult<u64> {
        Ok(self
            .db
            .read()
            .await
            .exec(
                QueryBuilder::search()
                    .from(USERS)
                    .where_()
                    .neighbor()
                    .and()
                    .not()
                    .edge_count_to(1)
                    .query(),
            )?
            .result as u64)
    }

    pub(crate) async fn user_id_from_token(&self, token: &str) -> ServerResult<DbId> {
        self.db.read().await.transaction(|t| {
            let (token_id, created) = t
                .exec(
                    QueryBuilder::select()
                        .values(CREATED)
                        .search()
                        .index(TOKEN)
                        .value(token)
                        .query(),
                )?
                .elements
                .first()
                .map(|e| (e.id, e.values[0].value.to_u64().unwrap_or_default()))
                .ok_or_else(|| token_not_found(token))?;

            if created < token_expiry_limit(self.token_expiry_seconds) {
                return Err(token_expired(token));
            }

            Ok(t.exec(
                QueryBuilder::search()
                    .from(token_id)
                    .limit(1)
                    .where_()
                    .neighbor()
                    .query(),
            )?
            .elements
            .first()
            .ok_or_else(|| token_not_found(token))?
            .id)
        })
    }

    pub(crate) async fn user_sessions(&self, user_id: DbId) -> ServerResult<Vec<UserSession>> {
        let expiry_limit = token_expiry_limit(self.token_expiry_seconds);
        self.db.read().await.transaction(|t| {
            Ok(t.exec(user_sessions_query(expiry_limit, user_id))?
                .elements
                .into_iter()
                .map(UserSession::from)
                .collect())
        })
    }

    pub(crate) async fn user_statuses(&self) -> ServerResult<Vec<UserStatus>> {
        self.db
            .read()
            .await
            .transaction(|t| -> ServerResult<Vec<UserStatus>> {
                let admin_id = t
                    .exec(QueryBuilder::select().aliases().ids(ADMIN).query())?
                    .elements[0]
                    .id;
                t.exec(
                    QueryBuilder::select()
                        .values([USERNAME])
                        .search()
                        .from(USERS)
                        .where_()
                        .neighbor()
                        .query(),
                )?
                .elements
                .into_iter()
                .map(|e| {
                    let sessions: Vec<UserSession> = t
                        .exec(user_sessions_query(
                            token_expiry_limit(self.token_expiry_seconds),
                            e.id,
                        ))?
                        .elements
                        .into_iter()
                        .map(UserSession::from)
                        .collect();

                    Ok(UserStatus {
                        username: e.values[0].value.to_string(),
                        login: !sessions.is_empty(),
                        admin: e.id == admin_id,
                        sessions,
                    })
                })
                .collect::<ServerResult<Vec<UserStatus>>>()
            })
    }

    pub(crate) async fn remove_expired_tokens(&self) -> ServerResult<()> {
        self.db.write().await.transaction_mut(|t| {
            let users = t
                .exec(
                    QueryBuilder::search()
                        .from(USERS)
                        .where_()
                        .neighbor()
                        .query(),
                )?
                .ids();

            for user in users {
                t.exec_mut(
                    QueryBuilder::remove()
                        .search()
                        .to(user)
                        .where_()
                        .neighbor()
                        .and()
                        .not()
                        .ids(USERS)
                        .and()
                        .key("created")
                        .value(Comparison::LessThan(
                            token_expiry_limit(self.token_expiry_seconds).into(),
                        ))
                        .query(),
                )?;
            }

            Ok(())
        })
    }

    pub(crate) async fn remove_token(&self, token: &str) -> ServerResult<()> {
        self.db.write().await.exec_mut(
            QueryBuilder::remove()
                .search()
                .index(TOKEN)
                .value(token)
                .query(),
        )?;
        Ok(())
    }

    pub(crate) async fn remove_tokens_except(&self, user: DbId, token: &str) -> ServerResult<()> {
        self.db.write().await.transaction_mut(|t| {
            t.exec_mut(
                QueryBuilder::remove()
                    .search()
                    .to(user)
                    .where_()
                    .neighbor()
                    .and()
                    .not()
                    .ids(USERS)
                    .and()
                    .not()
                    .key(TOKEN)
                    .value(token)
                    .query(),
            )?;

            Ok(())
        })
    }

    pub(crate) async fn remove_session(&self, user: DbId, session: i64) -> ServerResult<()> {
        self.db.write().await.transaction_mut(|t| {
            let removed = t.exec_mut(
                QueryBuilder::remove()
                    .search()
                    .to(user)
                    .limit(1)
                    .where_()
                    .neighbor()
                    .and()
                    .ids(session)
                    .and()
                    .keys(AGENT)
                    .query(),
            )?;

            if removed.result == 0 {
                return Err(session_not_found(session));
            }

            Ok(())
        })
    }
}

fn user_sessions_query(expiry_limit: u64, user_id: DbId) -> SelectValuesQuery {
    QueryBuilder::select()
        .values([AGENT, CREATED])
        .search()
        .to(user_id)
        .where_()
        .neighbor()
        .and()
        .key(CREATED)
        .value(Comparison::GreaterThanOrEqual(expiry_limit.into()))
        .query()
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
        .neighbor()
        .and()
        .key(OWNER)
        .value(owner)
        .and()
        .key(DB)
        .value(db)
        .query()
}

fn token_expired(token: &str) -> ServerError {
    ServerError::new(StatusCode::UNAUTHORIZED, &format!("token expired: {token}"))
}

fn token_not_found(token: &str) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("token not found: {token}"))
}

fn session_not_found(id: i64) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("session not found: {id}"))
}

fn user_not_found(name: &str) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("user not found: {name}"))
}

fn token_expiry_limit(token_expiry_seconds: u64) -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs()
        .saturating_sub(token_expiry_seconds)
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
    use agdb::test_utilities::test_file::TestFile;

    #[test]
    fn migrate_old_token_format() -> ServerResult<()> {
        let test_file = TestFile::new();

        {
            let mut db = Db::new(test_file.file_name())?;
            db.transaction_mut(|t| {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(USERS).query())?;
                t.exec_mut(QueryBuilder::insert().index(TOKEN).query())?;
                let admin = t
                    .exec_mut(
                        QueryBuilder::insert()
                            .nodes()
                            .aliases(ADMIN)
                            .values([[(USERNAME, "admin").into(), (TOKEN, "token0").into()]])
                            .query(),
                    )?
                    .ids();
                let mut users = t
                    .exec_mut(
                        QueryBuilder::insert()
                            .nodes()
                            .values([
                                [(USERNAME, "user1").into(), (TOKEN, "token1").into()],
                                [(USERNAME, "user2").into(), (TOKEN, "token2").into()],
                            ])
                            .query(),
                    )?
                    .ids();
                users.extend(admin);
                t.exec_mut(QueryBuilder::insert().edges().from(USERS).to(users).query())
            })?;
        }

        ServerDb::new(test_file.file_name(), 3600)?;

        let db = Db::new(test_file.file_name())?;
        let indexes = db.exec(QueryBuilder::select().indexes().query())?;
        assert_eq!(indexes.elements[0].values[0], (USERNAME, 3_u64).into());
        assert_eq!(indexes.elements[0].values[1], (TOKEN, 0_u64).into());
        let result = db.exec(
            QueryBuilder::search()
                .elements()
                .where_()
                .keys(TOKEN)
                .query(),
        )?;
        assert_eq!(result.result, 0, "{result:?}");

        Ok(())
    }

    #[tokio::test]
    async fn remove_expired_tokens() -> ServerResult<()> {
        let test_file = TestFile::new();
        let db = ServerDb::new(test_file.file_name(), 3600)?;

        let user1 = db
            .insert_user(ServerUser {
                db_id: None,
                username: "user1".to_string(),
                password: vec![],
                salt: vec![],
            })
            .await?;
        let user2 = db
            .insert_user(ServerUser {
                db_id: None,
                username: "user2".to_string(),
                password: vec![],
                salt: vec![],
            })
            .await?;

        let expired1 = db.save_token(user1, "token1", "agent").await?;
        let expired2 = db.save_token(user1, "token2", "agent").await?;
        db.save_token(user2, "token3", "agent").await?;
        db.save_token(user2, "token4", "agent").await?;

        db.db.write().await.exec_mut(
            QueryBuilder::insert()
                .values_uniform([(CREATED, 0_u64).into()])
                .ids([expired1, expired2])
                .query(),
        )?;

        assert_eq!(db.users_with_token().await?, 2);

        db.remove_expired_tokens().await?;
        assert_eq!(db.users_with_token().await?, 1);

        Ok(())
    }
}
