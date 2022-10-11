use agdb_db_error::DbError;
use agdb_graph::GraphIndex;
use agdb_graph::StorageGraph;
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn insert_edge() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();

    assert_eq!(graph.insert_edge(&from, &to), Ok(GraphIndex::from(-3_i64)));
}

#[test]
fn insert_edge_after_removed() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index = graph.insert_edge(&from, &to).unwrap();

    graph.remove_edge(&index).unwrap();

    assert_eq!(graph.insert_edge(&from, &to), Ok(index));
}

#[test]
fn insert_edge_after_several_removed() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let from = graph.insert_node().unwrap();
    let to = graph.insert_node().unwrap();
    let index1 = graph.insert_edge(&from, &to).unwrap();
    let index2 = graph.insert_edge(&from, &to).unwrap();
    graph.insert_edge(&from, &to).unwrap();

    graph.remove_edge(&index1).unwrap();
    graph.remove_edge(&index2).unwrap();

    assert_eq!(graph.insert_edge(&from, &to), Ok(index2));
}

#[test]
fn insert_edge_invalid_from() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();

    assert_eq!(
        graph.insert_edge(&GraphIndex::from(1), &GraphIndex::from(2)),
        Err(DbError::from("'1' is invalid index"))
    );
}

#[test]
fn insert_edge_invalid_to() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut graph = StorageGraph::try_from(storage).unwrap();
    let from = graph.insert_node().unwrap();

    assert_eq!(
        graph.insert_edge(&from, &GraphIndex::from(2)),
        Err(DbError::from("'2' is invalid index"))
    );
}
