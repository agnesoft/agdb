use super::graph_data::GraphData;
use super::graph_index::GraphIndex;
use crate::db::db_error::DbError;

pub struct GraphDataMemory {
    pub(crate) from: Vec<i64>,
    pub(crate) to: Vec<i64>,
    pub(crate) from_meta: Vec<i64>,
    pub(crate) to_meta: Vec<i64>,
}

impl GraphData for GraphDataMemory {
    fn capacity(&self) -> Result<u64, DbError> {
        Ok(self.from.len() as u64)
    }

    fn commit(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    fn free_index(&self) -> Result<i64, DbError> {
        Ok(self.from_meta[0])
    }

    fn from(&self, index: &GraphIndex) -> Result<i64, DbError> {
        Ok(self.from[index.as_usize()])
    }

    fn from_meta(&self, index: &GraphIndex) -> Result<i64, DbError> {
        Ok(self.from_meta[index.as_usize()])
    }

    fn grow(&mut self) -> Result<(), DbError> {
        self.from.push(0);
        self.to.push(0);
        self.from_meta.push(0);
        self.to_meta.push(0);

        Ok(())
    }

    fn node_count(&self) -> Result<u64, DbError> {
        Ok(self.to_meta[0] as u64)
    }

    fn set_from(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.from[index.as_usize()] = value;

        Ok(())
    }

    fn set_from_meta(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.from_meta[index.as_usize()] = value;

        Ok(())
    }

    fn set_node_count(&mut self, count: u64) -> Result<(), DbError> {
        self.to_meta[0] = count as i64;

        Ok(())
    }

    fn set_to(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.to[index.as_usize()] = value;

        Ok(())
    }

    fn set_to_meta(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.to_meta[index.as_usize()] = value;

        Ok(())
    }

    fn to(&self, index: &GraphIndex) -> Result<i64, DbError> {
        Ok(self.to[index.as_usize()])
    }

    fn to_meta(&self, index: &GraphIndex) -> Result<i64, DbError> {
        Ok(self.to_meta[index.as_usize()])
    }

    fn transaction(&mut self) {}
}

impl Default for GraphDataMemory {
    fn default() -> Self {
        Self {
            from: vec![0],
            to: vec![0],
            from_meta: vec![i64::MIN],
            to_meta: vec![0],
        }
    }
}
