use crate::graph_data_memory::GraphDataMemory;

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
