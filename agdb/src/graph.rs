use crate::collections::vec::DbVec;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use crate::StorageData;
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, Hash, PartialEq, PartialOrd)]
pub struct GraphIndex(pub i64);

impl GraphIndex {
    pub fn is_edge(&self) -> bool {
        self.0 < 0
    }

    pub fn is_node(&self) -> bool {
        0 < self.0
    }

    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }

    pub fn as_u64(&self) -> u64 {
        if self.is_edge() {
            (-self.0) as u64
        } else {
            self.0 as u64
        }
    }
}

impl From<i64> for GraphIndex {
    fn from(index: i64) -> Self {
        Self(index)
    }
}

impl StableHash for GraphIndex {
    fn stable_hash(&self) -> u64 {
        self.0.stable_hash()
    }
}

pub trait GraphData<D: StorageData> {
    fn capacity(&self) -> Result<u64, DbError>;
    fn commit(&mut self, storage: &mut Storage<D>, id: u64) -> Result<(), DbError>;
    fn free_index(&self, storage: &Storage<D>) -> Result<i64, DbError>;
    fn from(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError>;
    #[allow(clippy::wrong_self_convention)]
    fn from_meta(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError>;
    fn grow(&mut self, storage: &mut Storage<D>) -> Result<(), DbError>;
    fn node_count(&self, storage: &Storage<D>) -> Result<u64, DbError>;
    fn set_from(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        value: i64,
    ) -> Result<(), DbError>;
    fn set_from_meta(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        value: i64,
    ) -> Result<(), DbError>;
    fn set_node_count(&mut self, storage: &mut Storage<D>, count: u64) -> Result<(), DbError>;
    fn set_to(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        value: i64,
    ) -> Result<(), DbError>;
    fn set_to_meta(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        value: i64,
    ) -> Result<(), DbError>;
    fn to(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError>;
    fn to_meta(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError>;
    fn transaction(&mut self, storage: &mut Storage<D>) -> u64;
}

pub struct GraphDataStorageIndexes {
    from: StorageIndex,
    to: StorageIndex,
    from_meta: StorageIndex,
    to_meta: StorageIndex,
}

impl Serialize for GraphDataStorageIndexes {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(GraphDataStorageIndexes {
            from: StorageIndex::deserialize(bytes)?,
            to: StorageIndex::deserialize(
                &bytes[(StorageIndex::serialized_size_static() as usize)..],
            )?,
            from_meta: StorageIndex::deserialize(
                &bytes[(StorageIndex::serialized_size_static() as usize * 2)..],
            )?,
            to_meta: StorageIndex::deserialize(
                &bytes[(StorageIndex::serialized_size_static() as usize * 3)..],
            )?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 * StorageIndex::serialized_size_static() as usize);
        bytes.extend(self.from.serialize());
        bytes.extend(self.to.serialize());
        bytes.extend(self.from_meta.serialize());
        bytes.extend(self.to_meta.serialize());

        bytes
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl SerializeStatic for GraphDataStorageIndexes {}

pub struct GraphDataStorage<D>
where
    D: StorageData,
{
    storage: PhantomData<D>,
    storage_index: StorageIndex,
    from: DbVec<i64, D>,
    to: DbVec<i64, D>,
    from_meta: DbVec<i64, D>,
    to_meta: DbVec<i64, D>,
}

impl<D> GraphDataStorage<D>
where
    D: StorageData,
{
    pub fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        let id = storage.transaction();

        let mut from = DbVec::<i64, D>::new(storage)?;
        from.push(storage, &0)?;
        let mut to = DbVec::<i64, D>::new(storage)?;
        to.push(storage, &0)?;
        let mut from_meta = DbVec::<i64, D>::new(storage)?;
        from_meta.push(storage, &i64::MIN)?;
        let mut to_meta = DbVec::<i64, D>::new(storage)?;
        to_meta.push(storage, &0)?;

        let indexes = GraphDataStorageIndexes {
            from: from.storage_index(),
            to: to.storage_index(),
            from_meta: from_meta.storage_index(),
            to_meta: to_meta.storage_index(),
        };

        let index = storage.insert(&indexes)?;

        storage.commit(id)?;

        Ok(GraphDataStorage::<D> {
            storage: PhantomData,
            storage_index: index,
            from,
            to,
            from_meta,
            to_meta,
        })
    }

    pub fn from_storage(
        storage: &Storage<D>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let indexes = storage.value::<GraphDataStorageIndexes>(storage_index)?;

        let from = DbVec::<i64, D>::from_storage(storage, indexes.from)?;
        let to = DbVec::<i64, D>::from_storage(storage, indexes.to)?;
        let from_meta = DbVec::<i64, D>::from_storage(storage, indexes.from_meta)?;
        let to_meta = DbVec::<i64, D>::from_storage(storage, indexes.to_meta)?;

        Ok(GraphDataStorage::<D> {
            storage: PhantomData,
            storage_index,
            from,
            to,
            from_meta,
            to_meta,
        })
    }
}

impl<D> GraphData<D> for GraphDataStorage<D>
where
    D: StorageData,
{
    fn capacity(&self) -> Result<u64, DbError> {
        Ok(self.from.len())
    }

    fn commit(&mut self, storage: &mut Storage<D>, id: u64) -> Result<(), DbError> {
        storage.commit(id)
    }

    fn free_index(&self, storage: &Storage<D>) -> Result<i64, DbError> {
        self.from_meta.value(storage, 0)
    }

    fn from(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError> {
        self.from.value(storage, index.as_u64())
    }

    fn from_meta(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError> {
        self.from_meta.value(storage, index.as_u64())
    }

    fn grow(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        self.from.push(storage, &0)?;
        self.to.push(storage, &0)?;
        self.from_meta.push(storage, &0)?;
        self.to_meta.push(storage, &0)
    }

    fn node_count(&self, storage: &Storage<D>) -> Result<u64, DbError> {
        Ok(self.to_meta.value(storage, 0)? as u64)
    }

    fn set_from(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        value: i64,
    ) -> Result<(), DbError> {
        self.from.replace(storage, index.as_u64(), &value)?;
        Ok(())
    }

    fn set_from_meta(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        value: i64,
    ) -> Result<(), DbError> {
        self.from_meta.replace(storage, index.as_u64(), &value)?;
        Ok(())
    }

    fn set_node_count(&mut self, storage: &mut Storage<D>, count: u64) -> Result<(), DbError> {
        self.to_meta.replace(storage, 0, &(count as i64))?;
        Ok(())
    }

    fn set_to(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        value: i64,
    ) -> Result<(), DbError> {
        self.to.replace(storage, index.as_u64(), &value)?;
        Ok(())
    }

    fn set_to_meta(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        value: i64,
    ) -> Result<(), DbError> {
        self.to_meta.replace(storage, index.as_u64(), &value)?;
        Ok(())
    }

    fn to(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError> {
        self.to.value(storage, index.as_u64())
    }

    fn to_meta(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError> {
        self.to_meta.value(storage, index.as_u64())
    }

    fn transaction(&mut self, storage: &mut Storage<D>) -> u64 {
        storage.transaction()
    }
}

pub struct GraphNode<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    graph: &'a GraphImpl<D, Data>,
    index: GraphIndex,
    storage: &'a Storage<D>,
}

impl<'a, D, Data> GraphNode<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    #[allow(dead_code)]
    pub fn index(&self) -> GraphIndex {
        self.index
    }

    pub fn edge_iter_from(&'a self) -> GraphEdgeIterator<D, Data> {
        GraphEdgeIterator {
            graph: self.graph,
            index: self
                .graph
                .first_edge_from(self.storage, self.index)
                .unwrap_or_default(),
            storage: self.storage,
        }
    }

    pub fn edge_iter_to(&self) -> GraphEdgeReverseIterator<D, Data> {
        GraphEdgeReverseIterator {
            graph: self.graph,
            index: self
                .graph
                .first_edge_to(self.storage, self.index)
                .unwrap_or_default(),
            storage: self.storage,
        }
    }

    pub fn edge_count(&self) -> u64 {
        self.edge_count_from() + self.edge_count_to()
    }

    pub fn edge_count_from(&self) -> u64 {
        self.graph
            .edge_count_from(self.storage, self.index)
            .unwrap_or_default() as u64
    }

    pub fn edge_count_to(&self) -> u64 {
        self.graph
            .edge_count_to(self.storage, self.index)
            .unwrap_or_default() as u64
    }
}

pub struct GraphNodeIterator<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    graph: &'a GraphImpl<D, Data>,
    index: GraphIndex,
    storage: &'a Storage<D>,
}

impl<'a, D, Data> Iterator for GraphNodeIterator<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    type Item = GraphNode<'a, D, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self
            .graph
            .next_node(self.storage, self.index)
            .unwrap_or(None)
        {
            self.index = next;
            return Some(GraphNode {
                graph: self.graph,
                index: self.index,
                storage: self.storage,
            });
        }

        None
    }
}

pub struct GraphEdge<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    graph: &'a GraphImpl<D, Data>,
    index: GraphIndex,
    storage: &'a Storage<D>,
}

impl<'a, D, Data> GraphEdge<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    pub fn index(&self) -> GraphIndex {
        self.index
    }

    pub fn index_from(&self) -> GraphIndex {
        self.graph.edge_from(self.storage, self.index)
    }

    pub fn index_to(&self) -> GraphIndex {
        self.graph.edge_to(self.storage, self.index)
    }
}

pub struct GraphEdgeIterator<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    graph: &'a GraphImpl<D, Data>,
    index: GraphIndex,
    storage: &'a Storage<D>,
}

impl<'a, D, Data> Iterator for GraphEdgeIterator<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    type Item = GraphEdge<'a, D, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.index.is_valid() {
            return None;
        }

        let current_index = self.index;

        self.index = self
            .graph
            .next_edge_from(self.storage, self.index)
            .unwrap_or_default();

        Some(GraphEdge {
            graph: self.graph,
            index: current_index,
            storage: self.storage,
        })
    }
}

pub struct GraphEdgeReverseIterator<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    graph: &'a GraphImpl<D, Data>,
    index: GraphIndex,
    storage: &'a Storage<D>,
}

impl<'a, D, Data> Iterator for GraphEdgeReverseIterator<'a, D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    type Item = GraphEdge<'a, D, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.index.is_valid() {
            return None;
        }

        let current_index = self.index;

        self.index = self
            .graph
            .next_edge_to(self.storage, self.index)
            .unwrap_or_default();

        Some(GraphEdge {
            graph: self.graph,
            index: current_index,
            storage: self.storage,
        })
    }
}

pub struct GraphImpl<D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    data: Data,
    storage: PhantomData<D>,
}

impl<D, Data> GraphImpl<D, Data>
where
    Data: GraphData<D>,
    D: StorageData,
{
    pub fn edge<'a>(
        &'a self,
        storage: &'a Storage<D>,
        index: GraphIndex,
    ) -> Option<GraphEdge<D, Data>> {
        if self.validate_edge(storage, index).is_err() {
            return None;
        }

        Some(GraphEdge {
            graph: self,
            index,
            storage,
        })
    }

    pub fn node_count(&self, storage: &Storage<D>) -> Result<u64, DbError> {
        self.data.node_count(storage)
    }

    pub fn insert_edge(
        &mut self,
        storage: &mut Storage<D>,
        from: GraphIndex,
        to: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        self.validate_node(storage, from)?;
        self.validate_node(storage, to)?;

        let id = self.data.transaction(storage);
        let index = GraphIndex::from(-self.get_free_index(storage)?);
        self.set_edge(storage, index, from, to)?;
        self.data.commit(storage, id)?;

        Ok(index)
    }

    pub fn insert_node(&mut self, storage: &mut Storage<D>) -> Result<GraphIndex, DbError> {
        let id = self.data.transaction(storage);
        let index = GraphIndex::from(self.get_free_index(storage)?);
        let count = self.data.node_count(storage)?;

        self.data.set_node_count(storage, count + 1)?;
        self.data.commit(storage, id)?;

        Ok(index)
    }

    pub fn node<'a>(
        &'a self,
        storage: &'a Storage<D>,
        index: GraphIndex,
    ) -> Option<GraphNode<D, Data>> {
        if self.validate_node(storage, index).is_err() {
            return None;
        }

        Some(GraphNode {
            graph: self,
            index,
            storage,
        })
    }

    #[allow(dead_code)]
    pub fn node_iter<'a>(&'a self, storage: &'a Storage<D>) -> GraphNodeIterator<D, Data> {
        GraphNodeIterator {
            graph: self,
            index: GraphIndex::default(),
            storage,
        }
    }

    pub fn remove_edge(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        if self.validate_edge(storage, index).is_err() {
            return Ok(());
        }

        let id = self.data.transaction(storage);
        self.remove_from_edge(storage, index)?;
        self.remove_to_edge(storage, index)?;
        self.free_index(storage, GraphIndex::from(-index.0))?;

        self.data.commit(storage, id)
    }

    pub fn remove_node(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        if self.validate_node(storage, index).is_err() {
            return Ok(());
        }

        let id = self.data.transaction(storage);
        self.remove_from_edges(storage, index)?;
        self.remove_to_edges(storage, index)?;
        self.free_index(storage, index)?;

        let count = self.data.node_count(storage)?;
        self.data.set_node_count(storage, count - 1)?;

        self.data.commit(storage, id)
    }

    pub fn first_edge_from(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.from(storage, index)?))
    }

    pub fn first_edge_to(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.to(storage, index)?))
    }

    pub fn edge_from(&self, storage: &Storage<D>, index: GraphIndex) -> GraphIndex {
        GraphIndex::from(-self.data.from(storage, index).unwrap_or_default())
    }

    pub fn edge_to(&self, storage: &Storage<D>, index: GraphIndex) -> GraphIndex {
        GraphIndex::from(-self.data.to(storage, index).unwrap_or_default())
    }

    fn free_index(&mut self, storage: &mut Storage<D>, index: GraphIndex) -> Result<(), DbError> {
        let next_free = self.data.from_meta(storage, GraphIndex::default())?;
        self.data.set_from_meta(storage, index, next_free)?;
        self.data
            .set_from_meta(storage, GraphIndex::default(), -index.0)?;
        self.data.set_from(storage, index, 0)?;
        self.data.set_to(storage, index, 0)?;
        self.data.set_to_meta(storage, index, 0)
    }

    fn get_free_index(&mut self, storage: &mut Storage<D>) -> Result<i64, DbError> {
        let mut index = self.data.free_index(storage)?;

        if index == i64::MIN {
            index = self.data.capacity()? as i64;
            self.data.grow(storage)?;

            Ok(index)
        } else {
            let next = self.data.from_meta(storage, GraphIndex::from(-index))?;
            self.data
                .set_from_meta(storage, GraphIndex::default(), next)?;
            self.data
                .set_from_meta(storage, GraphIndex::from(-index), 0)?;

            Ok(-index)
        }
    }

    fn invalid_index(index: GraphIndex) -> DbError {
        DbError::from(format!("'{}' is invalid index", index.0))
    }

    fn is_removed_index(&self, storage: &Storage<D>, index: GraphIndex) -> Result<bool, DbError> {
        Ok(self.data.from_meta(storage, index)? < 0)
    }

    fn is_valid_edge(&self, storage: &Storage<D>, index: GraphIndex) -> Result<bool, DbError> {
        Ok(self.data.from(storage, index)? < 0)
    }

    fn is_valid_index(&self, storage: &Storage<D>, index: GraphIndex) -> Result<bool, DbError> {
        Ok(index.is_valid()
            && index.as_u64() < self.data.capacity()?
            && !self.is_removed_index(storage, index)?)
    }

    fn is_valid_node(&self, storage: &Storage<D>, index: GraphIndex) -> Result<bool, DbError> {
        Ok(0 <= self.data.from(storage, index)?)
    }

    fn next_edge_from(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.from_meta(storage, index)?))
    }

    fn next_edge_to(&self, storage: &Storage<D>, index: GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.to_meta(storage, index)?))
    }

    fn edge_count_from(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError> {
        self.data.from_meta(storage, index)
    }

    fn edge_count_to(&self, storage: &Storage<D>, index: GraphIndex) -> Result<i64, DbError> {
        self.data.to_meta(storage, index)
    }

    fn next_node(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<Option<GraphIndex>, DbError> {
        for i in (index.0 + 1)..(self.data.capacity()? as i64) {
            let next = GraphIndex::from(i);
            if self.is_valid_node(storage, next)? && !self.is_removed_index(storage, next)? {
                return Ok(Some(next));
            }
        }

        Ok(None)
    }

    fn remove_from_edge(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        let node_index = GraphIndex::from(-self.data.from(storage, index)?);
        let first_index = GraphIndex::from(-self.data.from(storage, node_index)?);
        let next = self.data.from_meta(storage, index)?;

        if first_index == index {
            self.data.set_from(storage, node_index, next)?;
        } else {
            let mut previous = first_index;

            while self.data.from_meta(storage, previous)? != -index.0 {
                previous = GraphIndex::from(self.data.from_meta(storage, previous)?);
            }

            self.data.set_from_meta(storage, previous, next)?;
        }

        let count = self.data.from_meta(storage, node_index)?;
        self.data.set_from_meta(storage, node_index, count - 1)
    }

    fn remove_from_edges(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        let mut edge = GraphIndex::from(-self.data.from(storage, index)?);

        while edge.is_valid() {
            self.remove_to_edge(storage, edge)?;
            let current_index = -edge.0;
            edge = GraphIndex::from(-self.data.from_meta(storage, edge)?);
            self.free_index(storage, GraphIndex::from(current_index))?;
        }

        Ok(())
    }

    fn remove_to_edge(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        let node_index = GraphIndex::from(-self.data.to(storage, index)?);
        let first_index = GraphIndex::from(-self.data.to(storage, node_index)?);
        let next = self.data.to_meta(storage, index)?;

        if first_index == index {
            self.data.set_to(storage, node_index, next)?;
        } else {
            let mut previous = first_index;

            while self.data.to_meta(storage, previous)? != -index.0 {
                previous = GraphIndex::from(self.data.to_meta(storage, previous)?);
            }

            self.data.set_to_meta(storage, previous, next)?;
        }

        let count = self.data.to_meta(storage, node_index)?;
        self.data.set_to_meta(storage, node_index, count - 1)
    }

    fn remove_to_edges(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        let mut edge_index = GraphIndex::from(-self.data.to(storage, index)?);

        while edge_index.is_valid() {
            self.remove_from_edge(storage, edge_index)?;
            let current_index = -edge_index.0;
            edge_index = GraphIndex::from(-self.data.to_meta(storage, edge_index)?);
            self.free_index(storage, GraphIndex::from(current_index))?;
        }

        Ok(())
    }

    fn set_edge(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        from: GraphIndex,
        to: GraphIndex,
    ) -> Result<(), DbError> {
        self.data.set_from(storage, index, -from.0)?;
        self.data.set_to(storage, index, -to.0)?;
        self.update_from_edge(storage, from, index)?;
        self.update_to_edge(storage, to, index)
    }

    fn update_from_edge(
        &mut self,
        storage: &mut Storage<D>,
        node: GraphIndex,
        edge: GraphIndex,
    ) -> Result<(), DbError> {
        let next = self.data.from(storage, node)?;
        self.data.set_from_meta(storage, edge, next)?;
        self.data.set_from(storage, node, -edge.0)?;

        let count = self.data.from_meta(storage, node)?;
        self.data.set_from_meta(storage, node, count + 1)
    }

    fn update_to_edge(
        &mut self,
        storage: &mut Storage<D>,
        node: GraphIndex,
        edge: GraphIndex,
    ) -> Result<(), DbError> {
        let next = self.data.to(storage, node)?;
        self.data.set_to_meta(storage, edge, next)?;
        self.data.set_to(storage, node, -edge.0)?;

        let count = self.data.to_meta(storage, node)?;
        self.data.set_to_meta(storage, node, count + 1)
    }

    fn validate_edge(&self, storage: &Storage<D>, index: GraphIndex) -> Result<(), DbError> {
        if !self.is_valid_index(storage, index)? || !self.is_valid_edge(storage, index)? {
            return Err(Self::invalid_index(index));
        }

        Ok(())
    }

    fn validate_node(&self, storage: &Storage<D>, index: GraphIndex) -> Result<(), DbError> {
        if !self.is_valid_index(storage, index)? || !self.is_valid_node(storage, index)? {
            return Err(Self::invalid_index(index));
        }

        Ok(())
    }
}

pub type DbGraph<D> = GraphImpl<D, GraphDataStorage<D>>;

impl<D> DbGraph<D>
where
    D: StorageData,
{
    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index
    }

    pub fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        Ok(DbGraph {
            data: GraphDataStorage::<D>::new(storage)?,
            storage: PhantomData,
        })
    }

    pub fn from_storage(storage: &Storage<D>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(DbGraph {
            data: GraphDataStorage::<D>::from_storage(storage, index)?,
            storage: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::file_storage_memory_mapped::FileStorageMemoryMapped;
    use crate::test_utilities::test_file::TestFile;
    use std::cmp::Ordering;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    #[allow(clippy::clone_on_copy)]
    #[test]
    fn derived_from_clone() {
        let index = GraphIndex(1);
        let other = index.clone();

        assert_eq!(index, other);
    }

    #[test]
    fn derived_from_debug() {
        let index = GraphIndex::default();
        format!("{index:?}");
    }

    #[test]
    fn derived_from_hash() {
        let mut hasher = DefaultHasher::new();
        GraphIndex(1).hash(&mut hasher);
        assert_ne!(hasher.finish(), 0);
    }

    #[test]
    fn derived_from_ord() {
        let index = GraphIndex::default();
        assert_eq!(index.cmp(&index), Ordering::Equal);
    }

    #[test]
    fn index_hashing() {
        let _ = GraphIndex(10).stable_hash();
    }

    #[test]
    fn ordering() {
        let mut indexes = vec![
            GraphIndex::default(),
            GraphIndex::from(100_i64),
            GraphIndex::from(-1_i64),
            GraphIndex::from(1_i64),
        ];
        indexes.sort();
        assert_eq!(
            indexes,
            vec![
                GraphIndex::from(-1_i64),
                GraphIndex::default(),
                GraphIndex::from(1_i64),
                GraphIndex::from(100_i64),
            ]
        )
    }

    #[test]
    fn serialized_size() {
        assert_eq!(
            GraphDataStorageIndexes {
                from: StorageIndex::default(),
                to: StorageIndex::default(),
                from_meta: StorageIndex::default(),
                to_meta: StorageIndex::default()
            }
            .serialized_size(),
            StorageIndex::default().serialized_size() * 4
        );
    }

    #[test]
    fn edge_from_index() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let from = graph.insert_node(&mut storage).unwrap();
        let to = graph.insert_node(&mut storage).unwrap();
        let index = graph.insert_edge(&mut storage, from, to).unwrap();

        assert_eq!(graph.edge(&storage, index).unwrap().index(), index);
    }

    #[test]
    fn edge_from_index_missing() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let graph = DbGraph::new(&mut storage).unwrap();

        assert!(graph.edge(&storage, GraphIndex::from(-3)).is_none());
    }

    #[test]
    fn edge_iteration() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node1, node2).unwrap();

        let mut actual = Vec::<GraphIndex>::new();

        for edge in graph.node(&storage, node1).unwrap().edge_iter_from() {
            actual.push(edge.index());
        }

        assert_eq!(actual, vec![edge3, edge2, edge1]);
    }

    #[test]
    fn insert_edge() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let from = graph.insert_node(&mut storage).unwrap();
        let to = graph.insert_node(&mut storage).unwrap();

        assert_eq!(
            graph.insert_edge(&mut storage, from, to),
            Ok(GraphIndex::from(-3_i64))
        );

        let from_node = graph.node(&storage, from).unwrap();

        assert_eq!(from_node.edge_count(), 1);
        assert_eq!(from_node.edge_count_from(), 1);
        assert_eq!(from_node.edge_count_to(), 0);

        let to_node = graph.node(&storage, to).unwrap();

        assert_eq!(to_node.edge_count(), 1);
        assert_eq!(to_node.edge_count_from(), 0);
        assert_eq!(to_node.edge_count_to(), 1);
    }

    #[test]
    fn insert_edge_after_removed() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let from = graph.insert_node(&mut storage).unwrap();
        let to = graph.insert_node(&mut storage).unwrap();
        let index = graph.insert_edge(&mut storage, from, to).unwrap();

        graph.remove_edge(&mut storage, index).unwrap();

        assert_eq!(graph.insert_edge(&mut storage, from, to), Ok(index));
    }

    #[test]
    fn insert_edge_after_several_removed() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let from = graph.insert_node(&mut storage).unwrap();
        let to = graph.insert_node(&mut storage).unwrap();
        let index1 = graph.insert_edge(&mut storage, from, to).unwrap();
        let index2 = graph.insert_edge(&mut storage, from, to).unwrap();
        graph.insert_edge(&mut storage, from, to).unwrap();

        graph.remove_edge(&mut storage, index1).unwrap();
        graph.remove_edge(&mut storage, index2).unwrap();

        assert_eq!(graph.insert_edge(&mut storage, from, to), Ok(index2));
    }

    #[test]
    fn insert_edge_invalid_from() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        assert_eq!(
            graph.insert_edge(&mut storage, GraphIndex::from(1), GraphIndex::from(2)),
            Err(DbError::from("'1' is invalid index"))
        );
    }

    #[test]
    fn insert_edge_invalid_to() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let from = graph.insert_node(&mut storage).unwrap();

        assert_eq!(
            graph.insert_edge(&mut storage, from, GraphIndex::from(2)),
            Err(DbError::from("'2' is invalid index"))
        );
    }

    #[test]
    fn insert_node() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        assert_eq!(graph.insert_node(&mut storage), Ok(GraphIndex::from(1)));
    }

    #[test]
    fn insert_node_after_removal() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        graph.insert_node(&mut storage).unwrap();
        let index = graph.insert_node(&mut storage).unwrap();
        graph.insert_node(&mut storage).unwrap();

        graph.remove_node(&mut storage, index).unwrap();

        assert_eq!(graph.insert_node(&mut storage).unwrap(), index);
    }

    #[test]
    fn node_count() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        assert_eq!(graph.node_count(&storage,).unwrap(), 0);

        graph.insert_node(&mut storage).unwrap();
        let index = graph.insert_node(&mut storage).unwrap();
        graph.insert_node(&mut storage).unwrap();

        assert_eq!(graph.node_count(&storage,).unwrap(), 3);

        graph.remove_node(&mut storage, index).unwrap();

        assert_eq!(graph.node_count(&storage,).unwrap(), 2);
    }

    #[test]
    fn node_from_index() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let index = graph.insert_node(&mut storage).unwrap();

        assert_eq!(graph.node(&storage, index).unwrap().index(), index);
    }

    #[test]
    fn node_from_index_missing() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let graph = DbGraph::new(&mut storage).unwrap();

        let node = graph.node(&storage, GraphIndex::from(1));

        assert!(node.is_none());
    }

    #[test]
    fn node_iteration() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let expected = vec![node1, node2, node3];
        let mut nodes = Vec::<GraphIndex>::new();

        for node in graph.node_iter(&storage) {
            nodes.push(node.index());
        }

        assert_eq!(nodes, expected);
    }

    #[test]
    fn node_iteration_with_removed_nodes() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();
        let node4 = graph.insert_node(&mut storage).unwrap();
        let node5 = graph.insert_node(&mut storage).unwrap();

        graph.remove_node(&mut storage, node2).unwrap();
        graph.remove_node(&mut storage, node5).unwrap();

        let expected = vec![node1, node3, node4];
        let mut nodes = Vec::<GraphIndex>::new();

        for node in graph.node_iter(&storage) {
            nodes.push(node.index());
        }

        assert_eq!(nodes, expected);
    }

    #[test]
    fn remove_edge_circular() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let node = graph.insert_node(&mut storage).unwrap();
        let index = graph.insert_edge(&mut storage, node, node).unwrap();

        graph.remove_edge(&mut storage, index).unwrap();

        assert!(graph.edge(&storage, index).is_none());
    }

    #[test]
    fn remove_edge_first() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let from = graph.insert_node(&mut storage).unwrap();
        let to = graph.insert_node(&mut storage).unwrap();
        let index1 = graph.insert_edge(&mut storage, from, to).unwrap();
        let index2 = graph.insert_edge(&mut storage, from, to).unwrap();
        let index3 = graph.insert_edge(&mut storage, from, to).unwrap();

        graph.remove_edge(&mut storage, index3).unwrap();

        assert!(graph.edge(&storage, index1).is_some());
        assert!(graph.edge(&storage, index2).is_some());
        assert!(graph.edge(&storage, index3).is_none());
    }

    #[test]
    fn remove_edge_last() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let from = graph.insert_node(&mut storage).unwrap();
        let to = graph.insert_node(&mut storage).unwrap();
        let index1 = graph.insert_edge(&mut storage, from, to).unwrap();
        let index2 = graph.insert_edge(&mut storage, from, to).unwrap();
        let index3 = graph.insert_edge(&mut storage, from, to).unwrap();

        graph.remove_edge(&mut storage, index1).unwrap();

        assert!(graph.edge(&storage, index1).is_none());
        assert!(graph.edge(&storage, index2).is_some());
        assert!(graph.edge(&storage, index3).is_some());
    }

    #[test]
    fn remove_edge_middle() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let from = graph.insert_node(&mut storage).unwrap();
        let to = graph.insert_node(&mut storage).unwrap();
        let index1 = graph.insert_edge(&mut storage, from, to).unwrap();
        let index2 = graph.insert_edge(&mut storage, from, to).unwrap();
        let index3 = graph.insert_edge(&mut storage, from, to).unwrap();

        graph.remove_edge(&mut storage, index2).unwrap();

        assert!(graph.edge(&storage, index1).is_some());
        assert!(graph.edge(&storage, index2).is_none());
        assert!(graph.edge(&storage, index3).is_some());

        assert_eq!(graph.node(&storage, from).unwrap().edge_count(), 2);
    }

    #[test]
    fn remove_edge_missing() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        graph
            .remove_edge(&mut storage, GraphIndex::from(-3))
            .unwrap();
    }

    #[test]
    fn remove_edge_only() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let from = graph.insert_node(&mut storage).unwrap();
        let to = graph.insert_node(&mut storage).unwrap();
        let index = graph.insert_edge(&mut storage, from, to).unwrap();

        graph.remove_edge(&mut storage, index).unwrap();

        assert!(graph.edge(&storage, index).is_none());
    }

    #[test]
    fn remove_node_circular_edge() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let index = graph.insert_node(&mut storage).unwrap();
        let edge = graph.insert_edge(&mut storage, index, index).unwrap();

        graph.remove_node(&mut storage, index).unwrap();

        assert!(graph.node(&storage, index).is_none());
        assert!(graph.edge(&storage, edge).is_none());
    }

    #[test]
    fn remove_node_only() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let index = graph.insert_node(&mut storage).unwrap();

        graph.remove_node(&mut storage, index).unwrap();

        assert!(graph.node(&storage, index).is_none());
    }

    #[test]
    fn remove_node_missing() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        graph
            .remove_node(&mut storage, GraphIndex::from(1))
            .unwrap();
    }

    #[test]
    fn remove_nodes_with_edges() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node1).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node1, node3).unwrap();
        let edge4 = graph.insert_edge(&mut storage, node2, node1).unwrap();
        let edge5 = graph.insert_edge(&mut storage, node3, node1).unwrap();
        let edge6 = graph.insert_edge(&mut storage, node3, node2).unwrap();
        let edge7 = graph.insert_edge(&mut storage, node2, node3).unwrap();

        graph.remove_node(&mut storage, node1).unwrap();

        assert!(graph.node(&storage, node1).is_none());
        assert!(graph.edge(&storage, edge1).is_none());
        assert!(graph.edge(&storage, edge2).is_none());
        assert!(graph.edge(&storage, edge3).is_none());
        assert!(graph.edge(&storage, edge4).is_none());
        assert!(graph.edge(&storage, edge5).is_none());

        assert!(graph.node(&storage, node2).is_some());
        assert!(graph.node(&storage, node3).is_some());
        assert!(graph.edge(&storage, edge6).is_some());
        assert!(graph.edge(&storage, edge7).is_some());
    }

    #[test]
    fn restore_from_file() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let storage_index;

        let node1;
        let node2;
        let node3;

        let edge1;
        let edge2;
        let edge3;

        {
            let mut graph = DbGraph::new(&mut storage).unwrap();

            storage_index = graph.storage_index();

            node1 = graph.insert_node(&mut storage).unwrap();
            node2 = graph.insert_node(&mut storage).unwrap();
            node3 = graph.insert_node(&mut storage).unwrap();

            edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
            edge2 = graph.insert_edge(&mut storage, node2, node3).unwrap();
            edge3 = graph.insert_edge(&mut storage, node3, node1).unwrap();
        }

        let graph = DbGraph::from_storage(&storage, storage_index).unwrap();

        assert!(graph.node(&storage, node1).is_some());
        assert!(graph.node(&storage, node2).is_some());
        assert!(graph.node(&storage, node3).is_some());
        assert!(graph.edge(&storage, edge1).is_some());
        assert!(graph.edge(&storage, edge2).is_some());
        assert!(graph.edge(&storage, edge3).is_some());
    }

    #[test]
    fn reuse_edge_index_for_node() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let n1 = graph.insert_node(&mut storage).unwrap();
        let n2 = graph.insert_node(&mut storage).unwrap();
        let e1 = graph.insert_edge(&mut storage, n1, n2).unwrap();
        graph.remove_edge(&mut storage, e1).unwrap();
        let n3 = graph.insert_node(&mut storage).unwrap();
        assert!(graph.node(&storage, n3).is_some());
    }
}
