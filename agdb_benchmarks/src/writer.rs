use crate::bench_result::BenchResult;
use crate::database::Database;
use crate::utilities::measured;
use crate::COMMENT_BODY;
use crate::POST_BODY;
use crate::POST_TITLE;
use crate::WRITER_COUNT;
use crate::WRITE_DELAY;
use crate::WRITE_POSTS;
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

pub(crate) struct Writer {
    id: DbId,
    db: Database,
    pub(crate) post_times: Vec<Duration>,
    pub(crate) comment_times: Vec<Duration>,
}

impl Writer {
    pub(crate) fn new(id: DbId, db: Database) -> Self {
        Self {
            id,
            db,
            post_times: vec![],
            comment_times: vec![],
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

        self.post_times.push(duration);

        Ok(())
    }

    pub(crate) fn write_comment(&mut self) -> BenchResult<()> {
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

            self.post_times.push(duration);
        }

        Ok(())
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

pub(crate) fn start_post_writers(db: &mut Database) -> BenchResult<Vec<JoinHandle<Writer>>> {
    let tasks =
        db.0.read()?
            .exec(
                &QueryBuilder::search()
                    .from("users")
                    .limit(WRITER_COUNT.into())
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

                    for _ in 0..WRITE_POSTS {
                        let _ = writer.write_post();
                        tokio::time::sleep(WRITE_DELAY).await;
                    }

                    writer
                })
            })
            .collect::<Vec<JoinHandle<Writer>>>();

    Ok(tasks)
}
