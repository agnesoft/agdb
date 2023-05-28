use crate::collections::vec::DbVec;
use crate::collections::vec::VecValue;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::rc::Rc;

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

    pub(crate) fn as_u64(&self) -> u64 {
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

impl Serialize for GraphIndex {
    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self(i64::deserialize(bytes)?))
    }

    fn serialized_size(&self) -> u64 {
        self.0.serialized_size()
    }
}

impl SerializeStatic for GraphIndex {}

impl VecValue for GraphIndex {
    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

pub trait GraphData {
    fn capacity(&self) -> Result<u64, DbError>;
    fn commit(&mut self, id: u64) -> Result<(), DbError>;
    fn free_index(&self) -> Result<i64, DbError>;
    fn from(&self, index: GraphIndex) -> Result<i64, DbError>;
    #[allow(clippy::wrong_self_convention)]
    fn from_meta(&self, index: GraphIndex) -> Result<i64, DbError>;
    fn grow(&mut self) -> Result<(), DbError>;
    fn node_count(&self) -> Result<u64, DbError>;
    fn set_from(&mut self, index: GraphIndex, value: i64) -> Result<(), DbError>;
    fn set_from_meta(&mut self, index: GraphIndex, value: i64) -> Result<(), DbError>;
    fn set_node_count(&mut self, count: u64) -> Result<(), DbError>;
    fn set_to(&mut self, index: GraphIndex, value: i64) -> Result<(), DbError>;
    fn set_to_meta(&mut self, index: GraphIndex, value: i64) -> Result<(), DbError>;
    fn to(&self, index: GraphIndex) -> Result<i64, DbError>;
    fn to_meta(&self, index: GraphIndex) -> Result<i64, DbError>;
    fn transaction(&mut self) -> u64;
}

pub(crate) struct GraphDataStorageIndexes {
    pub(crate) from: StorageIndex,
    pub(crate) to: StorageIndex,
    pub(crate) from_meta: StorageIndex,
    pub(crate) to_meta: StorageIndex,
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
        let mut bytes: Vec<u8> = vec![];
        bytes.reserve(4 * StorageIndex::serialized_size_static() as usize);

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

pub struct GraphDataStorage<S = FileStorage>
where
    S: Storage,
{
    pub(crate) storage: Rc<RefCell<S>>,
    pub(crate) storage_index: StorageIndex,
    pub(crate) from: DbVec<i64, S>,
    pub(crate) to: DbVec<i64, S>,
    pub(crate) from_meta: DbVec<i64, S>,
    pub(crate) to_meta: DbVec<i64, S>,
}

impl<S> GraphDataStorage<S>
where
    S: Storage,
{
    pub fn new(storage: Rc<RefCell<S>>) -> Result<Self, DbError> {
        let id = storage.borrow_mut().transaction();

        let mut from = DbVec::<i64, S>::new(storage.clone())?;
        from.push(&0)?;
        let mut to = DbVec::<i64, S>::new(storage.clone())?;
        to.push(&0)?;
        let mut from_meta = DbVec::<i64, S>::new(storage.clone())?;
        from_meta.push(&i64::MIN)?;
        let mut to_meta = DbVec::<i64, S>::new(storage.clone())?;
        to_meta.push(&0)?;

        let indexes = GraphDataStorageIndexes {
            from: from.storage_index(),
            to: to.storage_index(),
            from_meta: from_meta.storage_index(),
            to_meta: to_meta.storage_index(),
        };

        let index = storage.borrow_mut().insert(&indexes)?;

        storage.borrow_mut().commit(id)?;

        Ok(GraphDataStorage::<S> {
            storage,
            storage_index: index,
            from,
            to,
            from_meta,
            to_meta,
        })
    }

    pub fn from_storage(
        storage: Rc<RefCell<S>>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let indexes = storage
            .borrow_mut()
            .value::<GraphDataStorageIndexes>(storage_index)?;

        let from = DbVec::<i64, S>::from_storage(storage.clone(), indexes.from)?;
        let to = DbVec::<i64, S>::from_storage(storage.clone(), indexes.to)?;
        let from_meta = DbVec::<i64, S>::from_storage(storage.clone(), indexes.from_meta)?;
        let to_meta = DbVec::<i64, S>::from_storage(storage.clone(), indexes.to_meta)?;

        Ok(GraphDataStorage::<S> {
            storage,
            storage_index,
            from,
            to,
            from_meta,
            to_meta,
        })
    }
}

impl<Data> GraphData for GraphDataStorage<Data>
where
    Data: Storage,
{
    fn capacity(&self) -> Result<u64, DbError> {
        Ok(self.from.len())
    }

    fn commit(&mut self, id: u64) -> Result<(), DbError> {
        self.storage.borrow_mut().commit(id)
    }

    fn free_index(&self) -> Result<i64, DbError> {
        self.from_meta.value(0)
    }

    fn from(&self, index: GraphIndex) -> Result<i64, DbError> {
        self.from.value(index.as_u64())
    }

    fn from_meta(&self, index: GraphIndex) -> Result<i64, DbError> {
        self.from_meta.value(index.as_u64())
    }

    fn grow(&mut self) -> Result<(), DbError> {
        self.from.push(&0)?;
        self.to.push(&0)?;
        self.from_meta.push(&0)?;
        self.to_meta.push(&0)
    }

    fn node_count(&self) -> Result<u64, DbError> {
        Ok(self.to_meta.value(0)? as u64)
    }

    fn set_from(&mut self, index: GraphIndex, value: i64) -> Result<(), DbError> {
        self.from.replace(index.as_u64(), &value)?;
        Ok(())
    }

    fn set_from_meta(&mut self, index: GraphIndex, value: i64) -> Result<(), DbError> {
        self.from_meta.replace(index.as_u64(), &value)?;
        Ok(())
    }

    fn set_node_count(&mut self, count: u64) -> Result<(), DbError> {
        self.to_meta.replace(0, &(count as i64))?;
        Ok(())
    }

    fn set_to(&mut self, index: GraphIndex, value: i64) -> Result<(), DbError> {
        self.to.replace(index.as_u64(), &value)?;
        Ok(())
    }

    fn set_to_meta(&mut self, index: GraphIndex, value: i64) -> Result<(), DbError> {
        self.to_meta.replace(index.as_u64(), &value)?;
        Ok(())
    }

    fn to(&self, index: GraphIndex) -> Result<i64, DbError> {
        self.to.value(index.as_u64())
    }

    fn to_meta(&self, index: GraphIndex) -> Result<i64, DbError> {
        self.to_meta.value(index.as_u64())
    }

    fn transaction(&mut self) -> u64 {
        self.storage.borrow_mut().transaction()
    }
}

pub struct GraphNode<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

impl<'a, Data> GraphNode<'a, Data>
where
    Data: GraphData,
{
    #[allow(dead_code)]
    pub fn index(&self) -> GraphIndex {
        self.index
    }

    pub fn edge_iter_from(&self) -> GraphEdgeIterator<Data> {
        GraphEdgeIterator {
            graph: self.graph,
            index: self.graph.first_edge_from(self.index).unwrap_or_default(),
        }
    }

    pub fn edge_iter_to(&self) -> GraphEdgeReverseIterator<Data> {
        GraphEdgeReverseIterator {
            graph: self.graph,
            index: self.graph.first_edge_to(self.index).unwrap_or_default(),
        }
    }
}

pub struct GraphNodeIterator<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

impl<'a, Data> Iterator for GraphNodeIterator<'a, Data>
where
    Data: GraphData,
{
    type Item = GraphNode<'a, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.graph.next_node(self.index).unwrap_or(None) {
            self.index = next;
            return Some(GraphNode {
                graph: self.graph,
                index: self.index,
            });
        }

        None
    }
}

pub struct GraphEdge<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

impl<'a, Data> GraphEdge<'a, Data>
where
    Data: GraphData,
{
    pub fn index(&self) -> GraphIndex {
        self.index
    }

    pub fn index_from(&self) -> GraphIndex {
        self.graph.edge_from(self.index)
    }

    pub fn index_to(&self) -> GraphIndex {
        self.graph.edge_to(self.index)
    }
}

pub struct GraphEdgeIterator<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

impl<'a, Data> Iterator for GraphEdgeIterator<'a, Data>
where
    Data: GraphData,
{
    type Item = GraphEdge<'a, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.index.is_valid() {
            return None;
        }

        let current_index = self.index;

        self.index = self.graph.next_edge_from(self.index).unwrap_or_default();

        Some(GraphEdge {
            graph: self.graph,
            index: current_index,
        })
    }
}

pub struct GraphEdgeReverseIterator<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

impl<'a, Data> Iterator for GraphEdgeReverseIterator<'a, Data>
where
    Data: GraphData,
{
    type Item = GraphEdge<'a, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.index.is_valid() {
            return None;
        }

        let current_index = self.index;

        self.index = self.graph.next_edge_to(self.index).unwrap_or_default();

        Some(GraphEdge {
            graph: self.graph,
            index: current_index,
        })
    }
}

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
    pub fn edge(&self, index: GraphIndex) -> Option<GraphEdge<Data>> {
        if self.validate_edge(index).is_err() {
            return None;
        }

        Some(GraphEdge { graph: self, index })
    }

    #[allow(dead_code)]
    pub fn node_count(&self) -> Result<u64, DbError> {
        self.data.node_count()
    }

    pub fn insert_edge(&mut self, from: GraphIndex, to: GraphIndex) -> Result<GraphIndex, DbError> {
        self.validate_node(from)?;
        self.validate_node(to)?;

        let id = self.data.transaction();
        let index = GraphIndex::from(-self.get_free_index()?);
        self.set_edge(index, from, to)?;
        self.data.commit(id)?;

        Ok(index)
    }

    pub fn insert_node(&mut self) -> Result<GraphIndex, DbError> {
        let id = self.data.transaction();
        let index = GraphIndex::from(self.get_free_index()?);
        let count = self.data.node_count()?;
        self.data.set_node_count(count + 1)?;
        self.data.commit(id)?;

        Ok(index)
    }

    pub fn node(&self, index: GraphIndex) -> Option<GraphNode<Data>> {
        if self.validate_node(index).is_err() {
            return None;
        }

        Some(GraphNode { graph: self, index })
    }

    #[allow(dead_code)]
    pub fn node_iter(&self) -> GraphNodeIterator<Data> {
        GraphNodeIterator {
            graph: self,
            index: GraphIndex::default(),
        }
    }

    pub fn remove_edge(&mut self, index: GraphIndex) -> Result<(), DbError> {
        if self.validate_edge(index).is_err() {
            return Ok(());
        }

        let id = self.data.transaction();
        self.remove_from_edge(index)?;
        self.remove_to_edge(index)?;
        self.free_index(GraphIndex::from(-index.0))?;

        self.data.commit(id)
    }

    pub fn remove_node(&mut self, index: GraphIndex) -> Result<(), DbError> {
        if self.validate_node(index).is_err() {
            return Ok(());
        }

        let id = self.data.transaction();
        self.remove_from_edges(index)?;
        self.remove_to_edges(index)?;
        self.free_index(index)?;

        let count = self.data.node_count()?;
        self.data.set_node_count(count - 1)?;

        self.data.commit(id)
    }

    pub fn first_edge_from(&self, index: GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.from(index)?))
    }

    pub fn first_edge_to(&self, index: GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.to(index)?))
    }

    pub(crate) fn edge_from(&self, index: GraphIndex) -> GraphIndex {
        GraphIndex::from(-self.data.from(index).unwrap_or_default())
    }

    pub(crate) fn edge_to(&self, index: GraphIndex) -> GraphIndex {
        GraphIndex::from(-self.data.to(index).unwrap_or_default())
    }

    fn free_index(&mut self, index: GraphIndex) -> Result<(), DbError> {
        let next_free = self.data.from_meta(GraphIndex::default())?;
        self.data.set_from_meta(index, next_free)?;
        self.data.set_from_meta(GraphIndex::default(), -index.0)
    }

    fn get_free_index(&mut self) -> Result<i64, DbError> {
        let mut index = self.data.free_index()?;

        if index == i64::MIN {
            index = self.data.capacity()? as i64;
            self.data.grow()?;

            Ok(index)
        } else {
            let next = self.data.from_meta(GraphIndex::from(-index))?;
            self.data.set_from_meta(GraphIndex::default(), next)?;
            self.data.set_from_meta(GraphIndex::from(-index), 0)?;

            Ok(-index)
        }
    }

    fn invalid_index(index: GraphIndex) -> DbError {
        DbError::from(format!("'{}' is invalid index", index.0))
    }

    fn is_removed_index(&self, index: GraphIndex) -> Result<bool, DbError> {
        Ok(self.data.from_meta(index)? < 0)
    }

    fn is_valid_edge(&self, index: GraphIndex) -> Result<bool, DbError> {
        Ok(self.data.from(index)? < 0)
    }

    fn is_valid_index(&self, index: GraphIndex) -> Result<bool, DbError> {
        Ok(index.is_valid()
            && index.as_u64() < self.data.capacity()?
            && !self.is_removed_index(index)?)
    }

    fn is_valid_node(&self, index: GraphIndex) -> Result<bool, DbError> {
        Ok(0 <= self.data.from(index)?)
    }

    pub(crate) fn next_edge_from(&self, index: GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.from_meta(index)?))
    }

    pub(crate) fn next_edge_to(&self, index: GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex::from(-self.data.to_meta(index)?))
    }

    pub(crate) fn next_node(&self, index: GraphIndex) -> Result<Option<GraphIndex>, DbError> {
        for i in (index.0 + 1)..(self.data.capacity()? as i64) {
            let next = GraphIndex::from(i);
            if self.is_valid_node(next)? && !self.is_removed_index(next)? {
                return Ok(Some(next));
            }
        }

        Ok(None)
    }

    fn remove_from_edge(&mut self, index: GraphIndex) -> Result<(), DbError> {
        let node_index = GraphIndex::from(-self.data.from(index)?);
        let first_index = GraphIndex::from(-self.data.from(node_index)?);
        let next = self.data.from_meta(index)?;

        if first_index == index {
            self.data.set_from(node_index, next)?;
        } else {
            let mut previous = first_index;

            while self.data.from_meta(previous)? != -index.0 {
                previous = GraphIndex::from(self.data.from_meta(previous)?);
            }

            self.data.set_from_meta(previous, next)?;
        }

        let count = self.data.from_meta(node_index)?;
        self.data.set_from_meta(node_index, count - 1)
    }

    fn remove_from_edges(&mut self, index: GraphIndex) -> Result<(), DbError> {
        let mut edge = GraphIndex::from(-self.data.from(index)?);

        while edge.is_valid() {
            self.remove_to_edge(edge)?;
            let current_index = -edge.0;
            edge = GraphIndex::from(-self.data.from_meta(edge)?);
            self.free_index(GraphIndex::from(current_index))?;
        }

        Ok(())
    }

    fn remove_to_edge(&mut self, index: GraphIndex) -> Result<(), DbError> {
        let node_index = GraphIndex::from(-self.data.to(index)?);
        let first_index = GraphIndex::from(-self.data.to(node_index)?);
        let next = self.data.to_meta(index)?;

        if first_index == index {
            self.data.set_to(node_index, next)?;
        } else {
            let mut previous = first_index;

            while self.data.to_meta(previous)? != -index.0 {
                previous = GraphIndex::from(self.data.to_meta(previous)?);
            }

            self.data.set_to_meta(previous, next)?;
        }

        let count = self.data.to_meta(node_index)?;
        self.data.set_to_meta(node_index, count - 1)
    }

    fn remove_to_edges(&mut self, index: GraphIndex) -> Result<(), DbError> {
        let mut edge_index = GraphIndex::from(-self.data.to(index)?);

        while edge_index.is_valid() {
            self.remove_from_edge(edge_index)?;
            let current_index = -edge_index.0;
            edge_index = GraphIndex::from(-self.data.to_meta(edge_index)?);
            self.free_index(GraphIndex::from(current_index))?;
        }

        Ok(())
    }

    fn set_edge(
        &mut self,
        index: GraphIndex,
        from: GraphIndex,
        to: GraphIndex,
    ) -> Result<(), DbError> {
        self.data.set_from(index, -from.0)?;
        self.data.set_to(index, -to.0)?;
        self.update_from_edge(from, index)?;
        self.update_to_edge(to, index)
    }

    fn update_from_edge(&mut self, node: GraphIndex, edge: GraphIndex) -> Result<(), DbError> {
        let next = self.data.from(node)?;
        self.data.set_from_meta(edge, next)?;
        self.data.set_from(node, -edge.0)?;

        let count = self.data.from_meta(node)?;
        self.data.set_from_meta(node, count + 1)
    }

    fn update_to_edge(&mut self, node: GraphIndex, edge: GraphIndex) -> Result<(), DbError> {
        let next = self.data.to(node)?;
        self.data.set_to_meta(edge, next)?;
        self.data.set_to(node, -edge.0)?;

        let count = self.data.to_meta(node)?;
        self.data.set_to_meta(node, count + 1)
    }

    fn validate_edge(&self, index: GraphIndex) -> Result<(), DbError> {
        if !self.is_valid_index(index)? || !self.is_valid_edge(index)? {
            return Err(Self::invalid_index(index));
        }

        Ok(())
    }

    fn validate_node(&self, index: GraphIndex) -> Result<(), DbError> {
        if !self.is_valid_index(index)? || !self.is_valid_node(index)? {
            return Err(Self::invalid_index(index));
        }

        Ok(())
    }
}

pub type DbGraph<Data = FileStorage> = GraphImpl<GraphDataStorage<Data>>;

impl<Data> DbGraph<Data>
where
    Data: Storage,
{
    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index
    }

    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        Ok(DbGraph {
            data: GraphDataStorage::<Data>::new(storage)?,
        })
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(DbGraph {
            data: GraphDataStorage::<Data>::from_storage(storage, index)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;
    use std::cmp::Ordering;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

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

        assert_eq!(
            GraphIndex::default().serialized_size(),
            i64::serialized_size_static()
        );
    }

    #[test]
    fn edge_from_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index = graph.insert_edge(from, to).unwrap();

        assert_eq!(graph.edge(index).unwrap().index(), index);
    }

    #[test]
    fn edge_from_index_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let graph = DbGraph::new(storage).unwrap();

        assert!(graph.edge(GraphIndex::from(-3)).is_none());
    }

    #[test]
    fn edge_iteration() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(node1, node2).unwrap();
        let edge2 = graph.insert_edge(node1, node2).unwrap();
        let edge3 = graph.insert_edge(node1, node2).unwrap();

        let mut actual = Vec::<GraphIndex>::new();

        for edge in graph.node(node1).unwrap().edge_iter_from() {
            actual.push(edge.index());
        }

        assert_eq!(actual, vec![edge3, edge2, edge1]);
    }

    #[test]
    fn insert_edge() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();

        assert_eq!(graph.insert_edge(from, to), Ok(GraphIndex::from(-3_i64)));
    }

    #[test]
    fn insert_edge_after_removed() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index).unwrap();

        assert_eq!(graph.insert_edge(from, to), Ok(index));
    }

    #[test]
    fn insert_edge_after_several_removed() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index1 = graph.insert_edge(from, to).unwrap();
        let index2 = graph.insert_edge(from, to).unwrap();
        graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index1).unwrap();
        graph.remove_edge(index2).unwrap();

        assert_eq!(graph.insert_edge(from, to), Ok(index2));
    }

    #[test]
    fn insert_edge_invalid_from() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        assert_eq!(
            graph.insert_edge(GraphIndex::from(1), GraphIndex::from(2)),
            Err(DbError::from("'1' is invalid index"))
        );
    }

    #[test]
    fn insert_edge_invalid_to() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let from = graph.insert_node().unwrap();

        assert_eq!(
            graph.insert_edge(from, GraphIndex::from(2)),
            Err(DbError::from("'2' is invalid index"))
        );
    }

    #[test]
    fn insert_node() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        assert_eq!(graph.insert_node(), Ok(GraphIndex::from(1)));
    }

    #[test]
    fn insert_node_after_removal() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        graph.insert_node().unwrap();
        let index = graph.insert_node().unwrap();
        graph.insert_node().unwrap();

        graph.remove_node(index).unwrap();

        assert_eq!(graph.insert_node().unwrap(), index);
    }

    #[test]
    fn node_count() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        assert_eq!(graph.node_count().unwrap(), 0);

        graph.insert_node().unwrap();
        let index = graph.insert_node().unwrap();
        graph.insert_node().unwrap();

        assert_eq!(graph.node_count().unwrap(), 3);

        graph.remove_node(index).unwrap();

        assert_eq!(graph.node_count().unwrap(), 2);
    }

    #[test]
    fn node_from_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let index = graph.insert_node().unwrap();

        assert_eq!(graph.node(index).unwrap().index(), index);
    }

    #[test]
    fn node_from_index_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let graph = DbGraph::new(storage).unwrap();

        let node = graph.node(GraphIndex::from(1));

        assert!(node.is_none());
    }

    #[test]
    fn node_iteration() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
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
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();
        let node4 = graph.insert_node().unwrap();
        let node5 = graph.insert_node().unwrap();

        graph.remove_node(node2).unwrap();
        graph.remove_node(node5).unwrap();

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
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let node = graph.insert_node().unwrap();
        let index = graph.insert_edge(node, node).unwrap();

        graph.remove_edge(index).unwrap();

        assert!(graph.edge(index).is_none());
    }

    #[test]
    fn remove_edge_first() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index1 = graph.insert_edge(from, to).unwrap();
        let index2 = graph.insert_edge(from, to).unwrap();
        let index3 = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index3).unwrap();

        assert!(graph.edge(index1).is_some());
        assert!(graph.edge(index2).is_some());
        assert!(graph.edge(index3).is_none());
    }

    #[test]
    fn remove_edge_last() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index1 = graph.insert_edge(from, to).unwrap();
        let index2 = graph.insert_edge(from, to).unwrap();
        let index3 = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index1).unwrap();

        assert!(graph.edge(index1).is_none());
        assert!(graph.edge(index2).is_some());
        assert!(graph.edge(index3).is_some());
    }

    #[test]
    fn remove_edge_middle() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index1 = graph.insert_edge(from, to).unwrap();
        let index2 = graph.insert_edge(from, to).unwrap();
        let index3 = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index2).unwrap();

        assert!(graph.edge(index1).is_some());
        assert!(graph.edge(index2).is_none());
        assert!(graph.edge(index3).is_some());
    }

    #[test]
    fn remove_edge_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        graph.remove_edge(GraphIndex::from(-3)).unwrap();
    }

    #[test]
    fn remove_edge_only() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let from = graph.insert_node().unwrap();
        let to = graph.insert_node().unwrap();
        let index = graph.insert_edge(from, to).unwrap();

        graph.remove_edge(index).unwrap();

        assert!(graph.edge(index).is_none());
    }

    #[test]
    fn remove_node_circular_edge() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let index = graph.insert_node().unwrap();
        let edge = graph.insert_edge(index, index).unwrap();

        graph.remove_node(index).unwrap();

        assert!(graph.node(index).is_none());
        assert!(graph.edge(edge).is_none());
    }

    #[test]
    fn remove_node_only() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        let index = graph.insert_node().unwrap();

        graph.remove_node(index).unwrap();

        assert!(graph.node(index).is_none());
    }

    #[test]
    fn remove_node_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();
        graph.remove_node(GraphIndex::from(1)).unwrap();
    }

    #[test]
    fn remove_nodes_with_edges() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(node1, node2).unwrap();
        let edge2 = graph.insert_edge(node1, node1).unwrap();
        let edge3 = graph.insert_edge(node1, node3).unwrap();
        let edge4 = graph.insert_edge(node2, node1).unwrap();
        let edge5 = graph.insert_edge(node3, node1).unwrap();
        let edge6 = graph.insert_edge(node3, node2).unwrap();
        let edge7 = graph.insert_edge(node2, node3).unwrap();

        graph.remove_node(node1).unwrap();

        assert!(graph.node(node1).is_none());
        assert!(graph.edge(edge1).is_none());
        assert!(graph.edge(edge2).is_none());
        assert!(graph.edge(edge3).is_none());
        assert!(graph.edge(edge4).is_none());
        assert!(graph.edge(edge5).is_none());

        assert!(graph.node(node2).is_some());
        assert!(graph.node(node3).is_some());
        assert!(graph.edge(edge6).is_some());
        assert!(graph.edge(edge7).is_some());
    }

    #[test]
    fn restore_from_file() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let storage_index;

        let node1;
        let node2;
        let node3;

        let edge1;
        let edge2;
        let edge3;

        {
            let mut graph = DbGraph::new(storage.clone()).unwrap();

            storage_index = graph.storage_index();

            node1 = graph.insert_node().unwrap();
            node2 = graph.insert_node().unwrap();
            node3 = graph.insert_node().unwrap();

            edge1 = graph.insert_edge(node1, node2).unwrap();
            edge2 = graph.insert_edge(node2, node3).unwrap();
            edge3 = graph.insert_edge(node3, node1).unwrap();
        }

        let graph = DbGraph::from_storage(storage, storage_index).unwrap();

        assert!(graph.node(node1).is_some());
        assert!(graph.node(node2).is_some());
        assert!(graph.node(node3).is_some());
        assert!(graph.edge(edge1).is_some());
        assert!(graph.edge(edge2).is_some());
        assert!(graph.edge(edge3).is_some());
    }
}
