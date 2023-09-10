pub enum StorageMode {
    File,
    FileMemory,
    Memory,
}

pub struct DbConfig {
    pub storage_mode: StorageMode,
}
