pub(crate) trait GraphData {
    fn capacity(&self) -> u64;
    fn free_index(&self) -> i64;
    fn from(&self, index: i64) -> i64;
    fn from_meta(&self, index: i64) -> i64;
    fn node_count(&self) -> u64;
    fn resize(&mut self, capacity: u64);
    fn set_from(&mut self, index: i64, value: i64);
    fn set_from_meta(&mut self, index: i64, value: i64);
    fn set_node_count(&mut self, count: u64);
    fn set_to(&mut self, index: i64, value: i64);
    fn set_to_meta(&mut self, index: i64, value: i64);
    fn to(&self, index: i64) -> i64;
    fn to_meta(&self, index: i64) -> i64;
}
