#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DictionaryIndex {
    pub(crate) index: i64,
}

impl DictionaryIndex {
    pub fn as_u64(&self) -> u64 {
        self.value() as u64
    }

    pub fn as_usize(&self) -> usize {
        self.value() as usize
    }

    pub fn is_valid(&self) -> bool {
        0 < self.index
    }

    pub fn value(&self) -> i64 {
        self.index
    }
}
