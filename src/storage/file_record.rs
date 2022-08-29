#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct FileRecord {
    pub(crate) position: u64,
    pub(crate) size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn default_constructed() {
        let _record = FileRecord::default();
    }

    #[test]
    fn derived_from_debug() {
        let record = FileRecord::default();
        format!("{:?}", record);
    }

    #[test]
    fn derived_from_ord() {
        let record = FileRecord::default();
        assert_eq!(record.cmp(&record), Ordering::Equal);
    }
}
