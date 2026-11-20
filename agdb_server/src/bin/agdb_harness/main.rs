//! Runs a command against a shared agdb_server and a shared 3-node cluster:
//! `agdb_harness <command> [args...]`. The command gets their addresses and data
//! directories in `AGDB_SERVER_ADDRESS`, `AGDB_SERVER_DATA_DIR`,
//! `AGDB_CLUSTER_ADDRESSES` and `AGDB_CLUSTER_DATA_DIRS` (comma separated, the
//! leader first). The `agdb_server` binary must be built next to the harness.

mod cluster;
mod server;
mod test_error;

use agdb_api::config_impl::DEFAULT_CLUSTER_MAX_LOG_ENTRIES;
use server::HOST;
use server::TestServerImpl;
use std::process::ExitCode;
use test_error::TestError;
use tokio::process::Command;

const DEFAULT_PORT: u16 = 3000;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let Some((command, args)) = args.split_first() else {
        eprintln!("Usage: agdb_harness <command> [args...]");
        return ExitCode::from(2);
    };

    match run(command, args).await {
        Ok(code) => code,
        Err(e) => {
            eprintln!("agdb_harness: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(command: &str, args: &[String]) -> Result<ExitCode, TestError> {
    let mut config = server::default_config();
    config.bind = format!("{HOST}:{DEFAULT_PORT}");
    config.address = format!("http://{}", config.bind);

    if server::api_for_test(&config.address).status().await.is_ok() {
        return Err(TestError::new(format!(
            "{} is already in use",
            config.address
        )));
    }

    let server = TestServerImpl::with_config(config).await?;
    let cluster = cluster::start_cluster(3, false, DEFAULT_CLUSTER_MAX_LOG_ENTRIES).await?;
    let addresses: Vec<&str> = cluster.iter().map(|node| node.address.as_str()).collect();
    let data_dirs: Vec<&str> = cluster.iter().map(|node| node.data_dir.as_str()).collect();

    let status = shell(command, args)
        .env("AGDB_SERVER_ADDRESS", &server.address)
        .env("AGDB_SERVER_DATA_DIR", &server.data_dir)
        .env("AGDB_CLUSTER_ADDRESSES", addresses.join(","))
        .env("AGDB_CLUSTER_DATA_DIRS", data_dirs.join(","))
        .status()
        .await?;

    Ok(status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or(ExitCode::FAILURE, ExitCode::from))
}

/// Windows resolves only `.exe` files without the shell, the package manager shims are `.cmd`.
fn shell(command: &str, args: &[String]) -> Command {
    if cfg!(windows) {
        let mut shell = Command::new("cmd");
        shell.arg("/C").arg(command).args(args);
        shell
    } else {
        let mut shell = Command::new(command);
        shell.args(args);
        shell
    }
}
