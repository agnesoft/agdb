use agdb_db_error::DbError;
use std::error::Error;

#[test]
fn caused_by() {
    let error = DbError::from("file not found");
    let new_error = DbError::from("open error").caused_by(error);

    assert_eq!(
        new_error.cause,
        Some(Box::new(DbError::from("file not found")))
    );
}

#[test]
fn derived_from_debug() {
    let error = DbError::from("error");

    format!("{:?}", error);
}

#[test]
fn derived_from_display() {
    let file = file!();
    let col__ = column!();
    let line = line!();
    let error = DbError::from("file not found");

    assert_eq!(
        error.to_string(),
        format!(
            "file not found (at {}:{}:{})",
            file.replace('\\', "/"),
            line + 1,
            col__
        )
    );
}

#[test]
fn derived_from_partial_eq() {
    let left = DbError::from(std::io::Error::from(std::io::ErrorKind::NotFound));
    let right = DbError::from(std::io::Error::from(std::io::ErrorKind::NotFound));

    assert_eq!(left, right);
}

#[test]
fn derived_from_error() {
    let file = file!();
    let col__ = column!();
    let line = line!();
    let error = DbError::from("file not found");
    let new_error = DbError::from("open error").caused_by(error);

    assert_eq!(
        new_error.source().unwrap().to_string(),
        format!(
            "file not found (at {}:{}:{})",
            file.replace('\\', "/"),
            line + 1,
            col__
        )
    );
}

#[test]
fn from_io_error() {
    let _error = DbError::from(std::io::Error::from(std::io::ErrorKind::NotFound));
}

#[test]
fn from_utf8_error() {
    let _error = DbError::from(String::from_utf8(vec![0xdf, 0xff]).unwrap_err());
}

#[test]
fn source_none() {
    let error = DbError::from("file not found");

    assert!(error.source().is_none());
}
