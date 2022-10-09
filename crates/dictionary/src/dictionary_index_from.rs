use crate::dictionary_index::DictionaryIndex;

impl From<i64> for DictionaryIndex {
    fn from(index: i64) -> Self {
        Self { index }
    }
}
