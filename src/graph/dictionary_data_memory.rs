use super::dictionary_data::DictionaryData;

pub(crate) struct DictionaryDataMemory {}

impl DictionaryData for DictionaryDataMemory {
    fn commit(&mut self) -> Result<(), crate::DbError> {
        todo!()
    }

    fn transaction(&mut self) {
        todo!()
    }
}
