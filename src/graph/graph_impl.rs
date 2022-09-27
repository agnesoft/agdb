use super::graph_data::GraphData;
use super::graph_edge::GraphEdge;
use super::graph_node::GraphNode;
use super::graph_node_iterator::GraphNodeIterator;
use crate::DbError;

pub(crate) struct GraphImpl<Data: GraphData> {
    pub(super) data: Data,
}

#[allow(dead_code)]
impl<Data: GraphData> GraphImpl<Data> {
    pub(crate) fn edge(&self, index: i64) -> Option<GraphEdge<Data>> {
        if self.validate_edge(index).is_err() {
            return None;
        }

        Some(GraphEdge { graph: self, index })
    }

    pub(crate) fn node_count(&self) -> u64 {
        self.data.node_count()
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

    pub(crate) fn node(&self, index: i64) -> Option<GraphNode<Data>> {
        if self.validate_node(index).is_err() {
            return None;
        }

        Some(GraphNode { graph: self, index })
    }

    pub(crate) fn node_iter(&self) -> GraphNodeIterator<Data> {
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

    pub(super) fn first_edge_from(&self, index: i64) -> i64 {
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

            index
        } else {
            self.data.set_from_meta(0, self.data.from_meta(-index));

            -index
        }
    }

    fn invalid_index(index: i64) -> DbError {
        DbError::from(format!("'{}' is invalid index", index))
    }

    fn is_removed_index(&self, index: i64) -> bool {
        self.data.from_meta(index) < 0
    }

    fn is_valid_edge(&self, index: i64) -> bool {
        self.data.from(index) < 0
    }

    fn is_valid_index(&self, index: i64) -> bool {
        0 < index && (index as u64) < self.data.capacity() && !self.is_removed_index(index)
    }

    fn is_valid_node(&self, index: i64) -> bool {
        0 <= self.data.from(index)
    }

    pub(super) fn next_edge_from(&self, index: i64) -> i64 {
        -self.data.from_meta(-index)
    }

    pub(super) fn next_node(&self, index: i64) -> Option<i64> {
        ((index + 1)..(self.data.capacity() as i64))
            .find(|&index| self.is_valid_node(index) && !self.is_removed_index(index))
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
            let current = edge;
            edge = self.data.to_meta(edge);
            self.free_index(current);
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
