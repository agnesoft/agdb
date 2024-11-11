mod user_db;
mod user_db_storage;

use crate::config::Config;
use crate::error_code::ErrorCode;
use crate::password;
use crate::password::Password;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use crate::utilities::get_size;
use agdb::DbId;
use agdb::QueryResult;
use agdb::QueryType;
use agdb_api::AdminStatus;
use agdb_api::DbAudit;
use agdb_api::DbResource;
use agdb_api::DbType;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use agdb_api::Queries;
use agdb_api::ServerDatabase;
use axum::http::StatusCode;
use std::collections::HashMap;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::sync::RwLock;
use tokio::sync::RwLockReadGuard;
use tokio::sync::RwLockWriteGuard;
use user_db::UserDb;
use uuid::Uuid;

pub(crate) struct DbPoolImpl {
    server_db: ServerDb,
    pool: RwLock<HashMap<String, UserDb>>,
}

#[derive(Clone)]
pub(crate) struct DbPool(pub(crate) Arc<DbPoolImpl>);

impl DbPool {
    pub(crate) async fn new(config: &Config) -> ServerResult<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        let db_exists = Path::new(&config.data_dir)
            .join("agdb_server.agdb")
            .exists();

        Ok(if db_exists {
            let db_pool = Self(Arc::new(DbPoolImpl {
                server_db: ServerDb::load(&format!("mapped:{}/agdb_server.agdb", config.data_dir))?,
                pool: RwLock::new(HashMap::new()),
            }));

            if db_pool
                .0
                .server_db
                .find_user_id(&config.admin)
                .await
                .is_ok()
            {
                let admin_password = Password::create(&config.admin, &config.admin);
                db_pool
                    .0
                    .server_db
                    .insert_user(ServerUser {
                        db_id: None,
                        username: config.admin.clone(),
                        password: admin_password.password.to_vec(),
                        salt: admin_password.user_salt.to_vec(),
                        token: String::new(),
                    })
                    .await?;
            }

            for db in db_pool.0.server_db.dbs().await? {
                let (owner, db_name) = db.name.split_once('/').ok_or(ErrorCode::DbInvalid)?;
                let db_path = db_file(owner, db_name, config);
                std::fs::create_dir_all(db_audit_dir(owner, config))?;
                let server_db =
                    UserDb::new(&format!("{}:{}", db.db_type, db_path.to_string_lossy()))?;
                db_pool.0.pool.write().await.insert(db.name, server_db);
            }

            db_pool
        } else {
            let admin_password = Password::create(&config.admin, &config.admin);
            Self(Arc::new(DbPoolImpl {
                server_db: ServerDb::new(
                    &format!("mapped:{}/agdb_server.agdb", config.data_dir),
                    ServerUser {
                        db_id: None,
                        username: config.admin.clone(),
                        password: admin_password.password.to_vec(),
                        salt: admin_password.user_salt.to_vec(),
                        token: String::new(),
                    },
                )?,
                pool: RwLock::new(HashMap::new()),
            }))
        })
    }

    pub(crate) async fn add_db(
        &self,
        owner: &str,
        db: &str,
        db_type: DbType,
        config: &Config,
    ) -> ServerResult {
        let owner_id = self.0.server_db.user_id(owner).await?;
        let db_name = db_name(owner, db);

        if self
            .0
            .server_db
            .find_user_db_id(owner_id, &db_name)
            .await?
            .is_some()
        {
            return Err(ErrorCode::DbExists.into());
        }

        let db_path = Path::new(&config.data_dir).join(&db_name);
        std::fs::create_dir_all(db_audit_dir(owner, config))?;
        let path = db_path.to_str().ok_or(ErrorCode::DbInvalid)?.to_string();

        let server_db = UserDb::new(&format!("{}:{}", db_type, path)).map_err(|mut e| {
            e.status = ErrorCode::DbInvalid.into();
            e.description = format!("{}: {}", ErrorCode::DbInvalid.as_str(), e.description);
            e
        })?;

        let backup = if db_backup_file(owner, db, config).exists() {
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        } else {
            0
        };

        self.get_pool_mut().await.insert(db_name.clone(), server_db);
        self.0
            .server_db
            .insert_db(
                owner_id,
                Database {
                    db_id: None,
                    name: db_name.clone(),
                    db_type,
                    backup,
                },
            )
            .await?;

        Ok(())
    }

    pub(crate) async fn add_db_user(
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
        let db_id = self
            .0
            .server_db
            .find_user_db_id(user, &db_name)
            .await?
            .ok_or(db_not_found(&db_name))?;

        if !self.0.server_db.is_db_admin(user, db_id).await? {
            return Err(permission_denied("admin only"));
        }

        let user_id = self.0.server_db.user_id(username).await?;
        self.0.server_db.insert_db_user(db_id, user_id, role).await
    }

    pub(crate) async fn audit(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        config: &Config,
    ) -> ServerResult<DbAudit> {
        let db_name = db_name(owner, db);
        self.0
            .server_db
            .find_user_db_id(user, &db_name)
            .await?
            .ok_or(db_not_found(&db_name))?;

        if let Ok(log) = std::fs::OpenOptions::new()
            .read(true)
            .open(db_audit_file(owner, db, config))
        {
            return Ok(DbAudit(serde_json::from_reader(log)?));
        }

        Ok(DbAudit(vec![]))
    }

    pub(crate) async fn backup_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        config: &Config,
    ) -> ServerResult {
        let db_name = db_name(owner, db);
        let mut database = self.0.server_db.user_db(user, &db_name).await?;

        if !self
            .0
            .server_db
            .is_db_admin(user, database.db_id.unwrap())
            .await?
        {
            return Err(permission_denied("admin only"));
        }

        let pool = self.get_pool().await;
        let server_db = pool
            .get(&database.name)
            .ok_or(db_not_found(&database.name))?;

        self.do_backup(owner, db, config, server_db, &mut database)
            .await
    }

    pub(crate) async fn change_password(
        &self,
        mut user: ServerUser,
        new_password: &str,
    ) -> ServerResult {
        password::validate_password(new_password)?;
        let pswd = Password::create(&user.username, new_password);
        user.password = pswd.password.to_vec();
        user.salt = pswd.user_salt.to_vec();
        self.0.server_db.save_user(user).await
    }

    pub(crate) async fn clear_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        config: &Config,
        resource: DbResource,
    ) -> ServerResult<ServerDatabase> {
        let db_name = db_name(owner, db);
        let mut database = self.0.server_db.user_db(user, &db_name).await?;
        let role = self.0.server_db.user_db_role(user, &db_name).await?;

        if role != DbUserRole::Admin {
            return Err(permission_denied("admin only"));
        }

        match resource {
            DbResource::All => {
                self.do_clear_db(&database, owner, db, config, db_name)
                    .await?;
                do_clear_db_audit(owner, db, config)?;
                self.do_clear_db_backup(owner, db, config, &mut database)
                    .await?;
            }
            DbResource::Db => {
                self.do_clear_db(&database, owner, db, config, db_name)
                    .await?;
            }
            DbResource::Audit => {
                do_clear_db_audit(owner, db, config)?;
            }
            DbResource::Backup => {
                self.do_clear_db_backup(owner, db, config, &mut database)
                    .await?;
            }
        }

        let size = self
            .get_pool()
            .await
            .get(&database.name)
            .ok_or(db_not_found(&database.name))?
            .size()
            .await;

        Ok(ServerDatabase {
            name: database.name,
            db_type: database.db_type,
            role,
            size,
            backup: database.backup,
        })
    }

    async fn do_clear_db_backup(
        &self,
        owner: &str,
        db: &str,
        config: &Config,
        database: &mut Database,
    ) -> Result<(), ServerError> {
        let backup_file = if database.db_type == DbType::Memory {
            db_file(owner, db, config)
        } else {
            db_backup_file(owner, db, config)
        };
        if backup_file.exists() {
            std::fs::remove_file(&backup_file)?;
        }
        database.backup = 0;
        self.0.server_db.save_db(database).await?;
        Ok(())
    }

    async fn do_clear_db(
        &self,
        database: &Database,
        owner: &str,
        db: &str,
        config: &Config,
        db_name: String,
    ) -> Result<(), ServerError> {
        let mut pool = self.get_pool_mut().await;
        let server_db = pool
            .get_mut(&database.name)
            .ok_or(db_not_found(&database.name))?;
        *server_db = UserDb::new(&format!("{}:{}", DbType::Memory, database.name))?;
        if database.db_type != DbType::Memory {
            let main_file = db_file(owner, db, config);
            if main_file.exists() {
                std::fs::remove_file(&main_file)?;
            }

            let wal_file = db_file(owner, &format!(".{db}"), config);
            if wal_file.exists() {
                std::fs::remove_file(wal_file)?;
            }

            let db_path = Path::new(&config.data_dir).join(&db_name);
            let path = db_path.to_str().ok_or(ErrorCode::DbInvalid)?.to_string();
            *server_db = UserDb::new(&format!("{}:{}", database.db_type, path))?;
        }

        Ok(())
    }

    async fn do_backup(
        &self,
        owner: &str,
        db: &str,
        config: &Config,
        server_db: &UserDb,
        database: &mut Database,
    ) -> Result<(), ServerError> {
        let backup_path = if database.db_type == DbType::Memory {
            db_file(owner, db, config)
        } else {
            db_backup_file(owner, db, config)
        };
        if backup_path.exists() {
            std::fs::remove_file(&backup_path)?;
        } else {
            std::fs::create_dir_all(db_backup_dir(owner, config))?;
        }
        server_db
            .backup(backup_path.to_string_lossy().as_ref())
            .await?;

        database.backup = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        self.0.server_db.save_db(database).await?;
        Ok(())
    }

    pub(crate) async fn convert_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        db_type: DbType,
        config: &Config,
    ) -> ServerResult {
        let db_name = db_name(owner, db);
        let mut database = self.0.server_db.user_db(user, &db_name).await?;

        if database.db_type == db_type {
            return Ok(());
        }

        if !self
            .0
            .server_db
            .is_db_admin(user, database.db_id.unwrap())
            .await?
        {
            return Err(permission_denied("admin only"));
        }

        let mut pool = self.get_pool_mut().await;
        pool.remove(&db_name);

        let current_path = db_file(owner, db, config);
        let server_db = UserDb::new(&format!("{}:{}", db_type, current_path.to_string_lossy()))?;
        pool.insert(db_name, server_db);

        database.db_type = db_type;
        self.0.server_db.save_db(&database).await?;

        Ok(())
    }

    pub(crate) async fn copy_db(
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
        let database = self.0.server_db.user_db(user, &source_db).await?;

        if admin {
            user = self.0.server_db.user_id(new_owner).await?;
        } else {
            let username = self.0.server_db.user_name(user).await?;

            if new_owner != username {
                return Err(permission_denied("cannot copy db to another user"));
            }
        };

        if self
            .0
            .server_db
            .find_user_db_id(user, &target_db)
            .await?
            .is_some()
        {
            return Err(ErrorCode::DbExists.into());
        }

        let target_file = db_file(new_owner, new_db, config);

        if target_file.exists() {
            return Err(ErrorCode::DbExists.into());
        }

        std::fs::create_dir_all(Path::new(&config.data_dir).join(new_owner))?;
        let server_db = self
            .get_pool()
            .await
            .get(&source_db)
            .ok_or(db_not_found(&source_db))?
            .copy(target_file.to_string_lossy().as_ref())
            .await
            .map_err(|e| {
                ServerError::new(
                    ErrorCode::DbInvalid.into(),
                    &format!("db copy error: {}", e.description),
                )
            })?;
        self.get_pool_mut()
            .await
            .insert(target_db.clone(), server_db);
        self.0
            .server_db
            .insert_db(
                user,
                Database {
                    db_id: None,
                    name: target_db.clone(),
                    db_type: database.db_type,
                    backup: 0,
                },
            )
            .await?;

        Ok(())
    }

    pub(crate) async fn delete_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        config: &Config,
    ) -> ServerResult {
        self.remove_db(owner, db, user).await?;

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

        let audit_file = db_audit_file(owner, db, config);
        if audit_file.exists() {
            std::fs::remove_file(audit_file)?;
        }

        Ok(())
    }

    pub(crate) async fn exec(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        queries: Queries,
        config: &Config,
    ) -> ServerResult<Vec<QueryResult>> {
        let db_name = db_name(owner, db);
        let role = self.0.server_db.user_db_role(user, &db_name).await?;
        let required_role = required_role(&queries);

        if required_role == DbUserRole::Write && role == DbUserRole::Read {
            return Err(permission_denied("write rights required"));
        }

        let results = if required_role == DbUserRole::Read {
            self.get_pool()
                .await
                .get(&db_name)
                .ok_or(db_not_found(&db_name))?
                .exec(queries)
                .await
        } else {
            let username = self.0.server_db.user_name(user).await?;

            let (r, audit) = self
                .get_pool()
                .await
                .get(&db_name)
                .ok_or(db_not_found(&db_name))?
                .exec_mut(queries, &username)
                .await?;

            if !audit.is_empty() {
                let mut log = std::fs::OpenOptions::new()
                    .create(true)
                    .truncate(false)
                    .write(true)
                    .open(db_audit_file(owner, db, config))?;
                let len = log.seek(SeekFrom::End(0))?;
                if len == 0 {
                    serde_json::to_writer(&log, &audit)?;
                } else {
                    let mut data = serde_json::to_vec(&audit)?;
                    data[0] = b',';
                    log.seek(SeekFrom::End(-1))?;
                    log.write_all(&data)?;
                }
            }

            Ok(r)
        }?;

        Ok(results)
    }

    pub(crate) async fn find_dbs(&self) -> ServerResult<Vec<ServerDatabase>> {
        let pool = self.get_pool().await;
        let dbs = self.0.server_db.dbs().await?;

        let mut databases = Vec::with_capacity(dbs.len());

        for db in dbs {
            databases.push(ServerDatabase {
                db_type: db.db_type,
                role: DbUserRole::Admin,
                size: pool
                    .get(&db.name)
                    .ok_or(db_not_found(&db.name))?
                    .size()
                    .await,
                backup: db.backup,
                name: db.name,
            });
        }

        Ok(databases)
    }

    pub(crate) async fn find_user_dbs(&self, user: DbId) -> ServerResult<Vec<ServerDatabase>> {
        let user_dbs = self.0.server_db.user_dbs(user).await?;
        let mut dbs: Vec<ServerDatabase> = user_dbs
            .into_iter()
            .map(|(role, db)| ServerDatabase {
                name: db.name,
                db_type: db.db_type,
                role,
                size: 0,
                backup: db.backup,
            })
            .collect();

        for db in dbs.iter_mut() {
            db.size = self
                .get_pool()
                .await
                .get(&db.name)
                .ok_or(db_not_found(&db.name))?
                .size()
                .await;
        }

        Ok(dbs)
    }

    pub(crate) async fn db_users(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
    ) -> ServerResult<Vec<DbUser>> {
        let db = db_name(owner, db);
        let db_id = self.0.server_db.user_db_id(user, &db).await?;
        self.0.server_db.db_users(db_id).await
    }

    pub(crate) async fn optimize_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
    ) -> ServerResult<ServerDatabase> {
        let db_name = db_name(owner, db);
        let db = self.0.server_db.user_db(user, &db_name).await?;
        let role = self.0.server_db.user_db_role(user, &db_name).await?;

        if role == DbUserRole::Read {
            return Err(permission_denied("write rights required"));
        }

        let pool = self.get_pool().await;
        let server_db = pool.get(&db.name).ok_or(db_not_found(&db.name))?;
        server_db.optimize_storage().await?;
        let size = server_db.size().await;

        Ok(ServerDatabase {
            name: db.name,
            db_type: db.db_type,
            role,
            size,
            backup: db.backup,
        })
    }

    pub(crate) async fn remove_db_user(
        &self,
        owner: &str,
        db: &str,
        username: &str,
        user: DbId,
    ) -> ServerResult {
        if owner == username {
            return Err(permission_denied("cannot remove owner"));
        }

        let db_id = self
            .0
            .server_db
            .user_db_id(user, &db_name(owner, db))
            .await?;
        let user_id = self.0.server_db.user_id(username).await?;

        if user != user_id && !self.0.server_db.is_db_admin(user, db_id).await? {
            return Err(permission_denied("admin only"));
        }

        self.0.server_db.remove_db_user(db_id, user).await
    }

    pub(crate) async fn remove_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
    ) -> ServerResult<UserDb> {
        let user_name = self.0.server_db.user_name(user).await?;

        if owner != user_name {
            return Err(permission_denied("owner only"));
        }

        self.0.server_db.remove_db(user, db).await?;
        let db_name = db_name(owner, db);
        Ok(self.get_pool_mut().await.remove(&db_name).unwrap())
    }

    pub(crate) async fn remove_user(&self, username: &str, config: &Config) -> ServerResult {
        let db_names = self.0.server_db.remove_user(username).await?;

        for db in db_names.into_iter() {
            self.get_pool_mut().await.remove(&db);
        }

        let user_dir = Path::new(&config.data_dir).join(username);
        if user_dir.exists() {
            std::fs::remove_dir_all(user_dir)?;
        }

        Ok(())
    }

    pub(crate) async fn rename_db(
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
        let username = self.0.server_db.user_name(user).await?;

        if owner != username {
            return Err(permission_denied("owner only"));
        }

        let mut database = self.0.server_db.user_db(user, &db_name).await?;
        let target_name = db_file(new_owner, new_db, config);

        if target_name.exists() {
            return Err(ErrorCode::DbExists.into());
        }

        if new_owner != owner {
            std::fs::create_dir_all(Path::new(&config.data_dir).join(new_owner))?;
            self.add_db_user(owner, db, new_owner, DbUserRole::Admin, user)
                .await?;
        }

        let server_db = UserDb(
            self.get_pool()
                .await
                .get(&db_name)
                .ok_or(db_not_found(&db_name))?
                .0
                .clone(),
        );

        server_db
            .rename(target_name.to_string_lossy().as_ref())
            .await
            .map_err(|mut e| {
                e.status = ErrorCode::DbInvalid.into();
                e
            })?;

        let backup_path = db_backup_file(owner, db, config);
        if backup_path.exists() {
            let new_backup_path = db_backup_file(new_owner, new_db, config);
            let backups_dir = new_backup_path.parent().unwrap();
            std::fs::create_dir_all(backups_dir)?;
            std::fs::rename(backup_path, new_backup_path)?;
        }

        self.get_pool_mut()
            .await
            .insert(new_name.to_string(), server_db);

        database.name = new_name.to_string();
        self.0.server_db.save_db(&database).await?;

        self.get_pool_mut().await.remove(&db_name).unwrap();

        Ok(())
    }

    pub(crate) async fn restore_db(
        &self,
        owner: &str,
        db: &str,
        user: DbId,
        config: &Config,
    ) -> ServerResult {
        let db_name = db_name(owner, db);
        let mut database = self.0.server_db.user_db(user, &db_name).await?;

        if !self
            .0
            .server_db
            .is_db_admin(user, database.db_id.unwrap())
            .await?
        {
            return Err(permission_denied("admin only"));
        }

        let backup_path = if database.db_type == DbType::Memory {
            db_file(owner, db, config)
        } else {
            db_backup_file(owner, db, config)
        };

        if !backup_path.exists() {
            return Err(ServerError {
                description: "backup not found".to_string(),
                status: StatusCode::NOT_FOUND,
            });
        }

        let mut pool = self.get_pool_mut().await;
        pool.remove(&db_name);
        let current_path = db_file(owner, db, config);

        if database.db_type != DbType::Memory {
            let backup_temp = db_backup_dir(owner, config).join(db);
            std::fs::rename(&current_path, &backup_temp)?;
            std::fs::rename(&backup_path, &current_path)?;
            std::fs::rename(backup_temp, backup_path)?;
            database.backup = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        }

        let server_db = UserDb::new(&format!(
            "{}:{}",
            database.db_type,
            current_path.to_string_lossy()
        ))?;
        pool.insert(db_name, server_db);
        self.0.server_db.save_db(&database).await?;

        Ok(())
    }

    pub(crate) async fn user_token(&self, user: DbId) -> ServerResult<String> {
        let mut user_token = self.0.server_db.user_token(user).await?;

        if user_token.is_empty() {
            let token_uuid = Uuid::new_v4();
            user_token = token_uuid.to_string();
            self.0.server_db.save_token(user, &user_token).await?;
        }

        Ok(user_token)
    }

    pub(crate) async fn status(&self, config: &Config) -> ServerResult<AdminStatus> {
        Ok(AdminStatus {
            uptime: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - config.start_time,
            dbs: self.0.server_db.db_count().await?,
            users: self.0.server_db.user_count().await?,
            logged_in_users: self.0.server_db.user_token_count().await?,
            size: get_size(&config.data_dir).await?,
        })
    }

    async fn get_pool(&self) -> RwLockReadGuard<HashMap<String, UserDb>> {
        self.0.pool.read().await
    }

    async fn get_pool_mut(&self) -> RwLockWriteGuard<HashMap<String, UserDb>> {
        self.0.pool.write().await
    }
}

fn do_clear_db_audit(owner: &str, db: &str, config: &Config) -> Result<(), ServerError> {
    let audit_file = db_audit_file(owner, db, config);

    if audit_file.exists() {
        std::fs::remove_file(&audit_file)?;
    }

    Ok(())
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

fn db_audit_dir(owner: &str, config: &Config) -> PathBuf {
    Path::new(&config.data_dir).join(owner).join("audit")
}

fn db_audit_file(owner: &str, db: &str, config: &Config) -> PathBuf {
    db_audit_dir(owner, config).join(format!("{db}.log"))
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
