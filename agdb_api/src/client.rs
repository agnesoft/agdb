use crate::AdminStatus;
use crate::ChangePassword;
use crate::ClusterStatus;
use crate::DbAudit;
use crate::DbResource;
use crate::DbUser;
use crate::DbUserRole;
use crate::UserLogin;
use crate::UserStatus;
use crate::api_result::AgdbApiResult;
use crate::api_types::DbKind;
use crate::api_types::LogLevelFilter;
use crate::api_types::ServerDatabase;
use crate::api_types::UserCredentials;
use crate::http_client::HttpClient;
use agdb::QueryResult;
use agdb::QueryType;

#[cfg(feature = "api")]
#[cfg_attr(feature = "api", agdb::trait_def())]
pub trait AgdbApiClient: HttpClient + agdb::type_def::TypeDefinition {}

#[cfg(not(feature = "api"))]
pub trait AgdbApiClient: HttpClient {}

/// Typed API client for agdb server endpoints.
///
/// The methods on this type map to `/api/v1` routes from the server OpenAPI
/// specification and return either an HTTP status code or `(status, body)`.
/// It maintains a public field `token` for the logged-in user's token internally.
///
/// # Example
///
/// ```ignore
/// use agdb_api::{AgdbApi, ReqwestClient};
///
/// let api = AgdbApi::new(ReqwestClient::new(), "http://localhost:3000");
/// assert_eq!(api.base_url(), "http://localhost:3000/api/v1");
/// ```
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[cfg_attr(feature = "api", type_def(inherent))]
pub struct AgdbApi<T: AgdbApiClient> {
    client: T,
    address: String,
    base_url: String,
    pub token: Option<String>,
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl<T: AgdbApiClient> AgdbApi<T> {
    /// Creates a new API client.
    ///
    /// If `address` does not start with `http://` or `https://`, `http://`
    /// is prepended.
    pub fn new(client: T, address: &str) -> Self {
        let base = if address.starts_with("http") || address.starts_with("https") {
            address.to_string()
        } else {
            format!("http://{address}")
        };

        Self {
            client,
            address: address.to_string(),
            base_url: format!("{base}/api/v1"),
            token: None,
        }
    }

    /// Returns the original address passed to [`Self::new`].
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Returns the computed base URL (`{address}/api/v1`).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// `POST /admin/db/{owner}/{db}/add?db_type={db_type}`
    ///
    /// Adds a database for the specified user.
    ///
    /// Returns `201` when created.
    ///
    /// Common error responses: `401` unauthorized, `404` user not found,
    /// `465` database already exists.
    pub async fn admin_db_add(&self, owner: &str, db: &str, db_type: DbKind) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/add?db_type={db_type}")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `GET /admin/db/{owner}/{db}/audit`
    ///
    /// Returns the audit log for a database.
    ///
    /// Returns `(200, DbAudit)` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn admin_db_audit(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, DbAudit)> {
        self.client
            .get(
                &self.url(&format!("/admin/db/{owner}/{db}/audit")),
                &self.token,
            )
            .await
    }

    /// `POST /admin/db/{owner}/{db}/backup`
    ///
    /// Creates a backup for the database.
    ///
    /// Returns `201` when backup is created.
    ///
    /// Common error responses: `401` unauthorized, `404` database or user not found.
    pub async fn admin_db_backup(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/backup")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /admin/db/{owner}/{db}/clear?resource={resource}`
    ///
    /// Clears selected database resources such as the database file, audit log,
    /// or backup, then returns updated database metadata.
    ///
    /// Returns `(200, ServerDatabase)` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` user or database not found.
    pub async fn admin_db_clear(
        &self,
        owner: &str,
        db: &str,
        resource: DbResource,
    ) -> AgdbApiResult<(u16, ServerDatabase)> {
        self.client
            .post::<(), ServerDatabase>(
                &self.url(&format!("/admin/db/{owner}/{db}/clear?resource={resource}")),
                None,
                &self.token,
            )
            .await
    }

    /// `POST /admin/db/{owner}/{db}/convert?db_type={db_type}`
    ///
    /// Changes the database storage type.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` user or database not found.
    pub async fn admin_db_convert(
        &self,
        owner: &str,
        db: &str,
        db_type: DbKind,
    ) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/convert?db_type={db_type}")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /admin/db/{owner}/{db}/copy?new_owner={new_owner}&new_db={new_db}`
    ///
    /// Copies a database to a new owner, a new name, or both.
    ///
    /// Returns `201` when copied.
    ///
    /// Common error responses: `401` unauthorized, `404` database or user not found,
    /// `465` target database exists, `467` invalid target database name.
    pub async fn admin_db_copy(
        &self,
        owner: &str,
        db: &str,
        new_owner: &str,
        new_db: &str,
    ) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!(
                    "/admin/db/{owner}/{db}/copy?new_owner={new_owner}&new_db={new_db}"
                )),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `DELETE /admin/db/{owner}/{db}/delete`
    ///
    /// Permanently deletes a database and its associated resources.
    ///
    /// Returns `204` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` database not found.
    pub async fn admin_db_delete(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/admin/db/{owner}/{db}/delete")),
                &self.token,
            )
            .await
    }

    /// `POST /admin/db/{owner}/{db}/exec`
    ///
    /// Executes read-only queries against the database.
    ///
    /// Returns `(200, Vec<QueryResult>)` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` database not found.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use agdb_api::{AgdbApi, ReqwestClient};
    /// use agdb::QueryBuilder;
    ///
    /// let api = AgdbApi::new(ReqwestClient::new(), "http://localhost:3000");
    /// let queries = vec![QueryBuilder::select().node_count().query().into()];
    /// let (_status, results) = api.admin_db_exec("owner", "db", &queries).await?;
    /// ```
    pub async fn admin_db_exec(
        &self,
        owner: &str,
        db: &str,
        queries: &[QueryType],
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        self.client
            .post(
                &self.url(&format!("/admin/db/{owner}/{db}/exec")),
                Some(queries),
                &self.token,
            )
            .await
    }

    /// `POST /admin/db/{owner}/{db}/exec_mut`
    ///
    /// Executes mutable queries against the database.
    ///
    /// Returns `(200, Vec<QueryResult>)` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` database not found.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use agdb_api::{AgdbApi, ReqwestClient};
    /// use agdb::QueryBuilder;
    ///
    /// let api = AgdbApi::new(ReqwestClient::new(), "http://localhost:3000");
    /// let queries = vec![QueryBuilder::insert().nodes().count(1).query().into()];
    /// let (_status, results) = api.admin_db_exec_mut("owner", "db", &queries).await?;
    /// ```
    pub async fn admin_db_exec_mut(
        &self,
        owner: &str,
        db: &str,
        queries: &[QueryType],
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        self.client
            .post(
                &self.url(&format!("/admin/db/{owner}/{db}/exec_mut")),
                Some(queries),
                &self.token,
            )
            .await
    }

    /// `GET /admin/db/list`
    ///
    /// Lists all databases registered on the server.
    ///
    /// Returns `(200, Vec<ServerDatabase>)` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn admin_db_list(&self) -> AgdbApiResult<(u16, Vec<ServerDatabase>)> {
        self.client
            .get(&self.url("/admin/db/list"), &self.token)
            .await
    }

    /// `POST /admin/db/{owner}/{db}/optimize`
    ///
    /// Optimizes database storage, reclaiming fragmented space.
    ///
    /// Returns `(200, ServerDatabase)` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` database not found.
    pub async fn admin_db_optimize(
        &self,
        owner: &str,
        db: &str,
    ) -> AgdbApiResult<(u16, ServerDatabase)> {
        self.client
            .post::<(), ServerDatabase>(
                &self.url(&format!("/admin/db/{owner}/{db}/optimize")),
                None,
                &self.token,
            )
            .await
    }

    /// `POST /admin/db/{owner}/{db}/optimize?shrink_to_fit=true`
    ///
    /// Optimizes database storage and requests shrink-to-fit behavior
    /// aggressively reclaiming all unused space.
    ///
    /// Returns `(200, ServerDatabase)` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` database not found.
    pub async fn admin_db_optimize_shrink_to_fit(
        &self,
        owner: &str,
        db: &str,
    ) -> AgdbApiResult<(u16, ServerDatabase)> {
        self.client
            .post::<(), ServerDatabase>(
                &self.url(&format!(
                    "/admin/db/{owner}/{db}/optimize?shrink_to_fit=true"
                )),
                None,
                &self.token,
            )
            .await
    }

    /// `DELETE /admin/db/{owner}/{db}/remove`
    ///
    /// Disassociates a database and related resources from the server. This
    /// does NOT delete any resources (database, backups, audits etc.)
    /// from the server. Use /delete endpoint to physically delete a database instead.
    /// You can re-add the database with /add endpoint with the same name.
    ///
    /// This endpoints is useful for maintenance work or when adding/removing previously unmanaged
    /// database to the server.
    ///
    /// Returns `204` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` database not found.
    pub async fn admin_db_remove(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/admin/db/{owner}/{db}/remove")),
                &self.token,
            )
            .await
    }

    /// `POST /admin/db/{owner}/{db}/rename?new_owner={new_owner}&new_db={new_db}`
    ///
    /// Renames/moves a database to a different owner or name or both.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` database or user not found,
    /// `465` target database exists, `467` invalid target database name.
    pub async fn admin_db_rename(
        &self,
        owner: &str,
        db: &str,
        new_owner: &str,
        new_db: &str,
    ) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!(
                    "/admin/db/{owner}/{db}/rename?new_owner={new_owner}&new_db={new_db}"
                )),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /admin/db/{owner}/{db}/restore`
    ///
    /// Restores a database from backup while keeping the backup unchanged.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` backup not found.
    pub async fn admin_db_restore(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/restore")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /admin/db/{owner}/{db}/rollback`
    ///
    /// Rolls back a database by swapping the current database with the backup.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` backup not found.
    pub async fn admin_db_rollback(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/rollback")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `PUT /admin/db/{owner}/{db}/user/{username}/add?db_role={db_role}`
    ///
    /// Adds a database user or updates the user's database role. Owner's
    /// role cannot be changed.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` cannot change role of database owner,
    /// `404` user or database not found.
    pub async fn admin_db_user_add(
        &self,
        owner: &str,
        db: &str,
        username: &str,
        db_role: DbUserRole,
    ) -> AgdbApiResult<u16> {
        self.client
            .put::<()>(
                &self.url(&format!(
                    "/admin/db/{owner}/{db}/user/{username}/add?db_role={db_role}"
                )),
                None,
                &self.token,
            )
            .await
    }

    /// `GET /admin/db/{owner}/{db}/user/list`
    ///
    /// Lists users with access to the database and their current roles.
    ///
    /// Returns `(200, Vec<DbUser>)` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` database not found.
    pub async fn admin_db_user_list(
        &self,
        owner: &str,
        db: &str,
    ) -> AgdbApiResult<(u16, Vec<DbUser>)> {
        self.client
            .get(
                &self.url(&format!("/admin/db/{owner}/{db}/user/list")),
                &self.token,
            )
            .await
    }

    /// `DELETE /admin/db/{owner}/{db}/user/{username}/remove`
    ///
    /// Removes user access from a database. Owner cannot be
    /// removed from their database.
    ///
    /// Returns `204` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` cannot remove database owner,
    /// `404` user or database not found.
    pub async fn admin_db_user_remove(
        &self,
        owner: &str,
        db: &str,
        username: &str,
    ) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/admin/db/{owner}/{db}/user/{username}/remove")),
                &self.token,
            )
            .await
    }

    /// `POST /admin/shutdown`
    ///
    /// Requests server shutdown.
    ///
    /// Returns `200` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn admin_shutdown(&self) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(&self.url("/admin/shutdown"), None, &self.token)
            .await?
            .0)
    }

    /// `POST /admin/set_log_level?new_level={level}`
    ///
    /// Sets the server log level.
    ///
    /// Returns `200` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn admin_set_log_level(&self, level: LogLevelFilter) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/set_log_level?new_level={level}")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `GET /admin/status`
    ///
    /// Returns administrative server status information such
    /// as uptime, database and user counts, log level, occupied
    /// space etc.
    ///
    /// Returns `(200, AdminStatus)` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn admin_status(&self) -> AgdbApiResult<(u16, AdminStatus)> {
        self.client
            .get(&self.url("/admin/status"), &self.token)
            .await
    }

    /// `POST /admin/user/{username}/add`
    ///
    /// Creates a new user.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `461` password too short,
    /// `462` user name too short, `463` user already exists.
    pub async fn admin_user_add(&self, username: &str, password: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<UserCredentials, ()>(
                &self.url(&format!("/admin/user/{username}/add")),
                Some(UserCredentials {
                    password: password.to_string(),
                }),
                &self.token,
            )
            .await?
            .0)
    }

    /// `PUT /admin/user/{username}/change_password`
    ///
    /// Changes password for a specific user.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `464` user not found.
    pub async fn admin_user_change_password(
        &self,
        username: &str,
        password: &str,
    ) -> AgdbApiResult<u16> {
        self.client
            .put::<UserCredentials>(
                &self.url(&format!("/admin/user/{username}/change_password")),
                Some(UserCredentials {
                    password: password.to_string(),
                }),
                &self.token,
            )
            .await
    }

    /// `GET /admin/user/list`
    ///
    /// Lists all users and their active sessions.
    ///
    /// Returns `(200, Vec<UserStatus>)` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn admin_user_list(&self) -> AgdbApiResult<(u16, Vec<UserStatus>)> {
        self.client
            .get(&self.url("/admin/user/list"), &self.token)
            .await
    }

    /// `POST /admin/user/{username}/logout`
    ///
    /// Logs out all sessions for `username`.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` admin only, `404` user not found.
    pub async fn admin_user_logout(&self, username: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/user/{username}/logout")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /admin/user/{username}/logout?session={session}`
    ///
    /// Logs out a single session for `username`.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` admin only, `404` user or session not found.
    pub async fn admin_user_logout_session(
        &self,
        username: &str,
        session: &str,
    ) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/user/{username}/logout?session={session}")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /admin/user/logout_all`
    ///
    /// Logs out all users from all sessions except for the current admin user.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn admin_user_logout_all(&self) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(&self.url("/admin/user/logout_all"), None, &self.token)
            .await?
            .0)
    }

    /// `DELETE /admin/user/{username}/delete`
    ///
    /// Deletes a user. Deleting a user also deletes all databases owned by the user
    /// and logs out all sessions of the user.
    ///
    /// Returns `204` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` user not found.
    pub async fn admin_user_delete(&self, username: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/admin/user/{username}/delete")),
                &self.token,
            )
            .await
    }

    /// `POST /cluster/admin/user/{username}/logout`
    ///
    /// Logs out all sessions of a user across the cluster.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` admin only, `404` user not found.
    pub async fn cluster_admin_user_logout(&self, username: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/cluster/admin/user/{username}/logout")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /cluster/admin/user/{username}/logout?session={session}`
    ///
    /// Logs out one specific session of a user across the cluster.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` admin only, `404` user or session not found.
    pub async fn cluster_admin_user_logout_session(
        &self,
        username: &str,
        session: &str,
    ) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!(
                    "/cluster/admin/user/{username}/logout?session={session}"
                )),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /cluster/admin/user/logout_all`
    ///
    /// Logs out all users across the cluster except for the current admin user.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn cluster_admin_user_logout_all(&self) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url("/cluster/admin/user/logout_all"),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /cluster/user/login`
    ///
    /// Authenticates a user cluster-wide and stores the returned token in [`Self::token`].
    ///
    /// Returns `200` on success.
    ///
    /// Common error responses: `401` invalid credentials.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut api = agdb_api::AgdbApi::new(agdb_api::ReqwestClient::new(), "http://localhost:3000");
    /// let status = api.cluster_user_login("user", "password").await?;
    /// assert_eq!(status, 200);
    /// assert!(api.token.is_some());
    /// ```
    pub async fn cluster_user_login(
        &mut self,
        username: &str,
        password: &str,
    ) -> AgdbApiResult<u16> {
        let (status, token) = self
            .client
            .post::<UserLogin, String>(
                &self.url("/cluster/user/login"),
                Some(UserLogin {
                    username: username.to_string(),
                    password: password.to_string(),
                }),
                &None,
            )
            .await?;
        self.token = Some(token);
        Ok(status)
    }

    /// `POST /cluster/user/logout`
    ///
    /// Logs out the current session across the cluster and clears [`Self::token`]. This clears
    /// only the current session previously authenticated with [`Self::cluster_user_login`] and
    /// does not affect other sessions of the user if they exist.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn cluster_user_logout(&mut self) -> AgdbApiResult<u16> {
        let status = self
            .client
            .post::<(), ()>(&self.url("/cluster/user/logout"), None, &self.token)
            .await?
            .0;
        self.token = None;
        Ok(status)
    }

    /// `POST /cluster/user/logout?session=others`
    ///
    /// Logs out all sessions across the cluster except the current one.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn cluster_user_logout_others(&mut self) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url("/cluster/user/logout?session=others"),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /cluster/user/logout?session=all`
    ///
    /// Logs out all sessions across the cluster and clears [`Self::token`].
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn cluster_user_logout_all(&mut self) -> AgdbApiResult<u16> {
        let status = self
            .client
            .post::<(), ()>(
                &self.url("/cluster/user/logout?session=all"),
                None,
                &self.token,
            )
            .await?
            .0;
        self.token = None;
        Ok(status)
    }

    /// `POST /cluster/user/logout?session={session}`
    ///
    /// Logs out a specific session by id across the cluster.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` session not found.
    pub async fn cluster_user_logout_session(&self, session: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/cluster/user/logout?session={session}")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `GET /cluster/status`
    ///
    /// Returns cluster node status information such as leader identity,
    /// node availability, and node addresses.
    ///
    /// Returns `(200, Vec<ClusterStatus>)` on success.
    pub async fn cluster_status(&self) -> AgdbApiResult<(u16, Vec<ClusterStatus>)> {
        self.client.get(&self.url("/cluster/status"), &None).await
    }

    /// `POST /db/{owner}/{db}/add?db_type={db_type}`
    ///
    /// Adds a database owned by the currently authenticated user. Owner
    /// must be the current user.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` cannot add database to another user,
    /// `465` database already exists, `467` invalid database name.
    pub async fn db_add(&self, owner: &str, db: &str, db_type: DbKind) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/add?db_type={db_type}")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `GET /db/{owner}/{db}/audit`
    ///
    /// Returns database audit information, including executed mutable queries
    /// with timestamps and users.
    ///
    /// Returns `(200, DbAudit)` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` user or database not found.
    pub async fn db_audit(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, DbAudit)> {
        self.client
            .get(&self.url(&format!("/db/{owner}/{db}/audit")), &self.token)
            .await
    }

    /// `POST /db/{owner}/{db}/backup`
    ///
    /// Creates a database backup. Current database becomes the backup and the previous backup is overwritten if it exists.
    /// For ad-hoc or long term backups consider using /db_copy endpoint.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` admin role required or memory databases cannot be backed up,
    /// `404` user or database not found.
    pub async fn db_backup(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/backup")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /db/{owner}/{db}/clear?resource={resource}`
    ///
    /// Clears selected resources (audit, backup etc.) and returns updated database metadata.
    ///
    /// Returns `(200, ServerDatabase)` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` admin role required,
    /// `404` user or database not found.
    pub async fn db_clear(
        &self,
        owner: &str,
        db: &str,
        resource: DbResource,
    ) -> AgdbApiResult<(u16, ServerDatabase)> {
        self.client
            .post::<(), ServerDatabase>(
                &self.url(&format!("/db/{owner}/{db}/clear?resource={resource}")),
                None,
                &self.token,
            )
            .await
    }

    /// `POST /db/{owner}/{db}/convert?db_type={db_type}`
    ///
    /// Converts database storage type between in-memory, memory-mapped, and file-backed modes.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` admin role required,
    /// `404` user or database not found.
    pub async fn db_convert(&self, owner: &str, db: &str, db_type: DbKind) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/convert?db_type={db_type}")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /db/{owner}/{db}/copy?new_db={new_db}`
    ///
    /// Copies a database for the current user. Owner must be the current user. This is useful for creating ad-hoc
    /// backups, creating a snapshot or converting it to a different storage type with /convert endpoint.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` source database not found,
    /// `465` target database exists, `467` invalid target database name.
    pub async fn db_copy(&self, owner: &str, db: &str, new_db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/copy?new_db={new_db}")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `DELETE /db/{owner}/{db}/delete`
    ///
    /// Deletes a database and all associated resources. This action is permanent and cannot be undone.
    /// Use /remove endpoint to disassociate the database from the server without deleting the underlying resources.
    ///
    /// Returns `204` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` owner only, `404` database not found.
    pub async fn db_delete(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(&self.url(&format!("/db/{owner}/{db}/delete")), &self.token)
            .await
    }

    /// `POST /db/{owner}/{db}/exec`
    ///
    /// Executes read-only queries.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use agdb_api::{AgdbApi, ReqwestClient};
    /// use agdb::QueryBuilder;
    ///
    /// let api = AgdbApi::new(ReqwestClient::new(), "http://localhost:3000");
    /// let queries = vec![QueryBuilder::select().node_count().query().into()];
    /// let (_status, results) = api.db_exec("owner", "db", &queries).await?;
    /// ```
    ///
    /// Returns `(200, Vec<QueryResult>)` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` mutable queries are not allowed here,
    /// `404` database not found.
    pub async fn db_exec(
        &self,
        owner: &str,
        db: &str,
        queries: &[QueryType],
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        self.client
            .post(
                &self.url(&format!("/db/{owner}/{db}/exec")),
                Some(queries),
                &self.token,
            )
            .await
    }

    /// `POST /db/{owner}/{db}/exec_mut`
    ///
    /// Executes mutable queries.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use agdb_api::{AgdbApi, ReqwestClient};
    /// use agdb::QueryBuilder;
    ///
    /// let api = AgdbApi::new(ReqwestClient::new(), "http://localhost:3000");
    /// let queries = vec![QueryBuilder::insert().nodes().count(1).query().into()];
    /// let (_status, results) = api.db_exec_mut("owner", "db", &queries).await?;
    /// ```
    ///
    /// Returns `(200, Vec<QueryResult>)` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` write access required,
    /// `404` database not found.
    pub async fn db_exec_mut(
        &self,
        owner: &str,
        db: &str,
        queries: &[QueryType],
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        self.client
            .post(
                &self.url(&format!("/db/{owner}/{db}/exec_mut")),
                Some(queries),
                &self.token,
            )
            .await
    }

    /// `GET /db/list`
    ///
    /// Lists databases accessible to the current user
    /// and their metadata.
    ///
    /// Returns `(200, Vec<ServerDatabase>)` on success.
    ///
    /// Common error responses: `401` unauthorized.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut api = agdb_api::AgdbApi::new(agdb_api::ReqwestClient::new(), "http://localhost:3000");
    /// api.user_login("user", "password").await?;
    /// let (status, dbs) = api.db_list().await?;
    /// assert_eq!(status, 200);
    /// assert!(!dbs.is_empty());
    /// # Ok::<(), agdb_api::ApiError>(())
    /// ```
    pub async fn db_list(&self) -> AgdbApiResult<(u16, Vec<ServerDatabase>)> {
        self.client.get(&self.url("/db/list"), &self.token).await
    }

    /// `POST /db/{owner}/{db}/optimize`
    ///
    /// Optimizes database storage reclaiming fragmented space. This can improve performance
    /// and reduce storage usage after heavy updates or deletions.
    ///
    /// Returns `(200, ServerDatabase)` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` write access required,
    /// `404` database not found.
    pub async fn db_optimize(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, ServerDatabase)> {
        self.client
            .post::<(), ServerDatabase>(
                &self.url(&format!("/db/{owner}/{db}/optimize")),
                None,
                &self.token,
            )
            .await
    }

    /// `POST /db/{owner}/{db}/optimize?shrink_to_fit=true`
    ///
    /// Optimizes database storage with shrink-to-fit. This aggressively reclaims all unused space and can reduce
    /// storage usage to a minimum after heavy updates or deletions, but it can be slower than regular optimize
    /// and may require reallocating some of the reclaimed space upon subsequent updates. It is best suited when the
    /// database will stay mostly stable afterward or when minimizing storage usage matters more than update performance.
    ///
    /// Returns `(200, ServerDatabase)` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` write access required,
    /// `404` database not found.
    pub async fn db_optimize_shrink_to_fit(
        &self,
        owner: &str,
        db: &str,
    ) -> AgdbApiResult<(u16, ServerDatabase)> {
        self.client
            .post::<(), ServerDatabase>(
                &self.url(&format!("/db/{owner}/{db}/optimize?shrink_to_fit=true")),
                None,
                &self.token,
            )
            .await
    }

    /// `DELETE /db/{owner}/{db}/remove`
    ///
    /// Removes a database. This disassociates the database and all related resources (backups, audits etc.) from the server
    /// but does NOT delete them from the server. Use /delete endpoint to permanently delete a database instead.
    /// You can re-add the database with /add endpoint with the same name.
    ///
    /// Returns `204` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` owner only, `404` database not found.
    pub async fn db_remove(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(&self.url(&format!("/db/{owner}/{db}/remove")), &self.token)
            .await
    }

    /// `POST /db/{owner}/{db}/rename?new_db={new_db}`
    ///
    /// Renames a database owned by the current user.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` owner only, `404` user or database not found,
    /// `465` target database exists, `467` invalid target database name.
    pub async fn db_rename(&self, owner: &str, db: &str, new_db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/rename?new_db={new_db}")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /db/{owner}/{db}/restore`
    ///
    /// Restores database from backup while keeping the backup unchanged.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` admin role required,
    /// `404` backup not found.
    pub async fn db_restore(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/restore")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `POST /db/{owner}/{db}/rollback`
    ///
    /// Rolls back a database by swapping the current database with the backup.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` admin role required,
    /// `404` backup not found.
    pub async fn db_rollback(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/rollback")),
                None,
                &self.token,
            )
            .await?
            .0)
    }

    /// `PUT /db/{owner}/{db}/user/{username}/add?db_role={db_role}`
    ///
    /// Adds/updates database user role. Owner's role cannot be changed.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` cannot change database owner role,
    /// `404` user or database not found.
    pub async fn db_user_add(
        &self,
        owner: &str,
        db: &str,
        username: &str,
        db_role: DbUserRole,
    ) -> AgdbApiResult<u16> {
        self.client
            .put::<()>(
                &self.url(&format!(
                    "/db/{owner}/{db}/user/{username}/add?db_role={db_role}"
                )),
                None,
                &self.token,
            )
            .await
    }

    /// `GET /db/{owner}/{db}/user/list`
    ///
    /// Lists users assigned to the database and their role.
    ///
    /// Returns `(200, Vec<DbUser>)` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` database not found.
    pub async fn db_user_list(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, Vec<DbUser>)> {
        self.client
            .get(
                &self.url(&format!("/db/{owner}/{db}/user/list")),
                &self.token,
            )
            .await
    }

    /// `DELETE /db/{owner}/{db}/user/{username}/remove`
    ///
    /// Removes a user from the database. The owner cannot be removed.
    ///
    /// Returns `204` on success.
    ///
    /// Common error responses: `401` unauthorized, `403` owner only or cannot remove database owner,
    /// `404` user or database not found.
    pub async fn db_user_remove(
        &self,
        owner: &str,
        db: &str,
        username: &str,
    ) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/db/{owner}/{db}/user/{username}/remove")),
                &self.token,
            )
            .await
    }

    /// `GET /status`
    ///
    /// Lightweight service health endpoint. Returns no data.
    ///
    /// Returns `200` when server is up.
    pub async fn status(&self) -> AgdbApiResult<u16> {
        Ok(self.client.get::<()>(&self.url("/status"), &None).await?.0)
    }

    /// `POST /user/login`
    ///
    /// Authenticates a user in the current node and stores the returned token in [`Self::token`].
    ///
    /// Returns `200` on success.
    ///
    /// Common error responses: `401` invalid credentials.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut api = agdb_api::AgdbApi::new(agdb_api::ReqwestClient::new(), "http://localhost:3000");
    /// let status = api.user_login("user", "password").await?;
    /// assert_eq!(status, 200);
    /// assert!(api.token.is_some());
    /// # Ok::<(), agdb_api::ApiError>(())
    /// ```
    pub async fn user_login(&mut self, username: &str, password: &str) -> AgdbApiResult<u16> {
        let (status, token) = self
            .client
            .post::<UserLogin, String>(
                &self.url("/user/login"),
                Some(UserLogin {
                    username: username.to_string(),
                    password: password.to_string(),
                }),
                &None,
            )
            .await?;
        self.token = Some(token);
        Ok(status)
    }

    /// `POST /user/logout`
    ///
    /// Logs out current session from the current node and clears [`Self::token`].
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn user_logout(&mut self) -> AgdbApiResult<u16> {
        let status = self
            .client
            .post::<(), ()>(&self.url("/user/logout"), None, &self.token)
            .await?
            .0;
        self.token = None;
        Ok(status)
    }

    /// `POST /user/logout?session=others`
    ///
    /// Logs out all sessions except the current one from the current node.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn user_logout_others(&mut self) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(&self.url("/user/logout?session=others"), None, &self.token)
            .await?
            .0)
    }

    /// `POST /user/logout?session=all`
    ///
    /// Logs out all sessions from the current node and clears [`Self::token`].
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn user_logout_all(&mut self) -> AgdbApiResult<u16> {
        let status = self
            .client
            .post::<(), ()>(&self.url("/user/logout?session=all"), None, &self.token)
            .await?
            .0;
        self.token = None;
        Ok(status)
    }

    /// `POST /user/logout?session={session}`
    ///
    /// Logs out one specified session from the current node.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `404` session not found.
    pub async fn user_logout_session(&self, session: &str) -> AgdbApiResult<u16> {
        self.client
            .post::<(), ()>(
                &self.url(&format!("/user/logout?session={session}")),
                None,
                &self.token,
            )
            .await
            .map(|(status, _)| status)
    }

    /// `PUT /user/change_password`
    ///
    /// Changes password of the currently authenticated user.
    ///
    /// Returns `201` on success.
    ///
    /// Common error responses: `401` unauthorized, `461` password too short.
    pub async fn user_change_password(
        &self,
        old_password: &str,
        new_password: &str,
    ) -> AgdbApiResult<u16> {
        self.client
            .put(
                &self.url("/user/change_password"),
                Some(ChangePassword {
                    password: old_password.to_string(),
                    new_password: new_password.to_string(),
                }),
                &self.token,
            )
            .await
    }

    /// `GET /user/status`
    ///
    /// Returns status and active sessions of the current user.
    ///
    /// Returns `(200, UserStatus)` on success.
    ///
    /// Common error responses: `401` unauthorized.
    pub async fn user_status(&self) -> AgdbApiResult<(u16, UserStatus)> {
        self.client
            .get(&self.url("/user/status"), &self.token)
            .await
    }

    fn url(&self, uri: &str) -> String {
        format!("{}{uri}", self.base_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReqwestClient;

    #[test]
    fn address() {
        let client = AgdbApi::new(ReqwestClient::new(), "http://localhost:3000");
        assert_eq!(client.address(), "http://localhost:3000");
    }

    #[test]
    fn base_path() {
        let client = AgdbApi::new(ReqwestClient::new(), "http://localhost:3000/public");
        assert_eq!(client.address(), "http://localhost:3000/public");
        assert_eq!(client.base_url(), "http://localhost:3000/public/api/v1");
    }
}
