mod server_db;
mod server_db_storage;

use crate::config::Config;
use crate::db::server_db::ServerDb;
use crate::password::Password;
use crate::server_error::ServerResult;
use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::UserValue;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;

const SERVER_DB_NAME: &str = "mapped:agdb_server.agdb";

#[derive(UserValue)]
pub(crate) struct DbUser {
    pub(crate) db_id: Option<DbId>,
    pub(crate) name: String,
    pub(crate) password: Vec<u8>,
    pub(crate) salt: Vec<u8>,
    pub(crate) token: String,
}

#[derive(UserValue)]
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
                        .values(&DbUser {
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

    pub(crate) fn add_database(&self, user: DbId, database: Database) -> ServerResult {
        let db = ServerDb::new(&format!("{}:{}", database.db_type, database.name))?;
        self.get_pool_mut()?.insert(database.name.clone(), db);

        self.0.server_db.get_mut()?.transaction_mut(|t| {
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

    pub(crate) fn add_database_user(&self, database: DbId, user: DbId, role: &str) -> ServerResult {
        self.0.server_db.get_mut()?.exec_mut(
            &QueryBuilder::insert()
                .edges()
                .from(user)
                .to(database)
                .values_uniform(vec![("role", role).into()])
                .query(),
        )?;
        Ok(())
    }

    pub(crate) fn create_user(&self, user: DbUser) -> ServerResult {
        self.0.server_db.get_mut()?.transaction_mut(|t| {
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

    pub(crate) fn delete_database(&self, db: Database) -> ServerResult {
        let filename = self.remove_database(db)?.get()?.filename().to_string();
        let path = Path::new(&filename);

        if path.exists() {
            std::fs::remove_file(&filename)?;
            let dot_file = path
                .parent()
                .unwrap_or(Path::new("./"))
                .join(format!(".{filename}"));
            std::fs::remove_file(dot_file)?;
        }

        Ok(())
    }

    pub(crate) fn find_databases(&self) -> ServerResult<Vec<Database>> {
        Ok(self
            .0
            .server_db
            .get()?
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

    pub(crate) fn find_database_id(&self, name: &str) -> ServerResult<DbId> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::search()
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
            .ok_or(format!("Database '{name}' not found"))?
            .id)
    }

    pub(crate) fn find_users(&self) -> ServerResult<Vec<String>> {
        Ok(self
            .0
            .server_db
            .get()?
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

    pub(crate) fn find_user_databases(&self, user: DbId) -> ServerResult<Vec<Database>> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .from(user)
                            .where_()
                            .distance(CountComparison::Equal(2))
                            .query(),
                    )
                    .query(),
            )?
            .try_into()?)
    }

    pub(crate) fn find_user_database(&self, user: DbId, name: &str) -> ServerResult<Database> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .from(user)
                            .where_()
                            .distance(CountComparison::Equal(2))
                            .and()
                            .key("name")
                            .value(Comparison::Equal(name.into()))
                            .query(),
                    )
                    .query(),
            )?
            .try_into()?)
    }

    pub(crate) fn find_user(&self, name: &str) -> ServerResult<DbUser> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
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
            .try_into()?)
    }

    pub(crate) fn find_user_id(&self, name: &str) -> ServerResult<DbId> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::search()
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
            .ok_or(format!("User '{name}' not found"))?
            .id)
    }

    pub(crate) fn find_user_id_by_token(&self, token: &str) -> ServerResult<DbId> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::search()
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

    pub(crate) fn is_db_admin(&self, user: DbId, db: DbId) -> ServerResult<bool> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::search()
                    .from(user)
                    .to(db)
                    .limit(1)
                    .where_()
                    .distance(CountComparison::LessThanOrEqual(3))
                    .and()
                    .key("role")
                    .value(Comparison::Equal("admin".into()))
                    .query(),
            )?
            .result
            == 1)
    }

    pub(crate) fn remove_database(&self, db: Database) -> ServerResult<ServerDb> {
        self.0
            .server_db
            .get_mut()?
            .exec_mut(&QueryBuilder::remove().ids(db.db_id.unwrap()).query())?;

        Ok(self.get_pool_mut()?.remove(&db.name).unwrap())
    }

    pub(crate) fn save_token(&self, user: DbId, token: &str) -> ServerResult {
        self.0.server_db.get_mut()?.exec_mut(
            &QueryBuilder::insert()
                .values_uniform(vec![("token", token).into()])
                .ids(user)
                .query(),
        )?;
        Ok(())
    }

    pub(crate) fn save_user(&self, user: DbUser) -> ServerResult {
        self.0
            .server_db
            .get_mut()?
            .exec_mut(&QueryBuilder::insert().element(&user).query())?;
        Ok(())
    }

    // fn get_pool(&self) -> anyhow::Result<RwLockReadGuard<HashMap<String, ServerDb>>> {
    //     self.0.pool.read().map_err(map_error)
    // }

    fn get_pool_mut(&self) -> ServerResult<RwLockWriteGuard<HashMap<String, ServerDb>>> {
        Ok(self.0.pool.write()?)
    }
}
