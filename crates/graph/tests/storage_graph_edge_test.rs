use agdb_graph::StorageGraph;
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn edge_from_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();

    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index = graph.insert_edge(from, to).unwrap();

    assert_eq!(graph.edge(index).unwrap().index(), index);
}

#[test]
fn edge_from_index_missing() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let graph = StorageGraph::try_from(storage).unwrap();

    assert!(graph.edge(-3).is_none());
}

#[test]
fn edge_iteration() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();

    let edge1 = graph.insert_edge(node1, node2).unwrap();
    let edge2 = graph.insert_edge(node1, node2).unwrap();
    let edge3 = graph.insert_edge(node1, node2).unwrap();

    let mut actual = Vec::<i64>::new();

    for edge in graph.node(node1).unwrap().edge_from_iter() {
        actual.push(edge.index());
    }

    assert_eq!(actual, vec![edge3, edge2, edge1]);
}
