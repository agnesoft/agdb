mod user_db;
mod user_db_storage;

use crate::config::Config;
use crate::error_code::ErrorCode;
use crate::server_db::Database;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use crate::utilities::remove_file_if_exists;
use agdb::QueryResult;
use agdb_api::DbAudit;
use agdb_api::DbResource;
use agdb_api::DbType;
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
use user_db::UserDb;

#[derive(Clone)]
pub(crate) struct DbPool(pub(crate) Arc<RwLock<HashMap<String, UserDb>>>);

impl DbPool {
    async fn db(&self, name: &str) -> ServerResult<UserDb> {
        self.0
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| ServerError::new(StatusCode::NOT_FOUND, "db not found"))
    }

    pub(crate) async fn new(config: &Config, server_db: &ServerDb) -> ServerResult<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        let db_pool = Self(Arc::new(RwLock::new(HashMap::new())));

        for db in server_db.dbs().await? {
            let (owner, db_name) = db.name.split_once('/').ok_or(ErrorCode::DbInvalid)?;
            let db_path = db_file(owner, db_name, config);
            std::fs::create_dir_all(db_audit_dir(owner, config))?;
            let server_db = UserDb::new(&format!("{}:{}", db.db_type, db_path.to_string_lossy()))?;
            db_pool.0.write().await.insert(db.name, server_db);
        }

        Ok(db_pool)
    }

    pub(crate) async fn add_db(
        &self,
        owner: &str,
        db: &str,
        db_name: &str,
        db_type: DbType,
        config: &Config,
    ) -> ServerResult<u64> {
        let db_path = Path::new(&config.data_dir).join(db_name);
        let path = db_path.to_str().ok_or(ErrorCode::DbInvalid)?.to_string();

        std::fs::create_dir_all(db_audit_dir(owner, config))?;

        let user_db = UserDb::new(&format!("{}:{}", db_type, path)).map_err(|mut e| {
            e.status = ErrorCode::DbInvalid.into();
            e.description = format!("{}: {}", ErrorCode::DbInvalid.as_str(), e.description);
            e
        })?;

        let backup = if std::fs::exists(db_backup_file(owner, db, config))? {
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        } else {
            0
        };

        self.0.write().await.insert(db_name.to_string(), user_db);

        Ok(backup)
    }

    pub(crate) async fn audit(
        &self,
        owner: &str,
        db: &str,
        config: &Config,
    ) -> ServerResult<DbAudit> {
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
        db_name: &str,
        db_type: DbType,
        config: &Config,
    ) -> ServerResult<u64> {
        let user_db = self.db(db_name).await?;

        let backup_path = if db_type == DbType::Memory {
            db_file(owner, db, config)
        } else {
            db_backup_file(owner, db, config)
        };

        remove_file_if_exists(&backup_path)?;
        std::fs::create_dir_all(db_backup_dir(owner, config))?;

        user_db
            .backup(backup_path.to_string_lossy().as_ref())
            .await?;

        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
    }

    pub(crate) async fn clear_db(
        &self,
        owner: &str,
        db: &str,
        database: &mut Database,
        role: DbUserRole,
        config: &Config,
        resource: DbResource,
    ) -> ServerResult<ServerDatabase> {
        match resource {
            DbResource::All => {
                self.do_clear_db(owner, db, database, config).await?;
                remove_file_if_exists(db_audit_file(owner, db, config))?;
                self.do_clear_db_backup(owner, db, config, database).await?;
            }
            DbResource::Db => {
                self.do_clear_db(owner, db, database, config).await?;
            }
            DbResource::Audit => {
                remove_file_if_exists(db_audit_file(owner, db, config))?;
            }
            DbResource::Backup => {
                self.do_clear_db_backup(owner, db, config, database).await?;
            }
        }

        let size = self.db_size(&database.name).await?;

        Ok(ServerDatabase {
            name: database.name.clone(),
            db_type: database.db_type,
            role,
            size,
            backup: database.backup,
        })
    }

    pub(crate) async fn db_size(&self, name: &str) -> ServerResult<u64> {
        Ok(self
            .0
            .read()
            .await
            .get(name)
            .ok_or(db_not_found(name))?
            .size()
            .await)
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
        remove_file_if_exists(&backup_file)?;
        database.backup = 0;
        Ok(())
    }

    async fn do_clear_db(
        &self,
        owner: &str,
        db: &str,
        database: &Database,
        config: &Config,
    ) -> Result<(), ServerError> {
        let mut pool = self.0.write().await;
        let user_db = pool
            .get_mut(&database.name)
            .ok_or(db_not_found(&database.name))?;
        *user_db = UserDb::new(&format!("{}:{}", DbType::Memory, &database.name))?;
        if database.db_type != DbType::Memory {
            remove_file_if_exists(db_file(owner, db, config))?;
            remove_file_if_exists(db_file(owner, &format!(".{db}"), config))?;
            let db_path = Path::new(&config.data_dir).join(&database.name);
            let path = db_path.to_str().ok_or(ErrorCode::DbInvalid)?.to_string();
            *user_db = UserDb::new(&format!("{}:{path}", database.db_type))?;
        }

        Ok(())
    }

    pub(crate) async fn convert_db(
        &self,
        owner: &str,
        db: &str,
        db_name: &str,
        db_type: DbType,
        target_type: DbType,
        config: &Config,
    ) -> ServerResult {
        let mut user_db = self.0.write().await.remove(db_name).unwrap();
        let current_path = db_file(owner, db, config);

        if db_type == DbType::Memory {
            user_db
                .0
                .read()
                .await
                .backup(current_path.to_string_lossy().as_ref())?;
        }

        user_db = UserDb::new(&format!(
            "{}:{}",
            target_type,
            current_path.to_string_lossy()
        ))?;

        self.0.write().await.insert(db_name.to_string(), user_db);

        Ok(())
    }

    pub(crate) async fn copy_db(
        &self,
        source_db: &str,
        new_owner: &str,
        new_db: &str,
        target_db: &str,
        config: &Config,
    ) -> ServerResult {
        let target_file = db_file(new_owner, new_db, config);

        if std::fs::exists(&target_file)
            .map_err(|e| ServerError::new(ErrorCode::DbInvalid.into(), &e.to_string()))?
        {
            return Err(ErrorCode::DbExists.into());
        }

        std::fs::create_dir_all(Path::new(&config.data_dir).join(new_owner))?;

        let user_db = self
            .db(source_db)
            .await?
            .copy(target_file.to_string_lossy().as_ref())
            .await?;
        self.0.write().await.insert(target_db.to_string(), user_db);

        Ok(())
    }

    pub(crate) async fn delete_db(
        &self,
        owner: &str,
        db: &str,
        db_name: &str,
        config: &Config,
    ) -> ServerResult {
        self.remove_db(db_name).await?;
        remove_file_if_exists(db_file(owner, db, config))?;
        remove_file_if_exists(db_file(owner, &format!(".{db}"), config))?;
        remove_file_if_exists(db_backup_file(owner, db, config))?;
        remove_file_if_exists(db_audit_file(owner, db, config))
    }

    pub(crate) async fn exec(
        &self,
        db_name: &str,
        queries: Queries,
    ) -> ServerResult<Vec<QueryResult>> {
        self.db(db_name).await?.exec(queries).await
    }

    pub(crate) async fn exec_mut(
        &self,
        owner: &str,
        db: &str,
        db_name: &str,
        username: &str,
        queries: Queries,
        config: &Config,
    ) -> ServerResult<Vec<QueryResult>> {
        let (r, audit) = self.db(db_name).await?.exec_mut(queries, username).await?;

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
    }

    pub(crate) async fn optimize_db(&self, db_name: &str) -> ServerResult<u64> {
        let user_db = self.db(db_name).await?;
        user_db.optimize_storage().await?;
        Ok(user_db.size().await)
    }

    pub(crate) async fn remove_db(&self, db_name: &str) -> ServerResult<UserDb> {
        Ok(self.0.write().await.remove(db_name).unwrap())
    }

    pub(crate) async fn remove_user_dbs(
        &self,
        username: &str,
        dbs: &[String],
        config: &Config,
    ) -> ServerResult {
        for db in dbs {
            self.0.write().await.remove(db);
        }

        let user_dir = Path::new(&config.data_dir).join(username);
        if std::fs::exists(&user_dir)? {
            std::fs::remove_dir_all(&user_dir)?;
        }

        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) async fn rename_db(
        &self,
        owner: &str,
        db: &str,
        source_db: &str,
        new_owner: &str,
        new_db: &str,
        target_db: &str,
        config: &Config,
    ) -> ServerResult {
        let target_name = db_file(new_owner, new_db, config);

        if target_name.exists() {
            return Err(ErrorCode::DbExists.into());
        }

        if new_owner != owner {
            std::fs::create_dir_all(Path::new(&config.data_dir).join(new_owner))?;
        }

        let user_db = self.db(source_db).await?;

        user_db
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

        self.0.write().await.insert(target_db.to_string(), user_db);
        self.0.write().await.remove(source_db).unwrap();

        Ok(())
    }

    pub(crate) async fn restore_db(
        &self,
        owner: &str,
        db: &str,
        db_name: &str,
        db_type: DbType,
        config: &Config,
    ) -> ServerResult<Option<u64>> {
        let backup_path = if db_type == DbType::Memory {
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

        self.0.write().await.remove(db_name);
        let current_path = db_file(owner, db, config);

        let backup = if db_type != DbType::Memory {
            let backup_temp = db_backup_dir(owner, config).join(db);
            std::fs::rename(&current_path, &backup_temp)?;
            std::fs::rename(&backup_path, &current_path)?;
            std::fs::rename(backup_temp, backup_path)?;
            Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
        } else {
            None
        };

        let user_db = UserDb::new(&format!("{}:{}", db_type, current_path.to_string_lossy()))?;
        self.0.write().await.insert(db_name.to_string(), user_db);

        Ok(backup)
    }
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
