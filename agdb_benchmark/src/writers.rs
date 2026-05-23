use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::database::BENCHMARK_DATABASE;
use crate::database::BENCHMARK_USERNAME;
use crate::database::Database;
use crate::database::ServerDatabase;
use crate::database::bench_api;
use crate::queries::BenchComment;
use crate::queries::BenchPost;
use crate::results::TimingStats;
use crate::retry::RetryState;
use crate::users::benchmark_password;
use crate::users::comment_writer_username;
use crate::users::post_writer_username;
use crate::utilities::measured;
use crate::utilities::measured_async;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::StorageData;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use std::time::Duration;
use std::time::Instant;
use tokio::task::JoinHandle;

struct Writer<S: StorageData> {
    id: DbId,
    db: Database<S>,
    end: Duration,
    pub(crate) times: Vec<Duration>,
}

pub(crate) struct Writers<S: StorageData> {
    tasks: Vec<JoinHandle<Writer<S>>>,
}

struct ServerWriter {
    id: DbId,
    api: AgdbApi<ReqwestClient>,
    end: Duration,
    pub(crate) times: Vec<Duration>,
}

pub(crate) struct ServerWriters {
    tasks: Vec<JoinHandle<BenchResult<ServerWriter>>>,
}

impl<S: StorageData> Writer<S> {
    pub(crate) fn new(id: DbId, db: Database<S>) -> Self {
        Self {
            id,
            db,
            end: Duration::default(),
            times: vec![],
        }
    }

    fn write_post(&mut self, title: &str, body: &str) -> BenchResult<()> {
        let duration = measured(|| {
            self.db.0.write()?.transaction_mut(|t| -> BenchResult<()> {
                let id = t
                    .exec_mut(
                        QueryBuilder::insert()
                            .nodes()
                            .values(BenchPost {
                                title: title.to_string(),
                                body: body.to_string(),
                            })
                            .query(),
                    )?
                    .elements[0]
                    .id;

                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from([QueryId::from("posts"), self.id.into()])
                        .to(id)
                        .values([[].as_slice(), &[("authored", 1).into()]])
                        .query(),
                )?;

                Ok(())
            })?;

            Ok(())
        })?;

        self.times.push(duration);

        Ok(())
    }

    fn write_comment(&mut self, body: &str) -> BenchResult<bool> {
        if let Some(post_id) = self.last_post()? {
            let duration = measured(|| {
                self.db.0.write()?.transaction_mut(|t| -> BenchResult<()> {
                    let id = t
                        .exec_mut(
                            QueryBuilder::insert()
                                .nodes()
                                .values(BenchComment {
                                    body: body.to_string(),
                                })
                                .query(),
                        )?
                        .elements[0]
                        .id;

                    t.exec_mut(
                        QueryBuilder::insert()
                            .edges()
                            .from([post_id, self.id])
                            .to(id)
                            .values([[].as_slice(), &[("commented", 1).into()]])
                            .query(),
                    )?;

                    Ok(())
                })?;

                Ok(())
            })?;

            self.times.push(duration);
            return Ok(true);
        }

        Ok(false)
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

impl<S: StorageData> Writers<S> {
    pub(crate) async fn join(&mut self) -> BenchResult<TimingStats> {
        let mut writers = vec![];

        for task in self.tasks.iter_mut() {
            writers.push(task.await?);
        }

        let times: Vec<Duration> = writers.into_iter().flat_map(|w| w.times).collect();

        Ok(TimingStats::from_times(&times))
    }
}

impl ServerWriter {
    fn new(id: DbId, api: AgdbApi<ReqwestClient>) -> Self {
        Self {
            id,
            api,
            end: Duration::default(),
            times: vec![],
        }
    }

    async fn write_post(&mut self, title: &str, body: &str) -> BenchResult<()> {
        let duration = measured_async(async {
            self.api
                .db_exec_mut(
                    BENCHMARK_USERNAME,
                    BENCHMARK_DATABASE,
                    &[
                        QueryBuilder::insert()
                            .nodes()
                            .values(BenchPost {
                                title: title.to_string(),
                                body: body.to_string(),
                            })
                            .query()
                            .into(),
                        QueryBuilder::insert()
                            .edges()
                            .from([QueryId::from("posts"), self.id.into()])
                            .to(":0")
                            .values([[].as_slice(), &[("authored", 1).into()]])
                            .query()
                            .into(),
                    ],
                )
                .await?;

            Ok(())
        })
        .await?;

        self.times.push(duration);

        Ok(())
    }

    async fn write_comment(&mut self, body: &str) -> BenchResult<bool> {
        if let Some(post_id) = self.last_post().await? {
            let duration = measured_async(async {
                self.api
                    .db_exec_mut(
                        BENCHMARK_USERNAME,
                        BENCHMARK_DATABASE,
                        &[
                            QueryBuilder::insert()
                                .nodes()
                                .values(BenchComment {
                                    body: body.to_string(),
                                })
                                .query()
                                .into(),
                            QueryBuilder::insert()
                                .edges()
                                .from([post_id, self.id])
                                .to(":0")
                                .values([[].as_slice(), &[("commented", 1).into()]])
                                .query()
                                .into(),
                        ],
                    )
                    .await?;

                Ok(())
            })
            .await?;

            self.times.push(duration);
            return Ok(true);
        }

        Ok(false)
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

impl ServerWriters {
    pub(crate) async fn join(&mut self) -> BenchResult<TimingStats> {
        let mut writers = vec![];

        for task in self.tasks.iter_mut() {
            writers.push(task.await??);
        }

        let times: Vec<Duration> = writers.into_iter().flat_map(|w| w.times).collect();

        Ok(TimingStats::from_times(&times))
    }
}

pub(crate) fn start_post_writers<S: StorageData + Send + Sync + 'static>(
    db: &mut Database<S>,
    config: &Config,
) -> BenchResult<Writers<S>> {
    let start = Instant::now();
    let tasks =
        db.0.read()?
            .exec(
                QueryBuilder::search()
                    .from("users")
                    .limit(config.posters.count)
                    .where_()
                    .neighbor()
                    .query(),
            )?
            .elements
            .into_iter()
            .map(|e| {
                let id = e.id;
                let db = db.clone();
                let write_delay = Duration::from_millis(if config.posters.delay_ms == 0 {
                    0
                } else {
                    config.posters.delay_ms % id.0 as u64
                });
                let posts = config.posters.posts;
                let title = config.posters.title.to_string();
                let body = config.posters.body.to_string();

                tokio::task::spawn(async move {
                    let mut writer = Writer::new(id, db);

                    for i in 0..posts {
                        let _ = writer.write_post(&format!("{title} {i}"), &format!("{body} {i}"));
                        tokio::time::sleep(write_delay).await;
                    }

                    writer.end = start.elapsed();
                    writer
                })
            })
            .collect::<Vec<JoinHandle<Writer<S>>>>();

    Ok(Writers { tasks })
}

pub(crate) fn start_comment_writers<S: StorageData + Send + Sync + 'static>(
    db: &mut Database<S>,
    config: &Config,
) -> BenchResult<Writers<S>> {
    let start = Instant::now();
    let tasks =
        db.0.read()?
            .exec(
                QueryBuilder::search()
                    .from("users")
                    .offset(config.posters.count)
                    .limit(config.commenters.count)
                    .where_()
                    .neighbor()
                    .query(),
            )?
            .elements
            .into_iter()
            .map(|e| {
                let id = e.id;
                let db = db.clone();
                let write_delay = Duration::from_millis(if config.commenters.delay_ms == 0 {
                    0
                } else {
                    config.commenters.delay_ms % id.0 as u64
                });
                let comments = config.commenters.comments;
                let body = config.commenters.body.to_string();

                tokio::task::spawn(async move {
                    let mut writer = Writer::new(id, db);
                    let mut written = 0;

                    while written != comments {
                        if writer
                            .write_comment(&format!("{body} {written}"))
                            .unwrap_or(false)
                        {
                            written += 1;
                        }

                        tokio::time::sleep(write_delay).await;
                    }

                    writer.end = start.elapsed();
                    writer
                })
            })
            .collect::<Vec<JoinHandle<Writer<S>>>>();

    Ok(Writers { tasks })
}

pub(crate) async fn start_post_writers_server(
    db: &ServerDatabase,
    config: &Config,
) -> BenchResult<ServerWriters> {
    let start = Instant::now();
    let users = db
        .exec(&[QueryBuilder::search()
            .from("users")
            .limit(config.posters.count)
            .where_()
            .neighbor()
            .query()
            .into()])
        .await?;
    let address = db.address().to_string();
    let client = db.client().clone();
    let retry = config.server.retry;

    let tasks = users[0]
        .elements
        .iter()
        .enumerate()
        .map(|(index, e)| {
            let id = e.id;
            let address = address.clone();
            let client = client.clone();
            let write_delay = Duration::from_millis(if config.posters.delay_ms == 0 {
                0
            } else {
                config.posters.delay_ms % id.0 as u64
            });
            let posts = config.posters.posts;
            let title = config.posters.title.to_string();
            let body = config.posters.body.to_string();
            let username = post_writer_username(index as u64);

            tokio::task::spawn(async move {
                let mut api = bench_api(client, &address);
                let password = benchmark_password(&username);
                api.user_login(&username, &password).await?;
                let mut writer = ServerWriter::new(id, api);
                let mut retry_state = RetryState::new();
                let mut written = 0;

                while written != posts {
                    match writer
                        .write_post(&format!("{title} {written}"), &format!("{body} {written}"))
                        .await
                    {
                        Ok(_) => {
                            retry_state.reset();
                            written += 1;
                        }
                        Err(error) => {
                            retry_state
                                .on_failure(&retry, "post writer", &error.description)
                                .await?;
                        }
                    }
                    tokio::time::sleep(write_delay).await;
                }

                writer.end = start.elapsed();
                Ok(writer)
            })
        })
        .collect::<Vec<JoinHandle<BenchResult<ServerWriter>>>>();

    Ok(ServerWriters { tasks })
}

pub(crate) async fn start_comment_writers_server(
    db: &ServerDatabase,
    config: &Config,
) -> BenchResult<ServerWriters> {
    let start = Instant::now();
    let users = db
        .exec(&[QueryBuilder::search()
            .from("users")
            .offset(config.posters.count)
            .limit(config.commenters.count)
            .where_()
            .neighbor()
            .query()
            .into()])
        .await?;
    let address = db.address().to_string();
    let client = db.client().clone();
    let retry = config.server.retry;

    let tasks = users[0]
        .elements
        .iter()
        .enumerate()
        .map(|(index, e)| {
            let id = e.id;
            let address = address.clone();
            let client = client.clone();
            let write_delay = Duration::from_millis(if config.commenters.delay_ms == 0 {
                0
            } else {
                config.commenters.delay_ms % id.0 as u64
            });
            let comments = config.commenters.comments;
            let body = config.commenters.body.to_string();
            let username = comment_writer_username(index as u64);

            tokio::task::spawn(async move {
                let mut api = bench_api(client, &address);
                let password = benchmark_password(&username);
                api.user_login(&username, &password).await?;
                let mut writer = ServerWriter::new(id, api);
                let mut written = 0;
                let mut retry_state = RetryState::new();

                while written != comments {
                    match writer.write_comment(&format!("{body} {written}")).await {
                        Ok(true) => {
                            retry_state.reset();
                            written += 1;
                        }
                        Ok(false) => {}
                        Err(error) => {
                            retry_state
                                .on_failure(&retry, "comment writer", &error.description)
                                .await?;
                        }
                    }

                    tokio::time::sleep(write_delay).await;
                }

                writer.end = start.elapsed();
                Ok(writer)
            })
        })
        .collect::<Vec<JoinHandle<BenchResult<ServerWriter>>>>();

    Ok(ServerWriters { tasks })
}
