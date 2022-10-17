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

#[test]
fn cyclic_graph() {
    let mut graph = Graph::new();

    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();

    let edge1 = graph.insert_edge(&node1, &node2).unwrap();
    let edge2 = graph.insert_edge(&node1, &node2).unwrap();
    let edge3 = graph.insert_edge(&node2, &node3).unwrap();
    let edge4 = graph.insert_edge(&node2, &node3).unwrap();
    let edge5 = graph.insert_edge(&node3, &node1).unwrap();
    let edge6 = graph.insert_edge(&node3, &node1).unwrap();

    let result = GraphSearch::from(&graph).depth_first_search(&node1, &Handler::default());

    assert_eq!(
        result,
        vec![node1, edge2, node2, edge4, node3, edge6, edge1, edge3, edge5]
    );
}
