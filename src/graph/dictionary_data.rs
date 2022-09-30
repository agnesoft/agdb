use crate::DbError;

pub(crate) trait DictionaryData {
    fn commit(&mut self) -> Result<(), DbError>;
    fn transaction(&mut self);
}
