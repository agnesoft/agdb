use agdb_test_file::TestFile;
use std::fs::OpenOptions;
use std::path::Path;

fn ensure_file(filename: &str) {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(filename)
        .unwrap();
}

#[test]
fn default() {
    let caller = std::panic::Location::caller();
    let current_source_file = Path::new(caller.file())
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let test_file = TestFile::default();
    assert!(!test_file.file_name().is_empty());
    assert!(test_file.file_name().contains(&current_source_file));
}

#[test]
fn created_from_str_ref() {
    let filename = "./test_file-created_from_str_ref";
    let _test_file = TestFile::from(filename);
}

#[test]
fn created_from_string() {
    let filename = "./test_file-created_from_string".to_string();
    let _test_file = TestFile::from(filename);
}

#[test]
fn existing_file_is_deleted_on_construction() {
    let filename = "./test_file-existing_file_is_deleted_on_construction";
    ensure_file(filename);
    let _test_file = TestFile::from(filename);
    assert!(!Path::new(filename).exists());
}

#[test]
fn file_is_deleted_on_destruction() {
    let filename = "./test_file-file_is_deleted_on_destruction";

    {
        let _test_file = TestFile::from(filename);
        ensure_file(filename);
    }

    assert!(!Path::new(filename).exists());
}

#[test]
fn get_file_name() {
    let filename = "./test_file-get_file_name";
    let test_file = TestFile::from(filename);

    assert_eq!(test_file.file_name(), filename);
}

#[test]
fn hidden_file_is_deleted_on_construction() {
    let filename = "./test_file-hidden_file_is_deleted_on_construction";
    let hidden_filename = "./.test_file-hidden_file_is_deleted_on_construction";
    ensure_file(filename);
    ensure_file(hidden_filename);
    let _test_file = TestFile::from(filename);
    assert!(!Path::new(filename).exists());
    assert!(!Path::new(hidden_filename).exists());
}

#[test]
fn hidden_file_is_deleted_on_destruction() {
    let filename = "test_file-hidden_file_is_deleted_on_destruction";
    let hidden_filename = ".test_file-hidden_file_is_deleted_on_destruction";

    {
        let _test_file = TestFile::from(filename);
        ensure_file(filename);
        ensure_file(hidden_filename);
    }

    assert!(!Path::new(filename).exists());
    assert!(!Path::new(hidden_filename).exists());
}

#[test]
fn new() {
    let caller = std::panic::Location::caller();
    let current_source_file = Path::new(caller.file())
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let test_file = TestFile::new();
    assert!(!test_file.file_name().is_empty());
    assert!(test_file.file_name().contains(&current_source_file));
}
