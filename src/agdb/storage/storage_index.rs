#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let index = StorageIndex::from(1_u64);
        let bytes = index.serialize();
        let other = StorageIndex::deserialize(&bytes).unwrap();

        assert_eq!(index, other);
    }
}
