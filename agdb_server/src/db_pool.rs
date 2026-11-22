pub(crate) mod audit;
pub(crate) mod user_db;

use crate::config::Config;
use crate::error_code::ErrorCode;
use crate::server_db::Database;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use crate::utilities::remove_file_if_exists;
use agdb::QueryResult;
use agdb_api::DbAudit;
use agdb_api::DbKind;
use agdb_api::DbResource;
use agdb_api::Queries;
use axum::http::StatusCode;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::sync::RwLock;
use user_db::UserDb;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DbName {
    pub(crate) owner: String,
    pub(crate) db: String,
}

impl DbName {
    fn new(owner: &str, db: &str) -> Self {
        Self {
            owner: owner.to_string(),
            db: db.to_string(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct DbPool {
    pub(crate) pool: Arc<RwLock<HashMap<DbName, UserDb>>>,
    config: Config,
}

pub(crate) async fn new(config: Config, server_db: &ServerDb) -> ServerResult<DbPool> {
    std::fs::create_dir_all(&config.data_dir)?;

    let db_pool = DbPool {
        pool: Arc::new(RwLock::new(HashMap::new())),
        config,
    };

    db_pool.reload(server_db).await?;

    Ok(db_pool)
}

impl DbPool {
    pub(crate) async fn clear(&self) {
        self.pool.write().await.clear();
    }

    pub(crate) async fn reload(&self, server_db: &ServerDb) -> ServerResult<()> {
        let mut pool = self.pool.write().await;
        pool.clear();

        for db in server_db.dbs().await? {
            let db_path = db_file(&db.owner, &db.db, &self.config);
            std::fs::create_dir_all(Path::new(&self.config.data_dir).join(&db.owner))?;
            self.migrate_audit(&db.owner, &db.db)?;
            if let Err(e) = self.finish_audit_swap(&db.owner, &db.db) {
                crate::warn!("{}", e.description);
            }
            if db.db_type == DbKind::Memory && !db_path.exists() {
                let backup = db_backup_file(&db.owner, &db.db, &self.config);
                if backup.exists() {
                    std::fs::copy(&backup, &db_path)?;
                }
            }
            let user_db = UserDb::new(
                db_path.to_string_lossy().as_ref(),
                db.db_type,
                self.config.storage,
            )?;
            if db.db_type == DbKind::Memory {
                remove_file_if_exists(&db_path)?;
            }
            pool.insert(db.name(), user_db);
        }

        Ok(())
    }

    async fn db(&self, owner: &str, db: &str) -> ServerResult<UserDb> {
        self.pool
            .read()
            .await
            .get(&DbName::new(owner, db))
            .cloned()
            .ok_or_else(|| ServerError::new(StatusCode::NOT_FOUND, "db not found"))
    }

    pub(crate) async fn add_db(&self, owner: &str, db: &str, db_type: DbKind) -> ServerResult<u64> {
        let db_path = Path::new(&self.config.data_dir).join(owner).join(db);
        let path = db_path.to_str().ok_or(ErrorCode::DbInvalid)?.to_string();

        std::fs::create_dir_all(Path::new(&self.config.data_dir).join(owner))?;
        self.migrate_audit(owner, db)?;

        let user_db = UserDb::new(&path, db_type, self.config.storage).map_err(|mut e| {
            e.status = ErrorCode::DbInvalid.into();
            e.description = format!("{}: {}", ErrorCode::DbInvalid.as_str(), e.description);
            e
        })?;

        let backup = if std::fs::exists(db_backup_file(owner, db, &self.config))? {
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        } else {
            0
        };

        self.pool
            .write()
            .await
            .insert(DbName::new(owner, db), user_db);

        Ok(backup)
    }

    pub(crate) async fn audit(&self, owner: &str, db: &str) -> ServerResult<DbAudit> {
        let dir = db_audit_dir(owner, db, &self.config);
        Ok(DbAudit(
            tokio::task::spawn_blocking(move || audit::read(&dir)).await??,
        ))
    }

    pub(crate) async fn audit_trim(&self, owner: &str, db: &str, first_day: u64) -> ServerResult {
        self.db(owner, db).await?;
        audit::trim(&db_audit_dir(owner, db, &self.config), first_day)?;
        audit::trim(&db_backup_audit_dir(owner, db, &self.config), first_day)
    }

    fn migrate_audit(&self, owner: &str, db: &str) -> ServerResult {
        audit::migrate(
            &db_audit_legacy_file(owner, db, &self.config),
            &db_audit_dir(owner, db, &self.config),
        )?;
        audit::migrate(
            &db_backup_audit_legacy_file(owner, db, &self.config),
            &db_backup_audit_dir(owner, db, &self.config),
        )
    }

    pub(crate) async fn backup_db(&self, owner: &str, db: &str) -> ServerResult<u64> {
        let user_db = self.db(owner, db).await?;

        let backup_path = db_backup_file(owner, db, &self.config);

        remove_file_if_exists(&backup_path)?;
        std::fs::create_dir_all(db_backup_dir(owner, &self.config))?;

        let db_guard = user_db.0.read().await;
        db_guard.backup(backup_path.to_string_lossy().as_ref())?;

        audit::sync(
            &db_audit_dir(owner, db, &self.config),
            &db_backup_audit_dir(owner, db, &self.config),
        )?;

        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
    }

    pub(crate) async fn clear_db(
        &self,
        owner: &str,
        db: &str,
        database: &mut Database,
        resource: DbResource,
    ) -> ServerResult {
        match resource {
            DbResource::All => {
                self.do_clear_db(owner, db, database).await?;
                audit::remove(&db_audit_dir(owner, db, &self.config))?;
                self.do_clear_db_backup(owner, db, database).await?;
            }
            DbResource::Db => {
                self.do_clear_db(owner, db, database).await?;
            }
            DbResource::Audit => {
                audit::remove(&db_audit_dir(owner, db, &self.config))?;
            }
            DbResource::Backup => {
                self.do_clear_db_backup(owner, db, database).await?;
            }
        }

        Ok(())
    }

    pub(crate) async fn db_size(&self, owner: &str, db: &str) -> ServerResult<u64> {
        Ok(self.db(owner, db).await?.size().await)
    }

    async fn do_clear_db_backup(
        &self,
        owner: &str,
        db: &str,
        database: &mut Database,
    ) -> Result<(), ServerError> {
        let backup_file = db_backup_file(owner, db, &self.config);
        remove_file_if_exists(&backup_file)?;
        audit::remove(&db_backup_audit_dir(owner, db, &self.config))?;
        database.backup = 0;
        Ok(())
    }

    async fn do_clear_db(
        &self,
        owner: &str,
        db: &str,
        database: &Database,
    ) -> Result<(), ServerError> {
        let mut pool = self.pool.write().await;
        let name = database.name();
        let db_path = Path::new(&self.config.data_dir)
            .join(owner)
            .join(db)
            .to_str()
            .ok_or(ErrorCode::DbInvalid)?
            .to_string();
        let user_db = pool.get_mut(&name).ok_or(db_not_found(owner, db))?;
        *user_db = UserDb::new(&db_path, DbKind::Memory, self.config.storage)?;
        if database.db_type != DbKind::Memory {
            remove_file_if_exists(db_file(owner, db, &self.config))?;
            remove_file_if_exists(db_file(owner, &format!(".{db}"), &self.config))?;
            *user_db = UserDb::new(&db_path, database.db_type, self.config.storage)?;
        }

        Ok(())
    }

    pub(crate) async fn convert_db(
        &self,
        owner: &str,
        db: &str,
        db_type: DbKind,
        target_type: DbKind,
    ) -> ServerResult {
        let db_name = DbName::new(owner, db);
        let mut user_db = self.remove_db(owner, db).await?;

        match self
            .do_convert(&mut user_db, owner, db, db_type, target_type)
            .await
        {
            Ok(_) => {
                self.pool.write().await.insert(db_name, user_db);
                Ok(())
            }
            Err(e) => {
                self.pool.write().await.insert(db_name, user_db);
                Err(e)
            }
        }
    }

    async fn do_convert(
        &self,
        user_db: &mut UserDb,
        owner: &str,
        db: &str,
        db_type: DbKind,
        target_type: DbKind,
    ) -> Result<(), ServerError> {
        let current_path = db_file(owner, db, &self.config);
        if db_type == DbKind::Memory {
            user_db
                .0
                .read()
                .await
                .backup(current_path.to_string_lossy().as_ref())?;
        }
        *user_db = UserDb::new(
            current_path.to_string_lossy().as_ref(),
            target_type,
            self.config.storage,
        )?;
        if target_type == DbKind::Memory {
            remove_file_if_exists(&current_path)?;
        }

        Ok(())
    }

    pub(crate) async fn copy_db(
        &self,
        owner: &str,
        db: &str,
        new_owner: &str,
        new_db: &str,
    ) -> ServerResult {
        let target_file = db_file(new_owner, new_db, &self.config);

        if std::fs::exists(&target_file)
            .map_err(|e| ServerError::new(ErrorCode::DbInvalid.into(), &e.to_string()))?
        {
            return Err(ErrorCode::DbExists.into());
        }

        std::fs::create_dir_all(Path::new(&self.config.data_dir).join(new_owner))?;

        let target_db = DbName::new(new_owner, new_db);
        let cloned_db;

        {
            let user_db = self.db(owner, db).await?;
            let db_guard = user_db.0.read().await;
            cloned_db = db_guard.copy(target_file.to_string_lossy().as_ref())?;

            audit::sync(
                &db_audit_dir(owner, db, &self.config),
                &db_audit_dir(new_owner, new_db, &self.config),
            )?;
        }

        self.pool
            .write()
            .await
            .insert(target_db, UserDb(Arc::new(RwLock::new(cloned_db))));

        Ok(())
    }

    pub(crate) async fn delete_db(&self, owner: &str, db: &str) -> ServerResult {
        self.remove_db(owner, db).await?;
        remove_file_if_exists(db_file(owner, db, &self.config))?;
        remove_file_if_exists(db_file(owner, &format!(".{db}"), &self.config))?;
        remove_file_if_exists(db_backup_file(owner, db, &self.config))?;
        audit::remove(&db_backup_audit_dir(owner, db, &self.config))?;
        audit::remove_legacy(&db_backup_audit_legacy_file(owner, db, &self.config))?;
        audit::remove(&db_audit_dir(owner, db, &self.config))?;
        audit::remove_legacy(&db_audit_legacy_file(owner, db, &self.config))
    }

    pub(crate) async fn exec(
        &self,
        owner: &str,
        db: &str,
        queries: Queries,
    ) -> ServerResult<Vec<QueryResult>> {
        self.db(owner, db).await?.exec(queries).await
    }

    pub(crate) async fn exec_mut(
        &self,
        owner: &str,
        db: &str,
        username: &str,
        queries: Queries,
    ) -> ServerResult<Vec<QueryResult>> {
        let (r, audit) = self
            .db(owner, db)
            .await?
            .exec_mut(queries, username)
            .await?;

        if !audit.is_empty() {
            audit::append(&db_audit_dir(owner, db, &self.config), &audit)?;
        }

        Ok(r)
    }

    pub(crate) async fn optimize_db(&self, owner: &str, db: &str) -> ServerResult {
        let user_db = self.db(owner, db).await?;
        user_db.optimize_storage().await?;
        Ok(())
    }

    pub(crate) async fn remove_db(&self, owner: &str, db: &str) -> ServerResult<UserDb> {
        self.pool
            .write()
            .await
            .remove(&DbName::new(owner, db))
            .ok_or(ServerError {
                description: format!("db not found - cannot remove '{owner}/{db}'"),
                status: StatusCode::NOT_FOUND,
            })
    }

    pub(crate) async fn remove_user_dbs(&self, username: &str, dbs: &[DbName]) -> ServerResult {
        for db in dbs {
            self.pool.write().await.remove(db);
        }

        let user_dir = Path::new(&self.config.data_dir).join(username);
        if std::fs::exists(&user_dir)? {
            std::fs::remove_dir_all(&user_dir)?;
        }

        Ok(())
    }

    pub(crate) async fn rename_db(
        &self,
        owner: &str,
        db: &str,
        new_owner: &str,
        new_db: &str,
    ) -> ServerResult {
        let target_name = db_file(new_owner, new_db, &self.config);

        if target_name.exists() {
            return Err(ErrorCode::DbExists.into());
        }

        if new_owner != owner {
            std::fs::create_dir_all(Path::new(&self.config.data_dir).join(new_owner))?;
        }

        let source_db = DbName::new(owner, db);
        let target_db = DbName::new(new_owner, new_db);

        let user_db = self.remove_db(owner, db).await?;

        match self
            .do_rename(target_name, &user_db, owner, db, new_owner, new_db)
            .await
        {
            Ok(_) => {
                self.pool.write().await.insert(target_db, user_db);
                Ok(())
            }
            Err(e) => {
                self.pool.write().await.insert(source_db, user_db);
                Err(e)
            }
        }
    }

    async fn do_rename(
        &self,
        target_name: PathBuf,
        user_db: &UserDb,
        owner: &str,
        db: &str,
        new_owner: &str,
        new_db: &str,
    ) -> Result<(), ServerError> {
        user_db
            .rename(target_name.to_string_lossy().as_ref())
            .await
            .map_err(|mut e| {
                e.status = ErrorCode::DbInvalid.into();
                e
            })?;
        let backup_path = db_backup_file(owner, db, &self.config);
        if backup_path.exists() {
            let new_backup_path = db_backup_file(new_owner, new_db, &self.config);
            let backups_dir = new_backup_path.parent().ok_or(ServerError {
                description: format!(
                    "new backup path does not have a parent directory '{}' - cannot rename",
                    new_backup_path.display()
                ),
                status: StatusCode::NOT_FOUND,
            })?;
            std::fs::create_dir_all(backups_dir)?;
            std::fs::rename(backup_path, new_backup_path)?;
        }
        audit::rename(
            &db_backup_audit_dir(owner, db, &self.config),
            &db_backup_audit_dir(new_owner, new_db, &self.config),
        )?;
        audit::rename(
            &db_audit_dir(owner, db, &self.config),
            &db_audit_dir(new_owner, new_db, &self.config),
        )
    }

    pub(crate) async fn restore_db(&self, owner: &str, db: &str, db_type: DbKind) -> ServerResult {
        let backup_path = db_backup_file(owner, db, &self.config);

        if !backup_path.exists() {
            return Err(ServerError {
                description: "backup not found".to_string(),
                status: StatusCode::NOT_FOUND,
            });
        }

        self.migrate_audit(owner, db)?;
        self.remove_db(owner, db).await?;

        let db_name = DbName::new(owner, db);
        let current_path = db_file(owner, db, &self.config);
        std::fs::copy(&backup_path, &current_path)?;
        let user_db = UserDb::new(
            current_path.to_string_lossy().as_ref(),
            db_type,
            self.config.storage,
        )?;
        if db_type == DbKind::Memory {
            remove_file_if_exists(&current_path)?;
        }
        self.pool.write().await.insert(db_name, user_db);
        self.restore_audit_from_backup(owner, db)
    }

    pub(crate) async fn resync_db(
        &self,
        owner: &str,
        db: &str,
        db_type: DbKind,
        install_dir: &Path,
        config: &Config,
    ) -> ServerResult {
        match self.delete_db(owner, db).await {
            Ok(_) => {}
            Err(e) if e.status == StatusCode::NOT_FOUND => {}
            Err(e) => return Err(e),
        }

        let data_dir = Path::new(&config.data_dir);
        let db_src = install_dir.join(owner).join(db);
        let db_dst = db_file(owner, db, config);

        if let Some(parent) = db_dst.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if db_src.exists() {
            std::fs::copy(&db_src, &db_dst)?;
        } else if db_type != DbKind::Memory {
            return Err(ServerError::from(format!(
                "resync_db: snapshot missing main DB file for {owner}/{db}"
            )));
        }

        let installed = |path: &Path| install_dir.join(path.strip_prefix(data_dir).unwrap_or(path));

        for dst_path in [
            db_backup_file(owner, db, config),
            db_audit_legacy_file(owner, db, config),
            db_backup_audit_legacy_file(owner, db, config),
        ] {
            let src_path = installed(&dst_path);
            if src_path.is_file() {
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&src_path, &dst_path)?;
            }
        }

        for dst_path in [
            db_audit_dir(owner, db, config),
            db_backup_audit_dir(owner, db, config),
        ] {
            audit::sync(&installed(&dst_path), &dst_path)?;
        }

        self.migrate_audit(owner, db)?;

        let db_name = DbName::new(owner, db);
        let user_db = UserDb::new(
            db_dst.to_string_lossy().as_ref(),
            db_type,
            self.config.storage,
        )?;
        if db_type == DbKind::Memory {
            remove_file_if_exists(&db_dst)?;
        }
        self.pool.write().await.insert(db_name, user_db);
        Ok(())
    }

    pub(crate) async fn rollback_db(
        &self,
        owner: &str,
        db: &str,
        db_type: DbKind,
    ) -> ServerResult<Option<u64>> {
        let backup_path = db_backup_file(owner, db, &self.config);

        if !backup_path.exists() {
            return Err(ServerError {
                description: "backup not found".to_string(),
                status: StatusCode::NOT_FOUND,
            });
        }

        self.migrate_audit(owner, db)?;
        let db_name = DbName::new(owner, db);
        self.remove_db(owner, db).await?;

        let current_path = db_file(owner, db, &self.config);

        if db_type == DbKind::Memory {
            // For memory DBs: copy backup to main file, load it, then consume the main file.
            // The backup (.bak) is preserved so the DB can be rolled back again.
            std::fs::copy(&backup_path, &current_path)?;
            let user_db = UserDb::new(
                current_path.to_string_lossy().as_ref(),
                db_type,
                self.config.storage,
            )?;
            remove_file_if_exists(&current_path)?;
            self.pool.write().await.insert(db_name, user_db);
            self.restore_audit_from_backup(owner, db)?;
        } else {
            let backup_temp = db_swap_dir(owner, &self.config).join(format!("{db}.bak"));
            std::fs::create_dir_all(db_swap_dir(owner, &self.config))?;
            std::fs::rename(&current_path, &backup_temp)?;
            std::fs::rename(&backup_path, &current_path)?;
            std::fs::rename(backup_temp, &backup_path)?;
            let user_db = UserDb::new(
                current_path.to_string_lossy().as_ref(),
                db_type,
                self.config.storage,
            )?;
            self.pool.write().await.insert(db_name, user_db);
            self.swap_audit_with_backup(owner, db)?;
        }

        Ok(Some(
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        ))
    }

    fn restore_audit_from_backup(&self, owner: &str, db: &str) -> ServerResult {
        audit::sync(
            &db_backup_audit_dir(owner, db, &self.config),
            &db_audit_dir(owner, db, &self.config),
        )
    }

    fn swap_audit_with_backup(&self, owner: &str, db: &str) -> ServerResult {
        audit::swap(
            &db_audit_dir(owner, db, &self.config),
            &db_backup_audit_dir(owner, db, &self.config),
            &db_audit_swap_dir(owner, db, &self.config),
        )
    }

    fn finish_audit_swap(&self, owner: &str, db: &str) -> ServerResult {
        audit::finish_swap(
            &db_audit_dir(owner, db, &self.config),
            &db_backup_audit_dir(owner, db, &self.config),
            &db_audit_swap_dir(owner, db, &self.config),
        )
    }
}

fn db_not_found(owner: &str, db: &str) -> ServerError {
    ServerError::new(
        StatusCode::NOT_FOUND,
        &format!("db not found: {owner}/{db}"),
    )
}

pub(crate) fn db_backup_file(owner: &str, db: &str, config: &Config) -> PathBuf {
    db_backup_dir(owner, config).join(format!("{db}.bak"))
}

fn db_backup_dir(owner: &str, config: &Config) -> PathBuf {
    Path::new(&config.data_dir).join(owner).join("backups")
}

/// Holds the files of a rollback in progress, never a name derived from a db name.
fn db_swap_dir(owner: &str, config: &Config) -> PathBuf {
    db_backup_dir(owner, config).join(".swap")
}

fn db_audit_swap_dir(owner: &str, db: &str, config: &Config) -> PathBuf {
    db_swap_dir(owner, config).join(format!("{db}.audit"))
}

pub(crate) fn db_backup_audit_dir(owner: &str, db: &str, config: &Config) -> PathBuf {
    db_backup_dir(owner, config).join(format!("{db}.audit"))
}

fn db_backup_audit_legacy_file(owner: &str, db: &str, config: &Config) -> PathBuf {
    db_backup_dir(owner, config).join(format!("{db}.log"))
}

pub(crate) fn db_audit_dir(owner: &str, db: &str, config: &Config) -> PathBuf {
    Path::new(&config.data_dir)
        .join(owner)
        .join("audit")
        .join(db)
}

fn db_audit_legacy_file(owner: &str, db: &str, config: &Config) -> PathBuf {
    Path::new(&config.data_dir)
        .join(owner)
        .join("audit")
        .join(format!("{db}.log"))
}

pub(crate) fn db_file(owner: &str, db: &str, config: &Config) -> PathBuf {
    Path::new(&config.data_dir).join(owner).join(db)
}
