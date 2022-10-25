use agdb_graph::Graph;
use agdb_graph::GraphIndex;

#[test]
fn edge_from_index() {
    let mut graph = Graph::new();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index = graph.insert_edge(&from, &to).unwrap();

    assert_eq!(graph.edge(&index).unwrap().index(), index);
}

#[test]
fn edge_from_index_missing() {
    let graph = Graph::new();

    assert!(graph.edge(&GraphIndex::from(-3)).is_none());
}

#[test]
fn edge_iteration() {
    let mut graph = Graph::new();
    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();

    let edge1 = graph.insert_edge(&node1, &node2).unwrap();
    let edge2 = graph.insert_edge(&node1, &node2).unwrap();
    let edge3 = graph.insert_edge(&node1, &node2).unwrap();

    let mut actual = Vec::<GraphIndex>::new();

    for edge in graph.node(&node1).unwrap().edge_iter_from() {
        actual.push(edge.index());
    }

    assert_eq!(actual, vec![edge3, edge2, edge1]);
}

#[test]
fn edge_iteration_reverse() {
    let mut graph = Graph::new();
    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();

    let edge1 = graph.insert_edge(&node1, &node2).unwrap();
    let edge2 = graph.insert_edge(&node1, &node2).unwrap();
    let edge3 = graph.insert_edge(&node1, &node2).unwrap();

    let mut actual = Vec::<GraphIndex>::new();

    for edge in graph.node(&node2).unwrap().edge_iter_to() {
        actual.push(edge.index());
    }

    assert_eq!(actual, vec![edge3, edge2, edge1]);
}

#[test]
fn from_index() {
    let mut graph = Graph::new();
    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();

    let index = graph.insert_edge(&node1, &node2).unwrap();

    assert_eq!(graph.edge(&index).unwrap().index_from(), node1);
}

#[test]
fn to_index() {
    let mut graph = Graph::new();
    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();

    let index = graph.insert_edge(&node1, &node2).unwrap();

    assert_eq!(graph.edge(&index).unwrap().index_to(), node2);
}
