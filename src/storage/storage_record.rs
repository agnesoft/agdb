#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StorageRecord {
    pub(super) position: u64,
    pub(super) size: u64,
}

impl StorageRecord {
    pub(super) fn serialized_size() -> u64 {
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
