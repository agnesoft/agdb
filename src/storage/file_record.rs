#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FileRecord {
    pub(crate) pos: u64,
    pub(crate) size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_constructed() {
        let _record = FileRecord::default();
    }

    #[test]
    fn derived_from_debug() {
        let record = FileRecord::default();
        format!("{:?}", record);
    }
}
