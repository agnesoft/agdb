use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::config::DbType;
use agdb::DbImpl;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb::QueryType;
use agdb::StorageData;
use agdb_api::AgdbApi;
use agdb_api::AgdbApiError;
use agdb_api::DbKind;
use agdb_api::ReqwestClient;
use agdb_api::ServerDatabase as ServerDatabaseStat;
use reqwest::Client;
use reqwest::ClientBuilder;
use std::sync::Arc;
use std::sync::RwLock;

const ADMIN_USERNAME: &str = "admin";
const ADMIN_PASSWORD: &str = "admin";
pub(crate) const BENCHMARK_USERNAME: &str = "agdb_benchmark";
pub(crate) const BENCHMARK_PASSWORD: &str = "agdb_benchmark";
pub(crate) const BENCHMARK_DATABASE: &str = "benchmark";

pub(crate) struct Database<S: StorageData>(pub(crate) Arc<RwLock<DbImpl<S>>>);

pub(crate) struct DatabaseSize {
    pub(crate) original: u64,
    pub(crate) optimized: u64,
}

pub(crate) struct ServerDatabase {
    client: Client,
    api: AgdbApi<ReqwestClient>,
    owner: String,
    db: String,
}

impl<S: StorageData> Database<S> {
    pub(crate) fn new(config: &Config) -> BenchResult<Self> {
        remove_db_files(&config.db_name);
        let mut db = DbImpl::new(&config.db_name)?;
        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(["users", "posts"])
                .query(),
        )?;
        Ok(Self(Arc::new(RwLock::new(db))))
    }

    pub(crate) fn stat(&mut self, _config: &Config) -> BenchResult<DatabaseSize> {
        let original_size = self.0.read()?.size();

        self.0.write()?.optimize_storage()?;

        let db_size = self.0.read()?.size();

        Ok(DatabaseSize {
            original: original_size,
            optimized: db_size,
        })
    }
}

impl ServerDatabase {
    pub(crate) async fn new(config: &Config, address: &str, client: Client) -> BenchResult<Self> {
        let mut admin_api = admin_api(client.clone(), address);
        admin_api.user_login(ADMIN_USERNAME, ADMIN_PASSWORD).await?;

        ensure_benchmark_user(&admin_api).await?;
        reset_benchmark_database(&admin_api, config).await?;

        let mut api = bench_api(client.clone(), address);
        api.user_login(BENCHMARK_USERNAME, BENCHMARK_PASSWORD)
            .await?;
        bootstrap_server_database(&api).await?;

        Ok(Self {
            client,
            api,
            owner: BENCHMARK_USERNAME.to_string(),
            db: BENCHMARK_DATABASE.to_string(),
        })
    }

    pub(crate) async fn stat(&mut self, _config: &Config) -> BenchResult<DatabaseSize> {
        let original_size = self.database_stat().await?.size;
        let db_size = self.api.db_optimize(&self.owner, &self.db).await?.1.size;

        Ok(DatabaseSize {
            original: original_size,
            optimized: db_size,
        })
    }

    pub(crate) async fn exec_mut(&self, queries: &[QueryType]) -> BenchResult<Vec<QueryResult>> {
        Ok(self
            .api
            .db_exec_mut(&self.owner, &self.db, queries)
            .await?
            .1)
    }

    pub(crate) async fn exec(&self, queries: &[QueryType]) -> BenchResult<Vec<QueryResult>> {
        Ok(self.api.db_exec(&self.owner, &self.db, queries).await?.1)
    }

    async fn database_stat(&self) -> BenchResult<ServerDatabaseStat> {
        let databases = self.api.db_list().await?.1;

        databases
            .into_iter()
            .find(|database| database.owner == self.owner && database.db == self.db)
            .ok_or_else(|| AgdbApiError {
                status: 404,
                description: format!(
                    "database '{}'/ '{}' not found in server listing",
                    self.owner, self.db
                ),
            })
            .map_err(Into::into)
    }

    pub(crate) fn address(&self) -> &str {
        self.api.address()
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }
}

impl<S: StorageData> Clone for Database<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

async fn ensure_benchmark_user(api: &AgdbApi<ReqwestClient>) -> BenchResult<()> {
    match api
        .admin_user_add(BENCHMARK_USERNAME, BENCHMARK_PASSWORD)
        .await
    {
        Ok(_) => {}
        Err(error) if error.status == 463 => {}
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

async fn reset_benchmark_database(
    api: &AgdbApi<ReqwestClient>,
    config: &Config,
) -> BenchResult<()> {
    match api
        .admin_db_delete(BENCHMARK_USERNAME, BENCHMARK_DATABASE)
        .await
    {
        Ok(_) => {}
        Err(error) if error.status == 404 => {}
        Err(error) => return Err(error.into()),
    }

    api.admin_db_add(
        BENCHMARK_USERNAME,
        BENCHMARK_DATABASE,
        db_kind(&config.db_type),
    )
    .await?;

    Ok(())
}

async fn bootstrap_server_database(api: &AgdbApi<ReqwestClient>) -> BenchResult<()> {
    let queries = vec![
        QueryBuilder::insert()
            .nodes()
            .aliases(["users", "posts"])
            .query()
            .into(),
    ];

    api.db_exec_mut(BENCHMARK_USERNAME, BENCHMARK_DATABASE, &queries)
        .await?;

    Ok(())
}

fn db_kind(db_type: &DbType) -> DbKind {
    match db_type {
        DbType::File => DbKind::File,
        DbType::FileMapped => DbKind::Mapped,
        DbType::InMemory => DbKind::Memory,
    }
}

fn remove_db_files(db_name: &str) {
    let path = std::path::PathBuf::from(format!(".{db_name}"));
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    let path = std::path::PathBuf::from(db_name);
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

pub(crate) fn create_server_http_client(config: &Config) -> BenchResult<Client> {
    Ok(ClientBuilder::new()
        .danger_accept_invalid_certs(config.server.allow_invalid_certs)
        .http2_adaptive_window(true)
        .build()?)
}

pub(crate) fn admin_api(client: Client, address: &str) -> AgdbApi<ReqwestClient> {
    AgdbApi::new(ReqwestClient::with_client(client), address)
}

pub(crate) fn bench_api(client: Client, address: &str) -> AgdbApi<ReqwestClient> {
    AgdbApi::new(ReqwestClient::with_client(client), address)
}
