mod api_error;
mod api_result;
mod api_types;
mod http_client;

use agdb::QueryResult;
use agdb::QueryType;

use crate::api_result::AgdbApiResult;
use crate::api_types::DbType;
use crate::api_types::ServerDatabase;
use crate::http_client::HttpClient;

pub struct AgdbApi<T: HttpClient> {
    client: T,
    host: String,
    port: u16,
    token: Option<String>,
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

    pub async fn admin_db_add(
        &self,
        owner: &str,
        db: &str,
        db_type: DbType,
    ) -> AgdbApiResult<(u16, ())> {
        self.client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/add?db_type={db_type}")),
                &None,
                &self.token,
            )
            .await
    }

    pub async fn admin_db_backup(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, ())> {
        self.client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/backup")),
                &None,
                &self.token,
            )
            .await
    }

    pub async fn admin_db_copy(
        &self,
        owner: &str,
        db: &str,
        new_owner: &str,
        new_db: &str,
    ) -> AgdbApiResult<(u16, ())> {
        self.client
            .post::<(), ()>(
                &self.url(&format!(
                    "/admin/db/{owner}/{db}/copy?new_name={new_owner}/{new_db}"
                )),
                &None,
                &self.token,
            )
            .await
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
    ) -> AgdbApiResult<(u16, ())> {
        self.client
            .post::<(), ()>(
                &self.url(&format!(
                    "/admin/db/{owner}/{db}/reename?new_name={new_owner}/{new_db}"
                )),
                &None,
                &self.token,
            )
            .await
    }

    pub async fn admin_db_restore(&self, owner: &str, db: &str) -> AgdbApiResult<(u16, ())> {
        self.client
            .post::<(), ()>(
                &self.url(&format!("/admin/db/{owner}/{db}/restore")),
                &None,
                &self.token,
            )
            .await
    }

    fn url(&self, uri: &str) -> String {
        format!("{}:{}/api/v1{uri}", self.host, self.port)
    }
}
