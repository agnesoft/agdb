//! Compares the agdb of the working tree with the agdb of a git ref on the
//! workload of `baseline/benches`, see `baseline/README.md`.

use agplatform::CommandExt;
use agplatform::Env;
use agplatform::Error;
use agplatform::Fs;
use agplatform::Platform;
use std::path::Path;
use std::process::Command;

const USAGE: &str = "usage: agdb_benchmark baseline [<ref>] [-- <agbench args>]";

/// Runs the benchmark of the repository at `root` with the `args` that
/// follow `baseline`: the ref, HEAD by default, and the agbench options
/// after `--`.
pub(crate) fn run(platform: &impl Platform, root: &Path, args: &[String]) -> agplatform::Result {
    let split = args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len());

    if 1 < split {
        return Err(Error::other(USAGE));
    }

    let reference = args
        .first()
        .filter(|_| split == 1)
        .map_or("HEAD", String::as_str);
    let fs = platform.fs();
    let process = platform.process();
    // Cargo would resolve the workspace fields of a copy in the repository
    // from the repository's own manifest.
    let work = platform.env().tmp_dir().join("agdb_baseline");
    let base = work.join("base");
    let bench = work.join("bench");
    let index = work.join("index");
    let git = |args: &[&str]| {
        let mut command = Command::new("git");
        command
            .current_dir(root)
            .env("GIT_INDEX_FILE", &index)
            .args(args);
        command
    };

    if fs.exists(&work)? {
        fs.remove_dir_all(&work)?;
    }

    fs.create_dir_all(&base)?;
    fs.create_dir_all(&bench)?;
    git(&["read-tree", reference]).run_blocking(process)?;
    git(&["checkout-index", "--all"])
        .arg(format!("--prefix={}/", base.display()))
        .run_blocking(process)?;
    let manifest = git(&["show", &format!("{reference}:Cargo.toml")]).run_blocking(process)?;
    fs.write(base.join("Cargo.toml"), renamed(manifest.stdout_str()?)?)?;
    fs.write(bench.join("Cargo.toml"), bench_manifest(root, &base))?;
    fs.write(bench.join("lib.rs"), "")?;
    let commit = git(&["rev-parse", "--short", reference]).run_blocking(process)?;
    println!(
        "current: working tree, base: {reference} ({})",
        commit.stdout_str()?.trim()
    );

    Command::new("cargo")
        .current_dir(&bench)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .args(["bench", "--bench", "baseline", "--"])
        .args(args.get(split + 1..).unwrap_or_default())
        .interactive()
        .run_blocking(process)?;

    Ok(())
}

/// The workspace `manifest` with its version suffixed by `-base`: a
/// lockfile cannot hold two path packages of the same name and version.
fn renamed(manifest: &str) -> agplatform::Result<String> {
    let version = manifest
        .lines()
        .find_map(|line| line.strip_prefix("version = \"")?.strip_suffix('"'))
        .ok_or_else(|| Error::other("the workspace manifest has no version"))?;

    Ok(manifest
        .replace(&format!("\"{version}\""), &format!("\"{version}-base\""))
        .replace(&format!("={version}\""), &format!("={version}-base\"")))
}

/// The manifest of the bench of the agdb at `root` against the one at `base`.
fn bench_manifest(root: &Path, base: &Path) -> String {
    let root = root.display();
    let base = base.display();

    format!(
        r#"[package]
name = "agdb_baseline"
version = "0.0.0"
edition = "2024"
publish = false
autobenches = false

[workspace]

[lib]
path = "lib.rs"

[[bench]]
name = "baseline"
path = '{root}/agdb_benchmark/baseline/benches/baseline.rs'
harness = false

[dev-dependencies]
agbench = {{ path = '{root}/agbench' }}
agdb = {{ path = '{root}/agdb' }}
agdb_base = {{ package = "agdb", path = '{base}/agdb' }}
agplatform = {{ path = '{root}/agplatform', features = ["fs", "testing"] }}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::CommandOutput;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::path::PathBuf;

    const MANIFEST: &str = "[workspace.package]\nversion = \"0.18.0\"\n\n[workspace.dependencies]\nagdb = { version = \"=0.18.0\", path = \"agdb\" }\nserde = \"0.18.0\"\n";

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    /// The platform with the repository at `/repo` whose `reference` has
    /// the `manifest`.
    fn platform(reference: &str, manifest: &str) -> TestPlatform {
        let platform = test_platform();
        platform.env.set_var("TMPDIR", "/tmp").unwrap();
        platform.process.set_output("", CommandOutput::default());
        platform.process.set_output(
            format!("git show {reference}:Cargo.toml"),
            CommandOutput::succeeded(manifest),
        );
        platform.process.set_output(
            format!("git rev-parse --short {reference}"),
            CommandOutput::succeeded("abc1234\n"),
        );
        platform
    }

    fn work(platform: &TestPlatform) -> PathBuf {
        platform.env.tmp_dir().join("agdb_baseline")
    }

    mod run {
        use super::*;

        #[test]
        fn run_should_bench_working_tree_against_head() {
            let platform = platform("HEAD", MANIFEST);
            let work = work(&platform);

            run(&platform, Path::new("/repo"), &[]).unwrap();

            assert_eq!(
                platform.process.calls(),
                [
                    "git read-tree HEAD".to_string(),
                    format!(
                        "git checkout-index --all --prefix={}/",
                        work.join("base").display()
                    ),
                    "git show HEAD:Cargo.toml".to_string(),
                    "git rev-parse --short HEAD".to_string(),
                    "cargo bench --bench baseline --".to_string(),
                ]
            );
            let calls = platform.process.call_details();
            assert_eq!(
                calls[0].var("GIT_INDEX_FILE"),
                Some(work.join("index").as_os_str())
            );
            assert_eq!(calls[0].current_dir, Path::new("/repo"));
            assert_eq!(calls[4].current_dir, work.join("bench"));
            assert_eq!(
                calls[4].var("CARGO_TARGET_DIR"),
                Some(Path::new("/repo").join("target").as_os_str())
            );
            assert!(calls[4].options.interactive);
            assert_eq!(
                platform
                    .fs
                    .read_to_string(work.join("bench").join("Cargo.toml"))
                    .unwrap(),
                bench_manifest(Path::new("/repo"), &work.join("base"))
            );
            assert_eq!(
                platform
                    .fs
                    .read_to_string(work.join("bench").join("lib.rs"))
                    .unwrap(),
                ""
            );
        }

        #[test]
        fn run_ref_and_args_should_bench_against_ref_with_args() {
            let platform = platform("origin/main", MANIFEST);

            run(
                &platform,
                Path::new("/repo"),
                &args(&["origin/main", "--", "--filter", "select"]),
            )
            .unwrap();

            let calls = platform.process.calls();
            assert_eq!(calls[0], "git read-tree origin/main");
            assert_eq!(calls[4], "cargo bench --bench baseline -- --filter select");
        }

        #[test]
        fn run_args_should_bench_against_head_with_args() {
            let platform = platform("HEAD", MANIFEST);

            run(&platform, Path::new("/repo"), &args(&["--", "--check"])).unwrap();

            let calls = platform.process.calls();
            assert_eq!(calls[0], "git read-tree HEAD");
            assert_eq!(calls[4], "cargo bench --bench baseline -- --check");
        }

        #[test]
        fn run_should_suffix_base_version() {
            let platform = platform("HEAD", MANIFEST);

            run(&platform, Path::new("/repo"), &[]).unwrap();

            assert_eq!(
                platform
                    .fs
                    .read_to_string(work(&platform).join("base").join("Cargo.toml"))
                    .unwrap(),
                "[workspace.package]\nversion = \"0.18.0-base\"\n\n[workspace.dependencies]\nagdb = { version = \"=0.18.0-base\", path = \"agdb\" }\nserde = \"0.18.0-base\"\n"
            );
        }

        #[test]
        fn run_should_replace_previous_export() {
            let platform = platform("HEAD", MANIFEST);
            let stale = work(&platform).join("base").join("stale.rs");
            platform
                .fs
                .create_dir_all(work(&platform).join("base"))
                .unwrap();
            platform.fs.write(&stale, "").unwrap();

            run(&platform, Path::new("/repo"), &[]).unwrap();

            assert!(!platform.fs.exists(&stale).unwrap());
        }

        #[test]
        fn run_two_refs_should_fail() {
            let platform = platform("HEAD", MANIFEST);

            let error = run(&platform, Path::new("/repo"), &args(&["a", "b"])).unwrap_err();

            assert_eq!(error.description(), USAGE);
            assert!(platform.process.calls().is_empty());
        }

        #[test]
        fn run_manifest_without_version_should_fail() {
            let platform = platform("HEAD", "[workspace]\n");

            let error = run(&platform, Path::new("/repo"), &[]).unwrap_err();

            assert_eq!(error.description(), "the workspace manifest has no version");
        }

        #[test]
        fn run_failed_command_should_fail() {
            let commands = [
                "git read-tree",
                "git checkout-index",
                "git show",
                "git rev-parse",
                "cargo bench",
            ];

            for (failed, command) in commands.into_iter().enumerate() {
                let platform = platform("HEAD", MANIFEST);
                platform
                    .process
                    .set_output(command, CommandOutput::failed(128, "failed"));

                assert!(run(&platform, Path::new("/repo"), &[]).is_err());

                assert_eq!(platform.process.calls().len(), failed + 1, "{command}");
            }
        }

        #[test]
        fn run_non_utf8_output_should_fail() {
            for command in ["git show HEAD:Cargo.toml", "git rev-parse --short HEAD"] {
                let platform = platform("HEAD", MANIFEST);
                platform
                    .process
                    .set_output(command, CommandOutput::succeeded(vec![0xff]));

                assert!(run(&platform, Path::new("/repo"), &[]).is_err());

                assert_eq!(platform.process.calls().last().unwrap(), command);
            }
        }

        #[test]
        fn run_failed_io_should_fail() {
            for op in [FsOp::Metadata, FsOp::Remove, FsOp::CreateDir, FsOp::Open] {
                let mut fail = 0;

                loop {
                    let platform = platform("HEAD", MANIFEST);
                    platform.fs.create_dir_all(work(&platform)).unwrap();
                    platform.fs.set_io_errors(
                        op,
                        "",
                        (0..=fail).map(|call| (call == fail).then_some(std::io::ErrorKind::Other)),
                    );

                    if run(&platform, Path::new("/repo"), &[]).is_ok() {
                        break;
                    }

                    assert!(
                        platform
                            .process
                            .calls()
                            .iter()
                            .all(|call| !call.starts_with("cargo"))
                    );
                    fail += 1;
                }

                assert!(0 < fail, "op: {op:?}");
            }
        }
    }
}
