use agdb_graph::GraphIndex;

pub trait PathSearchHandler {
    fn process(&self, index: &GraphIndex, distance: &u64) -> u64;
}
