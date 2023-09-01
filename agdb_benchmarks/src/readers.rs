use crate::bench_result::BenchResult;
use crate::database::Database;
use crate::utilities;
use crate::utilities::measured;
use crate::COMMENTS_PER_READ;
use crate::COMMENTS_READERS;
use crate::POSTS_PER_READ;
use crate::POST_READERS;
use crate::READS_PER_COMMENTS_READER;
use crate::READS_PER_POST_READER;
use crate::READ_DELAY;
use agdb::DbId;
use agdb::QueryBuilder;
use std::time::Duration;
use tokio::task::JoinHandle;

struct Reader {
    db: Database,
    pub(crate) times: Vec<Duration>,
}

pub(crate) struct Readers(Vec<JoinHandle<Reader>>);

impl Reader {
    pub(crate) fn new(db: Database) -> Self {
        Self { db, times: vec![] }
    }

    fn read_comments(&mut self) -> BenchResult<bool> {
        if let Some(post_id) = self.last_post()? {
            let duration = measured(|| {
                let _comments = self.db.0.read()?.exec(
                    &QueryBuilder::select()
                        .ids(
                            QueryBuilder::search()
                                .from(post_id)
                                .limit(COMMENTS_PER_READ.into())
                                .where_()
                                .distance(agdb::CountComparison::Equal(2))
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

    fn read_posts(&mut self) -> BenchResult<bool> {
        let mut result = false;

        let duration = measured(|| {
            let posts = self.db.0.read()?.exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .from("posts")
                            .limit(POSTS_PER_READ.into())
                            .where_()
                            .distance(agdb::CountComparison::Equal(2))
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

impl Readers {
    pub(crate) async fn join_and_report(&mut self, description: &str) -> BenchResult<()> {
        let mut readers = vec![];

        for task in self.0.iter_mut() {
            readers.push(task.await?);
        }

        let times: Vec<Duration> = readers.into_iter().flat_map(|w| w.times).collect();

        utilities::report(description, times);

        Ok(())
    }
}

pub(crate) fn start_post_readers(db: &mut Database) -> BenchResult<Readers> {
    let mut tasks = vec![];

    for i in 0..POST_READERS {
        let db = db.clone();

        let handle = tokio::spawn(async move {
            let mut reader = Reader::new(db);
            let mut read = 0;
            let read_delay = Duration::from_millis(READ_DELAY.as_millis() as u64 % (i + 1) as u64);

            while read != READS_PER_POST_READER {
                tokio::time::sleep(read_delay).await;

                if reader.read_posts().unwrap_or(false) {
                    read += 1;
                }
            }

            reader
        });

        tasks.push(handle);
    }

    Ok(Readers(tasks))
}

pub(crate) fn start_comment_readers(db: &mut Database) -> BenchResult<Readers> {
    let mut tasks = vec![];

    for i in 0..COMMENTS_READERS {
        let db = db.clone();

        let handle = tokio::spawn(async move {
            let mut reader = Reader::new(db);
            let mut read = 0;
            let read_delay = Duration::from_millis(READ_DELAY.as_millis() as u64 % (i + 1) as u64);

            while read != READS_PER_COMMENTS_READER {
                tokio::time::sleep(read_delay).await;

                if reader.read_comments().unwrap_or(false) {
                    read += 1;
                }
            }

            reader
        });

        tasks.push(handle);
    }

    Ok(Readers(tasks))
}
