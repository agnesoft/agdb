use agdb_write_ahead_log::WriteAheadLogRecord;

#[test]
fn derived_from_debug() {
    let record = WriteAheadLogRecord::default();
    format!("{:?}", record);
}
