use super::db_value_index::DbValueIndex;
use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct DbKeyValueIndex {
    pub(crate) key: DbValueIndex,
    pub(crate) value: DbValueIndex,
}

impl StableHash for DbKeyValueIndex {
    fn stable_hash(&self) -> u64 {
        [self.key.value, self.value.value].concat().stable_hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;

    #[test]
    fn derived_from() {
        let index = DbKeyValueIndex::default();
        assert_eq!(index, index.clone());

        format!("{:?}", index);

        let mut hasher = DefaultHasher::new();
        DbKeyValueIndex::default().hash(&mut hasher);

        index.stable_hash();
    }
}
