use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DbId {
    pub id: i64,
}

impl StableHash for DbId {
    fn stable_hash(&self) -> u64 {
        self.id.stable_hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cmp::Ordering,
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    #[test]
    fn derived_from_hash() {
        let mut hasher = DefaultHasher::new();
        DbId { id: 1 }.hash(&mut hasher);
        assert_ne!(hasher.finish(), 0);
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(DbId { id: 1 }.cmp(&DbId { id: 1 }), Ordering::Equal);
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut ids = vec![DbId { id: 3 }, DbId { id: 0 }, DbId { id: -1 }];
        ids.sort();

        assert_eq!(ids, vec![DbId { id: -1 }, DbId { id: 0 }, DbId { id: 3 }]);
    }
}
