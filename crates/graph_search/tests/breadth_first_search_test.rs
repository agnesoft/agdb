use agdb_graph::Graph;
use agdb_graph::GraphIndex;
use agdb_graph_search::GraphSearch;
use agdb_graph_search::SearchControl;
use agdb_graph_search::SearchHandler;

#[test]
fn find_index() {
    let mut graph = Graph::new();

    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();
    let node4 = graph.insert_node().unwrap();

    let edge1 = graph.insert_edge(&node1, &node2).unwrap();
    let edge2 = graph.insert_edge(&node1, &node3).unwrap();
    let edge3 = graph.insert_edge(&node1, &node4).unwrap();

    struct Handler {}

    impl SearchHandler for Handler {
        fn process(&self, _index: &GraphIndex, _distance: &u64) -> SearchControl {
            SearchControl::Continue(true)
        }
    }

    let result = GraphSearch::from(&graph).breadth_first_search(&node1, &Handler {});

    assert_eq!(
        result,
        vec![node1, edge3, edge2, edge1, node4, node3, node2]
    );
}
