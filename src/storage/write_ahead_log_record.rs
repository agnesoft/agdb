#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub(crate) struct WriteAheadLogRecord {
    pub(crate) position: u64,
    pub(crate) bytes: Vec<u8>,
}
