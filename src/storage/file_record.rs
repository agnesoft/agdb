#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct FileRecord {
    pos: u64,
    size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_record_can_be_default_constructed() {
        let _record = FileRecord::default();
    }
}
