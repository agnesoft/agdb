use crate::utilities::serialize::Serialize;

#[derive(Clone, Copy, Default)]
pub struct FileRecord {
    pub index: u64,
    pub pos: u64,
    pub size: u64,
}

impl FileRecord {
    pub fn value_start(&self) -> u64 {
        self.pos + self.index.serialized_size() + self.size.serialized_size()
    }

    pub fn end(&self) -> u64 {
        self.value_start() + self.size
    }
}
