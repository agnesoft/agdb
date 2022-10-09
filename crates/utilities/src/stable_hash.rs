pub trait StableHash {
    fn stable_hash(&self) -> u64;
}

impl StableHash for i64 {
    fn stable_hash(&self) -> u64 {
        *self as u64
    }
}

impl StableHash for u64 {
    fn stable_hash(&self) -> u64 {
        *self
    }
}
