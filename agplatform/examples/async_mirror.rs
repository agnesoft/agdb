//! An async program mirroring the archives listed by an index: each one is
//! downloaded to disk and unpacked concurrently in its own tokio task.
//!
//! It uses the whole platform (env, fs, request and process) from tasks
//! spawned on the runtime, which need it shared in an `Arc`, every platform
//! is `Send + Sync`. The tests spawn the same tasks with a `TestPlatform`, so
//! the whole program runs in memory: no network, no disk and no `tar`.
//!
//! ```bash
//! MIRROR_DIR=/tmp/mirror cargo run --example async_mirror --all-features -- http://localhost:8080/index
//! ```

use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use agplatform::CommandExt;
use agplatform::Env;
use agplatform::Fs;
use agplatform::HttpRequest;
use agplatform::Platform;
use agplatform::Request;
use agplatform::RequestBuilderExt;
use agplatform::reqwest::Client;
use agplatform::reqwest::Url;
use tokio::task::JoinSet;

/// Returns the directory to mirror to, `MIRROR_DIR` or `mirror` in the
/// temporary directory.
fn mirror_dir(platform: &impl Platform) -> PathBuf {
    let env = platform.env();
    env.var("MIRROR_DIR")
        .map_or_else(|| env.tmp_dir().join("mirror"), PathBuf::from)
}

async fn mirror_archive(
    platform: &impl Platform,
    client: Client,
    url: Url,
    dir: &Path,
) -> agplatform::Result<PathBuf> {
    let Some(name) = url.path_segments().and_then(Iterator::last) else {
        return Err(agplatform::Error::request(format!("{url}: no file name")));
    };
    let archive = dir.join(name);
    let mut file = platform.fs().open_write(&archive)?;

    let mut interceptors = platform.request().interceptors();
    interceptors.insert(0, HttpRequest::retry(3, Duration::from_secs(1)));

    client
        .get(url.clone())
        .interceptors(interceptors)
        .send_async_with_sink(platform.request(), &mut |(), chunk: &[u8]| {
            file.write_all(chunk)
                .map_err(|error| agplatform::Error::fs(format!("{}: {error}", archive.display())))
        })
        .await?;
    drop(file);

    Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(dir)
        .timeout(Duration::from_secs(300))
        .run_async(platform.process())
        .await?;

    Ok(archive)
}

/// Mirrors the archives listed one per line by the index at `index_url`
/// to `dir`, all concurrently, and returns their paths. Stops at the first
/// failure, cancelling the other downloads and unpacking.
async fn mirror<P: Platform + 'static>(
    platform: &Arc<P>,
    index_url: &str,
    dir: PathBuf,
) -> agplatform::Result<Vec<PathBuf>> {
    let client = platform.request().client()?;
    let index_url = Url::parse(index_url).map_err(agplatform::Error::request)?;
    let index = client
        .get(index_url.clone())
        .send_async(platform.request())
        .await?;
    platform.fs().create_dir_all(&dir)?;
    let mut tasks = JoinSet::new();

    for line in index.text()?.lines().filter(|line| !line.is_empty()) {
        let url = index_url.join(line).map_err(agplatform::Error::request)?;
        let (platform, client, dir) = (platform.clone(), client.clone(), dir.clone());
        tasks.spawn(async move { mirror_archive(platform.as_ref(), client, url, &dir).await });
    }

    let mut archives = Vec::new();
    while let Some(archive) = tasks.join_next().await {
        archives.push(archive.expect("mirroring should not panic")?);
    }
    archives.sort();
    Ok(archives)
}

fn main() -> agplatform::Result<()> {
    let platform = Arc::new(agplatform::platform());
    let index_url = platform
        .env()
        .args()
        .nth(1)
        .map_or("http://localhost:8080/index".to_string(), |arg| {
            arg.to_string_lossy().into_owned()
        });
    let dir = mirror_dir(platform.as_ref());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime should start");

    for archive in runtime.block_on(mirror(&platform, &index_url, dir))? {
        println!("mirrored {}", archive.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::CommandOutput;
    use agplatform::RequestOutput;
    use agplatform::TestEnv;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_platform;

    const INDEX: &str = "https://mirror.example.com/archives/index";

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    fn respond(status: u16, body: &str) -> RequestOutput {
        RequestOutput {
            status,
            body: body.as_bytes().to_vec(),
            ..Default::default()
        }
    }

    fn exit(status: i32) -> CommandOutput {
        CommandOutput {
            status: Some(status),
            ..Default::default()
        }
    }

    fn unpack_call(dir: &Path, archive_name: &str) -> String {
        format!(
            "tar -xzf {} -C {}",
            dir.join(archive_name).display(),
            dir.display()
        )
    }

    /// Returns a platform serving an index of `a.tgz` and `b.tgz` whose
    /// archives `tar` unpacks.
    fn mirror_platform() -> TestPlatform {
        let platform = test_platform();
        platform
            .request
            .set_response("/index", respond(200, "a.tgz\nb.tgz\n"));
        platform
            .request
            .set_response("/a.tgz", respond(200, "archive a"));
        platform
            .request
            .set_response("/b.tgz", respond(200, "archive b"));
        platform.process.set_output("tar", exit(0));
        platform
    }

    mod mirror_dir {
        use super::*;

        #[test]
        fn mirror_dir_should_prefer_variable_over_tmp_dir() {
            #[cfg(unix)]
            const TMP_VAR: &str = "TMPDIR";
            #[cfg(windows)]
            const TMP_VAR: &str = "TMP";

            for (vars, expected) in [
                (vec![(TMP_VAR, "/tmp")], "/tmp/mirror"),
                (
                    vec![(TMP_VAR, "/tmp"), ("MIRROR_DIR", "/srv/mirror")],
                    "/srv/mirror",
                ),
            ] {
                let env = TestEnv::new().with_vars(vars.clone());
                let platform = test_platform().with_env(env);

                assert_eq!(
                    mirror_dir(&platform),
                    PathBuf::from(expected),
                    "vars: {vars:?}"
                );
            }
        }
    }

    mod mirror {
        use super::*;

        #[test]
        fn mirror_should_download_and_unpack_every_archive() {
            // GIVEN an index of two archives shared with the tasks
            let platform = Arc::new(mirror_platform());

            // WHEN mirroring it
            let archives = block_on(mirror(&platform, INDEX, PathBuf::from("/mirror"))).unwrap();

            // THEN both archives are on disk and were unpacked next to them
            assert_eq!(
                archives,
                [
                    PathBuf::from("/mirror/a.tgz"),
                    PathBuf::from("/mirror/b.tgz")
                ]
            );
            assert_eq!(platform.fs.read("/mirror/a.tgz").unwrap(), b"archive a");
            assert_eq!(platform.fs.read("/mirror/b.tgz").unwrap(), b"archive b");
            let mut unpacked = platform.process.calls();
            unpacked.sort();
            let dir = Path::new("/mirror");
            assert_eq!(
                unpacked,
                [unpack_call(dir, "a.tgz"), unpack_call(dir, "b.tgz")]
            );
        }

        #[test]
        fn mirror_should_resolve_archives_against_index_url() {
            let platform = Arc::new(mirror_platform());
            platform
                .request
                .set_response("/index", respond(200, "a.tgz\n../other/b.tgz\n"));

            block_on(mirror(&platform, INDEX, PathBuf::from("/mirror"))).unwrap();

            let mut calls = platform.request.calls();
            calls.sort();
            assert_eq!(
                calls,
                [
                    "GET https://mirror.example.com/archives/a.tgz",
                    "GET https://mirror.example.com/archives/index",
                    "GET https://mirror.example.com/other/b.tgz"
                ]
            );
        }

        #[test]
        fn mirror_failing_download_should_fail() {
            let platform = Arc::new(mirror_platform());
            platform
                .request
                .set_response("/b.tgz", respond(404, "gone"));

            let error = block_on(mirror(&platform, INDEX, PathBuf::from("/mirror"))).unwrap_err();

            assert_eq!(error.request_output().unwrap().status, 404);
        }

        #[test]
        fn mirror_failing_unpack_should_fail() {
            let platform = Arc::new(mirror_platform());
            let dir = Path::new("/mirror");
            platform
                .process
                .set_output(unpack_call(dir, "a.tgz"), exit(2));

            let error = block_on(mirror(&platform, INDEX, PathBuf::from("/mirror"))).unwrap_err();

            assert_eq!(error.command_output().unwrap().status, Some(2));
        }

        #[test]
        fn mirror_invalid_index_url_should_fail_without_requests() {
            let platform = Arc::new(mirror_platform());

            let error =
                block_on(mirror(&platform, "not a url", PathBuf::from("/mirror"))).unwrap_err();

            assert_eq!(error.kind(), agplatform::ErrorKind::Request);
            assert!(platform.request.calls().is_empty());
        }
    }
}
