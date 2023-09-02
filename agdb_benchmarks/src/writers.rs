use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::database::Database;
use crate::utilities;
use crate::utilities::measured;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::UserValue;
use std::time::Duration;
use tokio::task::JoinHandle;

#[derive(UserValue)]
struct Post {
    title: String,
    body: String,
}

#[derive(UserValue)]
struct Comment {
    body: String,
}

struct Writer {
    id: DbId,
    db: Database,
    pub(crate) times: Vec<Duration>,
}

pub(crate) struct Writers(Vec<JoinHandle<Writer>>);

impl Writer {
    pub(crate) fn new(id: DbId, db: Database) -> Self {
        Self {
            id,
            db,
            times: vec![],
        }
    }

    fn write_post(&mut self, title: &str, body: &str) -> BenchResult<()> {
        let duration = measured(|| {
            self.db.0.write()?.transaction_mut(|t| -> BenchResult<()> {
                let id = t
                    .exec_mut(
                        &QueryBuilder::insert()
                            .nodes()
                            .values(&Post {
                                title: title.to_string(),
                                body: body.to_string(),
                            })
                            .query(),
                    )?
                    .elements[0]
                    .id;

                t.exec_mut(
                    &QueryBuilder::insert()
                        .edges()
                        .from(vec![QueryId::from("posts"), self.id.into()])
                        .to(id)
                        .values(vec![vec![], vec![("authored", 1).into()]])
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
                            &QueryBuilder::insert()
                                .nodes()
                                .values(&Comment {
                                    body: body.to_string(),
                                })
                                .query(),
                        )?
                        .elements[0]
                        .id;

                    t.exec_mut(
                        &QueryBuilder::insert()
                            .edges()
                            .from(vec![post_id, self.id])
                            .to(id)
                            .values(vec![vec![], vec![("commented", 1).into()]])
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
                &QueryBuilder::search()
                    .depth_first()
                    .from("posts")
                    .limit(1)
                    .where_()
                    .distance(agdb::CountComparison::Equal(2))
                    .query(),
            )?
            .elements
            .get(0)
        {
            Ok(Some(post.id))
        } else {
            Ok(None)
        }
    }
}

impl Writers {
    pub(crate) async fn join_and_report(
        &mut self,
        description: &str,
        threads: u64,
        per_thread: u64,
        per_action: u64,
        config: &Config,
    ) -> BenchResult<()> {
        let mut writers = vec![];

        for task in self.0.iter_mut() {
            writers.push(task.await?);
        }

        let times: Vec<Duration> = writers.into_iter().flat_map(|w| w.times).collect();

        utilities::report(description, threads, per_thread, per_action, times, config);

        Ok(())
    }
}

pub(crate) fn start_post_writers(db: &mut Database, config: &Config) -> BenchResult<Writers> {
    let tasks =
        db.0.read()?
            .exec(
                &QueryBuilder::search()
                    .from("users")
                    .limit(config.posters.count)
                    .where_()
                    .distance(agdb::CountComparison::Equal(2))
                    .query(),
            )?
            .elements
            .into_iter()
            .map(|e| {
                let id = e.id;
                let db = db.clone();
                let write_delay = Duration::from_millis(config.posters.delay_ms % id.0 as u64);
                let posts = config.posters.posts;
                let title = config.posters.title.to_string();
                let body = config.posters.body.to_string();

                tokio::task::spawn(async move {
                    let mut writer = Writer::new(id, db);

                    for _ in 0..posts {
                        let _ = writer.write_post(&title, &body);
                        tokio::time::sleep(write_delay).await;
                    }

                    writer
                })
            })
            .collect::<Vec<JoinHandle<Writer>>>();

    Ok(Writers(tasks))
}

pub(crate) fn start_comment_writers(db: &mut Database, config: &Config) -> BenchResult<Writers> {
    let tasks =
        db.0.read()?
            .exec(
                &QueryBuilder::search()
                    .from("users")
                    .offset(config.posters.count)
                    .limit(config.commenters.count)
                    .where_()
                    .distance(agdb::CountComparison::Equal(2))
                    .query(),
            )?
            .elements
            .into_iter()
            .map(|e| {
                let id = e.id;
                let db = db.clone();
                let write_delay = Duration::from_millis(config.commenters.delay_ms % id.0 as u64);
                let comments = config.commenters.comments;
                let body = config.commenters.body.to_string();

                tokio::task::spawn(async move {
                    let mut writer = Writer::new(id, db);
                    let mut written = 0;

                    while written != comments {
                        if writer.write_comment(&body).unwrap_or(false) {
                            written += 1;
                        }

                        tokio::time::sleep(write_delay).await;
                    }

                    writer
                })
            })
            .collect::<Vec<JoinHandle<Writer>>>();

    Ok(Writers(tasks))
}
