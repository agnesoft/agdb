use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::Comparison;
use agdb::CountComparison;
use agdb::Db;
use agdb::DbId;
use agdb::DbUserValue;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::QueryResult;
use agdb::SearchQuery;
use agdb::UserValue;
use agdb_api::DbType;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use agdb_api::UserStatus;
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(UserValue)]
pub(crate) struct ServerUser {
    pub(crate) db_id: Option<DbId>,
    pub(crate) username: String,
    pub(crate) password: Vec<u8>,
    pub(crate) salt: Vec<u8>,
    pub(crate) token: String,
}

#[derive(Default, UserValue)]
struct Database {
    pub(crate) db_id: Option<DbId>,
    pub(crate) name: String,
    pub(crate) db_type: DbType,
    pub(crate) backup: u64,
}

#[derive(Clone)]
pub(crate) struct ServerDb(pub(crate) Arc<RwLock<Db>>);

const DBS: &str = "dbs";
const NAME: &str = "name";
const ROLE: &str = "role";
const TOKEN: &str = "token";
const USERS: &str = "users";
const USERNAME: &str = "username";

impl ServerDb {
    pub(crate) fn load(name: &str) -> ServerResult<Self> {
        Ok(Self(Arc::new(RwLock::new(Db::new(name)?))))
    }

    pub(crate) fn new(name: &str, admin: ServerUser) -> ServerResult<Self> {
        let mut db = Db::new(name)?;

        db.transaction_mut(|t| {
            t.exec_mut(QueryBuilder::insert().index(USERNAME).query())?;
            t.exec_mut(QueryBuilder::insert().index(TOKEN).query())?;
            t.exec_mut(QueryBuilder::insert().nodes().aliases([USERS, DBS]).query())?;
            let id = t
                .exec_mut(QueryBuilder::insert().element(&admin).query())?
                .elements[0]
                .id;
            t.exec_mut(QueryBuilder::insert().edges().from(USERS).to(id).query())
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
                        user: String::new(),
                        role: (&e.values[0].value).into(),
                    });
                } else {
                    users.last_mut().unwrap().user = e.values[0].value.to_string();
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

    pub(crate) async fn find_user_db_id(&self, user: DbId, db: &str) -> ServerResult<Option<DbId>> {
        Ok(self
            .0
            .read()
            .await
            .exec(find_user_db_query(user, db))?
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
                    .distance(CountComparison::Equal(2))
                    .and()
                    .key(ROLE)
                    .value(Comparison::Equal(DbUserRole::Admin.into()))
                    .query(),
            )?
            .result
            == 1)
    }

    pub(crate) async fn remove_db(&self, user: DbId, db: &str) -> ServerResult<()> {
        self.0.write().await.exec_mut(
            QueryBuilder::remove()
                .ids(find_user_db_query(user, db))
                .query(),
        )?;
        Ok(())
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

    pub(crate) async fn remove_user(&self, username: &str) -> ServerResult<Vec<String>> {
        let user = self.user_id(username).await?;
        let mut ids = vec![user];
        let mut dbs = vec![];

        self.user_dbs(user)
            .await?
            .into_iter()
            .for_each(|(_role, db)| {
                if let Some((owner, _)) = db.name.split_once('/') {
                    if owner == username {
                        ids.push(db.db_id.unwrap());
                        dbs.push(db.name);
                    }
                }
            });

        self.0
            .write()
            .await
            .exec_mut(QueryBuilder::remove().ids(ids).query())?;

        Ok(dbs)
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
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<ServerUser>()
                    .ids(find_user_query(username))
                    .query(),
            )?
            .try_into()
            .map_err(|_| user_not_found(username))?)
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

    pub(crate) async fn user_db(&self, user: DbId, db: &str) -> ServerResult<Database> {
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<Database>()
                    .ids(find_user_db_query(user, db))
                    .query(),
            )?
            .try_into()
            .map_err(|_| db_not_found(db))?)
    }

    pub(crate) async fn user_db_id(&self, user: DbId, db: &str) -> ServerResult<DbId> {
        Ok(self
            .find_user_db_id(user, db)
            .await?
            .ok_or(db_not_found(db))?)
    }

    pub(crate) async fn user_db_role(&self, user: DbId, db: &str) -> ServerResult<DbUserRole> {
        Ok((&self
            .0
            .read()
            .await
            .transaction(|t| -> Result<QueryResult, ServerError> {
                let db_id = t
                    .exec(find_user_db_query(user, db))?
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
                name: e.values[0].value.to_string(),
                login: !e.values[1].value.to_string().is_empty(),
                admin: false,
            })
            .collect())
    }
}

fn db_not_found(name: &str) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("db not found: {name}"))
}

fn find_user_db_query(user: DbId, db: &str) -> SearchQuery {
    QueryBuilder::search()
        .depth_first()
        .from(user)
        .limit(1)
        .where_()
        .distance(CountComparison::Equal(2))
        .and()
        .key(NAME)
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
