#[derive(Default)]
pub struct BitSet {
    #[allow(dead_code)]
    data: Vec<u8>,
}

#[allow(dead_code)]
impl BitSet {
    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn insert(&mut self, value: u64) {
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

    pub fn remove(&mut self, value: u64) {
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

        bitset.insert(10_u64);
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

        bitset.insert(10_u64);

        assert!(bitset.value(10_u64));
    }

    #[test]
    fn insert_multiple() {
        let mut bitset = BitSet::new();

        assert!(!bitset.value(10_u64));
        assert!(!bitset.value(11_u64));
        assert!(!bitset.value(2_u64));

        bitset.insert(10_u64);
        bitset.insert(11_u64);
        bitset.insert(2_u64);

        assert!(bitset.value(10_u64));
        assert!(bitset.value(11_u64));
        assert!(bitset.value(2_u64));
    }

    #[test]
    fn remove() {
        let mut bitset = BitSet::new();

        bitset.insert(10_u64);
        bitset.insert(11_u64);
        bitset.insert(2_u64);

        bitset.remove(11_u64);

        assert!(bitset.value(10_u64));
        assert!(!bitset.value(11_u64));
        assert!(bitset.value(2_u64));
    }

    #[test]
    fn remove_unset() {
        let mut bitset = BitSet::new();

        bitset.insert(10_u64);
        bitset.insert(11_u64);
        bitset.insert(2_u64);

        bitset.remove(9_u64);

        assert!(bitset.value(10_u64));
        assert!(bitset.value(11_u64));
        assert!(bitset.value(2_u64));
    }

    #[test]
    fn remove_beyond_length() {
        let mut bitset = BitSet::new();

        bitset.insert(10_u64);
        bitset.insert(11_u64);
        bitset.insert(2_u64);

        bitset.remove(150_u64);

        assert!(bitset.value(10_u64));
        assert!(bitset.value(11_u64));
        assert!(bitset.value(2_u64));
    }

    #[test]
    fn value_missing() {
        let mut bitset = BitSet::new();

        bitset.insert(5_u64);

        assert!(!bitset.value(2_u64));
    }

    #[test]
    fn value_beyond_length() {
        let bitset = BitSet::new();

        assert!(!bitset.value(10_u64));
    }
}
