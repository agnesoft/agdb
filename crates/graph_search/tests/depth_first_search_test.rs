use agdb_graph::Graph;
use agdb_graph::GraphIndex;
use agdb_graph_search::GraphSearch;
use agdb_graph_search::SearchControl;
use agdb_graph_search::SearchHandler;

struct Handler {
    pub processor: fn(&GraphIndex, &u64) -> SearchControl,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            processor: |_index: &GraphIndex, _distance: &u64| SearchControl::Continue(true),
        }
    }
}

impl SearchHandler for Handler {
    fn process(&self, index: &GraphIndex, distance: &u64) -> SearchControl {
        (self.processor)(index, distance)
    }
}

#[test]
fn empty_graph() {
    let graph = Graph::new();

    let result =
        GraphSearch::from(&graph).depth_first_search(&GraphIndex::default(), &Handler::default());

    assert_eq!(result, vec![]);
}
