use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DbId(pub i64);

impl StableHash for DbId {
    fn stable_hash(&self) -> u64 {
        self.0.stable_hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    #[test]
    fn derived_from_hash() {
        let mut hasher = DefaultHasher::new();
        DbId(1).hash(&mut hasher);
        assert_ne!(hasher.finish(), 0);
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(DbId(1).cmp(&DbId(1)), Ordering::Equal);
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut ids = vec![DbId(3), DbId(0), DbId(-1)];
        ids.sort();

        assert_eq!(ids, vec![DbId(-1), DbId(0), DbId(3)]);
    }
}
