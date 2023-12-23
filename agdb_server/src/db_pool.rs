pub(crate) mod server_db;
pub(crate) mod server_db_storage;

use crate::config::Config;
use crate::db_pool::server_db_storage::ServerDbStorage;
use crate::error_code::ErrorCode;
use crate::password::Password;
use crate::routes::db::user::DbUserRole;
use crate::routes::db::DbType;
use crate::routes::db::Queries;
use crate::routes::db::ServerDatabase;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbId;
use agdb::DbUserValue;
use agdb::DbValue;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryId;
use agdb::QueryResult;
use agdb::QueryType;
use agdb::SearchQuery;
use agdb::Transaction;
use agdb::TransactionMut;
use agdb::UserValue;
use axum::http::StatusCode;
use server_db::ServerDb;
use server_db::ServerDbImpl;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const SERVER_DB_NAME: &str = "mapped:agdb_server.agdb";

#[derive(UserValue)]
pub(crate) struct ServerUser {
    pub(crate) db_id: Option<DbId>,
    pub(crate) name: String,
    pub(crate) password: Vec<u8>,
    pub(crate) salt: Vec<u8>,
    pub(crate) token: String,
}

#[derive(Default, UserValue)]
pub(crate) struct Database {
    pub(crate) db_id: Option<DbId>,
    pub(crate) name: String,
    pub(crate) db_type: DbType,
    pub(crate) backup: u64,
}

#[allow(dead_code)]
pub(crate) struct DbPoolImpl {
    server_db: ServerDb,
    pool: RwLock<HashMap<String, ServerDb>>,
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

#[derive(Clone)]
pub(crate) struct DbPool(pub(crate) Arc<DbPoolImpl>);

impl DbPool {
    pub(crate) fn new(config: &Config) -> ServerResult<Self> {
        let db_exists = Path::new("agdb_server.agdb").exists();

        let db_pool = Self(Arc::new(DbPoolImpl {
            server_db: ServerDb::new(SERVER_DB_NAME)?,
            pool: RwLock::new(HashMap::new()),
        }));

        if !db_exists {
            let admin_password = Password::create(&config.admin, &config.admin);

            db_pool.0.server_db.get_mut()?.transaction_mut(|t| {
                t.exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .aliases(vec!["users", "dbs"])
                        .query(),
                )?;

                let admin = t.exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .values(&ServerUser {
                            db_id: None,
                            name: config.admin.clone(),
                            password: admin_password.password.to_vec(),
                            salt: admin_password.user_salt.to_vec(),
                            token: String::new(),
                        })
                        .query(),
                )?;

                t.exec_mut(
                    &QueryBuilder::insert()
                        .edges()
                        .from("users")
                        .to(admin)
                        .query(),
                )
            })?;
        }

        Ok(db_pool)
    }

    pub(crate) fn add_db(
        &self,
        owner: &str,
        db: &str,
        db_type: DbType,
        config: &Config,
    ) -> ServerResult {
        let owner_id = self.find_user_id(owner)?;
        let db_name = db_name(owner, db);

        if self.find_user_db(owner_id, &db_name).is_ok() {
            return Err(ErrorCode::DbExists.into());
        }

        let db_path = Path::new(&config.data_dir).join(&db_name);
        std::fs::create_dir_all(Path::new(&config.data_dir).join(owner))?;
        let path = db_path.to_str().ok_or(ErrorCode::DbInvalid)?.to_string();

        let server_db = ServerDb::new(&format!("{}:{}", db_type, path)).map_err(|mut e| {
            e.status = ErrorCode::DbInvalid.into();
            e.description = format!("{}: {}", ErrorCode::DbInvalid.as_str(), e.description);
            e
        })?;

        let backup = if db_backup_file(owner, db, config).exists() {
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        } else {
            0
        };

        self.get_pool_mut()?.insert(db_name.clone(), server_db);
        self.db_mut()?.transaction_mut(|t| {
            let db = t.exec_mut(
                &QueryBuilder::insert()
                    .nodes()
                    .values(&Database {
                        db_id: None,
                        name: db_name.clone(),
                        db_type,
                        backup,
                    })
                    .query(),
            )?;

            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(vec![QueryId::from(owner_id), "dbs".into()])
                    .to(db)
                    .values(vec![vec![("role", DbUserRole::Admin).into()], vec![]])
                    .query(),
            )
        })?;

        Ok(())
    }

    pub(crate) fn add_db_user(&self, db: DbId, user: DbId, role: DbUserRole) -> ServerResult {
        self.db_mut()?.transaction_mut(|t| {
            let existing_role = t.exec(
                &QueryBuilder::search()
                    .from(user)
                    .to(db)
                    .limit(1)
                    .where_()
                    .keys(vec!["role".into()])
                    .query(),
            )?;

            if existing_role.result == 1 {
                t.exec_mut(
                    &QueryBuilder::insert()
                        .values(vec![vec![("role", role).into()]])
                        .ids(existing_role)
                        .query(),
                )?;
            } else {
                t.exec_mut(
                    &QueryBuilder::insert()
                        .edges()
                        .from(user)
                        .to(db)
                        .values_uniform(vec![("role", role).into()])
                        .query(),
                )?;
            }

            Ok(())
        })
    }

    pub(crate) fn add_user(&self, user: ServerUser) -> ServerResult {
        self.db_mut()?.transaction_mut(|t| {
            let user = t.exec_mut(&QueryBuilder::insert().nodes().values(&user).query())?;

            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from("users")
                    .to(user)
                    .query(),
            )
        })?;
        Ok(())
    }

    pub(crate) fn backup_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        config: &Config,
    ) -> ServerResult {
        let db_name = db_name(owner, db);
        let mut database = self.find_user_db(user, &db_name)?;

        if !self.is_db_admin(user, database.db_id.unwrap())? {
            return Err(ServerError {
                description: "permission denied: admin permissions required".to_string(),
                status: StatusCode::FORBIDDEN,
            });
        }

        if database.db_type == DbType::Memory {
            return Err(ServerError {
                description: "memory db cannot have backup".to_string(),
                status: StatusCode::FORBIDDEN,
            });
        }

        let backup_path = db_backup_file(owner, db, config);

        if backup_path.exists() {
            std::fs::remove_file(&backup_path)?;
        } else {
            std::fs::create_dir_all(db_backup_dir(owner, config))?;
        }

        self.get_pool()?
            .get(&db_name)
            .ok_or(db_not_found(&db_name))?
            .get_mut()?
            .backup(backup_path.to_string_lossy().as_ref())?;

        database.backup = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        self.save_db(database)?;

        Ok(())
    }

    pub(crate) fn delete_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        config: &Config,
    ) -> ServerResult {
        self.remove_db(owner, db, user)?;

        let main_file = db_file(owner, db, config);
        if main_file.exists() {
            std::fs::remove_file(&main_file)?;
        }

        let wal_file = db_file(owner, &format!(".{db}"), config);
        if wal_file.exists() {
            std::fs::remove_file(wal_file)?;
        }

        let backup_file = db_backup_file(owner, db, config);
        if backup_file.exists() {
            std::fs::remove_file(backup_file)?;
        }

        Ok(())
    }

    pub(crate) fn exec(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        queries: &Queries,
    ) -> ServerResult<Vec<QueryResult>> {
        let db_name = db_name(owner, db);
        let role = self.find_user_db_role(user, &db_name)?;
        let required_role = required_role(queries);

        if required_role == DbUserRole::Write && role == DbUserRole::Read {
            return Err(ServerError {
                description: "permission denied: write permissions required".to_string(),
                status: StatusCode::FORBIDDEN,
            });
        }

        let pool = self.get_pool()?;
        let db = pool.get(&db_name).ok_or(db_not_found(&db_name))?;

        let results = if required_role == DbUserRole::Read {
            db.get()?.transaction(|t| {
                let mut results = vec![];

                for q in &queries.0 {
                    results.push(t_exec(t, q)?);
                }

                Ok(results)
            })
        } else {
            db.get_mut()?.transaction_mut(|t| {
                let mut results = vec![];

                for q in &queries.0 {
                    results.push(t_exec_mut(t, q)?);
                }

                Ok(results)
            })
        }
        .map_err(|e: QueryError| ServerError {
            description: e.to_string(),
            status: StatusCode::from_u16(470).unwrap(),
        })?;

        Ok(results)
    }

    pub(crate) fn find_dbs(&self) -> ServerResult<Vec<ServerDatabase>> {
        let dbs: Vec<Database> = self
            .db()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .from("dbs")
                            .where_()
                            .distance(CountComparison::Equal(2))
                            .query(),
                    )
                    .query(),
            )?
            .try_into()?;

        dbs.into_iter()
            .map(|db| {
                Ok(ServerDatabase {
                    db_type: db.db_type,
                    role: DbUserRole::Admin,
                    size: self
                        .get_pool()?
                        .get(&db.name)
                        .ok_or(db_not_found(&db.name))?
                        .get()?
                        .size(),
                    backup: db.backup,
                    name: db.name,
                })
            })
            .collect::<ServerResult<Vec<ServerDatabase>>>()
    }

    pub(crate) fn find_db(&self, db: &str) -> ServerResult<Database> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .depth_first()
                            .from("dbs")
                            .limit(1)
                            .where_()
                            .distance(CountComparison::Equal(2))
                            .and()
                            .key("name")
                            .value(Comparison::Equal(db.into()))
                            .query(),
                    )
                    .query(),
            )?
            .elements
            .get(0)
            .ok_or(db_not_found(db))?
            .try_into()?)
    }

    pub(crate) fn find_db_id(&self, name: &str) -> ServerResult<DbId> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::search()
                    .depth_first()
                    .from("dbs")
                    .limit(1)
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .and()
                    .key("name")
                    .value(Comparison::Equal(name.into()))
                    .query(),
            )?
            .elements
            .get(0)
            .ok_or(db_not_found(name))?
            .id)
    }

    pub(crate) fn find_users(&self) -> ServerResult<Vec<String>> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::select()
                    .values(vec!["name".into()])
                    .ids(
                        QueryBuilder::search()
                            .from("users")
                            .where_()
                            .distance(CountComparison::Equal(2))
                            .and()
                            .keys(vec!["name".into()])
                            .query(),
                    )
                    .query(),
            )?
            .elements
            .into_iter()
            .map(|e| e.values[0].value.to_string())
            .collect())
    }

    pub(crate) fn find_user_dbs(&self, user: DbId) -> ServerResult<Vec<ServerDatabase>> {
        let mut dbs = vec![];

        self.db()?
            .exec(
                &QueryBuilder::select()
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
            .elements
            .into_iter()
            .try_for_each(|e| -> ServerResult {
                if e.id.0 < 0 {
                    dbs.push(ServerDatabase {
                        role: (&e.values[0].value).into(),
                        ..Default::default()
                    });
                } else {
                    let db = Database::from_db_element(&e).unwrap_or_default();
                    let server_db = dbs.last_mut().unwrap();
                    server_db.db_type = db.db_type;
                    server_db.backup = db.backup;
                    server_db.size = self
                        .get_pool()?
                        .get(&db.name)
                        .ok_or(db_not_found(&db.name))?
                        .get()?
                        .size();
                    server_db.name = db.name;
                }
                Ok(())
            })?;

        Ok(dbs)
    }

    pub(crate) fn find_user_db_id(&self, user: DbId, db: &str) -> ServerResult<DbId> {
        let db_id_query = self.find_user_db_id_query(user, db);
        Ok(self
            .db()?
            .exec(&db_id_query)?
            .elements
            .get(0)
            .ok_or(db_not_found(db))?
            .id)
    }

    pub(crate) fn find_user_db(&self, user: DbId, db: &str) -> ServerResult<Database> {
        let db_id_query = self.find_user_db_id_query(user, db);
        Ok(self
            .db()?
            .transaction(|t| -> Result<QueryResult, ServerError> {
                let db_id = t
                    .exec(&db_id_query)?
                    .elements
                    .get(0)
                    .ok_or(db_not_found(db))?
                    .id;
                Ok(t.exec(&QueryBuilder::select().ids(db_id).query())?)
            })?
            .try_into()?)
    }

    pub(crate) fn find_user_db_role(&self, user: DbId, db: &str) -> ServerResult<DbUserRole> {
        let db_id_query = self.find_user_db_id_query(user, db);
        Ok((&self
            .db()?
            .transaction(|t| -> Result<QueryResult, ServerError> {
                let db_id = t
                    .exec(&db_id_query)?
                    .elements
                    .get(0)
                    .ok_or(db_not_found(db))?
                    .id;

                Ok(t.exec(
                    &QueryBuilder::select()
                        .ids(
                            QueryBuilder::search()
                                .depth_first()
                                .from(user)
                                .to(db_id)
                                .limit(1)
                                .where_()
                                .distance(CountComparison::LessThanOrEqual(2))
                                .and()
                                .keys(vec!["role".into()])
                                .query(),
                        )
                        .query(),
                )?)
            })?
            .elements[0]
            .values[0]
            .value)
            .into())
    }

    pub(crate) fn find_user(&self, name: &str) -> ServerResult<ServerUser> {
        let user_id = self.find_user_id(name)?;
        Ok(self
            .db()?
            .exec(&QueryBuilder::select().ids(user_id).query())?
            .try_into()?)
    }

    pub(crate) fn find_user_id(&self, name: &str) -> ServerResult<DbId> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::search()
                    .depth_first()
                    .from("users")
                    .limit(1)
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .and()
                    .key("name")
                    .value(Comparison::Equal(name.into()))
                    .query(),
            )?
            .elements
            .get(0)
            .ok_or(user_not_found(name))?
            .id)
    }

    pub(crate) fn find_user_id_by_token(&self, token: &str) -> ServerResult<DbId> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::search()
                    .depth_first()
                    .from("users")
                    .limit(1)
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .and()
                    .key("token")
                    .value(Comparison::Equal(token.into()))
                    .query(),
            )?
            .elements
            .get(0)
            .ok_or(format!("No user found for token '{token}'"))?
            .id)
    }

    pub(crate) fn db_user_id(&self, db: DbId, name: &str) -> ServerResult<DbId> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::search()
                    .depth_first()
                    .to(db)
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .and()
                    .key("name")
                    .value(Comparison::Equal(name.into()))
                    .query(),
            )?
            .elements
            .get(0)
            .ok_or(user_not_found(name))?
            .id)
    }

    pub(crate) fn db_users(&self, db: DbId) -> ServerResult<Vec<(String, DbUserRole)>> {
        let mut users = vec![];

        self.db()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .depth_first()
                            .to(db)
                            .where_()
                            .distance(CountComparison::LessThanOrEqual(2))
                            .and()
                            .where_()
                            .keys(vec!["role".into()])
                            .or()
                            .keys(vec!["password".into()])
                            .query(),
                    )
                    .query(),
            )?
            .elements
            .into_iter()
            .for_each(|e| {
                if e.id.0 < 0 {
                    users.push((String::new(), (&e.values[0].value).into()));
                } else {
                    users.last_mut().unwrap().0 = e.values[0].value.to_string();
                }
            });

        Ok(users)
    }

    pub(crate) fn is_db_admin(&self, user: DbId, db: DbId) -> ServerResult<bool> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::search()
                    .from(user)
                    .to(db)
                    .limit(1)
                    .where_()
                    .distance(CountComparison::LessThanOrEqual(2))
                    .and()
                    .key("role")
                    .value(Comparison::Equal(DbUserRole::Admin.into()))
                    .query(),
            )?
            .result
            == 1)
    }

    pub(crate) fn optimize_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
    ) -> ServerResult<ServerDatabase> {
        let db_name = db_name(owner, db);
        let db = self.find_user_db(user, &db_name)?;
        let role = self.find_user_db_role(user, &db_name)?;

        if role == DbUserRole::Read {
            return Err(ServerError {
                description: "permission denied: write permissions required".to_string(),
                status: StatusCode::FORBIDDEN,
            });
        }

        let pool = self.get_pool()?;
        let server_db = pool.get(&db.name).ok_or(db_not_found(&db.name))?;
        server_db.get_mut()?.optimize_storage()?;
        let size = server_db.get()?.size();

        Ok(ServerDatabase {
            name: db.name,
            db_type: db.db_type,
            role,
            size,
            backup: db.backup,
        })
    }

    pub(crate) fn remove_db_user(&self, db: DbId, user: DbId) -> ServerResult {
        self.db_mut()?.exec_mut(
            &QueryBuilder::remove()
                .ids(
                    QueryBuilder::search()
                        .from(user)
                        .to(db)
                        .limit(1)
                        .where_()
                        .edge()
                        .query(),
                )
                .query(),
        )?;
        Ok(())
    }

    pub(crate) fn remove_db(&self, owner: &str, db: &str, user: DbId) -> ServerResult<ServerDb> {
        let user_name = self.user_name(user)?;

        if owner != user_name {
            return Err(ServerError::new(
                StatusCode::FORBIDDEN,
                "permission denied: user must be a db owner",
            ));
        }

        let db_name = db_name(owner, db);
        let db_id = self.find_user_db_id(user, &db_name)?;

        self.db_mut()?
            .exec_mut(&QueryBuilder::remove().ids(db_id).query())?;

        Ok(self.get_pool_mut()?.remove(&db_name).unwrap())
    }

    pub(crate) fn rename_db(
        &self,
        owner: &str,
        db: &str,
        new_name: &str,
        user: DbId,
        config: &Config,
    ) -> ServerResult {
        let (new_owner, new_db) = new_name.split_once('/').ok_or(ErrorCode::DbInvalid)?;
        let username = self.user_name(user)?;

        if owner != username {
            return Err(ServerError::new(
                StatusCode::FORBIDDEN,
                "user must be a db owner",
            ));
        }

        let db_name = db_name(owner, db);
        let mut database = self.find_db(&db_name)?;

        if new_owner != owner {
            let new_owner_id = self.find_user_id(new_owner)?;
            std::fs::create_dir_all(Path::new(&config.data_dir).join(new_owner))?;
            self.add_db_user(database.db_id.unwrap(), new_owner_id, DbUserRole::Admin)?;
        }

        let server_db = self.get_pool_mut()?.remove(&db_name).unwrap();
        server_db
            .get_mut()?
            .rename(
                Path::new(&config.data_dir)
                    .join(new_name)
                    .to_string_lossy()
                    .as_ref(),
            )
            .map_err(|_| ErrorCode::DbInvalid)?;
        self.get_pool_mut()?.insert(new_name.to_string(), server_db);
        database.name = new_name.to_string();

        let backup_path = db_backup_file(owner, db, config);

        if backup_path.exists() {
            let new_backup_path = db_backup_file(new_owner, new_db, config);
            let backups_dir = new_backup_path.parent().unwrap();
            std::fs::create_dir_all(backups_dir)?;
            std::fs::rename(backup_path, new_backup_path)?;
        }

        self.db_mut()?
            .exec_mut(&QueryBuilder::insert().element(&database).query())?;

        Ok(())
    }

    pub(crate) fn restore_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        config: &Config,
    ) -> ServerResult {
        let db_name = db_name(owner, db);
        let mut database = self.find_user_db(user, &db_name)?;

        if !self.is_db_admin(user, database.db_id.unwrap())? {
            return Err(ServerError {
                description: "permission denied: must be a db admin".to_string(),
                status: StatusCode::FORBIDDEN,
            });
        }

        let backup_path = db_backup_file(owner, db, config);

        if !backup_path.exists() {
            return Err(ServerError {
                description: "backup not found".to_string(),
                status: StatusCode::NOT_FOUND,
            });
        }

        let current_path = db_file(owner, db, config);
        let backup_temp = db_backup_dir(owner, config).join(db);

        self.get_pool_mut()?.remove(&db_name);
        std::fs::rename(&current_path, &backup_temp)?;
        std::fs::rename(&backup_path, &current_path)?;
        std::fs::rename(backup_temp, backup_path)?;
        let server_db = ServerDb::new(&format!(
            "{}:{}",
            database.db_type,
            current_path.to_string_lossy()
        ))?;
        self.get_pool_mut()?.insert(db_name, server_db);
        database.backup = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        self.save_db(database)?;

        Ok(())
    }

    pub(crate) fn save_token(&self, user: DbId, token: &str) -> ServerResult {
        self.db_mut()?.exec_mut(
            &QueryBuilder::insert()
                .values_uniform(vec![("token", token).into()])
                .ids(user)
                .query(),
        )?;
        Ok(())
    }

    pub(crate) fn save_db(&self, db: Database) -> ServerResult {
        self.db_mut()?
            .exec_mut(&QueryBuilder::insert().element(&db).query())?;
        Ok(())
    }

    pub(crate) fn save_user(&self, user: ServerUser) -> ServerResult {
        self.db_mut()?
            .exec_mut(&QueryBuilder::insert().element(&user).query())?;
        Ok(())
    }

    pub(crate) fn user_name(&self, id: DbId) -> ServerResult<String> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::select()
                    .values(vec!["name".into()])
                    .ids(id)
                    .query(),
            )?
            .elements[0]
            .values[0]
            .value
            .to_string())
    }

    pub(crate) fn get_pool(&self) -> ServerResult<RwLockReadGuard<HashMap<String, ServerDb>>> {
        Ok(self.0.pool.read()?)
    }

    pub(crate) fn get_pool_mut(&self) -> ServerResult<RwLockWriteGuard<HashMap<String, ServerDb>>> {
        Ok(self.0.pool.write()?)
    }

    fn find_user_db_id_query(&self, user: DbId, db: &str) -> SearchQuery {
        QueryBuilder::search()
            .depth_first()
            .from(user)
            .limit(1)
            .where_()
            .distance(CountComparison::Equal(2))
            .and()
            .key("name")
            .value(Comparison::Equal(db.into()))
            .query()
    }

    fn db(&self) -> ServerResult<RwLockReadGuard<ServerDbImpl>> {
        self.0.server_db.get()
    }

    fn db_mut(&self) -> ServerResult<RwLockWriteGuard<ServerDbImpl>> {
        self.0.server_db.get_mut()
    }
}

fn user_not_found(name: &str) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("user not found: {name}"))
}

fn db_not_found(name: &str) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("db not found: {name}"))
}

fn db_backup_file(owner: &str, db: &str, config: &Config) -> PathBuf {
    db_backup_dir(owner, config).join(format!("{db}.bak"))
}

fn db_backup_dir(owner: &str, config: &Config) -> PathBuf {
    Path::new(&config.data_dir).join(owner).join("backups")
}

fn db_file(owner: &str, db: &str, config: &Config) -> PathBuf {
    Path::new(&config.data_dir).join(owner).join(db)
}

fn db_name(owner: &str, db: &str) -> String {
    format!("{owner}/{db}")
}

fn required_role(queries: &Queries) -> DbUserRole {
    for q in &queries.0 {
        match q {
            QueryType::InsertAlias(_)
            | QueryType::InsertEdges(_)
            | QueryType::InsertNodes(_)
            | QueryType::InsertValues(_)
            | QueryType::Remove(_)
            | QueryType::RemoveAliases(_)
            | QueryType::RemoveValues(_) => {
                return DbUserRole::Write;
            }
            _ => {}
        }
    }

    DbUserRole::Read
}

fn t_exec(t: &Transaction<ServerDbStorage>, q: &QueryType) -> Result<QueryResult, QueryError> {
    match q {
        QueryType::Search(q) => t.exec(q),
        QueryType::Select(q) => t.exec(q),
        QueryType::SelectAliases(q) => t.exec(q),
        QueryType::SelectAllAliases(q) => t.exec(q),
        QueryType::SelectKeys(q) => t.exec(q),
        QueryType::SelectKeyCount(q) => t.exec(q),
        QueryType::SelectValues(q) => t.exec(q),
        _ => unreachable!(),
    }
}

fn t_exec_mut(
    t: &mut TransactionMut<ServerDbStorage>,
    q: &QueryType,
) -> Result<QueryResult, QueryError> {
    match q {
        QueryType::Search(q) => t.exec(q),
        QueryType::Select(q) => t.exec(q),
        QueryType::SelectAliases(q) => t.exec(q),
        QueryType::SelectAllAliases(q) => t.exec(q),
        QueryType::SelectKeys(q) => t.exec(q),
        QueryType::SelectKeyCount(q) => t.exec(q),
        QueryType::SelectValues(q) => t.exec(q),
        QueryType::InsertAlias(q) => t.exec_mut(q),
        QueryType::InsertEdges(q) => t.exec_mut(q),
        QueryType::InsertNodes(q) => t.exec_mut(q),
        QueryType::InsertValues(q) => t.exec_mut(q),
        QueryType::Remove(q) => t.exec_mut(q),
        QueryType::RemoveAliases(q) => t.exec_mut(q),
        QueryType::RemoveValues(q) => t.exec_mut(q),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_pool::server_db::ServerDb;
    use agdb::QueryBuilder;

    #[test]
    #[should_panic]
    fn unreachable() {
        let db = ServerDb::new("memory:test").unwrap();
        db.get()
            .unwrap()
            .transaction(|t| t_exec(t, &QueryType::Remove(QueryBuilder::remove().ids(1).query())))
            .unwrap();
    }
}
