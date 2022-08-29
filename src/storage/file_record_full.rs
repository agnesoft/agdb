#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct FileRecordFull {
    pub(crate) index: i64,
    pub(crate) position: u64,
    pub(crate) size: u64,
}
