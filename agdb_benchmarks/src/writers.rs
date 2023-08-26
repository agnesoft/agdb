use crate::bench_result::BenchResult;
use crate::database::Database;
use crate::utilities;
use crate::utilities::measured;
use crate::COMMENTS_PER_WRITER;
use crate::COMMENT_BODY;
use crate::COMMENT_WRITER_COUNT;
use crate::POSTS_PER_WRITER;
use crate::POST_BODY;
use crate::POST_TITLE;
use crate::POST_WRITER_COUNT;
use crate::WRITE_DELAY;
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

    pub(crate) fn write_post(&mut self) -> BenchResult<()> {
        let duration = measured(|| {
            self.db.0.write()?.transaction_mut(|t| -> BenchResult<()> {
                let id = t
                    .exec_mut(
                        &QueryBuilder::insert()
                            .nodes()
                            .values(&Post {
                                title: POST_TITLE.to_string(),
                                body: POST_BODY.to_string(),
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

    pub(crate) fn write_comment(&mut self) -> BenchResult<bool> {
        if let Some(post_id) = self.last_post()? {
            let duration = measured(|| {
                self.db.0.write()?.transaction_mut(|t| -> BenchResult<()> {
                    let id = t
                        .exec_mut(
                            &QueryBuilder::insert()
                                .nodes()
                                .values(&Comment {
                                    body: COMMENT_BODY.to_string(),
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
    pub(crate) async fn join_and_report(&mut self, description: &str) -> BenchResult<()> {
        let mut writers = vec![];

        for task in self.0.iter_mut() {
            writers.push(task.await?);
        }

        let times: Vec<Duration> = writers.into_iter().flat_map(|w| w.times).collect();

        utilities::report(description, times);

        Ok(())
    }
}

pub(crate) fn start_post_writers(db: &mut Database) -> BenchResult<Writers> {
    let tasks =
        db.0.read()?
            .exec(
                &QueryBuilder::search()
                    .from("users")
                    .limit(POST_WRITER_COUNT.into())
                    .where_()
                    .distance(agdb::CountComparison::Equal(2))
                    .query(),
            )?
            .elements
            .into_iter()
            .map(|e| {
                let id = e.id;
                let db = db.clone();

                tokio::task::spawn(async move {
                    let mut writer = Writer::new(id, db);

                    for _ in 0..POSTS_PER_WRITER {
                        let _ = writer.write_post();
                        tokio::time::sleep(WRITE_DELAY).await;
                    }

                    writer
                })
            })
            .collect::<Vec<JoinHandle<Writer>>>();

    Ok(Writers(tasks))
}

pub(crate) fn start_comment_writers(db: &mut Database) -> BenchResult<Writers> {
    let tasks =
        db.0.read()?
            .exec(
                &QueryBuilder::search()
                    .from("users")
                    .offset(POST_WRITER_COUNT.into())
                    .limit(COMMENT_WRITER_COUNT.into())
                    .where_()
                    .distance(agdb::CountComparison::Equal(2))
                    .query(),
            )?
            .elements
            .into_iter()
            .map(|e| {
                let id = e.id;
                let db = db.clone();

                tokio::task::spawn(async move {
                    let mut writer = Writer::new(id, db);
                    let mut written = 0;

                    while written != COMMENTS_PER_WRITER {
                        if writer.write_comment().unwrap_or(false) {
                            written += 1;
                        }

                        tokio::time::sleep(WRITE_DELAY).await;
                    }

                    writer
                })
            })
            .collect::<Vec<JoinHandle<Writer>>>();

    Ok(Writers(tasks))
}
