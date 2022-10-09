use agdb_graph::Graph;

#[test]
fn remove_node_circular_edge() {
    let mut graph = Graph::new();
    let index = graph.insert_node().unwrap();
    let edge = graph.insert_edge(index, index).unwrap();

    graph.remove_node(index).unwrap();

    assert!(graph.node(index).is_none());
    assert!(graph.edge(edge).is_none());
}

#[test]
fn remove_node_only() {
    let mut graph = Graph::new();
    let index = graph.insert_node().unwrap();

    graph.remove_node(index).unwrap();

    assert!(graph.node(index).is_none());
}

#[test]
fn remove_node_missing() {
    let mut graph = Graph::new();
    graph.remove_node(1).unwrap();
}

#[test]
fn remove_nodes_with_edges() {
    let mut graph = Graph::new();

    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();

    let edge1 = graph.insert_edge(node1, node2).unwrap();
    let edge2 = graph.insert_edge(node1, node1).unwrap();
    let edge3 = graph.insert_edge(node1, node3).unwrap();
    let edge4 = graph.insert_edge(node2, node1).unwrap();
    let edge5 = graph.insert_edge(node3, node1).unwrap();

    let edge6 = graph.insert_edge(node3, node2).unwrap();
    let edge7 = graph.insert_edge(node2, node3).unwrap();

    graph.remove_node(node1).unwrap();

    assert!(graph.node(node1).is_none());
    assert!(graph.edge(edge1).is_none());
    assert!(graph.edge(edge2).is_none());
    assert!(graph.edge(edge3).is_none());
    assert!(graph.edge(edge4).is_none());
    assert!(graph.edge(edge5).is_none());

    assert!(graph.node(node2).is_some());
    assert!(graph.node(node3).is_some());
    assert!(graph.edge(edge6).is_some());
    assert!(graph.edge(edge7).is_some());
}
