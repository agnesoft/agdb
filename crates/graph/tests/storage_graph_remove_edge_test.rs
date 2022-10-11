use agdb_graph::{GraphIndex, StorageGraph};
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn remove_edge_circular() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let node = graph.insert_node().unwrap();
    let index = graph.insert_edge(&node, &node).unwrap();

    graph.remove_edge(&index).unwrap();

    assert!(graph.edge(&index).is_none());
}

#[test]
fn remove_edge_first() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index1 = graph.insert_edge(&from, &to).unwrap();
    let index2 = graph.insert_edge(&from, &to).unwrap();
    let index3 = graph.insert_edge(&from, &to).unwrap();

    graph.remove_edge(&index3).unwrap();

    assert!(graph.edge(&index1).is_some());
    assert!(graph.edge(&index2).is_some());
    assert!(graph.edge(&index3).is_none());
}

#[test]
fn remove_edge_last() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index1 = graph.insert_edge(&from, &to).unwrap();
    let index2 = graph.insert_edge(&from, &to).unwrap();
    let index3 = graph.insert_edge(&from, &to).unwrap();

    graph.remove_edge(&index1).unwrap();

    assert!(graph.edge(&index1).is_none());
    assert!(graph.edge(&index2).is_some());
    assert!(graph.edge(&index3).is_some());
}

#[test]
fn remove_edge_middle() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index1 = graph.insert_edge(&from, &to).unwrap();
    let index2 = graph.insert_edge(&from, &to).unwrap();
    let index3 = graph.insert_edge(&from, &to).unwrap();

    graph.remove_edge(&index2).unwrap();

    assert!(graph.edge(&index1).is_some());
    assert!(graph.edge(&index2).is_none());
    assert!(graph.edge(&index3).is_some());
}

#[test]
fn remove_edge_missing() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    graph.remove_edge(&GraphIndex::from(-3)).unwrap();
}

#[test]
fn remove_edge_only() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index = graph.insert_edge(&from, &to).unwrap();

    graph.remove_edge(&index).unwrap();

    assert!(graph.edge(&index).is_none());
}
