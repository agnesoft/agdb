pub mod graph_data;
pub mod graph_impl;
pub mod graph_index;
pub mod graph_storage;

mod graph_data_memory;
mod graph_data_storage;
mod graph_data_storage_indexes;
mod graph_edge;
mod graph_edge_iterator;
mod graph_edge_reverse_iterator;
mod graph_node;
mod graph_node_iterator;

use self::graph_data_memory::GraphDataMemory;
use self::graph_impl::GraphImpl;

pub type Graph = GraphImpl<GraphDataMemory>;

impl Graph {
    pub fn new() -> Graph {
        Graph {
            data: GraphDataMemory::default(),
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::db_error::DbError;
    use crate::graph::graph_index::GraphIndex;

    #[test]
    fn derived_from_default() {
        let _graph = Graph::default();
    }

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
    fn index_from() {
        let mut graph = Graph::new();
        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();

        let index = graph.insert_edge(&node1, &node2).unwrap();

        assert_eq!(graph.edge(&index).unwrap().index_from(), node1);
    }

    #[test]
    fn index_to() {
        let mut graph = Graph::new();
        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();

        let index = graph.insert_edge(&node1, &node2).unwrap();

        assert_eq!(graph.edge(&index).unwrap().index_to(), node2);
    }

    #[test]
    fn insert_edge() {
        let mut graph = Graph::new();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();

        assert_eq!(graph.insert_edge(&from, &to), Ok(GraphIndex::from(-3_i64)));
    }

    #[test]
    fn insert_edge_after_removed() {
        let mut graph = Graph::new();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index = graph.insert_edge(&from, &to).unwrap();

        graph.remove_edge(&index).unwrap();

        assert_eq!(graph.insert_edge(&from, &to), Ok(index));
    }

    #[test]
    fn insert_edge_after_several_removed() {
        let mut graph = Graph::new();
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
        let mut graph = Graph::new();

        assert_eq!(
            graph.insert_edge(&GraphIndex::from(1), &GraphIndex::from(2)),
            Err(DbError::from("'1' is invalid index"))
        );
    }

    #[test]
    fn insert_edge_invalid_to() {
        let mut graph = Graph::new();
        let from = graph.insert_node().unwrap();

        assert_eq!(
            graph.insert_edge(&from, &GraphIndex::from(2)),
            Err(DbError::from("'2' is invalid index"))
        );
    }

    #[test]
    fn insert_node() {
        let mut graph = Graph::new();

        assert_eq!(graph.insert_node(), Ok(GraphIndex::from(1)));
    }

    #[test]
    fn insert_node_after_removal() {
        let mut graph = Graph::new();
        graph.insert_node().unwrap();
        let index = graph.insert_node().unwrap();
        graph.insert_node().unwrap();

        graph.remove_node(&index).unwrap();

        assert_eq!(graph.insert_node(), Ok(index));
    }

    #[test]
    fn node_count() {
        let mut graph = Graph::new();

        assert_eq!(graph.node_count().unwrap(), 0);

        graph.insert_node().unwrap();
        let index = graph.insert_node().unwrap();
        graph.insert_node().unwrap();

        assert_eq!(graph.node_count(), Ok(3));

        graph.remove_node(&index).unwrap();

        assert_eq!(graph.node_count(), Ok(2));
    }

    #[test]
    fn node_from_index() {
        let mut graph = Graph::new();
        let index = graph.insert_node().unwrap();

        assert_eq!(graph.node(&index).unwrap().index(), index);
    }

    #[test]
    fn node_from_index_missing() {
        let graph = Graph::new();

        let node = graph.node(&GraphIndex::from(1));

        assert!(node.is_none());
    }

    #[test]
    fn node_iteration() {
        let mut graph = Graph::new();
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
        let mut graph = Graph::new();
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
        let mut graph = Graph::new();
        let node = graph.insert_node().unwrap();
        let index = graph.insert_edge(&node, &node).unwrap();

        graph.remove_edge(&index).unwrap();

        assert!(graph.edge(&index).is_none());
    }

    #[test]
    fn remove_edge_first() {
        let mut graph = Graph::new();
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
        let mut graph = Graph::new();
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
        let mut graph = Graph::new();
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
        let mut graph = Graph::new();
        graph.remove_edge(&GraphIndex::from(-3)).unwrap();
    }

    #[test]
    fn remove_edge_only() {
        let mut graph = Graph::new();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index = graph.insert_edge(&from, &to).unwrap();

        graph.remove_edge(&index).unwrap();

        assert!(graph.edge(&index).is_none());
    }

    #[test]
    fn remove_node_circular_edge() {
        let mut graph = Graph::new();
        let index = graph.insert_node().unwrap();
        let edge = graph.insert_edge(&index, &index).unwrap();

        graph.remove_node(&index).unwrap();

        assert!(graph.node(&index).is_none());
        assert!(graph.edge(&edge).is_none());
    }

    #[test]
    fn remove_node_only() {
        let mut graph = Graph::new();
        let index = graph.insert_node().unwrap();

        graph.remove_node(&index).unwrap();

        assert!(graph.node(&index).is_none());
    }

    #[test]
    fn remove_node_missing() {
        let mut graph = Graph::new();
        graph.remove_node(&GraphIndex::from(1)).unwrap();
    }

    #[test]
    fn remove_nodes_with_edges() {
        let mut graph = Graph::new();

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
}
