use super::graph_data::GraphData;
use agdb_db_error::DbError;

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

    fn from(&self, index: i64) -> Result<i64, DbError> {
        Ok(self.from[index as usize])
    }

    fn from_meta(&self, index: i64) -> Result<i64, DbError> {
        Ok(self.from_meta[index as usize])
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

    fn set_from(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.from[index as usize] = value;

        Ok(())
    }

    fn set_from_meta(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.from_meta[index as usize] = value;

        Ok(())
    }

    fn set_node_count(&mut self, count: u64) -> Result<(), DbError> {
        self.to_meta[0] = count as i64;

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

    fn transaction(&mut self) {}
}
