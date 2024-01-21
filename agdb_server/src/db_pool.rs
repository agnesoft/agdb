mod server_db;
mod server_db_storage;

use crate::config::Config;
use crate::db_pool::server_db_storage::ServerDbStorage;
use crate::error_code::ErrorCode;
use crate::password;
use crate::password::Password;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbId;
use agdb::DbUserValue;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryId;
use agdb::QueryIds;
use agdb::QueryResult;
use agdb::QueryType;
use agdb::SearchQuery;
use agdb::Transaction;
use agdb::TransactionMut;
use agdb::UserValue;
use agdb_api::DbType;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use agdb_api::Queries;
use agdb_api::ServerDatabase;
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
use uuid::Uuid;

const SERVER_DB_NAME: &str = "mapped:agdb_server.agdb";

#[derive(UserValue)]
pub(crate) struct ServerUser {
    pub(crate) db_id: Option<DbId>,
    pub(crate) name: String,
    pub(crate) password: Vec<u8>,
    pub(crate) salt: Vec<u8>,
    pub(crate) token: String,
}

#[derive(UserValue)]
struct Database {
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

    pub(crate) fn add_db_user(
        &self,
        owner: &str,
        db: &str,
        username: &str,
        role: DbUserRole,
        user: DbId,
    ) -> ServerResult {
        if owner == username {
            return Err(permission_denied("cannot change role of db owner"));
        }

        let db_name = db_name(owner, db);
        let db_id = self.find_user_db_id(user, &db_name)?;

        if !self.is_db_admin(user, db_id)? {
            return Err(permission_denied("admin only"));
        }

        let user_id = self.find_user_id(username)?;

        self.db_mut()?.transaction_mut(|t| {
            let existing_role = t.exec(
                &QueryBuilder::search()
                    .from(user_id)
                    .to(db_id)
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
                        .from(user_id)
                        .to(db_id)
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
            return Err(permission_denied("admin only"));
        }

        if database.db_type == DbType::Memory {
            return Err(permission_denied("memory db cannot have backup"));
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

    pub(crate) fn change_password(&self, mut user: ServerUser, new_password: &str) -> ServerResult {
        password::validate_password(new_password)?;
        let pswd = Password::create(&user.name, new_password);
        user.password = pswd.password.to_vec();
        user.salt = pswd.user_salt.to_vec();
        self.save_user(user)?;

        Ok(())
    }

    pub(crate) fn copy_db(
        &self,
        owner: &str,
        db: &str,
        new_name: &str,
        mut user: DbId,
        config: &Config,
        admin: bool,
    ) -> ServerResult {
        let (new_owner, new_db) = new_name.split_once('/').ok_or(ErrorCode::DbInvalid)?;
        let source_db = db_name(owner, db);
        let target_db = db_name(new_owner, new_db);
        let database = self.find_user_db(user, &source_db)?;

        if admin {
            user = self.find_user_id(new_owner)?;
        } else {
            let username = self.user_name(user)?;

            if new_owner != username {
                return Err(permission_denied("cannot copy db to another user"));
            }
        };

        if self.find_user_db(user, &target_db).is_ok() {
            return Err(ErrorCode::DbExists.into());
        }

        let target_file = db_file(new_owner, new_db, config);

        if target_file.exists() {
            return Err(ErrorCode::DbExists.into());
        }

        std::fs::create_dir_all(Path::new(&config.data_dir).join(new_owner))?;
        let server_db = self
            .get_pool()?
            .get(&source_db)
            .ok_or(db_not_found(&source_db))?
            .copy(target_file.to_string_lossy().as_ref())
            .map_err(|e| {
                ServerError::new(
                    ErrorCode::DbInvalid.into(),
                    &format!("db copy error: {}", e.description),
                )
            })?;
        self.get_pool_mut()?.insert(target_db.clone(), server_db);
        self.db_mut()?.transaction_mut(|t| {
            let db = t.exec_mut(
                &QueryBuilder::insert()
                    .nodes()
                    .values(&Database {
                        db_id: None,
                        name: target_db.clone(),
                        db_type: database.db_type,
                        backup: 0,
                    })
                    .query(),
            )?;

            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(vec![QueryId::from(user), "dbs".into()])
                    .to(db)
                    .values(vec![vec![("role", DbUserRole::Admin).into()], vec![]])
                    .query(),
            )
        })?;

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
        mut queries: Queries,
    ) -> ServerResult<Vec<QueryResult>> {
        let db_name = db_name(owner, db);
        let role = self.find_user_db_role(user, &db_name)?;
        let required_role = required_role(&queries);

        if required_role == DbUserRole::Write && role == DbUserRole::Read {
            return Err(permission_denied("write rights required"));
        }

        let pool = self.get_pool()?;
        let db = pool.get(&db_name).ok_or(db_not_found(&db_name))?;

        let results = if required_role == DbUserRole::Read {
            db.get()?.transaction(move |t| {
                let mut results = vec![];

                for q in queries.0.iter_mut() {
                    let result = t_exec(t, q, &results)?;
                    results.push(result);
                }

                Ok(results)
            })
        } else {
            db.get_mut()?.transaction_mut(move |t| {
                let mut results = vec![];

                for q in queries.0.iter_mut() {
                    let result = t_exec_mut(t, q, &results)?;
                    results.push(result);
                }

                Ok(results)
            })
        }
        .map_err(|e: QueryError| ServerError::new(ErrorCode::QueryError.into(), &e.description))?;

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
                    let db = Database::from_db_element(&e)?;
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
            .first()
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
            .first()
            .ok_or(format!("No user found for token '{token}'"))?
            .id)
    }

    pub(crate) fn get_user(&self, user: DbId) -> ServerResult<ServerUser> {
        Ok(self
            .db()?
            .exec(&QueryBuilder::select().ids(user).query())?
            .try_into()?)
    }

    pub(crate) fn db_users(&self, owner: &str, db: &str, user: DbId) -> ServerResult<Vec<DbUser>> {
        let db_id = self.find_user_db_id(user, &db_name(owner, db))?;
        let mut users = vec![];

        self.db()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .depth_first()
                            .to(db_id)
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
                    users.push(DbUser {
                        user: String::new(),
                        role: (&e.values[0].value).into(),
                    });
                } else {
                    users.last_mut().unwrap().user = e.values[0].value.to_string();
                }
            });

        Ok(users)
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
            return Err(permission_denied("write rights required"));
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

    pub(crate) fn remove_db_user(
        &self,
        owner: &str,
        db: &str,
        username: &str,
        user: DbId,
    ) -> ServerResult {
        if owner == username {
            return Err(permission_denied("cannot remove owner"));
        }

        let db_id = self.find_user_db_id(user, &db_name(owner, db))?;
        let user_id = self.find_db_user_id(db_id, username)?;

        if user != user_id && !self.is_db_admin(user, db_id)? {
            return Err(permission_denied("admin only"));
        }

        self.db_mut()?.exec_mut(
            &QueryBuilder::remove()
                .ids(
                    QueryBuilder::search()
                        .from(user_id)
                        .to(db_id)
                        .limit(1)
                        .where_()
                        .keys(vec!["role".into()])
                        .query(),
                )
                .query(),
        )?;
        Ok(())
    }

    pub(crate) fn remove_db(&self, owner: &str, db: &str, user: DbId) -> ServerResult<ServerDb> {
        let user_name = self.user_name(user)?;

        if owner != user_name {
            return Err(permission_denied("owner only"));
        }

        let db_name = db_name(owner, db);
        let db_id = self.find_user_db_id(user, &db_name)?;

        self.db_mut()?
            .exec_mut(&QueryBuilder::remove().ids(db_id).query())?;

        Ok(self.get_pool_mut()?.remove(&db_name).unwrap())
    }

    pub(crate) fn remove_user(&self, username: &str, config: &Config) -> ServerResult {
        let user_id = self.find_user_id(username)?;
        let dbs = self.find_user_databases(user_id)?;
        let mut ids = dbs
            .iter()
            .map(|db| db.db_id.unwrap())
            .collect::<Vec<DbId>>();
        ids.push(user_id);
        self.db_mut()?
            .exec_mut(&QueryBuilder::remove().ids(ids).query())?;

        for db in dbs.into_iter() {
            self.get_pool_mut()?.remove(&db.name);
        }

        let user_dir = Path::new(&config.data_dir).join(username);
        if user_dir.exists() {
            std::fs::remove_dir_all(user_dir)?;
        }

        Ok(())
    }

    pub(crate) fn rename_db(
        &self,
        owner: &str,
        db: &str,
        new_name: &str,
        user: DbId,
        config: &Config,
    ) -> ServerResult {
        let db_name = db_name(owner, db);

        if db_name == new_name {
            return Ok(());
        }

        let (new_owner, new_db) = new_name.split_once('/').ok_or(ErrorCode::DbInvalid)?;
        let username = self.user_name(user)?;

        if owner != username {
            return Err(permission_denied("owner only"));
        }

        let mut database = self.find_user_db(user, &db_name)?;
        let target_name = db_file(new_owner, new_db, config);

        if target_name.exists() {
            return Err(ErrorCode::DbExists.into());
        }

        if new_owner != owner {
            std::fs::create_dir_all(Path::new(&config.data_dir).join(new_owner))?;
            self.add_db_user(owner, db, new_owner, DbUserRole::Admin, user)?;
        }

        let server_db = ServerDb(
            self.get_pool()?
                .get(&db_name)
                .ok_or(db_not_found(&db_name))?
                .0
                .clone(),
        );

        server_db
            .get_mut()?
            .rename(target_name.to_string_lossy().as_ref())
            .map_err(|e| {
                ServerError::new(
                    ErrorCode::DbInvalid.into(),
                    &format!("db rename error: {}", e.description),
                )
            })?;

        let backup_path = db_backup_file(owner, db, config);
        if backup_path.exists() {
            let new_backup_path = db_backup_file(new_owner, new_db, config);
            let backups_dir = new_backup_path.parent().unwrap();
            std::fs::create_dir_all(backups_dir)?;
            std::fs::rename(backup_path, new_backup_path)?;
        }

        self.get_pool_mut()?.insert(new_name.to_string(), server_db);

        database.name = new_name.to_string();
        self.db_mut()?
            .exec_mut(&QueryBuilder::insert().element(&database).query())?;

        self.get_pool_mut()?.remove(&db_name).unwrap();

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
            return Err(permission_denied("admin only"));
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

        let mut pool = self.get_pool_mut()?;
        pool.remove(&db_name);
        std::fs::rename(&current_path, &backup_temp)?;
        std::fs::rename(&backup_path, &current_path)?;
        std::fs::rename(backup_temp, backup_path)?;
        let server_db = ServerDb::new(&format!(
            "{}:{}",
            database.db_type,
            current_path.to_string_lossy()
        ))?;
        pool.insert(db_name, server_db);
        database.backup = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        self.save_db(database)?;

        Ok(())
    }

    pub(crate) fn user_token(&self, user: DbId) -> ServerResult<String> {
        self.db_mut()?.transaction_mut(|t| {
            let mut user_token = t
                .exec(
                    &QueryBuilder::select()
                        .values(vec!["token".into()])
                        .ids(user)
                        .query(),
                )?
                .elements[0]
                .values[0]
                .value
                .to_string();

            if user_token.is_empty() {
                let token_uuid = Uuid::new_v4();
                user_token = token_uuid.to_string();
                t.exec_mut(
                    &QueryBuilder::insert()
                        .values_uniform(vec![("token", &user_token.clone()).into()])
                        .ids(user)
                        .query(),
                )?;
            }

            Ok(user_token)
        })
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

    fn db(&self) -> ServerResult<RwLockReadGuard<ServerDbImpl>> {
        self.0.server_db.get()
    }

    fn db_mut(&self) -> ServerResult<RwLockWriteGuard<ServerDbImpl>> {
        self.0.server_db.get_mut()
    }

    fn find_db_user_id(&self, db: DbId, name: &str) -> ServerResult<DbId> {
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
            .first()
            .ok_or(user_not_found(name))?
            .id)
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

    fn find_user_db(&self, user: DbId, db: &str) -> ServerResult<Database> {
        let db_id_query = self.find_user_db_id_query(user, db);
        Ok(self
            .db()?
            .transaction(|t| -> Result<QueryResult, ServerError> {
                let db_id = t
                    .exec(&db_id_query)?
                    .elements
                    .first()
                    .ok_or(db_not_found(db))?
                    .id;
                Ok(t.exec(&QueryBuilder::select().ids(db_id).query())?)
            })?
            .try_into()?)
    }

    fn find_user_databases(&self, user: DbId) -> ServerResult<Vec<Database>> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .depth_first()
                            .from(user)
                            .where_()
                            .distance(CountComparison::Equal(2))
                            .query(),
                    )
                    .query(),
            )?
            .try_into()?)
    }

    fn find_user_db_id(&self, user: DbId, db: &str) -> ServerResult<DbId> {
        let db_id_query = self.find_user_db_id_query(user, db);
        Ok(self
            .db()?
            .exec(&db_id_query)?
            .elements
            .first()
            .ok_or(db_not_found(db))?
            .id)
    }

    fn find_user_db_role(&self, user: DbId, db: &str) -> ServerResult<DbUserRole> {
        let db_id_query = self.find_user_db_id_query(user, db);
        Ok((&self
            .db()?
            .transaction(|t| -> Result<QueryResult, ServerError> {
                let db_id = t
                    .exec(&db_id_query)?
                    .elements
                    .first()
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

    fn get_pool(&self) -> ServerResult<RwLockReadGuard<HashMap<String, ServerDb>>> {
        Ok(self.0.pool.read()?)
    }

    fn get_pool_mut(&self) -> ServerResult<RwLockWriteGuard<HashMap<String, ServerDb>>> {
        Ok(self.0.pool.write()?)
    }

    fn is_db_admin(&self, user: DbId, db: DbId) -> ServerResult<bool> {
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

    fn save_db(&self, db: Database) -> ServerResult {
        self.db_mut()?
            .exec_mut(&QueryBuilder::insert().element(&db).query())?;
        Ok(())
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

fn permission_denied(message: &str) -> ServerError {
    ServerError::new(
        StatusCode::FORBIDDEN,
        &format!("permission denied: {}", message),
    )
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

fn t_exec(
    t: &Transaction<ServerDbStorage>,
    q: &mut QueryType,
    results: &[QueryResult],
) -> Result<QueryResult, QueryError> {
    match q {
        QueryType::Search(q) => {
            q.origin = id_or_result(q.origin.clone(), results)?;
            q.destination = id_or_result(q.destination.clone(), results)?;
            t.exec(q)
        }
        QueryType::Select(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(q)
        }
        QueryType::SelectAliases(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(q)
        }
        QueryType::SelectAllAliases(q) => t.exec(q),
        QueryType::SelectIndexes(q) => t.exec(q),
        QueryType::SelectKeys(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(q)
        }
        QueryType::SelectKeyCount(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(q)
        }
        QueryType::SelectValues(q) => {
            inject_results(&mut q.ids, results)?;
            t.exec(q)
        }
        _ => unreachable!(),
    }
}

fn t_exec_mut(
    t: &mut TransactionMut<ServerDbStorage>,
    q: &mut QueryType,
    results: &[QueryResult],
) -> Result<QueryResult, QueryError> {
    match q {
        QueryType::Search(q) => {
            q.origin = id_or_result(q.origin.clone(), results)?;
            q.destination = id_or_result(q.destination.clone(), results)?;
            t.exec(q)
        }
        QueryType::Select(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(q)
        }
        QueryType::SelectAliases(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(q)
        }
        QueryType::SelectAllAliases(q) => t.exec(q),
        QueryType::SelectIndexes(q) => t.exec(q),
        QueryType::SelectKeys(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(q)
        }
        QueryType::SelectKeyCount(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(q)
        }
        QueryType::SelectValues(q) => {
            inject_results(&mut q.ids, results)?;
            t.exec(q)
        }
        QueryType::InsertAlias(q) => {
            inject_results(&mut q.ids, results)?;
            t.exec_mut(q)
        }
        QueryType::InsertEdges(q) => {
            inject_results(&mut q.from, results)?;
            inject_results(&mut q.to, results)?;
            t.exec_mut(q)
        }
        QueryType::InsertNodes(q) => t.exec_mut(q),
        QueryType::InsertValues(q) => {
            inject_results(&mut q.ids, results)?;
            t.exec_mut(q)
        }
        QueryType::Remove(q) => {
            inject_results(&mut q.0, results)?;
            t.exec_mut(q)
        }
        QueryType::InsertIndex(q) => t.exec_mut(q),
        QueryType::RemoveAliases(q) => t.exec_mut(q),
        QueryType::RemoveIndex(q) => t.exec_mut(q),
        QueryType::RemoveValues(q) => {
            inject_results(&mut q.0.ids, results)?;
            t.exec_mut(q)
        }
    }
}

fn id_or_result(id: QueryId, results: &[QueryResult]) -> Result<QueryId, QueryError> {
    if let QueryId::Alias(alias) = &id {
        if let Some(index) = alias.strip_prefix(':') {
            if let Ok(index) = index.parse::<usize>() {
                return Ok(QueryId::Id(
                    results
                        .get(index)
                        .ok_or(QueryError::from(format!(
                            "Results index out of bounds '{index}' (> {})",
                            results.len()
                        )))?
                        .elements
                        .first()
                        .ok_or(QueryError::from("No lement found in the result"))?
                        .id,
                ));
            }
        }
    }

    Ok(id)
}

fn inject_results(ids: &mut QueryIds, results: &[QueryResult]) -> Result<(), QueryError> {
    if let QueryIds::Ids(ids) = ids {
        for i in 0..ids.len() {
            if let QueryId::Alias(alias) = &ids[i] {
                if let Some(index) = alias.strip_prefix(':') {
                    if let Ok(index) = index.parse::<usize>() {
                        let result_ids = results
                            .get(index)
                            .ok_or(QueryError::from(format!(
                                "Results index out of bounds '{index}' (> {})",
                                results.len()
                            )))?
                            .ids()
                            .into_iter()
                            .map(QueryId::Id)
                            .collect::<Vec<QueryId>>();
                        ids.splice(i..i + 1, result_ids.into_iter());
                    }
                }
            }
        }
    }

    Ok(())
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
            .transaction(|t| {
                t_exec(
                    t,
                    &mut QueryType::Remove(QueryBuilder::remove().ids(1).query()),
                    &[],
                )
            })
            .unwrap();
    }
}
