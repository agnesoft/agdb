#[derive(Clone, Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub struct StorageRecord {
    pub index: i64,
    pub position: u64,
    pub size: u64,
}

impl StorageRecord {
    pub fn serialized_size() -> u64 {
        (std::mem::size_of::<u64>() + std::mem::size_of::<u64>()) as u64
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn derived_from_debug() {
//         let record = StorageRecord::default();
//         format!("{:?}", record);
//     }
// }
