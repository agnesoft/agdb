use crate::DbError;

#[derive(Clone, Default)]
pub struct FileRecord {
    pub index: u64,
    pub pos: u64,
    pub size: u64,
}

pub struct FileRecords {
    records: Vec<FileRecord>,
}

impl FileRecords {
    pub fn records(&self) -> Vec<FileRecord> {
        let mut res = Vec::<FileRecord>::new();
        res.reserve(self.records.len());

        for record in &self.records {
            if record.index != 0 {
                res.push(record.clone());
            }
        }

        res.sort_by(|left, right| left.pos.cmp(&right.pos));

        res
    }

    pub fn new() -> Self {
        Self {
            records: vec![FileRecord::default()],
        }
    }

    pub fn new_record(&mut self, pos: u64, size: u64) -> FileRecord {
        for i in 1..self.records.len() {
            let mut record = &mut self.records[i];

            if record.index == 0 {
                record.index = i as u64;
                record.pos = pos;
                record.size = size;

                return record.clone();
            }
        }

        let record = FileRecord {
            index: self.records.len() as u64,
            pos,
            size,
        };
        self.records.push(record.clone());

        record
    }

    pub fn set_pos(&mut self, index: u64, pos: u64) {
        if let Some(i) = self.records.get_mut(index as usize) {
            i.pos = pos;
        }
    }

    pub fn set_size(&mut self, index: u64, size: u64) {
        if let Some(i) = self.records.get_mut(index as usize) {
            i.size = size;
        }
    }

    pub fn record(&self, index: u64) -> Result<FileRecord, DbError> {
        if let Some(i) = self.records.get(index as usize) {
            Ok(i.clone())
        } else {
            Err(DbError::from(format!(
                "FileStorage error: index {} not found",
                index
            )))
        }
    }

    pub fn remove_index(&mut self, index: u64) {
        if let Some(i) = self.records.get_mut(index as usize) {
            i.index = 0;
        }
    }
}
