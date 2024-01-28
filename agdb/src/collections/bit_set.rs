#[derive(Default)]
pub struct BitSet {
    data: Vec<u8>,
}

impl BitSet {
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn set(&mut self, value: u64) {
        let byte_index = value as usize / 8;
        let bit_index = value as usize % 8;

        if self.data.len() <= byte_index {
            self.data.resize(byte_index + 1, 0);
        }

        self.data[byte_index] |= 1 << bit_index;
    }

    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            data: Vec::with_capacity(capacity as usize / 8),
        }
    }

    #[allow(dead_code)]
    pub fn unset(&mut self, value: u64) {
        let byte_index = value as usize / 8;

        if byte_index < self.data.len() {
            let bit_index = value as usize % 8;
            self.data[byte_index] ^= 1 << bit_index;
        }
    }

    pub fn value(&self, value: u64) -> bool {
        let byte_index = value as usize / 8;

        if byte_index < self.data.len() {
            let bit_index = value as usize % 8;

            return self.data[byte_index] & (1 << bit_index) != 0;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear() {
        let mut bitset = BitSet::new();

        bitset.set(10_u64);
        bitset.clear();

        assert!(!bitset.value(10_u64));
    }

    #[test]
    fn derived_from_default() {
        let _bitset = BitSet::default();
    }

    #[test]
    fn insert() {
        let mut bitset = BitSet::new();

        assert!(!bitset.value(10_u64));

        bitset.set(10_u64);

        assert!(bitset.value(10_u64));
    }

    #[test]
    fn insert_multiple() {
        let mut bitset = BitSet::new();

        assert!(!bitset.value(10_u64));
        assert!(!bitset.value(11_u64));
        assert!(!bitset.value(2_u64));

        bitset.set(10_u64);
        bitset.set(11_u64);
        bitset.set(2_u64);

        assert!(bitset.value(10_u64));
        assert!(bitset.value(11_u64));
        assert!(bitset.value(2_u64));
    }

    #[test]
    fn unset() {
        let mut bitset = BitSet::new();

        bitset.set(10_u64);
        bitset.set(11_u64);
        bitset.set(2_u64);

        bitset.unset(11_u64);

        assert!(bitset.value(10_u64));
        assert!(!bitset.value(11_u64));
        assert!(bitset.value(2_u64));
    }

    #[test]
    fn remove_unset() {
        let mut bitset = BitSet::new();

        bitset.set(10_u64);
        bitset.set(11_u64);
        bitset.set(2_u64);

        bitset.unset(9_u64);

        assert!(bitset.value(10_u64));
        assert!(bitset.value(11_u64));
        assert!(bitset.value(2_u64));
    }

    #[test]
    fn remove_beyond_length() {
        let mut bitset = BitSet::new();

        bitset.set(10_u64);
        bitset.set(11_u64);
        bitset.set(2_u64);

        bitset.unset(150_u64);

        assert!(bitset.value(10_u64));
        assert!(bitset.value(11_u64));
        assert!(bitset.value(2_u64));
    }

    #[test]
    fn value_missing() {
        let mut bitset = BitSet::new();

        bitset.set(5_u64);

        assert!(!bitset.value(2_u64));
    }

    #[test]
    fn value_beyond_length() {
        let bitset = BitSet::new();

        assert!(!bitset.value(10_u64));
    }
}
