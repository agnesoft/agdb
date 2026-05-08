use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::database::BENCHMARK_DATABASE;
use crate::database::BENCHMARK_USERNAME;
use crate::database::Database;
use crate::database::ServerDatabase;
use crate::users::benchmark_password;
use crate::users::comment_reader_username;
use crate::users::post_reader_username;
use crate::utilities;
use crate::utilities::measured;
use crate::utilities::measured_async;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::StorageData;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use std::time::Duration;
use std::time::Instant;
use tokio::task::JoinHandle;

struct Reader<S: StorageData> {
    db: Database<S>,
    end: Duration,
    pub(crate) times: Vec<Duration>,
}

pub(crate) struct Readers<S: StorageData> {
    tasks: Vec<JoinHandle<Reader<S>>>,
}

struct ServerReader {
    api: AgdbApi<ReqwestClient>,
    end: Duration,
    pub(crate) times: Vec<Duration>,
}

pub(crate) struct ServerReaders {
    tasks: Vec<JoinHandle<BenchResult<ServerReader>>>,
}

impl<S: StorageData> Reader<S> {
    pub(crate) fn new(db: Database<S>) -> Self {
        Self {
            db,
            end: Duration::default(),
            times: vec![],
        }
    }

    fn read_comments(&mut self, limit: u64) -> BenchResult<bool> {
        if let Some(post_id) = self.last_post()? {
            let duration = measured(|| {
                let _comments = self.db.0.read()?.exec(
                    QueryBuilder::select()
                        .ids(
                            QueryBuilder::search()
                                .from(post_id)
                                .limit(limit)
                                .where_()
                                .neighbor()
                                .query(),
                        )
                        .query(),
                )?;
                Ok(())
            })?;

            self.times.push(duration);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn read_posts(&mut self, limit: u64) -> BenchResult<bool> {
        let mut result = false;

        let duration = measured(|| {
            let posts = self.db.0.read()?.exec(
                QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .from("posts")
                            .limit(limit)
                            .where_()
                            .neighbor()
                            .query(),
                    )
                    .query(),
            )?;

            result = posts.result != 0;

            Ok(())
        })?;

        if result {
            self.times.push(duration);
        }

        Ok(result)
    }

    fn last_post(&mut self) -> BenchResult<Option<DbId>> {
        if let Some(post) = self
            .db
            .0
            .read()?
            .exec(
                QueryBuilder::search()
                    .depth_first()
                    .from("posts")
                    .limit(1)
                    .where_()
                    .neighbor()
                    .query(),
            )?
            .elements
            .first()
        {
            Ok(Some(post.id))
        } else {
            Ok(None)
        }
    }
}

impl<S: StorageData> Readers<S> {
    pub(crate) async fn join_and_report(
        &mut self,
        description: &str,
        threads: u64,
        per_thread: u64,
        per_action: u64,
        config: &Config,
    ) -> BenchResult<()> {
        let mut readers = vec![];

        for task in self.tasks.iter_mut() {
            readers.push(task.await?);
        }

        let end = if let Some(r) = readers.iter().max_by_key(|r| r.end) {
            r.end
        } else {
            Duration::default()
        };
        let times: Vec<Duration> = readers.into_iter().flat_map(|w| w.times).collect();

        utilities::report(
            description,
            threads,
            per_thread,
            per_action,
            times,
            end,
            config,
        );

        Ok(())
    }
}

impl ServerReader {
    fn new(api: AgdbApi<ReqwestClient>) -> Self {
        Self {
            api,
            end: Duration::default(),
            times: vec![],
        }
    }

    async fn read_comments(&mut self, limit: u64) -> BenchResult<bool> {
        if let Some(post_id) = self.last_post().await? {
            let duration = measured_async(async {
                let _ = self
                    .api
                    .db_exec(
                        BENCHMARK_USERNAME,
                        BENCHMARK_DATABASE,
                        &[QueryBuilder::select()
                            .ids(
                                QueryBuilder::search()
                                    .from(post_id)
                                    .limit(limit)
                                    .where_()
                                    .neighbor()
                                    .query(),
                            )
                            .query()
                            .into()],
                    )
                    .await?;
                Ok(())
            })
            .await?;

            self.times.push(duration);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn read_posts(&mut self, limit: u64) -> BenchResult<bool> {
        let mut result = false;

        let duration = measured_async(async {
            let posts = self
                .api
                .db_exec(
                    BENCHMARK_USERNAME,
                    BENCHMARK_DATABASE,
                    &[QueryBuilder::select()
                        .ids(
                            QueryBuilder::search()
                                .from("posts")
                                .limit(limit)
                                .where_()
                                .neighbor()
                                .query(),
                        )
                        .query()
                        .into()],
                )
                .await?
                .1;

            result = posts[0].result != 0;

            Ok(())
        })
        .await?;

        if result {
            self.times.push(duration);
        }

        Ok(result)
    }

    async fn last_post(&self) -> BenchResult<Option<DbId>> {
        let result = self
            .api
            .db_exec(
                BENCHMARK_USERNAME,
                BENCHMARK_DATABASE,
                &[QueryBuilder::search()
                    .depth_first()
                    .from("posts")
                    .limit(1)
                    .where_()
                    .neighbor()
                    .query()
                    .into()],
            )
            .await?
            .1;

        if let Some(post) = result.first().and_then(|query| query.elements.first()) {
            Ok(Some(post.id))
        } else {
            Ok(None)
        }
    }
}

impl ServerReaders {
    pub(crate) async fn join_and_report(
        &mut self,
        description: &str,
        threads: u64,
        per_thread: u64,
        per_action: u64,
        config: &Config,
    ) -> BenchResult<()> {
        let mut readers = vec![];

        for task in self.tasks.iter_mut() {
            readers.push(task.await??);
        }

        let end = if let Some(r) = readers.iter().max_by_key(|r| r.end) {
            r.end
        } else {
            Duration::default()
        };
        let times: Vec<Duration> = readers.into_iter().flat_map(|r| r.times).collect();

        utilities::report(
            description,
            threads,
            per_thread,
            per_action,
            times,
            end,
            config,
        );

        Ok(())
    }
}

pub(crate) fn start_post_readers<S: StorageData + Send + Sync + 'static>(
    db: &mut Database<S>,
    config: &Config,
) -> BenchResult<Readers<S>> {
    let start = Instant::now();
    let mut tasks = vec![];

    for i in 0..config.post_readers.count {
        let db = db.clone();
        let limit = config.post_readers.posts;
        let read_delay = Duration::from_millis(if config.post_readers.delay_ms == 0 {
            0
        } else {
            config.post_readers.delay_ms % (i + 1)
        });
        let reads = config.post_readers.reads_per_reader;

        let handle = tokio::spawn(async move {
            let mut reader = Reader::new(db);
            let mut read = 0;

            while read != reads {
                tokio::time::sleep(read_delay).await;

                if reader.read_posts(limit).unwrap_or(false) {
                    read += 1;
                }
            }

            reader.end = start.elapsed();
            reader
        });

        tasks.push(handle);
    }

    Ok(Readers { tasks })
}

pub(crate) fn start_comment_readers<S: StorageData + Send + Sync + 'static>(
    db: &mut Database<S>,
    config: &Config,
) -> BenchResult<Readers<S>> {
    let start = Instant::now();
    let mut tasks = vec![];

    for i in 0..config.comment_readers.count {
        let db = db.clone();
        let read_delay = Duration::from_millis(if config.comment_readers.delay_ms == 0 {
            0
        } else {
            config.comment_readers.delay_ms % (i + 1)
        });
        let reads = config.comment_readers.reads_per_reader;
        let limit = config.comment_readers.comments;

        let handle = tokio::spawn(async move {
            let mut reader = Reader::new(db);
            let mut read = 0;

            while read != reads {
                tokio::time::sleep(read_delay).await;

                if reader.read_comments(limit).unwrap_or(false) {
                    read += 1;
                }
            }

            reader.end = start.elapsed();
            reader
        });

        tasks.push(handle);
    }

    Ok(Readers { tasks })
}

pub(crate) async fn start_post_readers_server(
    db: &ServerDatabase,
    config: &Config,
) -> BenchResult<ServerReaders> {
    let start = Instant::now();
    let mut tasks = vec![];
    let address = db.address().to_string();

    for i in 0..config.post_readers.count {
        let address = address.clone();
        let limit = config.post_readers.posts;
        let read_delay = Duration::from_millis(if config.post_readers.delay_ms == 0 {
            0
        } else {
            config.post_readers.delay_ms % (i + 1)
        });
        let reads = config.post_readers.reads_per_reader;
        let username = post_reader_username(i);

        let handle = tokio::spawn(async move {
            let mut api = AgdbApi::new(ReqwestClient::new(), &address);
            let password = benchmark_password(&username);
            api.user_login(&username, &password).await?;

            let mut reader = ServerReader::new(api);
            let mut read = 0;

            while read != reads {
                tokio::time::sleep(read_delay).await;

                if reader.read_posts(limit).await.unwrap_or(false) {
                    read += 1;
                } else {
                    println!("WTF: {read} out post {reads}");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }

            reader.end = start.elapsed();
            Ok(reader)
        });

        tasks.push(handle);
    }

    Ok(ServerReaders { tasks })
}

pub(crate) async fn start_comment_readers_server(
    db: &ServerDatabase,
    config: &Config,
) -> BenchResult<ServerReaders> {
    let start = Instant::now();
    let mut tasks = vec![];
    let address = db.address().to_string();

    for i in 0..config.comment_readers.count {
        let address = address.clone();
        let read_delay = Duration::from_millis(if config.comment_readers.delay_ms == 0 {
            0
        } else {
            config.comment_readers.delay_ms % (i + 1)
        });
        let reads = config.comment_readers.reads_per_reader;
        let limit = config.comment_readers.comments;
        let username = comment_reader_username(i);

        let handle = tokio::spawn(async move {
            let mut api = AgdbApi::new(ReqwestClient::new(), &address);
            let password = benchmark_password(&username);
            api.user_login(&username, &password).await?;

            let mut reader = ServerReader::new(api);
            let mut read = 0;

            while read != reads {
                tokio::time::sleep(read_delay).await;

                if reader.read_comments(limit).await.unwrap_or(false) {
                    read += 1;
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }

            reader.end = start.elapsed();
            Ok(reader)
        });

        tasks.push(handle);
    }

    Ok(ServerReaders { tasks })
}
