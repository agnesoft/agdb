use crate::DbError;
use crate::DbErrorType;
use crate::StorageData;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::serialize::u64_at;
use crate::utilities::stable_hash::StableHash;
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
        self.0.unsigned_abs()
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

/// The 16 bytes a node carries for its alias. The graph does not
/// interpret them, all zeroes is a node without an alias.
pub type GraphAlias = [u8; 16];

type Record = [i64; FIELDS];

const FIELDS: usize = 6;
const FIELD_SIZE: u64 = 8;
const RECORD_SIZE: u64 = FIELDS as u64 * FIELD_SIZE;
const HEADER_SIZE: u64 = 8;

// Node: first edge from, first edge to, edge count from, edge count to, alias.
// Edge: -origin, -destination, next edge from, next edge to,
// previous edge from, previous edge to. The links are the positive slots.
// Free slot: the next free slot as a negative number in FROM_META.
// Slot 0: the free list head, the node count and the free slot count.
const FROM: usize = 0;
const TO: usize = 1;
const FROM_META: usize = 2;
const TO_META: usize = 3;
const PREVIOUS_FROM: usize = 4;
const PREVIOUS_TO: usize = 5;
const ALIAS: usize = 4;
const FREE_HEAD: usize = FROM_META;
const NODE_COUNT: usize = TO_META;
const FREE_COUNT: usize = 4;
const NO_FREE: i64 = i64::MIN;
const ROOT: Record = [0, 0, NO_FREE, 0, 0, 0];

fn to_bytes(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn to_record(bytes: &[u8]) -> Record {
    let mut record = [0; FIELDS];

    for (i, field) in record.iter_mut().enumerate() {
        *field = u64_at(bytes, i) as i64;
    }

    record
}

pub struct GraphNode<'a, D>
where
    D: StorageData,
{
    graph: &'a DbGraph<D>,
    record: Record,
    storage: &'a Storage<D>,
}

impl<'a, D> GraphNode<'a, D>
where
    D: StorageData,
{
    pub fn edge_iter_from(&self) -> GraphEdgeIterator<'a, D> {
        GraphEdgeIterator {
            graph: self.graph,
            index: GraphIndex(-self.record[FROM]),
            storage: self.storage,
            next: FROM_META,
        }
    }

    pub fn edge_iter_to(&self) -> GraphEdgeIterator<'a, D> {
        GraphEdgeIterator {
            graph: self.graph,
            index: GraphIndex(-self.record[TO]),
            storage: self.storage,
            next: TO_META,
        }
    }

    pub fn edge_count(&self) -> u64 {
        self.edge_count_from() + self.edge_count_to()
    }

    pub fn edge_count_from(&self) -> u64 {
        self.record[FROM_META] as u64
    }

    pub fn edge_count_to(&self) -> u64 {
        self.record[TO_META] as u64
    }
}

pub struct GraphIterator<'a, D>
where
    D: StorageData,
{
    graph: &'a DbGraph<D>,
    slot: u64,
    storage: &'a Storage<D>,
}

impl<D> Iterator for GraphIterator<'_, D>
where
    D: StorageData,
{
    type Item = Result<GraphIndex, DbError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.slot < self.graph.len {
            let slot = self.slot;
            self.slot += 1;

            match self.graph.record(self.storage, slot) {
                Ok(record) if 0 <= record[FROM_META] => {
                    let slot = slot as i64;
                    return Some(Ok(GraphIndex(if record[FROM] < 0 { -slot } else { slot })));
                }
                Ok(_) => {}
                Err(error) => {
                    self.slot = self.graph.len;
                    return Some(Err(error));
                }
            }
        }

        None
    }
}

pub struct GraphEdge {
    index: GraphIndex,
    record: Record,
}

impl GraphEdge {
    pub fn index(&self) -> GraphIndex {
        self.index
    }

    pub fn index_from(&self) -> GraphIndex {
        GraphIndex(-self.record[FROM])
    }

    pub fn index_to(&self) -> GraphIndex {
        GraphIndex(-self.record[TO])
    }
}

/// Follows the edges from (`next` is FROM_META) or to (TO_META) a
/// node. A failed read is the last item.
pub struct GraphEdgeIterator<'a, D>
where
    D: StorageData,
{
    graph: &'a DbGraph<D>,
    index: GraphIndex,
    storage: &'a Storage<D>,
    next: usize,
}

impl<D> Iterator for GraphEdgeIterator<'_, D>
where
    D: StorageData,
{
    type Item = Result<GraphEdge, DbError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.index.is_valid() {
            return None;
        }

        let index = self.index;

        Some(match self.graph.record(self.storage, index.as_u64()) {
            Ok(record) => {
                self.index = GraphIndex(-record[self.next]);
                Ok(GraphEdge { index, record })
            }
            Err(error) => {
                self.index = GraphIndex::default();
                Err(error)
            }
        })
    }
}

/// The graph of nodes and edges stored as a single storage value of
/// a length followed by one record of 6 fields per element. The
/// element id is the position of its record (negative for edges) so
/// the elements share one id space. Removed elements form a list of
/// free records reused by the next inserted elements, last removed
/// first.
pub struct DbGraph<D>
where
    D: StorageData,
{
    storage_index: StorageIndex,
    len: u64,
    capacity: u64,
    storage: PhantomData<D>,
}

impl<D> DbGraph<D>
where
    D: StorageData,
{
    pub fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        let storage_index = storage.insert_bytes(&[1_u64.serialize(), to_bytes(&ROOT)].concat())?;

        Ok(Self {
            storage_index,
            len: 1,
            capacity: 1,
            storage: PhantomData,
        })
    }

    pub fn from_storage(
        storage: &Storage<D>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let size = storage.value_size(storage_index)?;
        let len = u64_at(
            &storage.value_as_bytes_at_size(storage_index, 0, HEADER_SIZE)?,
            0,
        );
        let capacity = (size - HEADER_SIZE) / RECORD_SIZE;

        if len == 0 || capacity < len {
            return Err(DbError::graph(
                DbErrorType::OutOfBounds,
                format!("Graph length ({len}) is not within its capacity (1..={capacity})"),
            ));
        }

        Ok(Self {
            storage_index,
            len,
            capacity,
            storage: PhantomData,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }

    /// The alias fields of the node at `index`, `None` if there is no
    /// such node.
    pub fn alias(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<Option<GraphAlias>, DbError> {
        Ok(self.node_record(storage, index)?.map(|record| {
            let mut alias = [0_u8; 16];
            alias.copy_from_slice(&to_bytes(&record[ALIAS..]));
            alias
        }))
    }

    pub fn edge(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<Option<GraphEdge>, DbError> {
        Ok(self
            .edge_record(storage, index)?
            .map(|record| GraphEdge { index, record }))
    }

    pub fn edge_count(&self, storage: &Storage<D>) -> Result<u64, DbError> {
        let root = self.record(storage, 0)?;
        Ok(self.len - 1 - root[NODE_COUNT] as u64 - root[FREE_COUNT] as u64)
    }

    pub fn edge_from(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex(-self.field(storage, index.as_u64(), FROM)?))
    }

    pub fn edge_to(&self, storage: &Storage<D>, index: GraphIndex) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex(-self.field(storage, index.as_u64(), TO)?))
    }

    pub fn edge_iter_from<'a>(
        &'a self,
        storage: &'a Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphEdgeIterator<'a, D>, DbError> {
        Ok(GraphEdgeIterator {
            graph: self,
            index: self.first_edge_from(storage, index)?,
            storage,
            next: FROM_META,
        })
    }

    pub fn edge_iter_to<'a>(
        &'a self,
        storage: &'a Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphEdgeIterator<'a, D>, DbError> {
        Ok(GraphEdgeIterator {
            graph: self,
            index: self.first_edge_to(storage, index)?,
            storage,
            next: TO_META,
        })
    }

    pub fn first_edge_from(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex(-self.field(storage, index.as_u64(), FROM)?))
    }

    pub fn first_edge_to(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex(-self.field(storage, index.as_u64(), TO)?))
    }

    pub fn insert_edge(
        &mut self,
        storage: &mut Storage<D>,
        from: GraphIndex,
        to: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        self.validate_node(storage, from)?;
        self.validate_node(storage, to)?;

        let id = storage.transaction();
        let edge = self.allocate(storage, 0)?;
        let origin = self.record(storage, from.as_u64())?;
        self.write(
            storage,
            from.as_u64(),
            FROM,
            &[edge, origin[TO], origin[FROM_META] + 1],
        )?;
        let destination = self.record(storage, to.as_u64())?;
        self.write(
            storage,
            to.as_u64(),
            TO,
            &[edge, destination[FROM_META], destination[TO_META] + 1],
        )?;

        if origin[FROM] != 0 {
            self.write(storage, origin[FROM] as u64, PREVIOUS_FROM, &[edge])?;
        }

        if destination[TO] != 0 {
            self.write(storage, destination[TO] as u64, PREVIOUS_TO, &[edge])?;
        }

        self.write(
            storage,
            edge as u64,
            FROM,
            &[-from.0, -to.0, origin[FROM], destination[TO], 0, 0],
        )?;
        storage.commit(id)?;

        Ok(GraphIndex(-edge))
    }

    pub fn insert_node(&mut self, storage: &mut Storage<D>) -> Result<GraphIndex, DbError> {
        let id = storage.transaction();
        let node = self.allocate(storage, 1)?;
        self.write(storage, node as u64, FROM, &[0; FIELDS])?;
        storage.commit(id)?;

        Ok(GraphIndex(node))
    }

    pub fn iter<'a>(&'a self, storage: &'a Storage<D>) -> GraphIterator<'a, D> {
        GraphIterator {
            graph: self,
            slot: 1,
            storage,
        }
    }

    pub fn node<'a>(
        &'a self,
        storage: &'a Storage<D>,
        index: GraphIndex,
    ) -> Result<Option<GraphNode<'a, D>>, DbError> {
        Ok(self.node_record(storage, index)?.map(|record| GraphNode {
            graph: self,
            record,
            storage,
        }))
    }

    pub fn node_count(&self, storage: &Storage<D>) -> Result<u64, DbError> {
        Ok(self.field(storage, 0, NODE_COUNT)? as u64)
    }

    pub fn remove_edge(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        if self.edge_record(storage, index)?.is_none() {
            return Ok(());
        }

        let id = storage.transaction();
        self.remove_edge_record(storage, index.as_u64())?;
        storage.commit(id)
    }

    pub fn remove_node(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        if self.node_record(storage, index)?.is_none() {
            return Ok(());
        }

        let id = storage.transaction();

        for head in [FROM, TO] {
            loop {
                let edge = self.field(storage, index.as_u64(), head)?;

                if edge == 0 {
                    break;
                }

                self.remove_edge_record(storage, edge as u64)?;
            }
        }

        self.free(storage, index.as_u64(), -1)?;
        storage.commit(id)
    }

    pub fn set_alias(
        &mut self,
        storage: &mut Storage<D>,
        index: GraphIndex,
        alias: &GraphAlias,
    ) -> Result<(), DbError> {
        self.validate_node(storage, index)?;
        self.write(
            storage,
            index.as_u64(),
            ALIAS,
            &[u64_at(alias, 0) as i64, u64_at(alias, 1) as i64],
        )
    }

    pub fn shrink_to_fit(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        storage.resize_value(self.storage_index, Self::offset(self.len))?;
        self.capacity = self.len;
        Ok(())
    }

    pub(crate) fn next_edge_from(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex(-self.field(
            storage,
            index.as_u64(),
            FROM_META,
        )?))
    }

    pub(crate) fn next_edge_to(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<GraphIndex, DbError> {
        Ok(GraphIndex(-self.field(storage, index.as_u64(), TO_META)?))
    }

    /// Takes the slot of the free list head or a new one at the end,
    /// adding `nodes` to the node count. The caller writes the record.
    fn allocate(&mut self, storage: &mut Storage<D>, nodes: i64) -> Result<i64, DbError> {
        let root = self.record(storage, 0)?;

        if root[FREE_HEAD] == NO_FREE {
            let slot = self.len;
            self.grow(storage)?;
            self.write(storage, 0, NODE_COUNT, &[root[NODE_COUNT] + nodes])?;
            Ok(slot as i64)
        } else {
            let slot = -root[FREE_HEAD];
            let next = self.field(storage, slot as u64, FREE_HEAD)?;
            self.write(
                storage,
                0,
                FREE_HEAD,
                &[next, root[NODE_COUNT] + nodes, root[FREE_COUNT] - 1],
            )?;
            Ok(slot)
        }
    }

    fn field(&self, storage: &Storage<D>, slot: u64, field: usize) -> Result<i64, DbError> {
        Ok(u64_at(
            &storage.value_as_bytes_at_size(
                self.storage_index,
                Self::offset(slot) + field as u64 * FIELD_SIZE,
                FIELD_SIZE,
            )?,
            0,
        ) as i64)
    }

    /// Pushes the slot to the free list, adding `nodes` to the node count.
    fn free(&mut self, storage: &mut Storage<D>, slot: u64, nodes: i64) -> Result<(), DbError> {
        let root = self.record(storage, 0)?;
        self.write(storage, slot, FROM, &[0, 0, root[FREE_HEAD], 0, 0, 0])?;
        self.write(
            storage,
            0,
            FREE_HEAD,
            &[
                -(slot as i64),
                root[NODE_COUNT] + nodes,
                root[FREE_COUNT] + 1,
            ],
        )
    }

    fn grow(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        if self.len == self.capacity {
            let capacity = self.capacity + std::cmp::max(self.capacity / 2, 1);
            storage.resize_value(self.storage_index, Self::offset(capacity))?;
            self.capacity = capacity;
        }

        storage.insert_bytes_at(self.storage_index, 0, &(self.len + 1).serialize())?;
        self.len += 1;
        Ok(())
    }

    fn invalid_index(index: GraphIndex) -> DbError {
        DbError::graph(
            DbErrorType::InvalidIndex,
            format!("'{}' is invalid index", index.0),
        )
    }

    /// The record of an existing element, `None` for the root,
    /// free and out of bounds slots.
    fn element(&self, storage: &Storage<D>, index: GraphIndex) -> Result<Option<Record>, DbError> {
        if !index.is_valid() || self.len <= index.as_u64() {
            return Ok(None);
        }

        let record = self.record(storage, index.as_u64())?;
        Ok((0 <= record[FROM_META]).then_some(record))
    }

    fn offset(slot: u64) -> u64 {
        HEADER_SIZE + slot * RECORD_SIZE
    }

    fn record(&self, storage: &Storage<D>, slot: u64) -> Result<Record, DbError> {
        Ok(to_record(&storage.value_as_bytes_at_size(
            self.storage_index,
            Self::offset(slot),
            RECORD_SIZE,
        )?))
    }

    fn remove_edge_record(&mut self, storage: &mut Storage<D>, slot: u64) -> Result<(), DbError> {
        let edge = self.record(storage, slot)?;
        self.unlink(
            storage,
            -edge[FROM],
            FROM,
            edge[FROM_META],
            edge[PREVIOUS_FROM],
        )?;
        self.unlink(storage, -edge[TO], TO, edge[TO_META], edge[PREVIOUS_TO])?;
        self.free(storage, slot, 0)
    }

    /// Removes an edge from the `head` (FROM or TO) list of the `node`
    /// given its `next` and `previous` edges in that list. The list
    /// links of an edge and the count of a node are 2 fields after the
    /// head and the previous links 4 fields after it.
    fn unlink(
        &mut self,
        storage: &mut Storage<D>,
        node: i64,
        head: usize,
        next: i64,
        previous: i64,
    ) -> Result<(), DbError> {
        let record = self.record(storage, node as u64)?;
        let count = record[head + 2] - 1;

        if previous == 0 {
            self.write(storage, node as u64, head, &[next, record[head + 1], count])?;
        } else {
            self.write(storage, previous as u64, head + 2, &[next])?;
            self.write(storage, node as u64, head + 2, &[count])?;
        }

        if next != 0 {
            self.write(storage, next as u64, head + 4, &[previous])?;
        }

        Ok(())
    }

    fn edge_record(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<Option<Record>, DbError> {
        Ok(self
            .element(storage, index)?
            .filter(|record| index.is_edge() && record[FROM] < 0))
    }

    fn node_record(
        &self,
        storage: &Storage<D>,
        index: GraphIndex,
    ) -> Result<Option<Record>, DbError> {
        Ok(self
            .element(storage, index)?
            .filter(|record| index.is_node() && 0 <= record[FROM]))
    }

    fn validate_node(&self, storage: &Storage<D>, index: GraphIndex) -> Result<Record, DbError> {
        self.node_record(storage, index)?
            .ok_or_else(|| Self::invalid_index(index))
    }

    fn write(
        &mut self,
        storage: &mut Storage<D>,
        slot: u64,
        field: usize,
        values: &[i64],
    ) -> Result<(), DbError> {
        storage.insert_bytes_at(
            self.storage_index,
            Self::offset(slot) + field as u64 * FIELD_SIZE,
            &to_bytes(values),
        )
    }
}

/// The graph of the databases of version 1: four vectors of the
/// `from`, `to`, `from_meta` and `to_meta` fields of the elements
/// without the previous edge links and the free slot count.
pub(crate) mod v1 {
    use super::*;

    pub(super) const CHUNK: u64 = 4096;

    pub(super) struct Vectors {
        pub(super) from: StorageIndex,
        pub(super) to: StorageIndex,
        pub(super) from_meta: StorageIndex,
        pub(super) to_meta: StorageIndex,
    }

    impl Serialize for Vectors {
        fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
            if bytes.len() < Self::serialized_size_static() as usize {
                return Err(DbError::graph(
                    DbErrorType::NotEnoughData,
                    format!(
                        "Graph v1 indexes deserialization error: got {} bytes, expected {}",
                        bytes.len(),
                        Self::serialized_size_static()
                    ),
                ));
            }

            Ok(Self {
                from: StorageIndex(u64_at(bytes, 0)),
                to: StorageIndex(u64_at(bytes, 1)),
                from_meta: StorageIndex(u64_at(bytes, 2)),
                to_meta: StorageIndex(u64_at(bytes, 3)),
            })
        }

        fn serialize(&self) -> Vec<u8> {
            [self.from.0, self.to.0, self.from_meta.0, self.to_meta.0]
                .iter()
                .flat_map(|index| index.to_le_bytes())
                .collect()
        }

        fn serialized_size(&self) -> u64 {
            Self::serialized_size_static()
        }
    }

    impl SerializeStatic for Vectors {
        fn serialized_size_static() -> u64 {
            StorageIndex::serialized_size_static() * 4
        }
    }

    /// Converts the version 1 graph at `index` and removes it. The
    /// fields are copied in chunks, then every edge links its next
    /// edges back to itself.
    pub(crate) fn convert<D: StorageData>(
        storage: &mut Storage<D>,
        index: StorageIndex,
    ) -> Result<DbGraph<D>, DbError> {
        let vectors = storage.value::<Vectors>(index)?;
        let columns = [vectors.from, vectors.to, vectors.from_meta, vectors.to_meta];
        let len = u64_at(&storage.value_as_bytes_at_size(vectors.from, 0, 8)?, 0);

        if len == 0 {
            return Err(DbError::graph(
                DbErrorType::NotEnoughData,
                "Graph v1 vectors are empty",
            ));
        }

        for column in columns {
            let column_len = u64_at(&storage.value_as_bytes_at_size(column, 0, 8)?, 0);

            if column_len != len {
                return Err(DbError::graph(
                    DbErrorType::OutOfBounds,
                    format!("Graph v1 vectors have different lengths ({column_len} != {len})"),
                ));
            }
        }

        let id = storage.transaction();
        let storage_index = storage.insert_zeroed(DbGraph::<D>::offset(len))?;
        storage.insert_bytes_at(storage_index, 0, &len.serialize())?;
        let mut graph = DbGraph {
            storage_index,
            len,
            capacity: len,
            storage: PhantomData,
        };
        let mut free = 0;

        for start in (0..len).step_by(CHUNK as usize) {
            let count = std::cmp::min(CHUNK, len - start);
            let mut fields = Vec::with_capacity(columns.len());

            for column in columns {
                fields.push(
                    storage
                        .value_as_bytes_at_size(column, 8 + start * 8, count * 8)?
                        .to_vec(),
                );
            }

            let mut records = Vec::with_capacity((count * RECORD_SIZE) as usize);

            for i in 0..count as usize {
                let record = [
                    u64_at(&fields[0], i) as i64,
                    u64_at(&fields[1], i) as i64,
                    u64_at(&fields[2], i) as i64,
                    u64_at(&fields[3], i) as i64,
                    0,
                    0,
                ];

                if start + (i as u64) != 0 && record[FROM_META] < 0 {
                    free += 1;
                }

                records.extend(to_bytes(&record));
            }

            storage.insert_bytes_at(storage_index, DbGraph::<D>::offset(start), &records)?;
        }

        graph.write(storage, 0, FREE_COUNT, &[free])?;

        for start in (1..len).step_by(CHUNK as usize) {
            let count = std::cmp::min(CHUNK, len - start);
            let records = storage
                .value_as_bytes_at_size(
                    storage_index,
                    DbGraph::<D>::offset(start),
                    count * RECORD_SIZE,
                )?
                .to_vec();

            for i in 0..count as usize {
                let record = to_record(&records[i * RECORD_SIZE as usize..]);
                let slot = start as i64 + i as i64;

                if 0 <= record[FROM_META] && record[FROM] < 0 {
                    for (next, previous) in [
                        (record[FROM_META], PREVIOUS_FROM),
                        (record[TO_META], PREVIOUS_TO),
                    ] {
                        if next != 0 {
                            if next < 0
                                || !graph
                                    .element(storage, GraphIndex(-next))?
                                    .is_some_and(|record| record[FROM] < 0)
                            {
                                return Err(DbError::graph(
                                    DbErrorType::InvalidIndex,
                                    format!("Graph v1 edge {} links invalid edge {next}", -slot),
                                ));
                            }

                            graph.write(storage, next as u64, previous, &[slot])?;
                        }
                    }
                }
            }
        }

        for column in columns {
            storage.remove(column)?;
        }

        storage.remove(index)?;
        storage.commit(id)?;

        Ok(graph)
    }

    #[cfg(test)]
    pub(crate) fn write<D: StorageData>(
        storage: &mut Storage<D>,
        records: &[[i64; 4]],
    ) -> StorageIndex {
        let mut columns = [vec![], vec![], vec![], vec![]];

        for (c, column) in columns.iter_mut().enumerate() {
            column.extend((records.len() as u64).serialize());

            for record in records {
                column.extend(record[c].to_le_bytes());
            }
        }

        let vectors = Vectors {
            from: storage.insert_bytes(&columns[0]).unwrap(),
            to: storage.insert_bytes(&columns[1]).unwrap(),
            from_meta: storage.insert_bytes(&columns[2]).unwrap(),
            to_meta: storage.insert_bytes(&columns[3]).unwrap(),
        };

        storage.insert(&vectors).unwrap()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::collections::tests::assert_io_failures_recover;
        use crate::collections::tests::storage;
        use crate::graph::tests::assert_consistent;
        use crate::graph::tests::random_graph;
        use crate::graph::tests::records;
        use crate::graph::tests::snapshot;

        /// The version 1 fields of the records, the first four.
        fn v1_records(records: &[Record]) -> Vec<[i64; 4]> {
            records
                .iter()
                .enumerate()
                .map(|(slot, r)| {
                    if slot == 0 {
                        [0, 0, r[FREE_HEAD], r[NODE_COUNT]]
                    } else {
                        [r[FROM], r[TO], r[FROM_META], r[TO_META]]
                    }
                })
                .collect()
        }

        mod vectors {
            use super::*;

            mod serialize {
                use super::*;

                mod serialized_size {
                    use super::*;

                    #[test]
                    fn serialized_size_should_be_size_of_four_indexes() {
                        let vectors = Vectors {
                            from: StorageIndex(1),
                            to: StorageIndex(2),
                            from_meta: StorageIndex(3),
                            to_meta: StorageIndex(4),
                        };

                        assert_eq!(vectors.serialized_size(), 32);
                        assert_eq!(vectors.serialize().len(), 32);
                    }
                }
            }
        }

        mod convert {
            use super::*;

            #[test]
            fn convert_should_restore_records_of_graph() {
                let mut storage = storage();
                let graph = random_graph(&mut storage, CHUNK as usize * 2 + 100, 5);
                let expected = records(&storage, &graph);
                let index = write(&mut storage, &v1_records(&expected));

                let converted = convert(&mut storage, index).unwrap();

                assert_eq!(converted.len, graph.len);
                assert_eq!(converted.capacity, graph.len);
                assert!(0 < expected[0][FREE_COUNT]);
                assert_eq!(records(&storage, &converted), expected);
                assert_consistent(&storage, &converted);
            }

            #[test]
            fn convert_should_remove_version_1_values() {
                let mut storage = storage();
                let index = write(&mut storage, &[[0, 0, i64::MIN, 1], [0, 0, 0, 0]]);
                let vectors = storage.value::<Vectors>(index).unwrap();

                let graph = convert(&mut storage, index).unwrap();

                for removed in [
                    index,
                    vectors.from,
                    vectors.to,
                    vectors.from_meta,
                    vectors.to_meta,
                ] {
                    assert!(storage.value_size(removed).is_err(), "{removed:?}");
                }
                assert_eq!(graph.node_count(&storage).unwrap(), 1);
                assert!(graph.node(&storage, GraphIndex(1)).unwrap().is_some());
            }

            #[test]
            fn convert_different_lengths_should_fail() {
                let mut storage = storage();
                let index = write(&mut storage, &[[0, 0, i64::MIN, 0]]);
                let vectors = storage.value::<Vectors>(index).unwrap();
                storage
                    .insert_bytes_at(vectors.to_meta, 0, &2_u64.serialize())
                    .unwrap();

                assert_eq!(
                    convert(&mut storage, index).err(),
                    Some(DbError::graph(
                        DbErrorType::OutOfBounds,
                        "Graph v1 vectors have different lengths (2 != 1)"
                    ))
                );
            }

            #[test]
            fn convert_empty_should_fail() {
                let mut storage = storage();
                let index = write(&mut storage, &[]);

                assert_eq!(
                    convert(&mut storage, index).err(),
                    Some(DbError::graph(
                        DbErrorType::NotEnoughData,
                        "Graph v1 vectors are empty"
                    ))
                );
            }

            #[test]
            fn convert_invalid_next_edge_should_fail() {
                for next in [3, -1, 1] {
                    let mut storage = storage();
                    let index = write(
                        &mut storage,
                        &[[0, 0, i64::MIN, 1], [2, 2, 1, 1], [-1, -1, 0, next]],
                    );

                    assert_eq!(
                        convert(&mut storage, index).err(),
                        Some(DbError::graph(
                            DbErrorType::InvalidIndex,
                            format!("Graph v1 edge -2 links invalid edge {next}")
                        ))
                    );
                }
            }

            #[test]
            fn convert_short_index_should_fail() {
                let mut storage = storage();
                let index = storage.insert(&1_u64).unwrap();

                assert_eq!(
                    convert(&mut storage, index).err(),
                    Some(DbError::graph(
                        DbErrorType::NotEnoughData,
                        "Graph v1 indexes deserialization error: got 8 bytes, expected 32"
                    ))
                );
            }

            #[test]
            fn convert_should_keep_version_1_values_on_io_failure() {
                let records = [
                    [0, 0, -5, 2],
                    [4, 0, 2, 0],
                    [0, 4, 0, 2],
                    [-1, -2, 0, 0],
                    [-1, -2, 3, 3],
                    [0, 0, i64::MIN, 0],
                ];

                assert_io_failures_recover(
                    |storage| write(storage, &records),
                    |storage, index| convert(storage, *index),
                    |storage, index| {
                        let graph = convert(storage, *index).unwrap();
                        assert_eq!(
                            snapshot(storage, &graph),
                            vec![
                                (1, vec![-4, -3], vec![]),
                                (2, vec![], vec![-4, -3]),
                                (-3, vec![1], vec![2]),
                                (-4, vec![1], vec![2]),
                            ]
                        );
                        assert_consistent(storage, &graph);
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::tests::TestStorage;
    use crate::collections::tests::assert_io_failures_recover;
    use crate::collections::tests::assert_read_failures;
    use crate::collections::tests::storage;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::test_platform::TestPlatform;

    pub(super) type TestGraph = DbGraph<FileStorageImpl<TestPlatform>>;
    pub(super) type Snapshot = Vec<(i64, Vec<i64>, Vec<i64>)>;

    fn graph(storage: &mut TestStorage) -> TestGraph {
        DbGraph::new(storage).unwrap()
    }

    fn nodes(storage: &mut TestStorage, graph: &mut TestGraph, count: usize) -> Vec<GraphIndex> {
        (0..count)
            .map(|_| graph.insert_node(storage).unwrap())
            .collect()
    }

    fn edge(storage: &mut TestStorage, graph: &mut TestGraph, from: i64, to: i64) -> GraphIndex {
        graph
            .insert_edge(storage, GraphIndex(from), GraphIndex(to))
            .unwrap()
    }

    /// Every element in iteration order with the edges from and to it,
    /// for an edge its origin and its destination.
    pub(super) fn snapshot(storage: &TestStorage, graph: &TestGraph) -> Snapshot {
        graph
            .iter(storage)
            .map(|index| {
                let index = index.unwrap();

                if let Some(node) = graph.node(storage, index).unwrap() {
                    (
                        index.0,
                        node.edge_iter_from()
                            .map(|e| e.unwrap().index().0)
                            .collect(),
                        node.edge_iter_to().map(|e| e.unwrap().index().0).collect(),
                    )
                } else {
                    let edge = graph.edge(storage, index).unwrap().unwrap();
                    (index.0, vec![edge.index_from().0], vec![edge.index_to().0])
                }
            })
            .collect()
    }

    pub(super) fn records(storage: &TestStorage, graph: &TestGraph) -> Vec<Record> {
        (0..graph.len)
            .map(|slot| graph.record(storage, slot).unwrap())
            .collect()
    }

    /// Checks the free list, the counters of the root and that the
    /// lists of the nodes link every edge in both directions.
    pub(super) fn assert_consistent(storage: &TestStorage, graph: &TestGraph) {
        let records = records(storage, graph);
        let root = records[0];
        let mut free = 0;
        let mut next = root[FREE_HEAD];

        while next != NO_FREE {
            assert!(records[(-next) as usize][FROM_META] < 0, "free {next}");
            free += 1;
            next = records[(-next) as usize][FROM_META];
        }

        let slots = &records[1..];
        let nodes = slots
            .iter()
            .filter(|r| 0 <= r[FROM_META] && 0 <= r[FROM])
            .count();
        let edges = slots
            .iter()
            .filter(|r| 0 <= r[FROM_META] && r[FROM] < 0)
            .count();
        assert_eq!(slots.iter().filter(|r| r[FROM_META] < 0).count(), free);
        assert_eq!(root[FREE_COUNT], free as i64);
        assert_eq!(root[NODE_COUNT], nodes as i64);
        assert_eq!(graph.edge_count(storage).unwrap(), edges as u64);
        let mut linked = 0;

        for (slot, record) in records.iter().enumerate().skip(1) {
            if record[FROM_META] < 0 || record[FROM] < 0 {
                continue;
            }

            for head in [FROM, TO] {
                let mut previous = 0;
                let mut edge = record[head];
                let mut count = 0;

                while edge != 0 {
                    let link = records[edge as usize];
                    assert_eq!(-link[head], slot as i64, "edge {edge} of {slot}");
                    assert_eq!(link[head + 4], previous, "previous of edge {edge}");
                    previous = edge;
                    edge = link[head + 2];
                    count += 1;
                }

                assert_eq!(record[head + 2], count, "count of {slot}");
                linked += count;
            }
        }

        assert_eq!(linked, 2 * edges as i64);
    }

    /// Checks that each failure of the `action` leaves the graph of the
    /// `setup` once the storage recovers.
    fn assert_io_failures_restore<R>(
        setup: impl Fn(&mut TestStorage) -> TestGraph,
        action: impl Fn(&mut TestStorage, &mut TestGraph) -> Result<R, DbError>,
    ) {
        let mut expected_storage = storage();
        let expected_graph = setup(&mut expected_storage);
        let expected = records(&expected_storage, &expected_graph);

        assert_io_failures_recover(setup, action, |storage, graph| {
            let graph = DbGraph::from_storage(storage, graph.storage_index()).unwrap();
            assert_eq!(records(storage, &graph), expected);
        });
    }

    /// A pseudo random graph with removed elements, one free slot per
    /// `free` elements.
    pub(super) fn random_graph(
        storage: &mut TestStorage,
        elements: usize,
        free: usize,
    ) -> TestGraph {
        let mut graph = graph(storage);
        let mut nodes = nodes(storage, &mut graph, elements / 3);
        let mut seed = 7_u64;
        let mut random = |n: usize| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as usize % n
        };

        while (graph.len as usize) <= elements {
            let from = nodes[random(nodes.len())];
            let to = nodes[random(nodes.len())];
            let edge = graph.insert_edge(storage, from, to).unwrap();

            if random(free) == 0 {
                graph.remove_edge(storage, edge).unwrap();
            }

            if random(free * 3) == 0 {
                let node = nodes.swap_remove(random(nodes.len()));
                graph.remove_node(storage, node).unwrap();
            }
        }

        let elements: Vec<GraphIndex> = graph
            .iter(storage)
            .step_by(free)
            .map(Result::unwrap)
            .collect();

        for index in elements {
            if index.is_node() {
                graph.remove_node(storage, index).unwrap();
            } else {
                graph.remove_edge(storage, index).unwrap();
            }
        }

        graph
    }

    mod graph_index {
        use super::*;

        mod is_edge {
            use super::*;

            #[test]
            fn is_edge_should_be_true_only_for_negative_indexes() {
                assert!(GraphIndex(-1).is_edge());
                assert!(!GraphIndex(0).is_edge());
                assert!(!GraphIndex(1).is_edge());
            }
        }

        mod is_node {
            use super::*;

            #[test]
            fn is_node_should_be_true_only_for_positive_indexes() {
                assert!(GraphIndex(1).is_node());
                assert!(!GraphIndex(0).is_node());
                assert!(!GraphIndex(-1).is_node());
            }
        }

        mod is_valid {
            use super::*;

            #[test]
            fn is_valid_should_be_false_only_for_zero() {
                assert!(GraphIndex(-1).is_valid());
                assert!(!GraphIndex(0).is_valid());
                assert!(GraphIndex(1).is_valid());
            }
        }

        mod as_u64 {
            use super::*;

            #[test]
            fn as_u64_should_return_slot() {
                assert_eq!(GraphIndex(-3).as_u64(), 3);
                assert_eq!(GraphIndex(0).as_u64(), 0);
                assert_eq!(GraphIndex(3).as_u64(), 3);
            }

            #[test]
            fn as_u64_min_should_return_magnitude() {
                assert_eq!(GraphIndex(i64::MIN).as_u64(), 1 << 63);
            }
        }

        mod from {
            use super::*;

            #[test]
            fn from_should_wrap_value() {
                assert_eq!(GraphIndex::from(-2), GraphIndex(-2));
            }
        }

        mod stable_hash {
            use super::*;

            #[allow(clippy::module_inception)]
            mod stable_hash {
                use super::*;

                #[test]
                fn stable_hash_should_hash_value() {
                    assert_eq!(GraphIndex(10).stable_hash(), 10_i64.stable_hash());
                }
            }
        }
    }

    mod graph_node {
        use super::*;

        mod edge_iter_from {
            use super::*;

            #[test]
            fn edge_iter_from_should_return_edges_newest_first() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 3);
                let first = edge(&mut storage, &mut graph, 1, 2);
                let second = edge(&mut storage, &mut graph, 1, 3);
                edge(&mut storage, &mut graph, 2, 1);

                let node = graph.node(&storage, GraphIndex(1)).unwrap().unwrap();
                let edges: Vec<GraphIndex> =
                    node.edge_iter_from().map(|e| e.unwrap().index()).collect();

                assert_eq!(edges, vec![second, first]);
            }

            #[test]
            fn edge_iter_from_read_failure_should_return_error() {
                assert_read_failures(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 2);
                        edge(storage, &mut graph, 1, 2);
                        edge(storage, &mut graph, 1, 1);
                        graph
                    },
                    |storage, graph| {
                        graph
                            .node(storage, GraphIndex(1))?
                            .unwrap()
                            .edge_iter_from()
                            .map(|edge| edge.map(|edge| edge.index()))
                            .collect::<Result<Vec<GraphIndex>, DbError>>()
                    },
                );
            }
        }

        mod edge_iter_to {
            use super::*;

            #[test]
            fn edge_iter_to_should_return_edges_newest_first() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 3);
                let first = edge(&mut storage, &mut graph, 1, 3);
                let second = edge(&mut storage, &mut graph, 2, 3);
                edge(&mut storage, &mut graph, 3, 1);

                let node = graph.node(&storage, GraphIndex(3)).unwrap().unwrap();
                let edges: Vec<GraphIndex> =
                    node.edge_iter_to().map(|e| e.unwrap().index()).collect();

                assert_eq!(edges, vec![second, first]);
            }

            #[test]
            fn edge_iter_to_read_failure_should_return_error() {
                assert_read_failures(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 2);
                        edge(storage, &mut graph, 2, 1);
                        edge(storage, &mut graph, 1, 1);
                        graph
                    },
                    |storage, graph| {
                        graph
                            .node(storage, GraphIndex(1))?
                            .unwrap()
                            .edge_iter_to()
                            .map(|edge| edge.map(|edge| edge.index()))
                            .collect::<Result<Vec<GraphIndex>, DbError>>()
                    },
                );
            }
        }

        mod edge_count {
            use super::*;

            #[test]
            fn edge_count_should_count_both_directions() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 2, 1);
                edge(&mut storage, &mut graph, 1, 1);

                let node = graph.node(&storage, GraphIndex(1)).unwrap().unwrap();

                assert_eq!(node.edge_count(), 4);
            }
        }

        mod edge_count_from {
            use super::*;

            #[test]
            fn edge_count_from_should_count_outgoing_edges() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 1, 2);
                let removed = edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 1, 1);
                edge(&mut storage, &mut graph, 2, 1);
                graph.remove_edge(&mut storage, removed).unwrap();

                let node = graph.node(&storage, GraphIndex(1)).unwrap().unwrap();

                assert_eq!(node.edge_count_from(), 2);
            }
        }

        mod edge_count_to {
            use super::*;

            #[test]
            fn edge_count_to_should_count_incoming_edges() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 1, 2);
                let removed = edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 2, 2);
                edge(&mut storage, &mut graph, 2, 1);
                graph.remove_edge(&mut storage, removed).unwrap();

                let node = graph.node(&storage, GraphIndex(2)).unwrap().unwrap();

                assert_eq!(node.edge_count_to(), 2);
            }
        }
    }

    mod graph_iterator {
        use super::*;

        mod iterator {
            use super::*;

            mod next {
                use super::*;

                #[test]
                fn next_should_skip_free_slots() {
                    let mut storage = storage();
                    let mut graph = graph(&mut storage);
                    nodes(&mut storage, &mut graph, 4);
                    edge(&mut storage, &mut graph, 1, 4);
                    let removed = edge(&mut storage, &mut graph, 4, 1);
                    edge(&mut storage, &mut graph, 4, 4);
                    graph.remove_node(&mut storage, GraphIndex(2)).unwrap();
                    graph.remove_edge(&mut storage, removed).unwrap();

                    let mut iterator = graph.iter(&storage);

                    assert_eq!(iterator.next().map(Result::unwrap), Some(GraphIndex(1)));
                    assert_eq!(iterator.next().map(Result::unwrap), Some(GraphIndex(3)));
                    assert_eq!(iterator.next().map(Result::unwrap), Some(GraphIndex(4)));
                    assert_eq!(iterator.next().map(Result::unwrap), Some(GraphIndex(-5)));
                    assert_eq!(iterator.next().map(Result::unwrap), Some(GraphIndex(-7)));
                    assert_eq!(iterator.next(), None);
                    assert_eq!(iterator.next(), None);
                }

                #[test]
                fn next_read_failure_should_return_error() {
                    assert_read_failures(
                        |storage| {
                            let mut graph = graph(storage);
                            nodes(storage, &mut graph, 2);
                            edge(storage, &mut graph, 1, 2);
                            graph
                        },
                        |storage, graph| {
                            graph
                                .iter(storage)
                                .collect::<Result<Vec<GraphIndex>, DbError>>()
                        },
                    );
                }
            }
        }
    }

    mod graph_edge {
        use super::*;

        mod index {
            use super::*;

            #[test]
            fn index_should_return_edge_index() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let index = edge(&mut storage, &mut graph, 1, 2);

                let edge = graph.edge(&storage, index).unwrap().unwrap();

                assert_eq!(edge.index(), GraphIndex(-3));
            }
        }

        mod index_from {
            use super::*;

            #[test]
            fn index_from_should_return_origin() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let index = edge(&mut storage, &mut graph, 2, 1);

                let edge = graph.edge(&storage, index).unwrap().unwrap();

                assert_eq!(edge.index_from(), GraphIndex(2));
            }
        }

        mod index_to {
            use super::*;

            #[test]
            fn index_to_should_return_destination() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let index = edge(&mut storage, &mut graph, 2, 1);

                let edge = graph.edge(&storage, index).unwrap().unwrap();

                assert_eq!(edge.index_to(), GraphIndex(1));
            }
        }
    }

    mod graph_edge_iterator {
        use super::*;

        mod iterator {
            use super::*;

            mod next {
                use super::*;

                #[test]
                fn next_should_follow_outgoing_edges_until_none() {
                    let mut storage = storage();
                    let mut graph = graph(&mut storage);
                    nodes(&mut storage, &mut graph, 2);
                    let first = edge(&mut storage, &mut graph, 1, 2);
                    let second = edge(&mut storage, &mut graph, 1, 1);
                    let node = graph.node(&storage, GraphIndex(1)).unwrap().unwrap();

                    let mut iterator = node.edge_iter_from();

                    assert_eq!(iterator.next().map(|e| e.unwrap().index()), Some(second));
                    assert_eq!(iterator.next().map(|e| e.unwrap().index()), Some(first));
                    assert!(iterator.next().is_none());
                    assert!(iterator.next().is_none());
                }

                #[test]
                fn next_should_follow_incoming_edges_until_none() {
                    let mut storage = storage();
                    let mut graph = graph(&mut storage);
                    nodes(&mut storage, &mut graph, 2);
                    let first = edge(&mut storage, &mut graph, 2, 1);
                    let second = edge(&mut storage, &mut graph, 1, 1);
                    let node = graph.node(&storage, GraphIndex(1)).unwrap().unwrap();

                    let mut iterator = node.edge_iter_to();

                    assert_eq!(iterator.next().map(|e| e.unwrap().index()), Some(second));
                    assert_eq!(iterator.next().map(|e| e.unwrap().index()), Some(first));
                    assert!(iterator.next().is_none());
                    assert!(iterator.next().is_none());
                }

                #[test]
                fn next_unreadable_edge_should_return_error_last() {
                    let mut storage = storage();
                    let mut graph = graph(&mut storage);
                    nodes(&mut storage, &mut graph, 2);
                    edge(&mut storage, &mut graph, 1, 2);
                    edge(&mut storage, &mut graph, 1, 2);
                    storage
                        .resize_value(graph.storage_index(), TestGraph::offset(4))
                        .unwrap();
                    let node = graph.node(&storage, GraphIndex(1)).unwrap().unwrap();
                    let mut iterator = node.edge_iter_from();

                    assert!(iterator.next().unwrap().is_err());
                    assert!(iterator.next().is_none());
                }
            }
        }
    }

    mod db_graph {
        use super::*;

        #[derive(Clone, Copy, PartialEq)]
        enum Slot {
            Free,
            Node,
            Edge(usize, usize),
        }

        /// The graph as plain vectors to compare the graph to.
        struct Model {
            slots: Vec<Slot>,
            from: Vec<Vec<usize>>,
            to: Vec<Vec<usize>>,
            free: Vec<usize>,
        }

        impl Model {
            fn allocate(&mut self, slot: Slot) -> usize {
                let index = self.free.pop().unwrap_or_else(|| {
                    self.slots.push(Slot::Free);
                    self.from.push(vec![]);
                    self.to.push(vec![]);
                    self.slots.len() - 1
                });
                self.slots[index] = slot;
                index
            }

            fn remove_edge(&mut self, edge: usize) {
                for list in self.from.iter_mut().chain(self.to.iter_mut()) {
                    list.retain(|e| *e != edge);
                }

                self.slots[edge] = Slot::Free;
                self.free.push(edge);
            }

            fn remove_node(&mut self, node: usize) {
                while let Some(edge) = self.from[node].first().copied() {
                    self.remove_edge(edge);
                }

                while let Some(edge) = self.to[node].first().copied() {
                    self.remove_edge(edge);
                }

                self.slots[node] = Slot::Free;
                self.free.push(node);
            }

            fn snapshot(&self) -> Snapshot {
                let ids = |edges: &Vec<usize>| edges.iter().map(|e| -(*e as i64)).collect();
                self.slots
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, kind)| match kind {
                        Slot::Free => None,
                        Slot::Node => {
                            Some((slot as i64, ids(&self.from[slot]), ids(&self.to[slot])))
                        }
                        Slot::Edge(from, to) => {
                            Some((-(slot as i64), vec![*from as i64], vec![*to as i64]))
                        }
                    })
                    .collect()
            }
        }

        #[test]
        fn db_graph_should_match_model_under_random_operations() {
            let mut storage = storage();
            let mut graph = graph(&mut storage);
            let mut model = Model {
                slots: vec![Slot::Free],
                from: vec![vec![]],
                to: vec![vec![]],
                free: vec![],
            };
            let mut seed = 11_u64;
            let mut random = |n: usize| {
                seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                (seed >> 33) as usize % n
            };

            for _ in 0..3000 {
                let nodes: Vec<usize> = (1..model.slots.len())
                    .filter(|s| model.slots[*s] == Slot::Node)
                    .collect();
                let edges: Vec<usize> = (1..model.slots.len())
                    .filter(|s| matches!(model.slots[*s], Slot::Edge(_, _)))
                    .collect();

                match random(10) {
                    0..=2 => {
                        let index = graph.insert_node(&mut storage).unwrap();
                        assert_eq!(index.0 as usize, model.allocate(Slot::Node));
                    }
                    3..=6 if !nodes.is_empty() => {
                        let from = nodes[random(nodes.len())];
                        let to = nodes[random(nodes.len())];
                        let index = edge(&mut storage, &mut graph, from as i64, to as i64);
                        let slot = model.allocate(Slot::Edge(from, to));
                        model.from[from].insert(0, slot);
                        model.to[to].insert(0, slot);
                        assert_eq!(index.0, -(slot as i64));
                    }
                    7..=8 if !edges.is_empty() => {
                        let edge = edges[random(edges.len())];
                        graph
                            .remove_edge(&mut storage, GraphIndex(-(edge as i64)))
                            .unwrap();
                        model.remove_edge(edge);
                    }
                    9 if !nodes.is_empty() => {
                        let node = nodes[random(nodes.len())];
                        graph
                            .remove_node(&mut storage, GraphIndex(node as i64))
                            .unwrap();
                        model.remove_node(node);
                    }
                    _ => {}
                }

                assert_consistent(&storage, &graph);
                assert_eq!(snapshot(&storage, &graph), model.snapshot());
            }
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_create_empty_graph() {
                let mut storage = storage();

                let graph = graph(&mut storage);

                assert_eq!(graph.node_count(&storage).unwrap(), 0);
                assert_eq!(graph.edge_count(&storage).unwrap(), 0);
                assert_eq!(graph.iter(&storage).next(), None);
                assert_eq!(records(&storage, &graph), vec![ROOT]);
            }

            #[test]
            fn new_should_return_io_failures() {
                crate::collections::tests::assert_io_failures(
                    |_| (),
                    |storage, _| DbGraph::new(storage),
                );
            }
        }

        mod from_storage {
            use super::*;

            #[test]
            fn from_storage_should_restore_graph() {
                let mut storage = storage();
                let graph = random_graph(&mut storage, 100, 4);

                let restored = DbGraph::from_storage(&storage, graph.storage_index()).unwrap();

                assert_eq!(restored.len, graph.len);
                assert_eq!(restored.capacity, graph.capacity);
                assert_eq!(snapshot(&storage, &restored), snapshot(&storage, &graph));
            }

            #[test]
            fn from_storage_zero_length_should_fail() {
                let mut storage = storage();
                let index = storage
                    .insert_bytes(&[0_u64.serialize(), to_bytes(&ROOT)].concat())
                    .unwrap();

                let error = DbGraph::from_storage(&storage, index).err().unwrap();

                assert_eq!(
                    error,
                    DbError::graph(
                        DbErrorType::OutOfBounds,
                        "Graph length (0) is not within its capacity (1..=1)"
                    )
                );
            }

            #[test]
            fn from_storage_length_over_capacity_should_fail() {
                let mut storage = storage();
                let index = storage
                    .insert_bytes(&[2_u64.serialize(), to_bytes(&ROOT)].concat())
                    .unwrap();

                let error = DbGraph::from_storage(&storage, index).err().unwrap();

                assert_eq!(
                    error,
                    DbError::graph(
                        DbErrorType::OutOfBounds,
                        "Graph length (2) is not within its capacity (1..=1)"
                    )
                );
            }

            #[test]
            fn from_storage_should_return_read_failures() {
                crate::collections::tests::assert_io_failures(
                    |storage| graph(storage).storage_index(),
                    |storage, index| DbGraph::from_storage(storage, *index),
                );
            }

            #[test]
            fn from_storage_missing_value_should_fail() {
                let storage = storage();

                assert!(DbGraph::from_storage(&storage, StorageIndex(10)).is_err());
            }
        }

        mod storage_index {
            use super::*;

            #[test]
            fn storage_index_should_return_index_of_value() {
                let mut storage = storage();
                let graph = graph(&mut storage);

                let size = storage.value_size(graph.storage_index()).unwrap();

                assert_eq!(size, DbGraph::<FileStorageImpl<TestPlatform>>::offset(1));
            }
        }

        mod alias {
            use super::*;

            #[test]
            fn alias_should_return_alias_fields_of_node() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let alias = [7; 16];
                graph
                    .set_alias(&mut storage, GraphIndex(2), &alias)
                    .unwrap();

                assert_eq!(graph.alias(&storage, GraphIndex(1)).unwrap(), Some([0; 16]));
                assert_eq!(graph.alias(&storage, GraphIndex(2)).unwrap(), Some(alias));
            }

            #[test]
            fn alias_no_node_should_return_none() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 3);
                let edge = edge(&mut storage, &mut graph, 1, 2);
                graph.remove_node(&mut storage, GraphIndex(3)).unwrap();

                for index in [
                    edge,
                    GraphIndex(-edge.0),
                    GraphIndex(0),
                    GraphIndex(3),
                    GraphIndex(5),
                ] {
                    assert_eq!(graph.alias(&storage, index).unwrap(), None, "{index:?}");
                }
            }

            #[test]
            fn alias_should_return_read_failure() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 1);
                storage
                    .resize_value(graph.storage_index(), HEADER_SIZE)
                    .unwrap();

                assert!(graph.alias(&storage, GraphIndex(1)).is_err());
            }
        }

        mod edge {
            use super::*;

            #[test]
            fn edge_should_return_existing_edge() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let index = edge(&mut storage, &mut graph, 1, 2);

                assert_eq!(
                    graph.edge(&storage, index).unwrap().map(|e| e.index()),
                    Some(index)
                );
            }

            #[test]
            fn edge_no_edge_should_return_none() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let index = edge(&mut storage, &mut graph, 1, 2);
                let removed = edge(&mut storage, &mut graph, 1, 2);
                graph.remove_edge(&mut storage, removed).unwrap();

                for index in [
                    GraphIndex(1),
                    GraphIndex(index.as_u64() as i64),
                    removed,
                    GraphIndex(-5),
                    GraphIndex(0),
                ] {
                    assert!(graph.edge(&storage, index).unwrap().is_none(), "{index:?}");
                }
            }

            #[test]
            fn edge_read_failure_should_return_error() {
                assert_read_failures(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 2);
                        edge(storage, &mut graph, 1, 2);
                        graph
                    },
                    |storage, graph| {
                        Ok(graph
                            .edge(storage, GraphIndex(-3))?
                            .map(|edge| edge.index_to()))
                    },
                );
            }
        }

        mod edge_count {
            use super::*;

            #[test]
            fn edge_count_should_count_existing_edges() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 3);
                edge(&mut storage, &mut graph, 1, 2);
                let removed = edge(&mut storage, &mut graph, 2, 3);
                edge(&mut storage, &mut graph, 3, 3);
                edge(&mut storage, &mut graph, 3, 1);
                graph.remove_edge(&mut storage, removed).unwrap();
                graph.remove_node(&mut storage, GraphIndex(1)).unwrap();

                assert_eq!(graph.edge_count(&storage).unwrap(), 1);
            }

            #[test]
            fn edge_count_should_return_read_failures() {
                crate::collections::tests::assert_io_failures(graph, |storage, graph| {
                    graph.edge_count(storage)
                });
            }
        }

        mod edge_from {
            use super::*;

            #[test]
            fn edge_from_should_return_origin() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let index = edge(&mut storage, &mut graph, 2, 1);

                assert_eq!(graph.edge_from(&storage, index).unwrap(), GraphIndex(2));
            }

            #[test]
            fn edge_from_out_of_bounds_should_fail() {
                let mut storage = storage();
                let graph = graph(&mut storage);

                assert!(graph.edge_from(&storage, GraphIndex(-1)).is_err());
            }
        }

        mod edge_to {
            use super::*;

            #[test]
            fn edge_to_should_return_destination() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let index = edge(&mut storage, &mut graph, 2, 1);

                assert_eq!(graph.edge_to(&storage, index).unwrap(), GraphIndex(1));
            }

            #[test]
            fn edge_to_out_of_bounds_should_fail() {
                let mut storage = storage();
                let graph = graph(&mut storage);

                assert!(graph.edge_to(&storage, GraphIndex(-1)).is_err());
            }
        }

        mod first_edge_from {
            use super::*;

            #[test]
            fn first_edge_from_should_return_newest_outgoing_edge() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 1, 2);
                let newest = edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 2, 1);

                assert_eq!(
                    graph.first_edge_from(&storage, GraphIndex(1)).unwrap(),
                    newest
                );
            }

            #[test]
            fn first_edge_from_no_edges_should_return_zero() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 2, 1);

                assert_eq!(
                    graph.first_edge_from(&storage, GraphIndex(1)).unwrap(),
                    GraphIndex(0)
                );
            }

            #[test]
            fn first_edge_from_out_of_bounds_should_fail() {
                let mut storage = storage();
                let graph = graph(&mut storage);

                assert!(graph.first_edge_from(&storage, GraphIndex(1)).is_err());
            }
        }

        mod first_edge_to {
            use super::*;

            #[test]
            fn first_edge_to_should_return_newest_incoming_edge() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 1, 2);
                let newest = edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 2, 1);

                assert_eq!(
                    graph.first_edge_to(&storage, GraphIndex(2)).unwrap(),
                    newest
                );
            }

            #[test]
            fn first_edge_to_no_edges_should_return_zero() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 2, 1);

                assert_eq!(
                    graph.first_edge_to(&storage, GraphIndex(2)).unwrap(),
                    GraphIndex(0)
                );
            }

            #[test]
            fn first_edge_to_out_of_bounds_should_fail() {
                let mut storage = storage();
                let graph = graph(&mut storage);

                assert!(graph.first_edge_to(&storage, GraphIndex(1)).is_err());
            }
        }

        mod insert_edge {
            use super::*;

            #[test]
            fn insert_edge_should_link_edge_first_in_both_lists() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 2, 1);

                let index = edge(&mut storage, &mut graph, 1, 2);

                assert_eq!(index, GraphIndex(-5));
                assert_consistent(&storage, &graph);
                assert_eq!(
                    snapshot(&storage, &graph),
                    vec![
                        (1, vec![-5, -3], vec![-4]),
                        (2, vec![-4], vec![-5, -3]),
                        (-3, vec![1], vec![2]),
                        (-4, vec![2], vec![1]),
                        (-5, vec![1], vec![2]),
                    ]
                );
            }

            #[test]
            fn insert_edge_self_loop_should_link_edge_in_both_lists_of_node() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 1);
                edge(&mut storage, &mut graph, 1, 1);

                let index = edge(&mut storage, &mut graph, 1, 1);

                assert_consistent(&storage, &graph);
                assert_eq!(
                    snapshot(&storage, &graph)[0],
                    (1, vec![index.0, -2], vec![index.0, -2])
                );
            }

            #[test]
            fn insert_edge_should_reuse_last_free_slot() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 4);
                graph.remove_node(&mut storage, GraphIndex(2)).unwrap();
                graph.remove_node(&mut storage, GraphIndex(3)).unwrap();

                let first = edge(&mut storage, &mut graph, 1, 4);
                let second = edge(&mut storage, &mut graph, 4, 1);
                let third = edge(&mut storage, &mut graph, 4, 4);

                assert_eq!(
                    [first, second, third],
                    [GraphIndex(-3), GraphIndex(-2), GraphIndex(-5)]
                );
                assert_consistent(&storage, &graph);
            }

            #[test]
            fn insert_edge_invalid_nodes_should_fail() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let edge = edge(&mut storage, &mut graph, 1, 2);
                graph.remove_node(&mut storage, GraphIndex(2)).unwrap();
                let expected = records(&storage, &graph);

                for (from, to, invalid) in [
                    (GraphIndex(1), GraphIndex(2), 2),
                    (GraphIndex(2), GraphIndex(1), 2),
                    (edge, GraphIndex(1), -3),
                    (GraphIndex(1), GraphIndex(9), 9),
                    (GraphIndex(0), GraphIndex(1), 0),
                ] {
                    assert_eq!(
                        graph.insert_edge(&mut storage, from, to),
                        Err(DbError::graph(
                            DbErrorType::InvalidIndex,
                            format!("'{invalid}' is invalid index")
                        ))
                    );
                }

                assert_eq!(records(&storage, &graph), expected);
            }

            #[test]
            fn insert_edge_should_restore_graph_on_io_failure() {
                assert_io_failures_restore(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 2);
                        edge(storage, &mut graph, 1, 2);
                        edge(storage, &mut graph, 2, 1);
                        graph
                    },
                    |storage, graph| graph.insert_edge(storage, GraphIndex(2), GraphIndex(1)),
                );
            }

            #[test]
            fn insert_edge_free_slot_should_restore_graph_on_io_failure() {
                assert_io_failures_restore(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 3);
                        edge(storage, &mut graph, 1, 2);
                        graph.remove_node(storage, GraphIndex(3)).unwrap();
                        graph
                    },
                    |storage, graph| graph.insert_edge(storage, GraphIndex(1), GraphIndex(2)),
                );
            }
        }

        mod insert_node {
            use super::*;

            #[test]
            fn insert_node_should_append_new_slots() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);

                let indexes = nodes(&mut storage, &mut graph, 10);

                assert_eq!(indexes, (1..=10).map(GraphIndex).collect::<Vec<_>>());
                assert_eq!(graph.len, 11);
                assert_eq!(graph.capacity, 13);
                assert_eq!(graph.node_count(&storage).unwrap(), 10);
                assert_consistent(&storage, &graph);
            }

            #[test]
            fn insert_node_should_reuse_last_free_slot() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let edge = edge(&mut storage, &mut graph, 1, 2);
                graph.remove_node(&mut storage, GraphIndex(1)).unwrap();

                let indexes = nodes(&mut storage, &mut graph, 3);

                assert_eq!(
                    indexes,
                    vec![
                        GraphIndex(1),
                        GraphIndex(edge.as_u64() as i64),
                        GraphIndex(4)
                    ]
                );
                assert_eq!(graph.alias(&storage, GraphIndex(3)).unwrap(), Some([0; 16]));
                assert_consistent(&storage, &graph);
            }

            #[test]
            fn insert_node_should_restore_graph_on_io_failure() {
                assert_io_failures_restore(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 2);
                        graph
                    },
                    |storage, graph| graph.insert_node(storage),
                );
            }

            #[test]
            fn insert_node_free_slot_should_restore_graph_on_io_failure() {
                assert_io_failures_restore(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 2);
                        graph.remove_node(storage, GraphIndex(1)).unwrap();
                        graph
                    },
                    |storage, graph| graph.insert_node(storage),
                );
            }
        }

        mod iter {
            use super::*;

            #[test]
            fn iter_should_return_elements_in_slot_order() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 2, 1);
                nodes(&mut storage, &mut graph, 1);

                let elements: Vec<GraphIndex> = graph.iter(&storage).map(Result::unwrap).collect();

                assert_eq!(
                    elements,
                    vec![GraphIndex(1), GraphIndex(2), GraphIndex(-3), GraphIndex(4)]
                );
            }

            #[test]
            fn iter_unreadable_slot_should_return_error_last() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                storage
                    .resize_value(
                        graph.storage_index(),
                        DbGraph::<FileStorageImpl<TestPlatform>>::offset(2),
                    )
                    .unwrap();

                let mut iterator = graph.iter(&storage);

                assert_eq!(iterator.next().map(Result::unwrap), Some(GraphIndex(1)));
                assert!(iterator.next().unwrap().is_err());
                assert!(iterator.next().is_none());
            }
        }

        mod node {
            use super::*;

            #[test]
            fn node_should_return_existing_node() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);

                let node = graph.node(&storage, GraphIndex(2)).unwrap().unwrap();

                assert_eq!(node.edge_count(), 0);
            }

            #[test]
            fn node_no_node_should_return_none() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 3);
                let index = edge(&mut storage, &mut graph, 1, 2);
                graph.remove_node(&mut storage, GraphIndex(3)).unwrap();

                for index in [
                    index,
                    GraphIndex(4),
                    GraphIndex(3),
                    GraphIndex(9),
                    GraphIndex(0),
                    GraphIndex(-1),
                ] {
                    assert!(graph.node(&storage, index).unwrap().is_none(), "{index:?}");
                }
            }

            #[test]
            fn node_read_failure_should_return_error() {
                assert_read_failures(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 2);
                        edge(storage, &mut graph, 1, 2);
                        graph
                    },
                    |storage, graph| {
                        Ok(graph
                            .node(storage, GraphIndex(1))?
                            .map(|node| node.edge_count()))
                    },
                );
            }
        }

        mod node_count {
            use super::*;

            #[test]
            fn node_count_should_count_existing_nodes() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 4);
                edge(&mut storage, &mut graph, 1, 2);
                graph.remove_node(&mut storage, GraphIndex(2)).unwrap();
                graph.remove_node(&mut storage, GraphIndex(2)).unwrap();
                nodes(&mut storage, &mut graph, 1);

                assert_eq!(graph.node_count(&storage).unwrap(), 4);
            }

            #[test]
            fn node_count_should_return_read_failures() {
                crate::collections::tests::assert_io_failures(graph, |storage, graph| {
                    graph.node_count(storage)
                });
            }
        }

        mod remove_edge {
            use super::*;

            fn star(storage: &mut TestStorage) -> TestGraph {
                let mut graph = graph(storage);
                nodes(storage, &mut graph, 2);

                for _ in 0..3 {
                    edge(storage, &mut graph, 1, 2);
                }

                graph
            }

            #[test]
            fn remove_edge_should_unlink_edge_at_any_position() {
                for (removed, remaining) in
                    [(-5, vec![-4, -3]), (-4, vec![-5, -3]), (-3, vec![-5, -4])]
                {
                    let mut storage = storage();
                    let mut graph = star(&mut storage);

                    graph
                        .remove_edge(&mut storage, GraphIndex(removed))
                        .unwrap();

                    assert_consistent(&storage, &graph);
                    assert_eq!(
                        snapshot(&storage, &graph)[..2],
                        [(1, remaining.clone(), vec![]), (2, vec![], remaining)]
                    );
                }
            }

            #[test]
            fn remove_edge_only_edge_should_leave_nodes_without_edges() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let index = edge(&mut storage, &mut graph, 1, 2);

                graph.remove_edge(&mut storage, index).unwrap();

                assert_consistent(&storage, &graph);
                assert_eq!(
                    snapshot(&storage, &graph),
                    vec![(1, vec![], vec![]), (2, vec![], vec![])]
                );
            }

            #[test]
            fn remove_edge_self_loop_should_unlink_edge_from_both_lists() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 1);
                edge(&mut storage, &mut graph, 1, 1);
                let index = edge(&mut storage, &mut graph, 1, 1);
                edge(&mut storage, &mut graph, 1, 1);

                graph.remove_edge(&mut storage, index).unwrap();

                assert_consistent(&storage, &graph);
                assert_eq!(
                    snapshot(&storage, &graph)[0],
                    (1, vec![-4, -2], vec![-4, -2])
                );
            }

            #[test]
            fn remove_edge_no_edge_should_do_nothing() {
                let mut storage = storage();
                let mut graph = star(&mut storage);
                graph.remove_edge(&mut storage, GraphIndex(-4)).unwrap();
                let expected = records(&storage, &graph);

                for index in [GraphIndex(-4), GraphIndex(1), GraphIndex(-9), GraphIndex(0)] {
                    graph.remove_edge(&mut storage, index).unwrap();
                }

                assert_eq!(records(&storage, &graph), expected);
            }

            #[test]
            fn remove_edge_should_restore_graph_on_io_failure() {
                assert_io_failures_restore(star, |storage, graph| {
                    graph.remove_edge(storage, GraphIndex(-4))
                });
            }
        }

        mod remove_node {
            use super::*;

            #[test]
            fn remove_node_should_remove_its_edges() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 3);
                edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 2, 1);
                edge(&mut storage, &mut graph, 2, 2);
                edge(&mut storage, &mut graph, 3, 1);
                edge(&mut storage, &mut graph, 3, 2);

                graph.remove_node(&mut storage, GraphIndex(2)).unwrap();

                assert_consistent(&storage, &graph);
                assert_eq!(
                    snapshot(&storage, &graph),
                    vec![
                        (1, vec![], vec![-7]),
                        (3, vec![-7], vec![]),
                        (-7, vec![3], vec![1])
                    ]
                );
                assert_eq!(graph.node_count(&storage).unwrap(), 2);
            }

            #[test]
            fn remove_node_should_free_edges_before_node() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 2, 1);
                graph.remove_node(&mut storage, GraphIndex(1)).unwrap();

                let indexes = nodes(&mut storage, &mut graph, 4);

                assert_eq!(
                    indexes,
                    vec![GraphIndex(1), GraphIndex(4), GraphIndex(3), GraphIndex(5)]
                );
            }

            #[test]
            fn remove_node_no_node_should_do_nothing() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 1, 2);
                graph.remove_node(&mut storage, GraphIndex(2)).unwrap();
                let expected = records(&storage, &graph);

                for index in [GraphIndex(2), GraphIndex(-3), GraphIndex(9), GraphIndex(0)] {
                    graph.remove_node(&mut storage, index).unwrap();
                }

                assert_eq!(records(&storage, &graph), expected);
            }

            #[test]
            fn remove_node_should_restore_graph_on_io_failure() {
                assert_io_failures_restore(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 2);
                        edge(storage, &mut graph, 1, 2);
                        edge(storage, &mut graph, 2, 2);
                        edge(storage, &mut graph, 2, 1);
                        graph
                    },
                    |storage, graph| graph.remove_node(storage, GraphIndex(2)),
                );
            }
        }

        mod set_alias {
            use super::*;

            #[test]
            fn set_alias_should_keep_edges() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 2, 1);
                let expected = snapshot(&storage, &graph);

                graph
                    .set_alias(&mut storage, GraphIndex(1), &[255; 16])
                    .unwrap();

                assert_eq!(
                    graph.alias(&storage, GraphIndex(1)).unwrap(),
                    Some([255; 16])
                );
                assert_eq!(snapshot(&storage, &graph), expected);
                assert_consistent(&storage, &graph);
            }

            #[test]
            fn set_alias_no_node_should_fail() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let index = edge(&mut storage, &mut graph, 1, 2);

                assert_eq!(
                    graph.set_alias(&mut storage, index, &[1; 16]),
                    Err(DbError::graph(
                        DbErrorType::InvalidIndex,
                        "'-3' is invalid index"
                    ))
                );
                assert_eq!(graph.alias(&storage, GraphIndex(1)).unwrap(), Some([0; 16]));
            }
        }

        mod shrink_to_fit {
            use super::*;

            #[test]
            fn shrink_to_fit_should_drop_spare_capacity() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 10);
                let expected = snapshot(&storage, &graph);

                graph.shrink_to_fit(&mut storage).unwrap();

                let restored = DbGraph::from_storage(&storage, graph.storage_index()).unwrap();
                assert_eq!(restored.capacity, 11);
                assert_eq!(graph.capacity, 11);
                assert_eq!(snapshot(&storage, &restored), expected);
                assert_eq!(graph.insert_node(&mut storage).unwrap(), GraphIndex(11));
                assert_consistent(&storage, &graph);
            }

            #[test]
            fn shrink_to_fit_should_restore_graph_on_io_failure() {
                assert_io_failures_restore(
                    |storage| {
                        let mut graph = graph(storage);
                        nodes(storage, &mut graph, 10);
                        graph
                    },
                    |storage, graph| graph.shrink_to_fit(storage),
                );
            }
        }

        mod next_edge_from {
            use super::*;

            #[test]
            fn next_edge_from_should_return_older_outgoing_edge() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let older = edge(&mut storage, &mut graph, 1, 2);
                edge(&mut storage, &mut graph, 2, 1);
                let newer = edge(&mut storage, &mut graph, 1, 1);

                assert_eq!(graph.next_edge_from(&storage, newer).unwrap(), older);
                assert_eq!(
                    graph.next_edge_from(&storage, older).unwrap(),
                    GraphIndex(0)
                );
            }

            #[test]
            fn next_edge_from_out_of_bounds_should_fail() {
                let mut storage = storage();
                let graph = graph(&mut storage);

                assert!(graph.next_edge_from(&storage, GraphIndex(-1)).is_err());
            }
        }

        mod next_edge_to {
            use super::*;

            #[test]
            fn next_edge_to_should_return_older_incoming_edge() {
                let mut storage = storage();
                let mut graph = graph(&mut storage);
                nodes(&mut storage, &mut graph, 2);
                let older = edge(&mut storage, &mut graph, 2, 1);
                edge(&mut storage, &mut graph, 1, 2);
                let newer = edge(&mut storage, &mut graph, 1, 1);

                assert_eq!(graph.next_edge_to(&storage, newer).unwrap(), older);
                assert_eq!(graph.next_edge_to(&storage, older).unwrap(), GraphIndex(0));
            }

            #[test]
            fn next_edge_to_out_of_bounds_should_fail() {
                let mut storage = storage();
                let graph = graph(&mut storage);

                assert!(graph.next_edge_to(&storage, GraphIndex(-1)).is_err());
            }
        }
    }
}
