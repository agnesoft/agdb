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
fn circular_path() {
    let mut graph = Graph::new();
    let node = graph.insert_node().unwrap();
    let _edge = graph.insert_edge(&node, &node).unwrap();

    let result = GraphSearch::from(&graph).path(&node, &node, &Handler::default());

    assert_eq!(result, vec![]);
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

#[test]
fn multi_edge_path() {
    let mut graph = Graph::new();

    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();

    let edge1 = graph.insert_edge(&node1, &node2).unwrap();
    let _edge2 = graph.insert_edge(&node1, &node2).unwrap();

    let edge3 = graph.insert_edge(&node2, &node3).unwrap();
    let _edge4 = graph.insert_edge(&node2, &node3).unwrap();

    let result = GraphSearch::from(&graph).path(&node1, &node3, &Handler::default());

    assert_eq!(result, vec![node1, edge1, node2, edge3, node3]);
}

#[test]
fn origin_is_destination() {
    let mut graph = Graph::new();
    let node = graph.insert_node().unwrap();

    let result = GraphSearch::from(&graph).path(&node, &node, &Handler::default());

    assert_eq!(result, vec![]);
}

#[test]
fn short_circuit_path() {
    let mut graph = Graph::new();

    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();

    let edge1 = graph.insert_edge(&node1, &node3).unwrap();
    let _edge2 = graph.insert_edge(&node1, &node2).unwrap();
    let _edge3 = graph.insert_edge(&node2, &node3).unwrap();

    let result = GraphSearch::from(&graph).path(&node1, &node3, &Handler::default());

    assert_eq!(result, vec![node1, edge1, node3]);
}

#[test]
fn single_path() {
    let mut graph = Graph::new();

    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();

    let edge1 = graph.insert_edge(&node1, &node2).unwrap();
    let edge2 = graph.insert_edge(&node2, &node3).unwrap();

    let result = GraphSearch::from(&graph).path(&node1, &node3, &Handler::default());

    assert_eq!(result, vec![node1, edge1, node2, edge2, node3]);
}

#[test]
fn skip_short_circuit_path() {
    let mut graph = Graph::new();

    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();

    let _edge1 = graph.insert_edge(&node1, &node3).unwrap();
    let edge2 = graph.insert_edge(&node1, &node2).unwrap();
    let edge3 = graph.insert_edge(&node2, &node3).unwrap();

    let result = GraphSearch::from(&graph).path(
        &node1,
        &node3,
        &Handler {
            processor: |index: &GraphIndex, _distance: &u64| {
                if index.value() == -4 {
                    return 0;
                }

                1
            },
        },
    );

    assert_eq!(result, vec![node1, edge2, node2, edge3, node3]);
}

#[test]
fn unconnected() {
    let mut graph = Graph::new();

    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();

    let _edge1 = graph.insert_edge(&node1, &node2).unwrap();

    let result = GraphSearch::from(&graph).path(&node1, &node3, &Handler::default());

    assert_eq!(result, vec![]);
}
