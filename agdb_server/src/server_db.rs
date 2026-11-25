use crate::action::ClusterAction;
use crate::config::Config;
use crate::db_pool::DbName;
use crate::password::Password;
use crate::raft::Log;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::AgdbSerialize;
use agdb::Db;
use agdb::DbElement;
use agdb::DbId;
use agdb::DbType;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::QueryResult;
use agdb::SearchQuery;
use agdb::SelectValuesQuery;
use agdb_api::DashboardName;
use agdb_api::DashboardSummary;
use agdb_api::DbKind;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use agdb_api::UserSession;
use agdb_api::UserStatus;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::collections::HashSet;
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
    pub(crate) expires_at: u64,
    pub(crate) agent: String,
    pub(crate) session: String,
}

#[derive(Default, DbType)]
pub(crate) struct Database {
    pub(crate) db_id: Option<DbId>,
    pub(crate) db: String,
    pub(crate) owner: String,
    pub(crate) db_type: DbKind,
    pub(crate) backup: u64,
    pub(crate) created: u64,
}

#[derive(Default, DbType)]
pub(crate) struct ServerDashboard {
    pub(crate) db_id: Option<DbId>,
    pub(crate) owner: String,
    pub(crate) name: String,
    pub(crate) config: String,
    pub(crate) updated: u64,
}

impl Database {
    pub(crate) fn name(&self) -> DbName {
        DbName {
            owner: self.owner.clone(),
            db: self.db.clone(),
        }
    }
}

/// The dashboard fields listed without the config.
#[derive(DbType)]
struct DashboardListing {
    db_id: Option<DbId>,
    owner: String,
    name: String,
    updated: u64,
}

/// What a user may do with a dashboard.
pub(crate) struct DashboardAccess {
    pub(crate) dashboard: ServerDashboard,
    /// The user's role; read on a public dashboard the user has no role on.
    pub(crate) role: DbUserRole,
    /// The user has a role on the dashboard (as its owner or shared).
    pub(crate) member: bool,
    pub(crate) public: bool,
}

/// A `user —role→ db/dashboard` edge.
struct RoleEdge {
    id: DbId,
    from: DbId,
    to: DbId,
    role: DbUserRole,
}

#[derive(Clone)]
pub(crate) struct ServerDb {
    pub(crate) db: Arc<RwLock<Db>>,
    pub(crate) token_expiry_seconds: u64,
}

const ADMIN: &str = "admin";
const CREATED: &str = "created";
const DASHBOARDS: &str = "dashboards";
const DB: &str = "db";
const DBS: &str = "dbs";
const EXPIRES_AT: &str = "expires_at";
const HOME_DASHBOARD: &str = "home_dashboard";
const OWNER: &str = "owner";
/// Alias of the node linking to every public dashboard.
const PUBLIC_DASHBOARDS: &str = "public_dashboards";
const NAME: &str = "name";
const ROLE: &str = "role";
const TOKEN: &str = "token";
const USERS: &str = "users";
const USERNAME: &str = "username";
pub(crate) const SERVER_DB_FILE: &str = "agdb_server.agdb";
const SESSION: &str = "session";

pub(crate) async fn new(
    config: &Config,
    mut shutdown_receiver: broadcast::Receiver<()>,
) -> ServerResult<ServerDb> {
    std::fs::create_dir_all(&config.data_dir)?;
    let db_name = format!("{}/{}", config.data_dir, SERVER_DB_FILE);
    let db = ServerDb::new(&db_name, config.token_expiry_seconds)?;

    db.db.write().await.set_storage_config(config.storage);

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
        crate::warn!("Token cleanup on startup failed: {e:?}");
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
                        crate::warn!("Token cleanup failed: {e:?}");
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
        let db = Db::new(name)?;
        crate::utilities::log_migration(name, &db);

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

            if !indexes.iter().any(|i| i == SESSION) {
                t.exec_mut(QueryBuilder::insert().index(SESSION).query())?;
            }

            if t.exec(QueryBuilder::select().ids(USERS).query()).is_err() {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(USERS).query())?;
            }

            if t.exec(QueryBuilder::select().ids(DBS).query()).is_err() {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(DBS).query())?;
            }

            if t.exec(QueryBuilder::select().ids(DASHBOARDS).query())
                .is_err()
            {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(DASHBOARDS).query())?;
            }

            if t.exec(QueryBuilder::select().ids(PUBLIC_DASHBOARDS).query())
                .is_err()
            {
                t.exec_mut(
                    QueryBuilder::insert()
                        .nodes()
                        .aliases(PUBLIC_DASHBOARDS)
                        .query(),
                )?;
            }

            // Give owners of dashboards created before sharing their admin role, remove in later versions
            let dashboards = t
                .exec(
                    QueryBuilder::select()
                        .values([OWNER])
                        .search()
                        .from(DASHBOARDS)
                        .where_()
                        .neighbor()
                        .query(),
                )?
                .elements;
            for dashboard in dashboards {
                let has_role = t
                    .exec(
                        QueryBuilder::search()
                            .to(dashboard.id)
                            .where_()
                            .distance(1)
                            .and()
                            .keys(ROLE)
                            .query(),
                    )?
                    .result
                    != 0;
                let owner = t
                    .exec(
                        QueryBuilder::search()
                            .index(USERNAME)
                            .value(dashboard.values[0].value.to_string())
                            .query(),
                    )?
                    .elements
                    .first()
                    .map(|e| e.id);
                if let Some(owner) = owner
                    && !has_role
                {
                    t.exec_mut(
                        QueryBuilder::insert()
                            .edges()
                            .from(owner)
                            .to(dashboard.id)
                            .values_uniform([(ROLE, DbUserRole::Admin).into()])
                            .query(),
                    )?;
                }
            }

            // Remove all old user tokens still on users from previous db version, remove in later versions
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

            //Populate created field for existing databases, remove in later versions
            t.exec_mut(
                QueryBuilder::insert()
                    .values_uniform([(CREATED, 0_u64).into()])
                    .search()
                    .from(DBS)
                    .where_()
                    .neighbor()
                    .and()
                    .not()
                    .keys(CREATED)
                    .query(),
            )?;

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
                    .distance_less_than_or_equal(2)
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

    pub(crate) async fn dashboard(&self, owner: &str, name: &str) -> ServerResult<ServerDashboard> {
        self.db
            .read()
            .await
            .exec(find_dashboard_query(owner, name))?
            .try_into()
            .map_err(|_| dashboard_not_found(name))
    }

    /// The dashboard and what `user` may do with it. Without a role the
    /// dashboard is not found, as with databases, unless it is public.
    pub(crate) async fn user_dashboard(
        &self,
        user: DbId,
        owner: &str,
        name: &str,
    ) -> ServerResult<DashboardAccess> {
        self.db
            .read()
            .await
            .transaction(|t| -> ServerResult<DashboardAccess> {
                let dashboard: ServerDashboard = t
                    .exec(find_dashboard_query(owner, name))?
                    .try_into()
                    .map_err(|_| dashboard_not_found(name))?;
                let id = dashboard.db_id.ok_or_else(|| dashboard_not_found(name))?;
                let role = role_edges(&t.exec(dashboard_edges_query(id))?)
                    .into_iter()
                    .find(|edge| edge.from == user)
                    .map(|edge| edge.role);
                let public = !t.exec(public_edge_query(id))?.ids().is_empty();
                match (role, public) {
                    (Some(role), _) => Ok(DashboardAccess {
                        dashboard,
                        role,
                        member: true,
                        public,
                    }),
                    (None, true) => Ok(DashboardAccess {
                        dashboard,
                        role: DbUserRole::Read,
                        member: false,
                        public,
                    }),
                    (None, false) => Err(dashboard_not_found(name)),
                }
            })
    }

    /// Whether the dashboard is public.
    pub(crate) async fn dashboard_public(&self, dashboard: DbId) -> ServerResult<bool> {
        Ok(!self
            .db
            .read()
            .await
            .exec(public_edge_query(dashboard))?
            .ids()
            .is_empty())
    }

    /// Dashboards `user` has a role on (owned and shared) and the public
    /// ones, without their (possibly large) configs, sorted by owner and name.
    pub(crate) async fn user_dashboards(&self, user: DbId) -> ServerResult<Vec<DashboardSummary>> {
        let mut dashboards =
            self.db
                .read()
                .await
                .transaction(|t| -> ServerResult<Vec<DashboardSummary>> {
                    let all: HashSet<DbId> =
                        t.exec(all_dashboards_query())?.ids().into_iter().collect();
                    let public: HashSet<DbId> = t
                        .exec(public_dashboards_query())?
                        .ids()
                        .into_iter()
                        .collect();
                    let roles: Vec<RoleEdge> = role_edges(&t.exec(user_edges_query(user))?)
                        .into_iter()
                        .filter(|edge| all.contains(&edge.to))
                        .collect();
                    let member: HashMap<DbId, DbUserRole> =
                        roles.iter().map(|edge| (edge.to, edge.role)).collect();
                    let ids: Vec<DbId> = member
                        .keys()
                        .copied()
                        .chain(public.iter().copied().filter(|id| !member.contains_key(id)))
                        .collect();

                    if ids.is_empty() {
                        return Ok(vec![]);
                    }

                    let listings: Vec<DashboardListing> = t
                        .exec(
                            QueryBuilder::select()
                                .elements::<DashboardListing>()
                                .ids(ids)
                                .query(),
                        )?
                        .try_into()?;

                    Ok(listings
                        .into_iter()
                        .map(|listing| {
                            let role = listing.db_id.and_then(|id| member.get(&id).copied());
                            summary(
                                listing,
                                role.unwrap_or(DbUserRole::Read),
                                role.is_some(),
                                &public,
                            )
                        })
                        .collect())
                })?;
        dashboards.sort_by(|a, b| (&a.owner, &a.name).cmp(&(&b.owner, &b.name)));
        Ok(dashboards)
    }

    /// Every dashboard for server admins, who act as their owners, sorted by
    /// owner and name.
    pub(crate) async fn dashboards(&self) -> ServerResult<Vec<DashboardSummary>> {
        let mut dashboards =
            self.db
                .read()
                .await
                .transaction(|t| -> ServerResult<Vec<DashboardSummary>> {
                    let public: HashSet<DbId> = t
                        .exec(public_dashboards_query())?
                        .ids()
                        .into_iter()
                        .collect();
                    let listings: Vec<DashboardListing> = t
                        .exec(
                            QueryBuilder::select()
                                .elements::<DashboardListing>()
                                .search()
                                .from(DASHBOARDS)
                                .where_()
                                .neighbor()
                                .query(),
                        )?
                        .try_into()?;
                    Ok(listings
                        .into_iter()
                        .map(|listing| summary(listing, DbUserRole::Admin, true, &public))
                        .collect())
                })?;
        dashboards.sort_by(|a, b| (&a.owner, &a.name).cmp(&(&b.owner, &b.name)));
        Ok(dashboards)
    }

    /// Makes the dashboard public or private. Users who can no longer see a
    /// private dashboard lose it as their home.
    pub(crate) async fn set_dashboard_public(
        &self,
        dashboard: DbId,
        public: bool,
    ) -> ServerResult<()> {
        self.db
            .write()
            .await
            .transaction_mut(|t| -> ServerResult<()> {
                let edges: Vec<DbId> = t.exec(public_edge_query(dashboard))?.ids();

                match (public, edges.is_empty()) {
                    (true, true) => {
                        t.exec_mut(
                            QueryBuilder::insert()
                                .edges()
                                .from(PUBLIC_DASHBOARDS)
                                .to(dashboard)
                                .query(),
                        )?;
                    }
                    (false, false) => {
                        t.exec_mut(QueryBuilder::remove().ids(edges).query())?;
                        let incoming = t
                            .exec(
                                QueryBuilder::select()
                                    .search()
                                    .to(dashboard)
                                    .where_()
                                    .distance(1)
                                    .query(),
                            )?
                            .elements;
                        let members: Vec<DbId> = incoming
                            .iter()
                            .filter(|edge| has_key(edge, ROLE))
                            .map(|edge| edge.from)
                            .collect();
                        let homes: Vec<DbId> = incoming
                            .iter()
                            .filter(|edge| {
                                has_key(edge, HOME_DASHBOARD) && !members.contains(&edge.from)
                            })
                            .map(|edge| edge.id)
                            .collect();
                        if !homes.is_empty() {
                            t.exec_mut(QueryBuilder::remove().ids(homes).query())?;
                        }
                    }
                    _ => {}
                }
                Ok(())
            })
    }

    /// Users with a role on the dashboard (including its owner) by name.
    pub(crate) async fn dashboard_users(&self, dashboard: DbId) -> ServerResult<Vec<DbUser>> {
        let mut users = self
            .db
            .read()
            .await
            .transaction(|t| -> ServerResult<Vec<DbUser>> {
                let edges = t.exec(
                    QueryBuilder::select()
                        .values([ROLE])
                        .search()
                        .to(dashboard)
                        .where_()
                        .distance(1)
                        .and()
                        .keys(ROLE)
                        .query(),
                )?;

                edges
                    .elements
                    .iter()
                    .map(|edge| {
                        let username = t
                            .exec(
                                QueryBuilder::select()
                                    .values(USERNAME)
                                    .ids(edge.from)
                                    .query(),
                            )?
                            .elements[0]
                            .values[0]
                            .value
                            .to_string();
                        Ok(DbUser {
                            username,
                            role: (&edge.values[0].value).into(),
                        })
                    })
                    .collect()
            })?;
        users.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(users)
    }

    /// Adds or changes the role of `user` on the dashboard.
    pub(crate) async fn insert_dashboard_user(
        &self,
        dashboard: DbId,
        user: DbId,
        role: DbUserRole,
    ) -> ServerResult<()> {
        self.db
            .write()
            .await
            .transaction_mut(|t| -> ServerResult<()> {
                let existing = role_edges(&t.exec(dashboard_edges_query(dashboard))?)
                    .into_iter()
                    .find(|edge| edge.from == user);

                if let Some(edge) = existing {
                    t.exec_mut(
                        QueryBuilder::insert()
                            .values([[(ROLE, role).into()]])
                            .ids(edge.id)
                            .query(),
                    )?;
                } else {
                    t.exec_mut(
                        QueryBuilder::insert()
                            .edges()
                            .from(user)
                            .to(dashboard)
                            .values_uniform([(ROLE, role).into()])
                            .query(),
                    )?;
                }
                Ok(())
            })
    }

    /// Removes the role of `user` on the dashboard, and the dashboard as the
    /// user's home with it unless it is public (so the user still sees it).
    pub(crate) async fn remove_dashboard_user(
        &self,
        dashboard: DbId,
        user: DbId,
    ) -> ServerResult<()> {
        self.db
            .write()
            .await
            .transaction_mut(|t| -> ServerResult<()> {
                let public = !t.exec(public_edge_query(dashboard))?.ids().is_empty();
                let edges: Vec<DbId> = t
                    .exec(dashboard_edges_query(dashboard))?
                    .elements
                    .into_iter()
                    .filter(|edge| edge.from == user)
                    .filter(|edge| !public || !has_key(edge, HOME_DASHBOARD))
                    .map(|edge| edge.id)
                    .collect();

                if !edges.is_empty() {
                    t.exec_mut(QueryBuilder::remove().ids(edges).query())?;
                }
                Ok(())
            })
    }

    /// The dashboard `user` opens by default, if any.
    pub(crate) async fn home_dashboard(&self, user: DbId) -> ServerResult<Option<DashboardName>> {
        self.db
            .read()
            .await
            .transaction(|t| -> ServerResult<Option<DashboardName>> {
                let Some(edge) = t.exec(home_edges_query(user))?.elements.into_iter().next() else {
                    return Ok(None);
                };
                let dashboard = &t
                    .exec(
                        QueryBuilder::select()
                            .values([OWNER, NAME])
                            .ids(edge.to)
                            .query(),
                    )?
                    .elements[0];
                Ok(Some(DashboardName {
                    owner: dashboard.values[0].value.to_string(),
                    name: dashboard.values[1].value.to_string(),
                }))
            })
    }

    /// Replaces the home dashboard of `user`; `None` clears it.
    pub(crate) async fn set_home_dashboard(
        &self,
        user: DbId,
        dashboard: Option<DbId>,
    ) -> ServerResult<()> {
        self.db
            .write()
            .await
            .transaction_mut(|t| -> ServerResult<()> {
                let existing: Vec<DbId> = t
                    .exec(home_edges_query(user))?
                    .elements
                    .into_iter()
                    .map(|edge| edge.id)
                    .collect();

                if !existing.is_empty() {
                    t.exec_mut(QueryBuilder::remove().ids(existing).query())?;
                }

                if let Some(dashboard) = dashboard {
                    t.exec_mut(
                        QueryBuilder::insert()
                            .edges()
                            .from(user)
                            .to(dashboard)
                            .values_uniform([(HOME_DASHBOARD, true).into()])
                            .query(),
                    )?;
                }
                Ok(())
            })
    }

    pub(crate) async fn remove_dashboard(&self, owner: &str, name: &str) -> ServerResult<()> {
        self.db.write().await.exec_mut(
            QueryBuilder::remove()
                .search()
                .from(DASHBOARDS)
                .where_()
                .neighbor()
                .and()
                .key(OWNER)
                .value(owner)
                .and()
                .key(NAME)
                .value(name)
                .query(),
        )?;
        Ok(())
    }

    pub(crate) async fn remove_dashboards(&self, owner: &str) -> ServerResult<()> {
        self.db.write().await.exec_mut(
            QueryBuilder::remove()
                .search()
                .from(DASHBOARDS)
                .where_()
                .neighbor()
                .and()
                .key(OWNER)
                .value(owner)
                .query(),
        )?;
        Ok(())
    }

    pub(crate) async fn rename_dashboard(
        &self,
        owner: &str,
        name: &str,
        new_name: &str,
        updated: u64,
    ) -> ServerResult<()> {
        self.db
            .write()
            .await
            .transaction_mut(|t| -> ServerResult<()> {
                let mut dashboard: ServerDashboard = t
                    .exec(find_dashboard_query(owner, name))?
                    .try_into()
                    .map_err(|_| dashboard_not_found(name))?;
                // An add or rename applied after the request was checked may have taken the name.
                if t.exec(find_dashboard_query(owner, new_name))?.result != 0 {
                    return Err(dashboard_exists(new_name));
                }
                dashboard.name = new_name.to_string();
                dashboard.updated = updated;
                t.exec_mut(QueryBuilder::insert().element(&dashboard).query())?;
                Ok(())
            })
    }

    /// With `create` inserts a new dashboard owned (with the admin role) by
    /// its `owner`; an existing dashboard is an error (a concurrent add won).
    /// Otherwise replaces the config of an existing dashboard.
    pub(crate) async fn save_dashboard(
        &self,
        mut dashboard: ServerDashboard,
        create: bool,
    ) -> ServerResult<()> {
        self.db
            .write()
            .await
            .transaction_mut(|t| -> ServerResult<()> {
                dashboard.db_id = t
                    .exec(find_dashboard_query(&dashboard.owner, &dashboard.name))?
                    .elements
                    .first()
                    .map(|e| e.id);

                match (create, dashboard.db_id) {
                    (true, Some(_)) => return Err(dashboard_exists(&dashboard.name)),
                    // A remove or rename applied after the request was checked.
                    (false, None) => return Err(dashboard_not_found(&dashboard.name)),
                    _ => {}
                }

                if create {
                    let owner = t
                        .exec(
                            QueryBuilder::search()
                                .index(USERNAME)
                                .value(&dashboard.owner)
                                .query(),
                        )?
                        .elements
                        .first()
                        .map(|e| e.id)
                        .ok_or_else(|| user_not_found(&dashboard.owner))?;
                    let id = t
                        .exec_mut(QueryBuilder::insert().element(&dashboard).query())?
                        .elements[0]
                        .id;
                    t.exec_mut(
                        QueryBuilder::insert()
                            .edges()
                            .from([QueryId::from(owner), DASHBOARDS.into()])
                            .to(id)
                            .values([vec![(ROLE, DbUserRole::Admin).into()], vec![]])
                            .query(),
                    )?;
                } else {
                    t.exec_mut(QueryBuilder::insert().element(&dashboard).query())?;
                }
                Ok(())
            })
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
            let (id, expires_at) = t
                .exec(
                    QueryBuilder::select()
                        .values(EXPIRES_AT)
                        .search()
                        .index(TOKEN)
                        .value(token)
                        .query(),
                )?
                .elements
                .first()
                .map(|e| (e.id, e.values[0].value.to_u64().unwrap_or_default()))
                .ok_or_else(|| token_not_found(token))?;

            if expires_at < current_timestamp() {
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
                    .distance_less_than_or_equal(2)
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

        self.remove_tokens(user).await?;
        self.remove_dashboards(username).await?;

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
        self.db.write().await.exec_mut(
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
        agent: String,
        session: String,
    ) -> ServerResult<DbId> {
        let created = current_timestamp();
        let expires_at = created + self.token_expiry_seconds;
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
                                created,
                                expires_at,
                                agent,
                                session,
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
                        .distance_less_than_or_equal(2)
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

        // Each edge precedes its node. Users also point at dashboards (role and
        // home edges), which are skipped.
        let mut role = None;
        for e in elements {
            if e.id.0 < 0 {
                role = e
                    .values
                    .iter()
                    .find(|kv| kv.key == ROLE.into())
                    .map(|kv| DbUserRole::from(&kv.value));
            } else if let Some(role) = role.take()
                && has_key(&e, DB)
            {
                dbs.push((role, Database::from_db_element(&e)?));
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
            let (token_id, expires_at) = t
                .exec(
                    QueryBuilder::select()
                        .values(EXPIRES_AT)
                        .search()
                        .index(TOKEN)
                        .value(token)
                        .query(),
                )?
                .elements
                .first()
                .map(|e| (e.id, e.values[0].value.to_u64().unwrap_or_default()))
                .ok_or_else(|| token_not_found(token))?;

            if expires_at < current_timestamp() {
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
        Ok(self
            .db
            .read()
            .await
            .exec(user_sessions_query(user_id))?
            .try_into()?)
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
                    let sessions: Vec<UserSession> =
                        t.exec(user_sessions_query(e.id))?.try_into()?;

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
                        .key(EXPIRES_AT)
                        .less_than(current_timestamp())
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

    pub(crate) async fn remove_session(&self, session: String) -> ServerResult<()> {
        self.db.write().await.transaction_mut(|t| {
            let removed = t.exec_mut(
                QueryBuilder::remove()
                    .search()
                    .index(SESSION)
                    .value(&session)
                    .query(),
            )?;

            if removed.result == 0 {
                return Err(session_not_found(&session));
            }

            Ok(())
        })
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn user_sessions_query(user_id: DbId) -> SelectValuesQuery {
    QueryBuilder::select()
        .elements::<UserSession>()
        .search()
        .to(user_id)
        .where_()
        .neighbor()
        .and()
        .key(EXPIRES_AT)
        .greater_than_or_equal(current_timestamp())
        .query()
}

pub(crate) fn dashboard_exists(name: &str) -> ServerError {
    ServerError::new(
        StatusCode::CONFLICT,
        &format!("dashboard already exists: {name}"),
    )
}

pub(crate) fn dashboard_not_found(name: &str) -> ServerError {
    ServerError::new(
        StatusCode::NOT_FOUND,
        &format!("dashboard not found: {name}"),
    )
}

fn db_not_found(name: &str) -> ServerError {
    ServerError::new(StatusCode::NOT_FOUND, &format!("db not found: {name}"))
}

fn find_dashboard_query(owner: &str, name: &str) -> SelectValuesQuery {
    QueryBuilder::select()
        .element::<ServerDashboard>()
        .search()
        .from(DASHBOARDS)
        .where_()
        .neighbor()
        .and()
        .key(OWNER)
        .value(owner)
        .and()
        .key(NAME)
        .value(name)
        .query()
}

fn all_dashboards_query() -> SearchQuery {
    QueryBuilder::search()
        .from(DASHBOARDS)
        .where_()
        .neighbor()
        .query()
}

/// The edge that makes the dashboard public, if it is.
fn public_edge_query(dashboard: DbId) -> SearchQuery {
    QueryBuilder::search()
        .from(PUBLIC_DASHBOARDS)
        .to(dashboard)
        .where_()
        .edge()
        .query()
}

fn public_dashboards_query() -> SearchQuery {
    QueryBuilder::search()
        .from(PUBLIC_DASHBOARDS)
        .where_()
        .neighbor()
        .query()
}

fn summary(
    listing: DashboardListing,
    role: DbUserRole,
    member: bool,
    public: &HashSet<DbId>,
) -> DashboardSummary {
    DashboardSummary {
        public: listing.db_id.is_some_and(|id| public.contains(&id)),
        owner: listing.owner,
        name: listing.name,
        updated: listing.updated,
        role,
        member,
    }
}

fn has_key(element: &DbElement, key: &str) -> bool {
    element.values.iter().any(|kv| kv.key == key.into())
}

/// The outgoing edges of a user: roles on dbs and dashboards, and the home
/// dashboard. Each carries its destination in `to`.
fn user_edges_query(user: DbId) -> SelectValuesQuery {
    QueryBuilder::select()
        .search()
        .from(user)
        .where_()
        .distance(1)
        .query()
}

/// The incoming edges of a dashboard: the roles of its users, their homes and
/// the public alias. Far fewer than the outgoing edges of a user.
fn dashboard_edges_query(dashboard: DbId) -> SelectValuesQuery {
    QueryBuilder::select()
        .search()
        .to(dashboard)
        .where_()
        .distance(1)
        .query()
}

/// The home dashboard edge of a user (at most one).
fn home_edges_query(user: DbId) -> SelectValuesQuery {
    QueryBuilder::select()
        .search()
        .from(user)
        .where_()
        .distance(1)
        .and()
        .keys(HOME_DASHBOARD)
        .query()
}

fn role_edges(result: &QueryResult) -> Vec<RoleEdge> {
    result
        .elements
        .iter()
        .filter_map(|edge| {
            let role = edge.values.iter().find(|kv| kv.key == ROLE.into())?;
            Some(RoleEdge {
                id: edge.id,
                from: edge.from,
                to: edge.to,
                role: (&role.value).into(),
            })
        })
        .collect()
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

fn session_not_found(session: &str) -> ServerError {
    ServerError::new(
        StatusCode::NOT_FOUND,
        &format!("session not found: {session}"),
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::DbKeyOrder;
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
        assert_eq!(indexes.elements[0].values[1], (SESSION, 0_u64).into());
        assert_eq!(indexes.elements[0].values[2], (TOKEN, 0_u64).into());
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

    fn test_dashboard(owner: &str, name: &str) -> ServerDashboard {
        ServerDashboard {
            db_id: None,
            owner: owner.to_string(),
            name: name.to_string(),
            config: "{}".to_string(),
            updated: 1,
        }
    }

    async fn test_user(db: &ServerDb, username: &str) -> ServerResult<DbId> {
        db.insert_user(ServerUser {
            db_id: None,
            username: username.to_string(),
            password: vec![],
            salt: vec![],
        })
        .await
    }

    async fn dashboard_id(db: &ServerDb, owner: &str, name: &str) -> ServerResult<DbId> {
        Ok(db.dashboard(owner, name).await?.db_id.unwrap())
    }

    fn names(dashboards: Vec<DashboardSummary>) -> Vec<String> {
        dashboards
            .into_iter()
            .map(|d| format!("{}/{}:{}", d.owner, d.name, d.role))
            .collect()
    }

    #[tokio::test]
    async fn dashboard_conflicts_checked_when_applied() -> ServerResult<()> {
        let test_file = TestFile::new();
        let db = ServerDb::new(test_file.file_name(), 3600)?;
        let user = test_user(&db, "user").await?;
        db.save_dashboard(test_dashboard("user", "a"), true).await?;
        db.save_dashboard(test_dashboard("user", "b"), true).await?;

        // An add that lost a race to another add does not replace it.
        let error = db
            .save_dashboard(test_dashboard("user", "a"), true)
            .await
            .unwrap_err();
        assert_eq!(error.status, StatusCode::CONFLICT);
        db.save_dashboard(test_dashboard("user", "a"), false)
            .await?;

        // A save that lost a race to a remove does not recreate it.
        let error = db
            .save_dashboard(test_dashboard("user", "c"), false)
            .await
            .unwrap_err();
        assert_eq!(error.status, StatusCode::NOT_FOUND);

        // A rename onto a name taken meanwhile does not create a duplicate.
        let error = db.rename_dashboard("user", "a", "b", 2).await.unwrap_err();
        assert_eq!(error.status, StatusCode::CONFLICT);
        assert_eq!(
            names(db.user_dashboards(user).await?),
            vec!["user/a:admin", "user/b:admin"]
        );
        Ok(())
    }

    #[tokio::test]
    async fn dashboard_shared_roles() -> ServerResult<()> {
        let test_file = TestFile::new();
        let db = ServerDb::new(test_file.file_name(), 3600)?;
        let owner = test_user(&db, "owner").await?;
        let user = test_user(&db, "user").await?;
        db.save_dashboard(test_dashboard("owner", "ops"), true)
            .await?;
        db.save_dashboard(test_dashboard("user", "own"), true)
            .await?;
        let ops = dashboard_id(&db, "owner", "ops").await?;

        assert!(matches!(
            db.user_dashboard(user, "owner", "ops").await,
            Err(e) if e.status == StatusCode::NOT_FOUND
        ));

        db.insert_dashboard_user(ops, user, DbUserRole::Read)
            .await?;
        db.set_home_dashboard(user, Some(ops)).await?;
        // The home edge next to the role edge changes neither the role nor the users.
        db.insert_dashboard_user(ops, user, DbUserRole::Write)
            .await?;
        assert_eq!(
            db.user_dashboard(user, "owner", "ops").await?.role,
            DbUserRole::Write
        );
        assert_eq!(
            names(db.user_dashboards(user).await?),
            vec!["owner/ops:write", "user/own:admin"]
        );
        assert_eq!(
            db.dashboard_users(ops).await?,
            vec![
                DbUser {
                    username: "owner".to_string(),
                    role: DbUserRole::Admin,
                },
                DbUser {
                    username: "user".to_string(),
                    role: DbUserRole::Write,
                },
            ]
        );
        assert_eq!(
            names(db.user_dashboards(owner).await?),
            vec!["owner/ops:admin"]
        );

        // Removing the role removes the dashboard as home too.
        db.remove_dashboard_user(ops, user).await?;
        assert_eq!(db.home_dashboard(user).await?, None);
        assert_eq!(
            names(db.user_dashboards(user).await?),
            vec!["user/own:admin"]
        );
        Ok(())
    }

    #[tokio::test]
    async fn dashboard_home() -> ServerResult<()> {
        let test_file = TestFile::new();
        let db = ServerDb::new(test_file.file_name(), 3600)?;
        let user = test_user(&db, "user").await?;
        db.save_dashboard(test_dashboard("user", "a"), true).await?;
        db.save_dashboard(test_dashboard("user", "b"), true).await?;
        let a = dashboard_id(&db, "user", "a").await?;
        let b = dashboard_id(&db, "user", "b").await?;
        let home = |name: &str| {
            Some(DashboardName {
                owner: "user".to_string(),
                name: name.to_string(),
            })
        };

        assert_eq!(db.home_dashboard(user).await?, None);
        db.set_home_dashboard(user, Some(a)).await?;
        db.set_home_dashboard(user, Some(b)).await?;
        assert_eq!(db.home_dashboard(user).await?, home("b"));

        // The home follows a rename and goes with a remove.
        db.rename_dashboard("user", "b", "c", 2).await?;
        assert_eq!(db.home_dashboard(user).await?, home("c"));
        db.remove_dashboard("user", "c").await?;
        assert_eq!(db.home_dashboard(user).await?, None);

        db.set_home_dashboard(user, Some(a)).await?;
        db.set_home_dashboard(user, None).await?;
        assert_eq!(db.home_dashboard(user).await?, None);
        // Clearing only removed the home, not the owner's role.
        assert_eq!(names(db.user_dashboards(user).await?), vec!["user/a:admin"]);
        Ok(())
    }

    #[tokio::test]
    async fn user_dbs_skip_dashboards() -> ServerResult<()> {
        let test_file = TestFile::new();
        let db = ServerDb::new(test_file.file_name(), 3600)?;
        let owner = test_user(&db, "owner").await?;
        let user = test_user(&db, "user").await?;
        db.insert_db(
            user,
            Database {
                db: "data".to_string(),
                owner: "user".to_string(),
                ..Default::default()
            },
        )
        .await?;
        db.save_dashboard(test_dashboard("user", "own"), true)
            .await?;
        db.save_dashboard(test_dashboard("owner", "ops"), true)
            .await?;
        let ops = dashboard_id(&db, "owner", "ops").await?;
        db.insert_dashboard_user(ops, user, DbUserRole::Read)
            .await?;
        db.set_home_dashboard(user, Some(ops)).await?;
        db.set_dashboard_public(ops, true).await?;

        let dbs = db.user_dbs(user).await?;
        assert_eq!(dbs.len(), 1);
        assert_eq!(dbs[0].0, DbUserRole::Admin);
        assert_eq!(dbs[0].1.db, "data");
        assert!(db.user_dbs(owner).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn dashboard_public() -> ServerResult<()> {
        let test_file = TestFile::new();
        let db = ServerDb::new(test_file.file_name(), 3600)?;
        test_user(&db, "owner").await?;
        let member = test_user(&db, "member").await?;
        let viewer = test_user(&db, "viewer").await?;
        db.save_dashboard(test_dashboard("owner", "ops"), true)
            .await?;
        db.save_dashboard(test_dashboard("owner", "other"), true)
            .await?;
        let ops = dashboard_id(&db, "owner", "ops").await?;
        db.insert_dashboard_user(ops, member, DbUserRole::Write)
            .await?;
        let listed = |dashboards: Vec<DashboardSummary>| {
            dashboards
                .into_iter()
                .map(|d| {
                    format!(
                        "{}/{}:{}:{}:{}",
                        d.owner, d.name, d.role, d.public, d.member
                    )
                })
                .collect::<Vec<_>>()
        };

        assert!(!db.dashboard_public(ops).await?);
        assert!(db.user_dashboards(viewer).await?.is_empty());

        // Twice is the same as once.
        db.set_dashboard_public(ops, true).await?;
        db.set_dashboard_public(ops, true).await?;
        assert!(db.dashboard_public(ops).await?);
        let access = db.user_dashboard(viewer, "owner", "ops").await?;
        assert_eq!(access.role, DbUserRole::Read);
        assert!(!access.member && access.public);
        let access = db.user_dashboard(member, "owner", "ops").await?;
        assert_eq!(access.role, DbUserRole::Write);
        assert!(access.member && access.public);
        assert_eq!(
            listed(db.user_dashboards(viewer).await?),
            vec!["owner/ops:read:true:false"]
        );
        assert_eq!(
            listed(db.user_dashboards(member).await?),
            vec!["owner/ops:write:true:true"]
        );
        assert_eq!(
            listed(db.dashboards().await?),
            vec!["owner/ops:admin:true:true", "owner/other:admin:false:true"]
        );
        // The public link is not a user.
        assert_eq!(db.dashboard_users(ops).await?.len(), 2);

        // A member removed from a public dashboard keeps it as home.
        db.set_home_dashboard(viewer, Some(ops)).await?;
        db.set_home_dashboard(member, Some(ops)).await?;
        db.remove_dashboard_user(ops, member).await?;
        assert!(db.home_dashboard(member).await?.is_some());
        db.insert_dashboard_user(ops, member, DbUserRole::Read)
            .await?;

        // Private again: only users with a role keep it as home.
        db.set_dashboard_public(ops, false).await?;
        db.set_dashboard_public(ops, false).await?;
        assert!(!db.dashboard_public(ops).await?);
        assert_eq!(db.home_dashboard(viewer).await?, None);
        assert!(db.home_dashboard(member).await?.is_some());
        assert!(matches!(
            db.user_dashboard(viewer, "owner", "ops").await,
            Err(e) if e.status == StatusCode::NOT_FOUND
        ));
        assert!(db.user_dashboards(viewer).await?.is_empty());

        // A rename keeps it public, a remove drops the public link.
        db.set_dashboard_public(ops, true).await?;
        db.rename_dashboard("owner", "ops", "sales", 2).await?;
        assert_eq!(
            listed(db.user_dashboards(viewer).await?),
            vec!["owner/sales:read:true:false"]
        );
        db.remove_dashboard("owner", "sales").await?;
        assert!(db.user_dashboards(viewer).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn migrate_dashboard_owner_role() -> ServerResult<()> {
        let test_file = TestFile::new();

        {
            let db = ServerDb::new(test_file.file_name(), 3600)?;
            test_user(&db, "user").await?;
            // A dashboard saved before owners got their role edge.
            db.db
                .read()
                .await
                .transaction_mut(|t| -> ServerResult<()> {
                    let id = t
                        .exec_mut(
                            QueryBuilder::insert()
                                .element(test_dashboard("user", "old"))
                                .query(),
                        )?
                        .elements[0]
                        .id;
                    t.exec_mut(
                        QueryBuilder::insert()
                            .edges()
                            .from(DASHBOARDS)
                            .to(id)
                            .query(),
                    )?;
                    Ok(())
                })?;
        }

        let db = ServerDb::new(test_file.file_name(), 3600)?;
        let user = db.user_id("user").await?;
        assert_eq!(
            names(db.user_dashboards(user).await?),
            vec!["user/old:admin"]
        );
        // Starting again adds no second role.
        let db = ServerDb::new(test_file.file_name(), 3600)?;
        let id = dashboard_id(&db, "user", "old").await?;
        assert_eq!(db.dashboard_users(id).await?.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn migrate_missing_db_created_only() -> ServerResult<()> {
        let test_file = TestFile::new();
        let existing_created = 123_u64;

        {
            let mut db = Db::new(test_file.file_name())?;
            db.transaction_mut(|t| {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(DBS).query())?;
                let dbs = t.exec_mut(
                    QueryBuilder::insert()
                        .nodes()
                        .values(vec![
                            vec![
                                (DB, "db_missing").into(),
                                (OWNER, "owner").into(),
                                ("db_type", "memory").into(),
                                ("backup", 0).into(),
                            ],
                            vec![
                                (DB, "db_existing").into(),
                                (OWNER, "owner").into(),
                                ("db_type", "memory").into(),
                                ("backup", 0).into(),
                                (CREATED, existing_created).into(),
                            ],
                        ])
                        .query(),
                )?;

                t.exec_mut(QueryBuilder::insert().edges().from(DBS).to(dbs).query())
            })?;
        }

        let db = ServerDb::new(test_file.file_name(), 3600)?;
        let dbs: Vec<Database> = db
            .db
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<Database>()
                    .search()
                    .from(DBS)
                    .order_by(DbKeyOrder::Asc(CREATED.into()))
                    .where_()
                    .neighbor()
                    .query(),
            )?
            .try_into()?;

        assert_eq!(dbs[0].created, 0_u64);
        assert_eq!(dbs[1].created, 123_u64);

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

        let expired1 = db
            .save_token(user1, "token1", "agent".to_string(), String::new())
            .await?;
        let expired2 = db
            .save_token(user1, "token2", "agent".to_string(), String::new())
            .await?;
        db.save_token(user2, "token3", "agent".to_string(), String::new())
            .await?;
        db.save_token(user2, "token4", "agent".to_string(), String::new())
            .await?;

        db.db.write().await.exec_mut(
            QueryBuilder::insert()
                .values_uniform([(EXPIRES_AT, 0_u64).into()])
                .ids([expired1, expired2])
                .query(),
        )?;

        assert_eq!(db.users_with_token().await?, 2);

        db.remove_expired_tokens().await?;
        assert_eq!(db.users_with_token().await?, 1);

        Ok(())
    }
}
