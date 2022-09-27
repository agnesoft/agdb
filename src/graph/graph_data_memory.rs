use super::graph_data::GraphData;
use crate::DbError;

pub(crate) struct GraphDataMemory {
    pub(super) from: Vec<i64>,
    pub(super) to: Vec<i64>,
    pub(super) from_meta: Vec<i64>,
    pub(super) to_meta: Vec<i64>,
    pub(super) node_count: u64,
}

impl GraphData for GraphDataMemory {
    fn capacity(&self) -> Result<u64, DbError> {
        Ok(self.from.len() as u64)
    }

    fn free_index(&self) -> Result<i64, DbError> {
        Ok(self.from_meta[0])
    }

    fn from(&self, index: i64) -> Result<i64, DbError> {
        Ok(self.from[index as usize])
    }

    fn from_meta(&self, index: i64) -> Result<i64, DbError> {
        Ok(self.from_meta[index as usize])
    }

    fn node_count(&self) -> Result<u64, DbError> {
        Ok(self.node_count)
    }

    fn resize(&mut self, capacity: u64) -> Result<(), DbError> {
        self.from.resize(capacity as usize, 0);
        self.to.resize(capacity as usize, 0);
        self.from_meta.resize(capacity as usize, 0);
        self.to_meta.resize(capacity as usize, 0);

        Ok(())
    }

    fn set_from(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.from[index as usize] = value;

        Ok(())
    }

    fn set_from_meta(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.from_meta[index as usize] = value;

        Ok(())
    }

    fn set_node_count(&mut self, count: u64) -> Result<(), DbError> {
        self.node_count = count;

        Ok(())
    }

    fn set_to(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.to[index as usize] = value;

        Ok(())
    }

    fn set_to_meta(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.to_meta[index as usize] = value;

        Ok(())
    }

    fn to(&self, index: i64) -> Result<i64, DbError> {
        Ok(self.to[index as usize])
    }

    fn to_meta(&self, index: i64) -> Result<i64, DbError> {
        Ok(self.to_meta[index as usize])
    }
}
