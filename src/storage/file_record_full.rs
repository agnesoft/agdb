#[allow(dead_code)]
#[derive(Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct FileRecordFull {
    pub(crate) index: i64,
    pub(crate) position: u64,
    pub(crate) size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn default_constructed() {
        let _record = FileRecordFull::default();
    }

    #[test]
    fn derived_from_debug() {
        let record = FileRecordFull::default();
        format!("{:?}", record);
    }

    #[test]
    fn derived_from_ord() {
        let record = FileRecordFull::default();
        assert_eq!(record.cmp(&record), Ordering::Equal);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(FileRecordFull::default(), FileRecordFull::default());
    }
}
