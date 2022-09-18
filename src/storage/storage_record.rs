#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StorageRecord {
    pub(crate) position: u64,
    pub(crate) size: u64,
}

#[allow(dead_code)]
impl StorageRecord {
    pub(crate) fn serialized_size() -> u64 {
        (std::mem::size_of::<u64>() + std::mem::size_of::<u64>()) as u64
    }
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
