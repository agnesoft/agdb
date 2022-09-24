use self::graph_edge::GraphEdge;
use self::graph_impl::GraphImpl;
use self::graph_node::GraphNode;
use self::graph_node_iterator::GraphNodeIterator;
use crate::DbError;

mod graph_edge;
mod graph_edge_iterator;
mod graph_impl;
mod graph_node;
mod graph_node_iterator;

pub(crate) struct Graph {
    data: GraphImpl,
}

#[allow(dead_code)]
impl Graph {
    pub(crate) fn new() -> Graph {
        Graph {
            data: GraphImpl {
                from: vec![0],
                to: vec![0],
                from_meta: vec![i64::MAX],
                to_meta: vec![0],
                node_count: 0,
            },
        }
    }

    pub(crate) fn edge(&self, index: i64) -> Option<GraphEdge> {
        if self.validate_edge(index).is_err() {
            return None;
        }

        Some(GraphEdge {
            graph: &self.data,
            index,
        })
    }

    pub(crate) fn insert_edge(&mut self, from: i64, to: i64) -> Result<i64, DbError> {
        self.validate_node(from)?;
        self.validate_node(to)?;

        let index;
        let first_from = self.data.from[from as usize];
        let first_to = self.data.to[to as usize];

        if let Some(free_index) = self.free_index() {
            index = free_index;
            self.data.from[index as usize] = from;
            self.data.to[index as usize] = to;
            self.data.from_meta[index as usize] = first_from;
            self.data.to_meta[index as usize] = first_to;
        } else {
            index = self.data.from.len() as i64;
            self.data.from.push(from);
            self.data.to.push(to);
            self.data.from_meta.push(first_from);
            self.data.to_meta.push(first_to)
        }

        self.data.from[from as usize] = -index;
        self.data.from_meta[from as usize] -= 1;
        self.data.to[to as usize] = -index;
        self.data.to_meta[to as usize] -= 1;

        Ok(-index)
    }

    pub(crate) fn insert_node(&mut self) -> i64 {
        let index;

        if let Some(free_index) = self.free_index() {
            index = free_index;
            self.data.from[index as usize] = 0;
            self.data.to[index as usize] = 0;
            self.data.from_meta[index as usize] = 0;
            self.data.to_meta[index as usize] = 0;
        } else {
            index = self.data.from.len() as i64;
            self.data.from.push(0);
            self.data.to.push(0);
            self.data.from_meta.push(0);
            self.data.to_meta.push(0);
        }

        self.data.node_count += 1;

        index
    }

    pub(crate) fn node(&self, index: i64) -> Option<GraphNode> {
        if self.validate_node(index).is_err() {
            return None;
        }

        Some(GraphNode {
            graph: &self.data,
            index,
        })
    }

    pub(crate) fn node_iter(&self) -> GraphNodeIterator {
        GraphNodeIterator {
            graph: &self.data,
            index: 0,
        }
    }

    pub(crate) fn remove_edge(&mut self, edge: i64) {
        if self.validate_edge(edge).is_err() {
            return;
        }

        {
            let from = self.data.from[(-edge) as usize];
            let mut previous = self.data.from[from as usize];
            let next = self.data.from_meta[(-edge) as usize];

            if previous == edge {
                self.data.from[from as usize] = next;
            } else {
                while self.data.from_meta[(-previous) as usize] != edge {
                    previous = self.data.from_meta[(-previous) as usize];
                }

                self.data.from_meta[(-previous) as usize] = next;
            }

            self.data.from_meta[from as usize] += 1;

            self.data.from_meta[(-edge) as usize] = self.data.from_meta[0];
            self.data.from_meta[0] = (-edge) as usize as i64;
        }

        {
            let to = self.data.to[(-edge) as usize];
            let mut previous = self.data.to[to as usize];
            let next = self.data.to_meta[(-edge) as usize];

            if previous == edge {
                self.data.to[to as usize] = next;
            } else {
                while self.data.to_meta[(-previous) as usize] != edge {
                    previous = self.data.to_meta[(-previous) as usize];
                }

                self.data.to_meta[(-previous) as usize] = next;
            }

            self.data.to_meta[to as usize] += 1;
            self.data.to_meta[(-edge) as usize] = 0;
        }
    }

    fn free_index(&mut self) -> Option<i64> {
        let index = self.data.from_meta[0];

        if index != i64::MAX {
            let next = self.data.from_meta[index as usize];
            self.data.from_meta[0] = next;
            return Some(index);
        }

        None
    }

    fn validate_edge(&self, index: i64) -> Result<(), DbError> {
        if let Some(meta) = self.data.from_meta.get((-index) as usize) {
            if *meta <= 0 {
                return Ok(());
            }
        }

        Err(DbError::from(format!("'{}' is not a valid edge", index)))
    }

    fn validate_node(&self, index: i64) -> Result<(), DbError> {
        if let Some(meta) = self.data.from_meta.get(index as usize) {
            if *meta <= 0 {
                return Ok(());
            }
        }

        Err(DbError::from(format!("'{}' is not a valid node", index)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_iteration() {
        let mut graph = Graph::new();
        let node1 = graph.insert_node();
        let node2 = graph.insert_node();

        let edge1 = graph.insert_edge(node1, node2).unwrap();
        let edge2 = graph.insert_edge(node1, node2).unwrap();
        let edge3 = graph.insert_edge(node1, node2).unwrap();

        let mut actual = Vec::<i64>::new();

        for edge in graph.node(node1).unwrap().edge_from_iter() {
            actual.push(edge.index());
        }

        assert_eq!(actual, vec![edge3, edge2, edge1]);
    }

    #[test]
    fn edge_from_index() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();
        let index = graph.insert_edge(from, to).unwrap();

        assert_eq!(graph.edge(index).unwrap().index(), index);
    }

    #[test]
    fn edge_from_index_missing() {
        let graph = Graph::new();

        assert!(graph.edge(-3).is_none());
    }

    #[test]
    fn insert_edge() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();

        assert_eq!(graph.insert_edge(from, to), Ok(-3_i64));
    }

    #[test]
    fn insert_edge_after_removed() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();
        let index = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index);

        assert_eq!(graph.insert_edge(from, to).unwrap(), index);
    }

    #[test]
    fn insert_edge_after_several_removed() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();
        let index1 = graph.insert_edge(from, to).unwrap();
        let index2 = graph.insert_edge(from, to).unwrap();
        graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index1);
        graph.remove_edge(index2);

        assert_eq!(graph.insert_edge(from, to).unwrap(), index2);
    }

    #[test]
    fn insert_edge_invalid_from() {
        let mut graph = Graph::new();

        assert_eq!(
            graph.insert_edge(1, 2),
            Err(DbError::from("'1' is not a valid node"))
        );
    }

    #[test]
    fn insert_edge_invalid_to() {
        let mut graph = Graph::new();
        let from = graph.insert_node();

        assert_eq!(
            graph.insert_edge(from, 2),
            Err(DbError::from("'2' is not a valid node"))
        );
    }

    #[test]
    fn insert_node() {
        let mut graph = Graph::new();

        assert_eq!(graph.insert_node(), 1);
    }

    #[test]
    fn insert_node_after_removal() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();
        let index = graph.insert_edge(from, to).unwrap();
        graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index);

        assert_eq!(graph.insert_node(), (-index));
    }

    #[test]
    fn node_from_index() {
        let mut graph = Graph::new();
        let index = graph.insert_node();

        assert_eq!(graph.node(index).unwrap().index(), index);
    }

    #[test]
    fn node_from_index_missing() {
        let graph = Graph::new();

        let node = graph.node(1);

        assert!(node.is_none());
    }

    #[test]
    fn node_iteration() {
        let mut graph = Graph::new();
        let node1 = graph.insert_node();
        let node2 = graph.insert_node();
        let node3 = graph.insert_node();

        let expected = vec![node1, node2, node3];
        let mut nodes = Vec::<i64>::new();

        for node in graph.node_iter() {
            nodes.push(node.index());
        }

        assert_eq!(nodes, expected);
    }

    #[test]
    fn remove_only_edge() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();
        let index = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index);

        assert!(graph.edge(index).is_none());
    }

    #[test]
    fn remove_first_edge() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();
        let index1 = graph.insert_edge(from, to).unwrap();
        let index2 = graph.insert_edge(from, to).unwrap();
        let index3 = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index3);

        assert!(graph.edge(index1).is_some());
        assert!(graph.edge(index2).is_some());
        assert!(graph.edge(index3).is_none());
    }

    #[test]
    fn remove_last_edge() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();
        let index1 = graph.insert_edge(from, to).unwrap();
        let index2 = graph.insert_edge(from, to).unwrap();
        let index3 = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index1);

        assert!(graph.edge(index1).is_none());
        assert!(graph.edge(index2).is_some());
        assert!(graph.edge(index3).is_some());
    }

    #[test]
    fn remove_middle_edge() {
        let mut graph = Graph::new();
        let from = graph.insert_node();
        let to = graph.insert_node();
        let index1 = graph.insert_edge(from, to).unwrap();
        let index2 = graph.insert_edge(from, to).unwrap();
        let index3 = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index2);

        assert!(graph.edge(index1).is_some());
        assert!(graph.edge(index2).is_none());
        assert!(graph.edge(index3).is_some());
    }
    #[test]
    fn remove_missing_edge() {
        let mut graph = Graph::new();
        graph.remove_edge(-3);
    }

    #[test]
    fn remove_self_edge() {
        let mut graph = Graph::new();
        let node = graph.insert_node();
        let index = graph.insert_edge(node, node).unwrap();

        graph.remove_edge(index);

        assert!(graph.edge(index).is_none());
    }
}
