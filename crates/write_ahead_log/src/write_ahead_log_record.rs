#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WriteAheadLogRecord {
    pub position: u64,
    pub bytes: Vec<u8>,
}
