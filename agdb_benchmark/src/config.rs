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
pub enum DbType {
    File,
    FileMapped,
    InMemory,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct RetryConfig {
    pub(crate) base_delay_ms: u64,
    pub(crate) max_delay_ms: u64,
    pub(crate) max_consecutive_failures: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) allow_invalid_certs: bool,
    pub(crate) retry: RetryConfig,
    pub(crate) memory_poll_interval_ms: u64,
    pub(crate) memory_end_delay_ms: u64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct TargetsConfig {
    pub(crate) embedded: bool,
    pub(crate) local_server: Option<String>,
    pub(crate) remote_server: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) db_name: String,
    pub(crate) targets: TargetsConfig,
    pub(crate) db_type: DbType,
    pub(crate) server: ServerConfig,
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
            targets: TargetsConfig::default(),
            db_type: DbType::FileMapped,
            server: ServerConfig::default(),
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

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 100,
            max_delay_ms: 5000,
            max_consecutive_failures: 10,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            allow_invalid_certs: false,
            retry: RetryConfig::default(),
            memory_poll_interval_ms: 2000,
            memory_end_delay_ms: 15_000,
        }
    }
}

impl Default for TargetsConfig {
    fn default() -> Self {
        Self {
            embedded: true,
            local_server: None,
            remote_server: None,
        }
    }
}
