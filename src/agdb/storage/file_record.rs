use crate::utilities::serialize::SerializeFixedSized;

#[derive(Clone, Copy, Default)]
pub struct FileRecord {
    pub index: u64,
    pub pos: u64,
    pub size: u64,
}

impl FileRecord {
    pub fn value_start(&self) -> u64 {
        self.pos + u64::serialized_size() * 2
    }

    pub fn end(&self) -> u64 {
        self.value_start() + self.size
    }
}
