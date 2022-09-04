#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StorageRecord {
    pub(crate) position: u64,
    pub(crate) size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let record = StorageRecord::default();
        format!("{:?}", record);
    }
}
