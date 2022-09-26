use self::graph_data::GraphData;
use self::graph_edge::GraphEdge;
use self::graph_node::GraphNode;
use self::graph_node_iterator::GraphNodeIterator;
use crate::DbError;

mod graph_data;
mod graph_edge;
mod graph_edge_iterator;
mod graph_node;
mod graph_node_iterator;

pub(crate) struct Graph {
    data: GraphData,
}

#[allow(dead_code)]
impl Graph {
    pub(crate) fn new() -> Graph {
        Graph {
            data: GraphData {
                from: vec![0],
                to: vec![0],
                from_meta: vec![i64::MIN],
                to_meta: vec![0],
                node_count: 0,
            },
        }
    }

    pub(crate) fn edge(&self, index: i64) -> Option<GraphEdge> {
        if self.validate_edge(index).is_err() {
            return None;
        }

        Some(GraphEdge { graph: self, index })
    }

    pub(crate) fn insert_edge(&mut self, from: i64, to: i64) -> Result<i64, DbError> {
        self.validate_node(from)?;
        self.validate_node(to)?;

        let index = self.get_free_index();
        self.set_edge(index, from, to);

        Ok(-index)
    }

    pub(crate) fn insert_node(&mut self) -> i64 {
        let index = self.get_free_index();
        let count = self.data.node_count();
        self.data.set_node_count(count + 1);

        index
    }

    pub(crate) fn node(&self, index: i64) -> Option<GraphNode> {
        if self.validate_node(index).is_err() {
            return None;
        }

        Some(GraphNode { graph: self, index })
    }

    pub(crate) fn node_iter(&self) -> GraphNodeIterator {
        GraphNodeIterator {
            graph: self,
            index: 0,
        }
    }

    pub(crate) fn remove_edge(&mut self, index: i64) {
        if self.validate_edge(index).is_err() {
            return;
        }

        self.remove_from_edge(-index);
        self.remove_to_edge(-index);
        self.free_index(-index);
    }

    pub(crate) fn remove_node(&mut self, index: i64) {
        if self.validate_node(index).is_err() {
            return;
        }

        self.remove_from_edges(index);
        self.remove_to_edges(index);
        self.free_index(index);

        let count = self.data.node_count();
        self.data.set_node_count(count - 1);
    }

    fn first_edge_from(&self, index: i64) -> i64 {
        -self.data.from(index)
    }

    fn free_index(&mut self, index: i64) {
        let next_free = self.data.from_meta(0);
        self.data.set_from_meta(index, next_free);
        self.data.set_from_meta(0, -index);
    }

    fn get_free_index(&mut self) -> i64 {
        let mut index = self.data.free_index();

        if index == i64::MIN {
            index = self.data.capacity() as i64;
            self.data.resize((index + 1) as u64);
            return index;
        } else {
            self.data.set_from_meta(0, self.data.from_meta(-index));
            return -index;
        }
    }

    fn invalid_index(index: i64) -> DbError {
        DbError::from(format!("'{}' is invalid index", index))
    }

    fn is_valid_edge(&self, index: i64) -> bool {
        self.data.from(index) < 0
    }

    fn is_valid_index(&self, index: i64) -> bool {
        0 < index && (index as u64) < self.data.capacity() && 0 <= self.data.from_meta(index)
    }

    fn is_valid_node(&self, index: i64) -> bool {
        0 <= self.data.from(index)
    }

    fn next_edge_from(&self, index: i64) -> i64 {
        -self.data.from_meta(-index)
    }

    fn next_node(&self, index: i64) -> Option<i64> {
        for index in (index + 1)..(self.data.capacity() as i64) {
            if self.is_valid_node(index) {
                return Some(index);
            }
        }

        None
    }

    fn remove_from_edge(&mut self, index: i64) {
        let node = -self.data.from(index);
        let first = self.data.from(node);
        let next = self.data.from_meta(index);

        if first == index {
            self.data.set_from(node, next);
        } else {
            let mut previous = first;

            while self.data.from_meta(previous) != index {
                previous = self.data.from_meta(previous);
            }

            self.data.set_from_meta(previous, next);
        }

        let count = self.data.from_meta(node);
        self.data.set_from_meta(node, count - 1);
    }

    fn remove_from_edges(&mut self, index: i64) {
        let mut edge = self.data.from(index);

        while edge != 0 {
            self.remove_to_edge(edge);
            let current = edge;
            edge = self.data.from_meta(edge);
            self.free_index(current);
        }
    }

    fn remove_to_edge(&mut self, index: i64) {
        let node = -self.data.to(index);
        let first = self.data.to(node);
        let next = self.data.to_meta(index);

        if first == index {
            self.data.set_to(node, next);
        } else {
            let mut previous = first;

            while self.data.to_meta(previous) != index {
                previous = self.data.to_meta(previous);
            }

            self.data.set_to_meta(previous, next);
        }

        let count = self.data.to_meta(node);
        self.data.set_to_meta(node, count - 1);
    }

    fn remove_to_edges(&mut self, index: i64) {
        let mut edge = self.data.to(index);

        while edge != 0 {
            self.remove_from_edge(edge);
            self.free_index(index);
            edge = self.data.to_meta(edge);
        }
    }

    fn set_edge(&mut self, index: i64, from: i64, to: i64) {
        self.data.set_from(index, -from);
        self.data.set_to(index, -to);
        self.update_from_edge(from, index);
        self.update_to_edge(to, index);
    }

    fn update_from_edge(&mut self, node: i64, edge: i64) {
        let next = self.data.from(node);
        self.data.set_from_meta(edge, next);
        self.data.set_from(node, edge);

        let count = self.data.from_meta(node);
        self.data.set_from_meta(node, count + 1);
    }

    fn update_to_edge(&mut self, node: i64, edge: i64) {
        let next = self.data.to(node);
        self.data.set_to_meta(edge, next);
        self.data.set_to(node, edge);

        let count = self.data.to_meta(node);
        self.data.set_to_meta(node, count + 1);
    }

    fn validate_edge(&self, index: i64) -> Result<(), DbError> {
        if !self.is_valid_index(-index) || !self.is_valid_edge(-index) {
            return Err(Self::invalid_index(index));
        }

        Ok(())
    }

    fn validate_node(&self, index: i64) -> Result<(), DbError> {
        if !self.is_valid_index(index) || !self.is_valid_node(index) {
            return Err(Self::invalid_index(index));
        }

        Ok(())
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
            Err(DbError::from("'1' is invalid index"))
        );
    }

    #[test]
    fn insert_edge_invalid_to() {
        let mut graph = Graph::new();
        let from = graph.insert_node();

        assert_eq!(
            graph.insert_edge(from, 2),
            Err(DbError::from("'2' is invalid index"))
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
        graph.insert_node();
        let index = graph.insert_node();
        graph.insert_node();

        graph.remove_node(index);

        assert_eq!(graph.insert_node(), index);
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

    #[test]
    fn remove_only_node() {
        let mut graph = Graph::new();
        let index = graph.insert_node();

        graph.remove_node(index);

        assert!(graph.node(index).is_none());
    }

    #[test]
    fn remove_node_with_self_edge() {
        let mut graph = Graph::new();
        let index = graph.insert_node();
        let edge = graph.insert_edge(index, index).unwrap();

        graph.remove_node(index);

        assert!(graph.node(index).is_none());
        assert!(graph.edge(edge).is_none());
    }
}
