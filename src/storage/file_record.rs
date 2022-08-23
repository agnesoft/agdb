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
    fn file_record_can_be_default_constructed() {
        let _record = FileRecord::default();
    }
}
