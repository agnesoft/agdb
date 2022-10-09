use agdb_test_utilities::TestFile;
use agdb_write_ahead_log::WriteAheadLog;
use agdb_write_ahead_log::WriteAheadLogRecord;

#[test]
fn clear() {
    let test_file = TestFile::new();

    let mut wal = WriteAheadLog::try_from(test_file.file_name()).unwrap();
    let record = WriteAheadLogRecord {
        position: 1,
        bytes: vec![1_u8; 5],
    };

    wal.insert(record).unwrap();
    wal.clear().unwrap();

    assert_eq!(wal.records(), Ok(vec![]));
}

#[test]
fn filename_constructed() {
    let test_file = TestFile::new();
    WriteAheadLog::try_from(test_file.file_name()).unwrap();
}

#[test]
fn insert() {
    let test_file = TestFile::from(".\\write_ahead_log_test.rs-insert.testfile");

    let mut wal = WriteAheadLog::try_from(test_file.file_name()).unwrap();
    let record = WriteAheadLogRecord {
        position: 1,
        bytes: vec![1_u8; 5],
    };

    wal.insert(record.clone()).unwrap();

    assert_eq!(wal.records(), Ok(vec![record]));
}

#[test]
fn insert_empty() {
    let test_file = TestFile::from("./write_ahead_log_test.rs-insert_empty.testfile");

    let mut wal = WriteAheadLog::try_from(test_file.file_name()).unwrap();
    let record = WriteAheadLogRecord {
        position: 16,
        bytes: vec![],
    };

    wal.insert(record.clone()).unwrap();

    assert_eq!(wal.records(), Ok(vec![record]));
}

#[test]
fn records() {
    let test_file = TestFile::from("write_ahead_log_test.rs-records.testfile");

    let mut wal = WriteAheadLog::try_from(test_file.file_name()).unwrap();
    let record1 = WriteAheadLogRecord {
        position: 1,
        bytes: vec![1_u8; 5],
    };
    let record2 = WriteAheadLogRecord {
        position: 15,
        bytes: vec![2_u8; 3],
    };

    wal.insert(record1.clone()).unwrap();
    wal.insert(record2.clone()).unwrap();

    assert_eq!(wal.records(), Ok(vec![record1, record2]));
}
