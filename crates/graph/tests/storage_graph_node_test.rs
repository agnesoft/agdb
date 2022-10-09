use agdb_graph::StorageGraph;
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn node_from_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let index = graph.insert_node().unwrap();

    assert_eq!(graph.node(index).unwrap().index(), index);
}

#[test]
fn node_from_index_missing() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let graph = StorageGraph::try_from(storage).unwrap();

    let node = graph.node(1);

    assert!(node.is_none());
}

#[test]
fn node_iteration() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();

    let expected = vec![node1, node2, node3];
    let mut nodes = Vec::<i64>::new();

    for node in graph.node_iter() {
        nodes.push(node.index());
    }

    assert_eq!(nodes, expected);
}

#[test]
fn node_iteration_with_removed_nodes() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let node1 = graph.insert_node().unwrap();
    let node2 = graph.insert_node().unwrap();
    let node3 = graph.insert_node().unwrap();
    let node4 = graph.insert_node().unwrap();
    let node5 = graph.insert_node().unwrap();

    graph.remove_node(node2).unwrap();
    graph.remove_node(node5).unwrap();

    let expected = vec![node1, node3, node4];
    let mut nodes = Vec::<i64>::new();

    for node in graph.node_iter() {
        nodes.push(node.index());
    }

    assert_eq!(nodes, expected);
}
