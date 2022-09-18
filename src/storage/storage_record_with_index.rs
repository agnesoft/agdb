#[derive(Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct StorageRecordWithIndex {
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
        let _record = StorageRecordWithIndex::default();
    }

    #[test]
    fn derived_from_debug() {
        let record = StorageRecordWithIndex::default();
        format!("{:?}", record);
    }

    #[test]
    fn derived_from_ord() {
        let record = StorageRecordWithIndex::default();
        assert_eq!(record.cmp(&record), Ordering::Equal);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            StorageRecordWithIndex::default(),
            StorageRecordWithIndex::default()
        );
    }
}
