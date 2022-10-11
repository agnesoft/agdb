use agdb_graph::Graph;
use agdb_graph::GraphIndex;

#[test]
fn insert_node() {
    let mut graph = Graph::new();

    assert_eq!(graph.insert_node(), Ok(GraphIndex::from(1)));
}

#[test]
fn insert_node_after_removal() {
    let mut graph = Graph::new();
    graph.insert_node().unwrap();
    let index = graph.insert_node().unwrap();
    graph.insert_node().unwrap();

    graph.remove_node(&index).unwrap();

    assert_eq!(graph.insert_node(), Ok(index));
}

#[test]
fn node_count() {
    let mut graph = Graph::new();

    assert_eq!(graph.node_count().unwrap(), 0);

    graph.insert_node().unwrap();
    let index = graph.insert_node().unwrap();
    graph.insert_node().unwrap();

    assert_eq!(graph.node_count(), Ok(3));

    graph.remove_node(&index).unwrap();

    assert_eq!(graph.node_count(), Ok(2));
}
