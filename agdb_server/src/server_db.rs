use crate::action::ClusterAction;
use crate::config::Config;
use crate::db_pool::DbName;
use crate::password::Password;
use crate::raft::Log;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::AgdbSerialize;
use agdb::CountComparison;
use agdb::Db;
use agdb::DbId;
use agdb::DbType;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::QueryResult;
use agdb::SearchQuery;
use agdb_api::DbKind;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use agdb_api::UserStatus;
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

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
    pub(crate) created: SystemTime,
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
pub(crate) struct ServerDb(pub(crate) Arc<RwLock<Db>>);

const ADMIN: &str = "admin";
const DB: &str = "db";
const DBS: &str = "dbs";
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

            if t.exec(QueryBuilder::select().ids(USERS).query()).is_err() {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(USERS).query())?;
            }

            if t.exec(QueryBuilder::select().ids(DBS).query()).is_err() {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(DBS).query())?;
            }

            Ok(())
        })?;

        Ok(Self(Arc::new(RwLock::new(db))))
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
            .exec(
                QueryBuilder::select()
                    .search()
                    .from(ADMIN)
                    .limit(1)
                    .where_()
                    .neighbor()
                    .and()
                    .key(TOKEN)
                    .value(token)
                    .query(),
            )?
            .result
            == 1)
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
                    .value(DbUserRole::Admin)
                    .query(),
            )?
            .result
            == 1)
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
            QueryBuilder::remove()
                .search()
                .from(USERS)
                .where_()
                .distance(4)
                .and()
                .keys(TOKEN)
                .and()
                .not_beyond()
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
        self.0
            .write()
            .await
            .transaction_mut(|t| -> ServerResult<()> {
                let token_id = t
                    .exec_mut(
                        QueryBuilder::insert()
                            .element(&UserToken {
                                db_id: None,
                                token: token.to_string(),
                                created: SystemTime::now(),
                            })
                            .query(),
                    )?
                    .elements[0]
                    .id;

                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from(user)
                        .to(token_id)
                        .query(),
                )?;

                Ok(())
            })
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
                            .distance(1)
                            .or()
                            .where_()
                            .neighbor()
                            .and()
                            .keys(DB)
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
        self.0.read().await.transaction(|t| {
            let token_id = t
                .exec(QueryBuilder::search().index(TOKEN).value(token).query())?
                .elements
                .first()
                .ok_or_else(|| token_not_found(token))?
                .id;

            Ok(t.exec(
                QueryBuilder::search()
                    .to(token_id)
                    .limit(1)
                    .where_()
                    .neighbor()
                    .query(),
            )?
            .elements
            .first()
            .ok_or_else(|| user_not_found(token))?
            .id)
        })
    }

    pub(crate) async fn user_statuses(&self) -> ServerResult<Vec<UserStatus>> {
        self.0
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
                    Ok(UserStatus {
                        username: e.values[0].value.to_string(),
                        login: t
                            .exec(QueryBuilder::select().edge_count_from().ids(e.id).query())?
                            .elements[0]
                            .values[0]
                            .value
                            .to_u64()?
                            > 0,
                        admin: e.id == admin_id,
                    })
                })
                .collect::<ServerResult<Vec<UserStatus>>>()
            })
    }

    pub(crate) async fn remove_token(&self, token: &str) -> ServerResult<()> {
        self.0.write().await.exec_mut(
            QueryBuilder::remove()
                .ids(self.user_token_id(token).await?)
                .query(),
        )?;
        Ok(())
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
        .neighbor()
        .and()
        .key(OWNER)
        .value(owner)
        .and()
        .key(DB)
        .value(db)
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
