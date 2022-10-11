use agdb_graph::Graph;
use agdb_graph::GraphIndex;

#[test]
fn node_from_index() {
    let mut graph = Graph::new();
    let index = graph.insert_node().unwrap();

    assert_eq!(graph.node(&index).unwrap().index(), index);
}

#[test]
fn node_from_index_missing() {
    let graph = Graph::new();

    let node = graph.node(&GraphIndex::from(1));

    assert!(node.is_none());
}

#[test]
fn node_iteration() {
    let mut graph = Graph::new();
    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();

    let expected = vec![node1, node2, node3];
    let mut nodes = Vec::<GraphIndex>::new();

    for node in graph.node_iter() {
        nodes.push(node.index());
    }

    assert_eq!(nodes, expected);
}

#[test]
fn node_iteration_with_removed_nodes() {
    let mut graph = Graph::new();
    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();
    let node4 = graph.insert_node().unwrap();
    let node5 = graph.insert_node().unwrap();

    graph.remove_node(&node2).unwrap();
    graph.remove_node(&node5).unwrap();

    let expected = vec![node1, node3, node4];
    let mut nodes = Vec::<GraphIndex>::new();

    for node in graph.node_iter() {
        nodes.push(node.index());
    }

    assert_eq!(nodes, expected);
}
