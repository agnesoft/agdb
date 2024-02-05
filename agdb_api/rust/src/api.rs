use crate::api_result::AgdbApiResult;
use crate::api_types::DbType;
use crate::api_types::ServerDatabase;
use crate::api_types::UserCredentials;
use crate::http_client::HttpClient;
use crate::ChangePassword;
use crate::DbAudit;
use crate::DbUser;
use crate::DbUserRole;
use crate::UserLogin;
use crate::UserStatus;
use agdb::QueryResult;
use agdb::QueryType;

pub struct AgdbApi<T: HttpClient> {
    client: T,
    host: String,
    port: u16,
    pub token: Option<String>,
}

impl<T: HttpClient> AgdbApi<T> {
    pub fn new(client: T, host: &str, port: u16) -> Self {
        Self {
            client,
            host: host.to_string(),
            port,
            token: None,
        }
    }

    pub async fn admin_db_add(&self, owner: &str, db: &str, db_type: DbType) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/add?db_type={db_type}")),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn admin_db_audit(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, DbAudit)> {
        self.client
            .get(
                &self.url(&format!("/admin/db/{owner}/{db}/audit")),
                &self.token,
            )
            .await
    }

    pub async fn admin_db_backup(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/backup")),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

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
                    "/admin/db/{owner}/{db}/copy?new_name={new_owner}/{new_db}"
                )),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn admin_db_delete(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/admin/db/{owner}/{db}/delete")),
                &self.token,
            )
            .await
    }

    pub async fn admin_db_exec(
        &self,
        owner: &str,
        db: &str,
        queries: &Vec<QueryType>,
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        self.client
            .post(
                &self.url(&format!("/admin/db/{owner}/{db}/exec")),
                &Some(queries),
                &self.token,
            )
            .await
    }

    pub async fn admin_db_list(&self) -> AgdbApiResult<(u16, Vec<ServerDatabase>)> {
        self.client
            .get(&self.url("/admin/db/list"), &self.token)
            .await
    }

    pub async fn admin_db_optimize(
        &self,
        owner: &str,
        db: &str,
    ) -> AgdbApiResult<(u16, ServerDatabase)> {
        self.client
            .post::<(), ServerDatabase>(
                &self.url(&format!("/admin/db/{owner}/{db}/optimize")),
                &None,
                &self.token,
            )
            .await
    }

    pub async fn admin_db_remove(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/admin/db/{owner}/{db}/remove")),
                &self.token,
            )
            .await
    }

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
                    "/admin/db/{owner}/{db}/rename?new_name={new_owner}/{new_db}"
                )),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn admin_db_restore(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/restore")),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn admin_db_user_add(
        &self,
        owner: &str,
        db: &str,
        user: &str,
        db_role: DbUserRole,
    ) -> AgdbApiResult<u16> {
        self.client
            .put::<()>(
                &self.url(&format!(
                    "/admin/db/{owner}/{db}/user/{user}/add?db_role={db_role}"
                )),
                &None,
                &self.token,
            )
            .await
    }

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

    pub async fn admin_db_user_remove(
        &self,
        owner: &str,
        db: &str,
        user: &str,
    ) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/admin/db/{owner}/{db}/user/{user}/remove")),
                &self.token,
            )
            .await
    }

    pub async fn admin_shutdown(&self) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(&self.url("/admin/shutdown"), &None, &self.token)
            .await?
            .0)
    }

    pub async fn admin_user_add(&self, user: &str, password: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<UserCredentials, ()>(
                &self.url(&format!("/admin/user/{user}/add")),
                &Some(UserCredentials {
                    password: password.to_string(),
                }),
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn admin_user_change_password(
        &self,
        user: &str,
        password: &str,
    ) -> AgdbApiResult<u16> {
        self.client
            .put::<UserCredentials>(
                &self.url(&format!("/admin/user/{user}/change_password")),
                &Some(UserCredentials {
                    password: password.to_string(),
                }),
                &self.token,
            )
            .await
    }

    pub async fn admin_user_list(&self) -> AgdbApiResult<(u16, Vec<UserStatus>)> {
        self.client
            .get(&self.url("/admin/user/list"), &self.token)
            .await
    }

    pub async fn admin_user_remove(&self, user: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/admin/user/{user}/remove")),
                &self.token,
            )
            .await
    }

    pub async fn db_add(&self, owner: &str, db: &str, db_type: DbType) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/add?db_type={db_type}")),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn db_audit(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, DbAudit)> {
        self.client
            .get(&self.url(&format!("/db/{owner}/{db}/audit")), &self.token)
            .await
    }

    pub async fn db_backup(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/backup")),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn db_copy(
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
                    "/db/{owner}/{db}/copy?new_name={new_owner}/{new_db}"
                )),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn db_delete(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(&self.url(&format!("/db/{owner}/{db}/delete")), &self.token)
            .await
    }

    pub async fn db_exec(
        &self,
        owner: &str,
        db: &str,
        queries: &Vec<QueryType>,
    ) -> AgdbApiResult<(u16, Vec<QueryResult>)> {
        self.client
            .post(
                &self.url(&format!("/db/{owner}/{db}/exec")),
                &Some(queries),
                &self.token,
            )
            .await
    }

    pub async fn db_list(&self) -> AgdbApiResult<(u16, Vec<ServerDatabase>)> {
        self.client.get(&self.url("/db/list"), &self.token).await
    }

    pub async fn db_optimize(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, ServerDatabase)> {
        self.client
            .post::<(), ServerDatabase>(
                &self.url(&format!("/db/{owner}/{db}/optimize")),
                &None,
                &self.token,
            )
            .await
    }

    pub async fn db_remove(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(&self.url(&format!("/db/{owner}/{db}/remove")), &self.token)
            .await
    }

    pub async fn db_rename(
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
                    "/db/{owner}/{db}/rename?new_name={new_owner}/{new_db}"
                )),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn db_restore(&self, owner: &str, db: &str) -> AgdbApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/db/{owner}/{db}/restore")),
                &None,
                &self.token,
            )
            .await?
            .0)
    }

    pub async fn db_user_add(
        &self,
        owner: &str,
        db: &str,
        user: &str,
        db_role: DbUserRole,
    ) -> AgdbApiResult<u16> {
        self.client
            .put::<()>(
                &self.url(&format!(
                    "/db/{owner}/{db}/user/{user}/add?db_role={db_role}"
                )),
                &None,
                &self.token,
            )
            .await
    }

    pub async fn db_user_list(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, Vec<DbUser>)> {
        self.client
            .get(
                &self.url(&format!("/db/{owner}/{db}/user/list")),
                &self.token,
            )
            .await
    }

    pub async fn db_user_remove(&self, owner: &str, db: &str, user: &str) -> AgdbApiResult<u16> {
        self.client
            .delete(
                &self.url(&format!("/db/{owner}/{db}/user/{user}/remove")),
                &self.token,
            )
            .await
    }

    pub async fn status(&self) -> AgdbApiResult<u16> {
        Ok(self.client.get::<()>(&self.url("/status"), &None).await?.0)
    }

    pub async fn user_login(&mut self, user: &str, password: &str) -> AgdbApiResult<u16> {
        let (status, token) = self
            .client
            .post::<UserLogin, String>(
                &self.url("/user/login"),
                &Some(UserLogin {
                    username: user.to_string(),
                    password: password.to_string(),
                }),
                &None,
            )
            .await?;
        self.token = Some(token);
        Ok(status)
    }

    pub async fn user_logout(&mut self) -> AgdbApiResult<u16> {
        let status = self
            .client
            .post::<(), ()>(&self.url("/user/logout"), &None, &self.token)
            .await?
            .0;
        self.token = None;
        Ok(status)
    }

    pub async fn user_change_password(
        &self,
        old_password: &str,
        new_pasword: &str,
    ) -> AgdbApiResult<u16> {
        self.client
            .put(
                &self.url("/user/change_password"),
                &Some(ChangePassword {
                    password: old_password.to_string(),
                    new_password: new_pasword.to_string(),
                }),
                &self.token,
            )
            .await
    }

    fn url(&self, uri: &str) -> String {
        format!("{}:{}/api/v1{uri}", self.host, self.port)
    }
}
