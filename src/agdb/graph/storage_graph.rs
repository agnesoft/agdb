use super::graph_data_storage::GraphDataStorage;
use super::graph_impl::GraphImpl;
use crate::db::db_error::DbError;
use crate::storage::storage_file::StorageFile;
use crate::storage::storage_index::StorageIndex;
use crate::storage::Storage;
use std::cell::RefCell;
use std::rc::Rc;

pub type StorageGraph<Data = StorageFile> = GraphImpl<GraphDataStorage<Data>>;

#[allow(dead_code)]
impl<Data> StorageGraph<Data>
where
    Data: Storage,
{
    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index.clone()
    }
}

impl<Data> TryFrom<Rc<RefCell<Data>>> for StorageGraph<Data>
where
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: Rc<RefCell<Data>>) -> Result<Self, Self::Error> {
        Ok(StorageGraph {
            data: GraphDataStorage::<Data>::try_from(storage)?,
        })
    }
}

impl<Data: Storage> TryFrom<(Rc<RefCell<Data>>, StorageIndex)> for StorageGraph<Data> {
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        Ok(StorageGraph {
            data: GraphDataStorage::<Data>::try_from(storage_with_index)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::graph_index::GraphIndex;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn edge_from_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut graph = StorageGraph::try_from(storage).unwrap();

        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index = graph.insert_edge(&from, &to).unwrap();

        assert_eq!(graph.edge(&index).unwrap().index(), index);
    }

    #[test]
    fn edge_from_index_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let graph = StorageGraph::try_from(storage).unwrap();

        assert!(graph.edge(&GraphIndex::from(-3)).is_none());
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

    #[test]
    fn node_from_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut graph = StorageGraph::try_from(storage).unwrap();
        let index = graph.insert_node().unwrap();

        assert_eq!(graph.node(&index).unwrap().index(), index);
    }

    #[test]
    fn node_from_index_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let graph = StorageGraph::try_from(storage).unwrap();

        let node = graph.node(&GraphIndex::from(1));

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
        let mut nodes = Vec::<GraphIndex>::new();

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

        graph.remove_node(&node2).unwrap();
        graph.remove_node(&node5).unwrap();

        let expected = vec![node1, node3, node4];
        let mut nodes = Vec::<GraphIndex>::new();

        for node in graph.node_iter() {
            nodes.push(node.index());
        }

        assert_eq!(nodes, expected);
    }

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

    #[test]
    fn remove_node_circular_edge() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut graph = StorageGraph::try_from(storage).unwrap();
        let index = graph.insert_node().unwrap();
        let edge = graph.insert_edge(&index, &index).unwrap();

        graph.remove_node(&index).unwrap();

        assert!(graph.node(&index).is_none());
        assert!(graph.edge(&edge).is_none());
    }

    #[test]
    fn remove_node_only() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut graph = StorageGraph::try_from(storage).unwrap();
        let index = graph.insert_node().unwrap();

        graph.remove_node(&index).unwrap();

        assert!(graph.node(&index).is_none());
    }

    #[test]
    fn remove_node_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut graph = StorageGraph::try_from(storage).unwrap();
        graph.remove_node(&GraphIndex::from(1)).unwrap();
    }

    #[test]
    fn remove_nodes_with_edges() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut graph = StorageGraph::try_from(storage).unwrap();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(&node1, &node2).unwrap();
        let edge2 = graph.insert_edge(&node1, &node1).unwrap();
        let edge3 = graph.insert_edge(&node1, &node3).unwrap();
        let edge4 = graph.insert_edge(&node2, &node1).unwrap();
        let edge5 = graph.insert_edge(&node3, &node1).unwrap();

        let edge6 = graph.insert_edge(&node3, &node2).unwrap();
        let edge7 = graph.insert_edge(&node2, &node3).unwrap();

        graph.remove_node(&node1).unwrap();

        assert!(graph.node(&node1).is_none());
        assert!(graph.edge(&edge1).is_none());
        assert!(graph.edge(&edge2).is_none());
        assert!(graph.edge(&edge3).is_none());
        assert!(graph.edge(&edge4).is_none());
        assert!(graph.edge(&edge5).is_none());

        assert!(graph.node(&node2).is_some());
        assert!(graph.node(&node3).is_some());
        assert!(graph.edge(&edge6).is_some());
        assert!(graph.edge(&edge7).is_some());
    }

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

            edge1 = graph.insert_edge(&node1, &node2).unwrap();
            edge2 = graph.insert_edge(&node2, &node3).unwrap();
            edge3 = graph.insert_edge(&node3, &node1).unwrap();
        }

        let graph = StorageGraph::try_from((storage, index)).unwrap();

        assert!(graph.node(&node1).is_some());
        assert!(graph.node(&node2).is_some());
        assert!(graph.node(&node3).is_some());
        assert!(graph.edge(&edge1).is_some());
        assert!(graph.edge(&edge2).is_some());
        assert!(graph.edge(&edge3).is_some());
    }
}
