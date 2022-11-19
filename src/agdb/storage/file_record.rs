use crate::utilities::serialize::SerializeFixedSized;
use crate::DbIndex;

#[derive(Clone, Default)]
pub struct FileRecord {
    pub index: u64,
    pub pos: u64,
    pub size: u64,
}

impl FileRecord {
    pub fn value_start(&self) -> u64 {
        self.pos + DbIndex::serialized_size()
    }

    pub fn end(&self) -> u64 {
        self.value_start() + self.size
    }
}
