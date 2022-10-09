use agdb_graph::Graph;

#[test]
fn remove_edge_circular() {
    let mut graph = Graph::new();
    let node = graph.insert_node().unwrap();
    let index = graph.insert_edge(node, node).unwrap();

    graph.remove_edge(index).unwrap();

    assert!(graph.edge(index).is_none());
}

#[test]
fn remove_edge_first() {
    let mut graph = Graph::new();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index1 = graph.insert_edge(from, to).unwrap();
    let index2 = graph.insert_edge(from, to).unwrap();
    let index3 = graph.insert_edge(from, to).unwrap();

    graph.remove_edge(index3).unwrap();

    assert!(graph.edge(index1).is_some());
    assert!(graph.edge(index2).is_some());
    assert!(graph.edge(index3).is_none());
}

#[test]
fn remove_edge_last() {
    let mut graph = Graph::new();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index1 = graph.insert_edge(from, to).unwrap();
    let index2 = graph.insert_edge(from, to).unwrap();
    let index3 = graph.insert_edge(from, to).unwrap();

    graph.remove_edge(index1).unwrap();

    assert!(graph.edge(index1).is_none());
    assert!(graph.edge(index2).is_some());
    assert!(graph.edge(index3).is_some());
}

#[test]
fn remove_edge_middle() {
    let mut graph = Graph::new();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index1 = graph.insert_edge(from, to).unwrap();
    let index2 = graph.insert_edge(from, to).unwrap();
    let index3 = graph.insert_edge(from, to).unwrap();

    graph.remove_edge(index2).unwrap();

    assert!(graph.edge(index1).is_some());
    assert!(graph.edge(index2).is_none());
    assert!(graph.edge(index3).is_some());
}

#[test]
fn remove_edge_missing() {
    let mut graph = Graph::new();
    graph.remove_edge(-3).unwrap();
}

#[test]
fn remove_edge_only() {
    let mut graph = Graph::new();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index = graph.insert_edge(from, to).unwrap();

    graph.remove_edge(index).unwrap();

    assert!(graph.edge(index).is_none());
}
