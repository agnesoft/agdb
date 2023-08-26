use crate::database::Database;
use bench_result::BenchResult;
use num_format::Locale;
use std::time::Duration;

mod bench_error;
mod bench_result;
mod database;
mod users;
mod utilities;
mod writers;

pub(crate) const BENCH_DATABASE: &str = "db.agdb";
pub(crate) const LOCALE: Locale = Locale::cs;
pub(crate) const PADDING: usize = 30;
pub(crate) const CELL_PADDING: usize = 5;

pub(crate) const USER_COUNT: u32 = 1000;

pub(crate) const POST_WRITER_COUNT: u32 = 100;
pub(crate) const POSTS_PER_WRITER: u32 = 100;
pub(crate) const POST_TITLE: &str = "Title of the testing post";
pub(crate) const POST_BODY: &str = "Body of the testing post should be longer than the title";

pub(crate) const COMMENT_WRITER_COUNT: u32 = 100;
pub(crate) const COMMENTS_PER_WRITER: u32 = 100;
pub(crate) const COMMENT_BODY: &str = "This is a testing comment of a post.";

pub(crate) const WRITE_DELAY: Duration = Duration::from_millis(100);

#[tokio::main]
async fn main() -> BenchResult<()> {
    println!("Running agdb benchmark\n\n");
    utilities::print_header();

    let mut db = Database::new()?;
    users::setup_users(&mut db)?;
    let mut posters = writers::start_post_writers(&mut db)?;
    let mut commenters = writers::start_comment_writers(&mut db)?;

    posters
        .join_and_report(&format!(
            "{POST_WRITER_COUNT} posters * {POSTS_PER_WRITER} posts"
        ))
        .await?;
    commenters
        .join_and_report(&format!(
            "{POST_WRITER_COUNT} commenters * {COMMENTS_PER_WRITER} comments"
        ))
        .await?;

    println!("---");
    db.stat()
}
