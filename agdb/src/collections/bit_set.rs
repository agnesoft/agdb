#[derive(Default)]
pub struct BitSet {
    data: Vec<u8>,
}

impl BitSet {
    #[inline]
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

    #[inline]
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

    mod bit_set {
        use super::*;

        mod set {
            use super::*;

            #[test]
            fn set_should_set_only_the_value() {
                let mut bitset = BitSet::new();

                bitset.set(10);

                assert!(bitset.value(10));
                assert!(!bitset.value(9));
                assert!(!bitset.value(11));
            }

            #[test]
            fn set_should_keep_other_values() {
                let mut bitset = BitSet::new();

                for value in [10, 11, 2, 100] {
                    bitset.set(value);
                }

                for value in [10, 11, 2, 100] {
                    assert!(bitset.value(value), "value: {value}");
                }
            }
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_have_no_values() {
                let bitset = BitSet::new();

                assert!(!bitset.value(0));
            }
        }

        mod value {
            use super::*;

            #[test]
            fn value_beyond_values_should_be_false() {
                let mut bitset = BitSet::new();
                bitset.set(5);

                assert!(!bitset.value(150));
            }
        }
    }
}
