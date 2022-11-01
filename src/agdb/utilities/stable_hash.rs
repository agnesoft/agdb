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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64() {
        assert_eq!(10_i64.stable_hash(), 10_u64);
    }

    #[test]
    fn u64() {
        assert_eq!(10_u64.stable_hash(), 10_u64);
    }
}
