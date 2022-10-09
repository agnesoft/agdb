use agdb_graph::StorageGraph;
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn restore_from_file() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let index;

    let node1;
    let node2;
    let node3;

    let edge1;
    let edge2;
    let edge3;

    {
        let mut graph = StorageGraph::try_from(storage.clone()).unwrap();

        index = graph.storage_index();

        node1 = graph.insert_node().unwrap();
        node2 = graph.insert_node().unwrap();
        node3 = graph.insert_node().unwrap();

        edge1 = graph.insert_edge(node1, node2).unwrap();
        edge2 = graph.insert_edge(node2, node3).unwrap();
        edge3 = graph.insert_edge(node3, node1).unwrap();
    }

    let graph = StorageGraph::try_from((storage, index)).unwrap();

    assert!(graph.node(node1).is_some());
    assert!(graph.node(node2).is_some());
    assert!(graph.node(node3).is_some());
    assert!(graph.edge(edge1).is_some());
    assert!(graph.edge(edge2).is_some());
    assert!(graph.edge(edge3).is_some());
}
