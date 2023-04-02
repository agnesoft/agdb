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
