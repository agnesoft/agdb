use assert_cmd::prelude::*;

const BINARY: &str = "agdb_ci";

struct TestData {
    path: std::path::PathBuf,
}

fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn read_dirs(src: impl AsRef<std::path::Path>) -> std::io::Result<Vec<String>> {
    let mut files = vec![];
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            files.extend(read_dirs(entry.path())?);
        } else {
            let content = std::fs::read_to_string(entry.path())?;
            files.push(content);
        }
    }
    Ok(files)
}

impl TestData {
    fn new() -> Result<Self, std::io::Error> {
        let from = std::env::current_dir()?
            .join("tests")
            .join("test_data_before");
        let path = std::env::current_dir()?.join(format!("agdb_ci_{}", std::process::id()));
        copy_dir_all(from, &path)?;
        Ok(Self { path })
    }
}

impl Drop for TestData {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).unwrap();
    }
}

#[test]
fn replace_version() {
    let temp_dir = TestData::new().unwrap();
    let output = std::process::Command::cargo_bin(BINARY)
        .unwrap()
        .current_dir(&temp_dir.path)
        .output()
        .unwrap();

    assert!(output.status.success());

    let changed = read_dirs(&temp_dir.path).unwrap();
    let expected = read_dirs(
        std::env::current_dir()
            .unwrap()
            .join("tests")
            .join("test_data_after"),
    )
    .unwrap();

    for (actual, expected) in changed.iter().zip(expected.iter()) {
        assert_eq!(actual, expected);
    }
}
