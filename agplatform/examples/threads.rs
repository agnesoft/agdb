//! Checksums of the files of a directory computed by `sha256sum` processes
//! run from several threads, written to a `SHA256SUMS` manifest.
//!
//! The platform is shared by reference with scoped threads, every platform
//! is `Send + Sync`, so the threads get the same `TestPlatform` in the tests.
//! `TestProcess` and `TestFs` are safe to use from many threads at once and
//! record the commands of all of them.
//!
//! ```bash
//! cargo run --example threads --features fs,process -- src
//! ```

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use agplatform::CommandExt;
use agplatform::Env;
use agplatform::FileType;
use agplatform::Fs;
use agplatform::Platform;

fn checksum(platform: &impl Platform, file: &Path) -> agplatform::Result<String> {
    let output = Command::new("sha256sum")
        .arg(file)
        .run_blocking(platform.process())?;
    let line = output.stdout_str()?;
    Ok(line
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string())
}

/// Returns the checksums of the files in `dir` sorted by their path,
/// computed by `threads` threads.
fn checksums(
    platform: &impl Platform,
    dir: &Path,
    threads: usize,
) -> agplatform::Result<Vec<(PathBuf, String)>> {
    let mut files = Vec::new();
    for entry in platform.fs().read_dir(dir)? {
        let entry = entry?;
        if entry.file_type() == FileType::File {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();

    let per_thread = files.len().div_ceil(threads.max(1)).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = files
            .chunks(per_thread)
            .map(|files| {
                scope.spawn(move || {
                    files
                        .iter()
                        .map(|file| Ok((file.clone(), checksum(platform, file)?)))
                        .collect::<agplatform::Result<Vec<_>>>()
                })
            })
            .collect();

        let mut checksums = Vec::new();
        for worker in workers {
            checksums.extend(worker.join().expect("workers should not panic")?);
        }
        Ok(checksums)
    })
}

/// Writes the `checksums` of the files in `dir` to its `SHA256SUMS` in the
/// format of `sha256sum --check`.
fn write_manifest(
    platform: &impl Platform,
    dir: &Path,
    checksums: &[(PathBuf, String)],
) -> agplatform::Result<()> {
    let manifest: String = checksums
        .iter()
        .map(|(file, checksum)| {
            let name = file.file_name().unwrap_or(file.as_os_str());
            format!("{checksum}  {}\n", name.display())
        })
        .collect();
    platform.fs().write(dir.join("SHA256SUMS"), manifest)
}

fn main() -> agplatform::Result<()> {
    let platform = agplatform::platform();
    let dir = platform
        .env()
        .args()
        .nth(1)
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let threads = std::thread::available_parallelism().map_or(1, usize::from);

    let checksums = checksums(&platform, &dir, threads)?;
    write_manifest(&platform, &dir, &checksums)?;
    println!(
        "{} files written to {}",
        checksums.len(),
        dir.join("SHA256SUMS").display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::CommandOutput;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_platform;
    use std::collections::HashMap;

    fn sha256sum(platform: &TestPlatform, file: &str, checksum: &str) {
        platform.process.set_output(
            format!("sha256sum {file}"),
            CommandOutput {
                status: Some(0),
                stdout: format!("{checksum}  {file}\n").into_bytes(),
                ..Default::default()
            },
        );
    }

    fn platform_with_files(files: &[&str]) -> TestPlatform {
        let platform = test_platform();
        platform.fs.create_dir_all("/data").unwrap();
        let mut checksums = HashMap::new();
        for (index, file) in files.iter().enumerate() {
            platform.fs.write(file, "").unwrap();
            let checksum = format!("{index:04}");
            checksums.insert(
                Path::new(file)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                checksum.clone(),
            );
            sha256sum(&platform, file, &checksum);
        }
        for entry in platform.fs.read_dir("/data").unwrap() {
            let entry = entry.unwrap();
            if entry.file_type() != FileType::File {
                continue;
            }
            let name = entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            if let Some(checksum) = checksums.get(&name) {
                sha256sum(&platform, &entry.path().display().to_string(), checksum);
            }
        }
        platform
    }

    mod checksum {
        use super::*;

        #[test]
        fn checksum_should_return_first_field_of_sha256sum() {
            let platform = platform_with_files(&["/data/a"]);

            let checksum = checksum(&platform, Path::new("/data/a")).unwrap();

            assert_eq!(checksum, "0000");
        }
    }

    mod checksums {
        use super::*;

        #[test]
        fn checksums_should_hash_every_file_once_whatever_the_threads() {
            for threads in [0, 1, 2, 3, 8] {
                // GIVEN a directory with files and a sub-directory
                let platform = platform_with_files(&["/data/c", "/data/a", "/data/b"]);
                platform.fs.create_dir_all("/data/sub").unwrap();

                // WHEN computing their checksums on threads
                let checksums = checksums(&platform, Path::new("/data"), threads).unwrap();

                // THEN each file was hashed once by any of the threads and
                // the checksums are sorted by path
                let mut calls = platform.process.calls();
                calls.sort();
                assert_eq!(
                    calls
                        .iter()
                        .map(|call| {
                            assert!(call.starts_with("sha256sum "), "call: {call}");
                            Path::new(call.split_once(' ').unwrap().1)
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string()
                        })
                        .collect::<Vec<_>>(),
                    ["a", "b", "c"],
                    "threads: {threads}"
                );
                assert_eq!(
                    checksums
                        .iter()
                        .map(|(path, checksum)| {
                            (
                                path.file_name().unwrap().to_string_lossy().to_string(),
                                checksum.clone(),
                            )
                        })
                        .collect::<Vec<_>>(),
                    [
                        ("a".to_string(), "0001".to_string()),
                        ("b".to_string(), "0002".to_string()),
                        ("c".to_string(), "0000".to_string())
                    ],
                    "threads: {threads}"
                );
            }
        }

        #[test]
        fn checksums_failing_process_should_fail() {
            let platform = platform_with_files(&["/data/a", "/data/b"]);
            platform.process.set_output(
                "sha256sum",
                CommandOutput {
                    status: Some(1),
                    ..Default::default()
                },
            );

            let error = checksums(&platform, Path::new("/data"), 2).unwrap_err();

            assert_eq!(error.command_output().unwrap().status, Some(1));
        }

        #[test]
        fn checksums_missing_dir_should_fail() {
            let platform = test_platform();

            let error = checksums(&platform, Path::new("/data"), 2).unwrap_err();

            assert_eq!(error.kind(), agplatform::ErrorKind::Fs);
            assert!(platform.process.calls().is_empty());
        }
    }

    mod write_manifest {
        use super::*;

        #[test]
        fn write_manifest_should_list_checksums_with_file_names() {
            // GIVEN checksums of absolute paths as returned by `read_dir`
            // of a relative directory
            let platform = platform_with_files(&[]);
            let checksums = [
                (PathBuf::from("/data/a"), "0001".to_string()),
                (PathBuf::from("/data/b"), "0002".to_string()),
            ];

            // WHEN writing their manifest to the relative directory
            write_manifest(&platform, Path::new("data"), &checksums).unwrap();

            // THEN it names the files relative to it

            assert_eq!(
                platform.fs.read_to_string("/data/SHA256SUMS").unwrap(),
                "0001  a\n0002  b\n"
            );
        }
    }
}
