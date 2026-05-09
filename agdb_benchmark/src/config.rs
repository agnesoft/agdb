use crate::bench_result::BenchResult;
use num_format::Locale;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::path::Path;

pub(crate) const BENCH_CONFIG_FILE: &str = "agdb_benchmark.yaml";

#[derive(Serialize, Deserialize)]
pub(crate) struct PostWriters {
    pub(crate) count: u64,
    pub(crate) posts: u64,
    pub(crate) delay_ms: u64,
    pub(crate) title: String,
    pub(crate) body: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CommentWriters {
    pub(crate) count: u64,
    pub(crate) comments: u64,
    pub(crate) delay_ms: u64,
    pub(crate) body: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostReaders {
    pub(crate) count: u64,
    pub(crate) posts: u64,
    pub(crate) reads_per_reader: u64,
    pub(crate) delay_ms: u64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CommentReaders {
    pub(crate) count: u64,
    pub(crate) comments: u64,
    pub(crate) reads_per_reader: u64,
    pub(crate) delay_ms: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkMode {
    Embedded,
    Server { address: String },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbType {
    File,
    FileMapped,
    InMemory,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) db_name: String,
    pub(crate) mode: BenchmarkMode,
    pub(crate) db_type: DbType,
    pub(crate) locale: Locale,
    pub(crate) padding: u64,
    pub(crate) cell_padding: u64,
    pub(crate) posters: PostWriters,
    pub(crate) commenters: CommentWriters,
    pub(crate) post_readers: PostReaders,
    pub(crate) comment_readers: CommentReaders,
}

impl Config {
    pub(crate) fn new(config_file: &str) -> BenchResult<Self> {
        let path = Path::new(config_file);

        if !path.exists() {
            println!("Using default config (saved to '{config_file}')");
            let config = Self::default();
            let file = File::create(path)?;
            serde_yaml::to_writer(file, &config)?;
            Ok(config)
        } else {
            println!("Using existing config from '{config_file}'");
            let file = File::open(path)?;
            Ok(serde_yaml::from_reader(file)?)
        }
    }

    pub(crate) fn user_count(&self) -> u64 {
        self.posters.count + self.commenters.count
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_name: "agdb_benchmark.agdb".to_string(),
            mode: BenchmarkMode::Embedded,
            db_type: DbType::FileMapped,
            locale: Locale::cs,
            padding: 20,
            cell_padding: 10,
            posters: PostWriters {
                count: 10,
                posts: 100,
                delay_ms: 100,
                title: "Title of the testing post".to_string(),
                body: "Body of the testing post should be longer than the title".to_string(),
            },
            commenters: CommentWriters {
                count: 10,
                comments: 100,
                delay_ms: 100,
                body: "This is a testing comment of a post.".to_string(),
            },
            post_readers: PostReaders {
                count: 100,
                posts: 10,
                delay_ms: 100,
                reads_per_reader: 100,
            },
            comment_readers: CommentReaders {
                count: 100,
                comments: 10,
                delay_ms: 100,
                reads_per_reader: 100,
            },
        }
    }
}
