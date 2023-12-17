mod server_db;
mod server_db_storage;

use crate::config::Config;
use crate::error_code::ErrorCode;
use crate::password::Password;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbId;
use agdb::DbUserValue;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::UserValue;
use server_db::ServerDb;
use server_db::ServerDbImpl;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

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
    pub(crate) db_type: String,
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

    pub(crate) fn add_db(&self, user: DbId, database: Database, config: &Config) -> ServerResult {
        let db_path = Path::new(&config.data_dir).join(&database.name);
        let user_dir = db_path.parent().ok_or(ErrorCode::DbInvalid)?;
        std::fs::create_dir_all(user_dir)?;
        let path = db_path.to_str().ok_or(ErrorCode::DbInvalid)?.to_string();

        let db = ServerDb::new(&format!("{}:{}", database.db_type, path)).map_err(|mut e| {
            e.status = ErrorCode::DbInvalid.into();
            e.description = format!("{}: {}", ErrorCode::DbInvalid.as_str(), e.description);
            e
        })?;
        self.get_pool_mut()?.insert(database.name.clone(), db);

        self.db_mut()?.transaction_mut(|t| {
            let db = t.exec_mut(&QueryBuilder::insert().nodes().values(&database).query())?;

            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(vec![QueryId::from(user), "dbs".into()])
                    .to(db)
                    .values(vec![vec![("role", "admin").into()], vec![]])
                    .query(),
            )
        })?;
        Ok(())
    }

    pub(crate) fn add_db_user(&self, db: DbId, user: DbId, role: &str) -> ServerResult {
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

    pub(crate) fn create_user(&self, user: ServerUser) -> ServerResult {
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

    pub(crate) fn delete_db(&self, db: Database, config: &Config) -> ServerResult {
        let path = Path::new(&config.data_dir).join(&db.name);
        self.remove_db(db)?;

        if path.exists() {
            let main_file_name = path
                .file_name()
                .ok_or(ErrorCode::DbInvalid)?
                .to_string_lossy();
            std::fs::remove_file(&path)?;
            let dot_file = path
                .parent()
                .ok_or(ErrorCode::DbInvalid)?
                .join(format!(".{main_file_name}"));
            std::fs::remove_file(dot_file)?;
        }

        Ok(())
    }

    pub(crate) fn find_dbs(&self) -> ServerResult<Vec<Database>> {
        Ok(self
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
            .try_into()?)
    }

    pub(crate) fn find_db(&self, db: &str) -> ServerResult<Database> {
        Ok(self
            .0
            .server_db
            .get()?
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
            .ok_or(ServerError::new(
                ErrorCode::DbNotFound.into(),
                &format!("{}: {db}", ErrorCode::DbNotFound.as_str()),
            ))?
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
            .ok_or(ServerError::new(
                ErrorCode::DbNotFound.into(),
                &format!("{}: {name}", ErrorCode::DbNotFound.as_str()),
            ))?
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

    pub(crate) fn find_user_dbs(&self, user: DbId) -> ServerResult<Vec<(Database, String)>> {
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
            .for_each(|e| {
                if e.id.0 < 0 {
                    dbs.push((Database::default(), e.values[0].value.to_string()));
                } else {
                    dbs.last_mut().unwrap().0 = Database::from_db_element(&e).unwrap_or_default();
                }
            });

        Ok(dbs)
    }

    pub(crate) fn find_user_db(&self, user: DbId, db: &str) -> ServerResult<Database> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .depth_first()
                            .from(user)
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
            .ok_or(ServerError::new(
                ErrorCode::DbNotFound.into(),
                &format!("{}: {db}", ErrorCode::DbNotFound.as_str()),
            ))?
            .try_into()?)
    }

    pub(crate) fn find_user(&self, name: &str) -> ServerResult<ServerUser> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .depth_first()
                            .from("users")
                            .limit(1)
                            .where_()
                            .distance(CountComparison::Equal(2))
                            .and()
                            .key("name")
                            .value(Comparison::Equal(name.into()))
                            .query(),
                    )
                    .query(),
            )?
            .elements
            .get(0)
            .ok_or(ServerError::new(
                ErrorCode::UserNotFound.into(),
                &format!("{}: {name}", ErrorCode::UserNotFound.as_str()),
            ))?
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
            .ok_or(ServerError::new(
                ErrorCode::UserNotFound.into(),
                &format!("{}: {name}", ErrorCode::UserNotFound.as_str()),
            ))?
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

    pub(crate) fn db_admins(&self, db: DbId) -> ServerResult<Vec<DbId>> {
        Ok(self
            .db()?
            .exec(
                &QueryBuilder::search()
                    .to(db)
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .and()
                    .beyond()
                    .where_()
                    .node()
                    .or()
                    .key("role")
                    .value(Comparison::Equal("admin".into()))
                    .query(),
            )?
            .ids())
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
            .ok_or(ErrorCode::UserNotFound)?
            .id)
    }

    pub(crate) fn db_users(&self, db: DbId) -> ServerResult<Vec<(String, String)>> {
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
                    users.push((String::new(), e.values[0].value.to_string()));
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
                    .value(Comparison::Equal("admin".into()))
                    .query(),
            )?
            .result
            == 1)
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

    pub(crate) fn remove_db(&self, db: Database) -> ServerResult<ServerDb> {
        self.db_mut()?
            .exec_mut(&QueryBuilder::remove().ids(db.db_id.unwrap()).query())?;

        Ok(self.get_pool_mut()?.remove(&db.name).unwrap())
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

    pub(crate) fn save_user(&self, user: ServerUser) -> ServerResult {
        self.db_mut()?
            .exec_mut(&QueryBuilder::insert().element(&user).query())?;
        Ok(())
    }

    // fn get_pool(&self) -> anyhow::Result<RwLockReadGuard<HashMap<String, ServerDb>>> {
    //     self.0.pool.read().map_err(map_error)
    // }

    fn get_pool_mut(&self) -> ServerResult<RwLockWriteGuard<HashMap<String, ServerDb>>> {
        Ok(self.0.pool.write()?)
    }

    fn db(&self) -> ServerResult<RwLockReadGuard<ServerDbImpl>> {
        self.0.server_db.get()
    }

    fn db_mut(&self) -> ServerResult<RwLockWriteGuard<ServerDbImpl>> {
        self.0.server_db.get_mut()
    }
}