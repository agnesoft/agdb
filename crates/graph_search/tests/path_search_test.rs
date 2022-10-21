use agdb_graph::Graph;
use agdb_graph::GraphIndex;
use agdb_graph_search::GraphSearch;
use agdb_graph_search::PathSearchHandler;

struct Handler {
    pub processor: fn(&GraphIndex, &u64) -> u64,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            processor: |_index: &GraphIndex, _distance: &u64| 1_u64,
        }
    }
}

impl PathSearchHandler for Handler {
    fn process(&self, index: &GraphIndex, distance: &u64) -> u64 {
        (self.processor)(index, distance)
    }
}

#[test]
fn empty_graph() {
    let graph = Graph::new();

    let result = GraphSearch::from(&graph).path(
        &GraphIndex::default(),
        &GraphIndex::default(),
        &Handler::default(),
    );

    assert_eq!(result, vec![]);
}
