use super::graph_data::GraphData;
use super::graph_edge::GraphEdge;
use super::graph_index::GraphIndex;
use super::graph_node::GraphNode;
use super::graph_node_iterator::GraphNodeIterator;
use crate::db_error::DbError;

pub struct GraphImpl<Data>
where
    Data: GraphData,
{
    pub(crate) data: Data,
}

impl<Data> GraphImpl<Data>
where
    Data: GraphData,
{
    pub fn edge(&self, index: &GraphIndex) -> Option<GraphEdge<Data>> {
        if self.validate_edge(index).is_err() {
            return None;
        }

        Some(GraphEdge {
            graph: self,
            index: index.clone(),
        })
    }

    pub fn node_count(&self) -> Result<u64, DbError> {
        self.data.node_count()
    }

    pub fn insert_edge(
        &mut self,
        from: &GraphIndex,
        to: &GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        self.validate_node(from)?;
        self.validate_node(to)?;

        self.data.transaction();
        let index = GraphIndex::from(-self.get_free_index()?);
        self.set_edge(&index, from, to)?;
        self.data.commit()?;

        Ok(index)
    }

    pub fn insert_node(&mut self) -> Result<GraphIndex, DbError> {
        self.data.transaction();
        let index = GraphIndex::from(self.get_free_index()?);
        let count = self.data.node_count()?;
        self.data.set_node_count(count + 1)?;
        self.data.commit()?;

        Ok(index)
    }

    pub fn node(&self, index: &GraphIndex) -> Option<GraphNode<Data>> {
        if self.validate_node(index).is_err() {
            return None;
        }

        Some(GraphNode {
            graph: self,
            index: index.clone(),
        })
    }

    pub fn node_iter(&self) -> GraphNodeIterator<Data> {
        GraphNodeIterator {
            graph: self,
            index: GraphIndex::default(),
        }
    }

    pub fn remove_edge(&mut self, index: &GraphIndex) -> Result<(), DbError> {
        if self.validate_edge(index).is_err() {
            return Ok(());
        }

        self.data.transaction();
        self.remove_from_edge(index)?;
        self.remove_to_edge(index)?;
        self.free_index(&GraphIndex::from(-index.value()))?;

        self.data.commit()
    }

    pub fn remove_node(&mut self, index: &GraphIndex) -> Result<(), DbError> {
        if self.validate_node(index).is_err() {
            return Ok(());
        }

        self.data.transaction();
        self.remove_from_edges(index)?;
        self.remove_to_edges(index)?;
        self.free_index(index)?;

        let count = self.data.node_count()?;
        self.data.set_node_count(count - 1)?;

        self.data.commit()
    }

    pub fn first_edge_from(&self, index: &GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.from(index)?))
    }

    pub fn first_edge_to(&self, index: &GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.to(index)?))
    }

    pub(crate) fn edge_from(&self, index: &GraphIndex) -> GraphIndex {
        GraphIndex::from(-self.data.from(index).unwrap_or_default())
    }

    pub(crate) fn edge_to(&self, index: &GraphIndex) -> GraphIndex {
        GraphIndex::from(-self.data.to(index).unwrap_or_default())
    }

    fn free_index(&mut self, index: &GraphIndex) -> Result<(), DbError> {
        let next_free = self.data.from_meta(&GraphIndex::default())?;
        self.data.set_from_meta(index, next_free)?;
        self.data
            .set_from_meta(&GraphIndex::default(), -index.value())
    }

    fn get_free_index(&mut self) -> Result<i64, DbError> {
        let mut index = self.data.free_index()?;

        if index == i64::MIN {
            index = self.data.capacity()? as i64;
            self.data.grow()?;

            Ok(index)
        } else {
            let next = self.data.from_meta(&GraphIndex::from(-index))?;
            self.data.set_from_meta(&GraphIndex::default(), next)?;

            Ok(-index)
        }
    }

    fn invalid_index(index: &GraphIndex) -> DbError {
        DbError::from(format!("'{}' is invalid index", index.value()))
    }

    fn is_removed_index(&self, index: &GraphIndex) -> Result<bool, DbError> {
        Ok(self.data.from_meta(index)? < 0)
    }

    fn is_valid_edge(&self, index: &GraphIndex) -> Result<bool, DbError> {
        Ok(self.data.from(index)? < 0)
    }

    fn is_valid_index(&self, index: &GraphIndex) -> Result<bool, DbError> {
        Ok(index.is_valid()
            && index.as_u64() < self.data.capacity()?
            && !self.is_removed_index(index)?)
    }

    fn is_valid_node(&self, index: &GraphIndex) -> Result<bool, DbError> {
        Ok(0 <= self.data.from(index)?)
    }

    pub(crate) fn next_edge_from(&self, index: &GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.from_meta(index)?))
    }

    pub(crate) fn next_edge_to(&self, index: &GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.to_meta(index)?))
    }

    pub(crate) fn next_node(&self, index: &GraphIndex) -> Result<Option<GraphIndex>, DbError> {
        for i in (index.value() + 1)..(self.data.capacity()? as i64) {
            let next = GraphIndex::from(i);
            if self.is_valid_node(&next)? && !self.is_removed_index(&next)? {
                return Ok(Some(next));
            }
        }

        Ok(None)
    }

    fn remove_from_edge(&mut self, index: &GraphIndex) -> Result<(), DbError> {
        let node = GraphIndex::from(-self.data.from(index)?);
        let first = GraphIndex::from(-self.data.from(&node)?);
        let next = self.data.from_meta(index)?;

        if first == *index {
            self.data.set_from(&node, next)?;
        } else {
            let mut previous = first;

            while self.data.from_meta(&previous)? != -index.value() {
                previous = GraphIndex::from(self.data.from_meta(&previous)?);
            }

            self.data.set_from_meta(&previous, next)?;
        }

        let count = self.data.from_meta(&node)?;
        self.data.set_from_meta(&node, count - 1)
    }

    fn remove_from_edges(&mut self, index: &GraphIndex) -> Result<(), DbError> {
        let mut edge = GraphIndex::from(-self.data.from(index)?);

        while edge.is_valid() {
            self.remove_to_edge(&edge)?;
            let current_index = -edge.value();
            edge = GraphIndex::from(-self.data.from_meta(&edge)?);
            self.free_index(&GraphIndex::from(current_index))?;
        }

        Ok(())
    }

    fn remove_to_edge(&mut self, index: &GraphIndex) -> Result<(), DbError> {
        let node = GraphIndex::from(-self.data.to(index)?);
        let first = GraphIndex::from(-self.data.to(&node)?);
        let next = self.data.to_meta(index)?;

        if first == *index {
            self.data.set_to(&node, next)?;
        } else {
            let mut previous = first;

            while self.data.to_meta(&previous)? != -index.value() {
                previous = GraphIndex::from(self.data.to_meta(&previous)?);
            }

            self.data.set_to_meta(&previous, next)?;
        }

        let count = self.data.to_meta(&node)?;
        self.data.set_to_meta(&node, count - 1)
    }

    fn remove_to_edges(&mut self, index: &GraphIndex) -> Result<(), DbError> {
        let mut edge = GraphIndex::from(-self.data.to(index)?);

        while edge.is_valid() {
            self.remove_from_edge(&edge)?;
            let current_index = -edge.value();
            edge = GraphIndex::from(-self.data.to_meta(&edge)?);
            self.free_index(&GraphIndex::from(current_index))?;
        }

        Ok(())
    }

    fn set_edge(
        &mut self,
        index: &GraphIndex,
        from: &GraphIndex,
        to: &GraphIndex,
    ) -> Result<(), DbError> {
        self.data.set_from(index, -from.value())?;
        self.data.set_to(index, -to.value())?;
        self.update_from_edge(from, index)?;
        self.update_to_edge(to, index)
    }

    fn update_from_edge(&mut self, node: &GraphIndex, edge: &GraphIndex) -> Result<(), DbError> {
        let next = self.data.from(node)?;
        self.data.set_from_meta(edge, next)?;
        self.data.set_from(node, -edge.value())?;

        let count = self.data.from_meta(node)?;
        self.data.set_from_meta(node, count + 1)
    }

    fn update_to_edge(&mut self, node: &GraphIndex, edge: &GraphIndex) -> Result<(), DbError> {
        let next = self.data.to(node)?;
        self.data.set_to_meta(edge, next)?;
        self.data.set_to(node, -edge.value())?;

        let count = self.data.to_meta(node)?;
        self.data.set_to_meta(node, count + 1)
    }

    fn validate_edge(&self, index: &GraphIndex) -> Result<(), DbError> {
        if !self.is_valid_index(index)? || !self.is_valid_edge(index)? {
            return Err(Self::invalid_index(index));
        }

        Ok(())
    }

    fn validate_node(&self, index: &GraphIndex) -> Result<(), DbError> {
        if !self.is_valid_index(index)? || !self.is_valid_node(index)? {
            return Err(Self::invalid_index(index));
        }

        Ok(())
    }
}
