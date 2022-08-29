#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FileRecord {
    pub(crate) position: u64,
    pub(crate) size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let record = FileRecord::default();
        format!("{:?}", record);
    }
}
