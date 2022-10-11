use agdb_graph::GraphIndex;
use agdb_graph::StorageGraph;
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn insert_node() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();

    assert_eq!(graph.insert_node(), Ok(GraphIndex::from(1)));
}

#[test]
fn insert_node_after_removal() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    graph.insert_node().unwrap();
    let index = graph.insert_node().unwrap();
    graph.insert_node().unwrap();

    graph.remove_node(&index).unwrap();

    assert_eq!(graph.insert_node().unwrap(), index);
}

#[test]
fn node_count() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();

    assert_eq!(graph.node_count().unwrap(), 0);

    graph.insert_node().unwrap();
    let index = graph.insert_node().unwrap();
    graph.insert_node().unwrap();

    assert_eq!(graph.node_count().unwrap(), 3);

    graph.remove_node(&index).unwrap();

    assert_eq!(graph.node_count().unwrap(), 2);
}
