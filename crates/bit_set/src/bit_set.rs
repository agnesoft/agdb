#[derive(Default)]
pub struct BitSet {
    data: Vec<u8>,
}

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
