use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::collections::array::ARRAY_SIZE;
use crate::collections::array::SegmentedArray;
use crate::storage::StorageSlice;
use crate::storage::alloc::Allocator;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;
use crate::storage::root::IdWidth;
use crate::storage::root::Layout;
use crate::utilities::serialize::u64_at;

const RECORDS_SIZE: usize = ARRAY_SIZE + 24;
/// The bytes of an encoded [`GraphHeader`].
pub(crate) const GRAPH_SIZE: usize = 2 * RECORDS_SIZE;

// Node, 32 B: [first edge from u64][first edge to u64]
// [edge count from u32][edge count to u32][data u64].
// Edge, fields of the id width: [from][to][next edge from][next edge to]
// [previous edge from][previous edge to] then [data u64], 32 B with
// `U32` ids, 64 B with `U64` ones (8 B of 0 at the end).
// Free: [VACANT][next free slot] in fields of the width, VACANT all ones
// for a node and 0 (no node) for an edge.
const NODE_SHIFT: u32 = 5;
const NODE_FIRST: u64 = 0;
const NODE_COUNT: u64 = 16;
const NODE_DATA: u64 = 24;
const EDGE_NEXT: u64 = 2;
const EDGE_PREVIOUS: u64 = 4;
const EDGE_DATA: u64 = 6;
/// The record buffer, the largest record.
const RECORD: usize = 64;

/// The graph: nodes and edges in separate arrays of records, so node `n`
/// and edge `-n` may both exist. The id of an element is the slot of its
/// record, negative for edges.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphHeader {
    nodes: Records,
    edges: Records,
    /// The highest slot of either array.
    limit: u64,
}

/// An array of records of which the slot 0 is never used. Removed
/// records form a list of free records reused last removed first.
#[derive(Clone, Debug, PartialEq)]
struct Records {
    array: SegmentedArray,
    /// The bytes of the fields of ids.
    width: u64,
    vacant: u64,
    len: u64,
    free: u64,
    frees: u64,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Node {
    pub(crate) first_from: DbId,
    pub(crate) first_to: DbId,
    pub(crate) from_count: u64,
    pub(crate) to_count: u64,
    /// Kept for the users of the graph, 0 in a new node.
    pub(crate) data: u64,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Edge {
    pub(crate) from: DbId,
    pub(crate) to: DbId,
    pub(crate) next_from: DbId,
    pub(crate) next_to: DbId,
    pub(crate) previous_from: DbId,
    pub(crate) previous_to: DbId,
    /// Kept for the users of the graph, 0 in a new edge.
    pub(crate) data: u64,
}

/// The ids of the nodes then of the edges in slot order. A failed read
/// is the last item.
pub(crate) struct Elements<'a, V> {
    graph: &'a GraphHeader,
    view: &'a V,
    edges: bool,
    slot: u64,
}

fn invalid(id: DbId) -> DbError {
    DbError::graph(
        DbErrorType::InvalidIndex,
        format!("'{}' is invalid index", id.0),
    )
}

fn node_id(slot: u64) -> DbId {
    DbId(slot as i64)
}

fn edge_id(slot: u64) -> DbId {
    DbId(-(slot as i64))
}

/// The `width` (4 or 8) bytes at `at` of the `bytes` of a record.
fn uint(bytes: &[u8], at: u64, width: u64) -> u64 {
    let at = at as usize;

    if width == 4 {
        return u64::from(u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()));
    }

    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

impl GraphHeader {
    pub(crate) fn new(layout: Layout) -> Self {
        let (width, edge_shift, limit) = match layout.ids {
            IdWidth::U32 => (4, 5, u64::from(u32::MAX)),
            IdWidth::U64 => (8, 6, i64::MAX as u64),
        };

        Self {
            nodes: Records::new(layout.page_shift, NODE_SHIFT, 8, u64::MAX),
            edges: Records::new(layout.page_shift, edge_shift, width, 0),
            limit,
        }
    }

    pub(crate) fn decode(bytes: &[u8], layout: Layout) -> Self {
        let mut graph = Self::new(layout);
        graph.nodes.decode(&bytes[..RECORDS_SIZE]);
        graph.edges.decode(&bytes[RECORDS_SIZE..]);
        graph
    }

    pub(crate) fn encode(&self) -> [u8; GRAPH_SIZE] {
        let mut bytes = [0_u8; GRAPH_SIZE];
        self.nodes.encode(&mut bytes[..RECORDS_SIZE]);
        self.edges.encode(&mut bytes[RECORDS_SIZE..]);
        bytes
    }

    pub(crate) fn node_count(&self) -> u64 {
        self.nodes.count()
    }

    pub(crate) fn edge_count(&self) -> u64 {
        self.edges.count()
    }

    pub(crate) fn elements<'a, V: View>(&'a self, view: &'a V) -> Elements<'a, V> {
        Elements {
            graph: self,
            view,
            edges: false,
            slot: 1,
        }
    }

    pub(crate) fn node(&self, view: &impl View, id: DbId) -> Result<Option<Node>, DbError> {
        if id.0 < 0 {
            return Ok(None);
        }

        Ok(self.nodes.record(view, id.0 as u64)?.map(|bytes| Node {
            first_from: edge_id(u64_at(&bytes, 0)),
            first_to: edge_id(u64_at(&bytes, 1)),
            from_count: uint(&bytes, NODE_COUNT, 4),
            to_count: uint(&bytes, NODE_COUNT + 4, 4),
            data: u64_at(&bytes, NODE_DATA as usize / 8),
        }))
    }

    pub(crate) fn edge(&self, view: &impl View, id: DbId) -> Result<Option<Edge>, DbError> {
        if 0 < id.0 {
            return Ok(None);
        }

        let width = self.edges.width;
        Ok(self.edges.record(view, id.0.unsigned_abs())?.map(|bytes| {
            let field = |field: u64| uint(&bytes, field * width, width);
            Edge {
                from: node_id(field(0)),
                to: node_id(field(1)),
                next_from: edge_id(field(EDGE_NEXT)),
                next_to: edge_id(field(EDGE_NEXT + 1)),
                previous_from: edge_id(field(EDGE_PREVIOUS)),
                previous_to: edge_id(field(EDGE_PREVIOUS + 1)),
                data: uint(&bytes, EDGE_DATA * width, 8),
            }
        }))
    }

    /// Writes the `data` of the element `id`.
    pub(crate) fn set_data(
        &self,
        view: &mut impl ViewMut,
        id: DbId,
        data: u64,
    ) -> Result<(), DbError> {
        let (records, at) = if 0 < id.0 {
            (&self.nodes, NODE_DATA)
        } else {
            (&self.edges, EDGE_DATA * self.edges.width)
        };
        let slot = id.0.unsigned_abs();

        if records.record(view, slot)?.is_none() {
            return Err(invalid(id));
        }

        records.write(view, slot, at, &data.to_le_bytes())
    }

    pub(crate) fn insert_node(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
    ) -> Result<DbId, DbError> {
        let slot = self.nodes.take(view, alloc, self.limit, "node")?;
        self.nodes.write(view, slot, 0, &[0; 1 << NODE_SHIFT])?;
        Ok(node_id(slot))
    }

    pub(crate) fn insert_edge(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        from: DbId,
        to: DbId,
    ) -> Result<DbId, DbError> {
        let Some(origin) = self.node(view, from)? else {
            return Err(invalid(from));
        };
        let Some(destination) = self.node(view, to)? else {
            return Err(invalid(to));
        };
        let ends = [(from, origin.from_count), (to, destination.to_count)];

        for (node, count) in ends {
            if count == u64::from(u32::MAX) {
                return Err(DbError::graph(
                    DbErrorType::OutOfBounds,
                    format!("Node '{}' has too many edges ({count})", node.0),
                ));
            }
        }

        let slot = self.edges.take(view, alloc, self.limit, "edge")?;
        let next = [-origin.first_from.0 as u64, -destination.first_to.0 as u64];
        let width = self.edges.width;
        let mut record = [0_u8; RECORD];
        let fields = [from.0 as u64, to.0 as u64, next[0], next[1]];

        for (field, value) in fields.into_iter().enumerate() {
            record[field * width as usize..][..width as usize]
                .copy_from_slice(&value.to_le_bytes()[..width as usize]);
        }

        self.edges
            .write(view, slot, 0, &record[..8 * width as usize])?;

        for (direction, ((node, count), next)) in ends.into_iter().zip(next).enumerate() {
            let direction = direction as u64;

            if next != 0 {
                self.edges
                    .set(view, next, (EDGE_PREVIOUS + direction) * width, slot)?;
            }

            let node = node.0 as u64;
            self.nodes
                .set(view, node, NODE_FIRST + direction * 8, slot)?;
            self.nodes
                .set_u32(view, node, NODE_COUNT + direction * 4, count + 1)?;
        }

        Ok(edge_id(slot))
    }

    /// Removes the node `id` with its edges, if it exists.
    pub(crate) fn remove_node(&mut self, view: &mut impl ViewMut, id: DbId) -> Result<(), DbError> {
        if self.node(view, id)?.is_none() {
            return Ok(());
        }

        let slot = id.0 as u64;

        for direction in 0..2 {
            loop {
                let edge = self.nodes.get(view, slot, NODE_FIRST + direction * 8)?;

                if edge == 0 {
                    break;
                }

                self.remove_edge_slot(view, edge)?;
            }
        }

        self.nodes.release(view, slot)
    }

    /// Removes the edge `id`, if it exists.
    pub(crate) fn remove_edge(&mut self, view: &mut impl ViewMut, id: DbId) -> Result<(), DbError> {
        if self.edge(view, id)?.is_none() {
            return Ok(());
        }

        self.remove_edge_slot(view, id.0.unsigned_abs())
    }

    fn remove_edge_slot(&mut self, view: &mut impl ViewMut, slot: u64) -> Result<(), DbError> {
        let width = self.edges.width;
        let bytes = self.edges.read(view, slot)?;
        let field = |field: u64| uint(&bytes, field * width, width);
        let links = [0, 1].map(|direction| {
            (
                field(direction),
                field(EDGE_NEXT + direction),
                field(EDGE_PREVIOUS + direction),
            )
        });

        for (direction, (node, next, previous)) in links.into_iter().enumerate() {
            self.unlink(view, node, direction as u64, next, previous)?;
        }

        self.edges.release(view, slot)
    }

    /// Removes an edge from the list of the `node` in the `direction` (0
    /// from, 1 to) given its `next` and `previous` edges in that list.
    fn unlink(
        &self,
        view: &mut impl ViewMut,
        node: u64,
        direction: u64,
        next: u64,
        previous: u64,
    ) -> Result<(), DbError> {
        let width = self.edges.width;
        let count = self.nodes.get_u32(view, node, NODE_COUNT + direction * 4)? - 1;

        if previous == 0 {
            self.nodes
                .set(view, node, NODE_FIRST + direction * 8, next)?;
        } else {
            self.edges
                .set(view, previous, (EDGE_NEXT + direction) * width, next)?;
        }

        self.nodes
            .set_u32(view, node, NODE_COUNT + direction * 4, count)?;

        if next != 0 {
            self.edges
                .set(view, next, (EDGE_PREVIOUS + direction) * width, previous)?;
        }

        Ok(())
    }
}

impl Records {
    fn new(page_shift: u32, record_shift: u32, width: u64, vacant: u64) -> Self {
        Self {
            array: SegmentedArray::new(page_shift, record_shift),
            width,
            vacant,
            len: 1,
            free: 0,
            frees: 0,
        }
    }

    fn decode(&mut self, bytes: &[u8]) {
        self.array.decode(&bytes[..ARRAY_SIZE]);
        let words = &bytes[ARRAY_SIZE..];
        self.len = u64_at(words, 0);
        self.free = u64_at(words, 1);
        self.frees = u64_at(words, 2);
    }

    fn encode(&self, bytes: &mut [u8]) {
        bytes[..ARRAY_SIZE].copy_from_slice(&self.array.encode());
        let words = [self.len, self.free, self.frees];

        for (chunk, word) in bytes[ARRAY_SIZE..]
            .as_chunks_mut::<8>()
            .0
            .iter_mut()
            .zip(words)
        {
            *chunk = word.to_le_bytes();
        }
    }

    fn count(&self) -> u64 {
        self.len - 1 - self.frees
    }

    /// The bytes of the record `slot`, `None` for the slot 0, free and
    /// out of bounds slots.
    fn record<'a>(
        &self,
        view: &'a impl View,
        slot: u64,
    ) -> Result<Option<StorageSlice<'a>>, DbError> {
        if slot == 0 || self.len <= slot {
            return Ok(None);
        }

        let bytes = self.read(view, slot)?;
        Ok((uint(&bytes, 0, self.width) != self.vacant).then_some(bytes))
    }

    fn read<'a>(&self, view: &'a impl View, slot: u64) -> Result<StorageSlice<'a>, DbError> {
        view.read(self.array.pos(slot), self.array.record_size())
    }

    fn write(
        &self,
        view: &mut impl ViewMut,
        slot: u64,
        at: u64,
        bytes: &[u8],
    ) -> Result<(), DbError> {
        view.write(self.array.pos(slot) + at, bytes)
    }

    /// The id field at `at` of the record `slot`.
    fn get(&self, view: &impl View, slot: u64, at: u64) -> Result<u64, DbError> {
        Ok(uint(
            &view.read(self.array.pos(slot) + at, self.width)?,
            0,
            self.width,
        ))
    }

    fn set(&self, view: &mut impl ViewMut, slot: u64, at: u64, id: u64) -> Result<(), DbError> {
        self.write(view, slot, at, &id.to_le_bytes()[..self.width as usize])
    }

    fn get_u32(&self, view: &impl View, slot: u64, at: u64) -> Result<u64, DbError> {
        Ok(uint(&view.read(self.array.pos(slot) + at, 4)?, 0, 4))
    }

    fn set_u32(
        &self,
        view: &mut impl ViewMut,
        slot: u64,
        at: u64,
        value: u64,
    ) -> Result<(), DbError> {
        self.write(view, slot, at, &value.to_le_bytes()[..4])
    }

    /// Takes the slot of the free list head or a new one at the end, at
    /// most `limit`.
    fn take(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        limit: u64,
        kind: &str,
    ) -> Result<u64, DbError> {
        if self.free != 0 {
            let slot = self.free;
            self.free = self.get(view, slot, self.width)?;
            self.frees -= 1;
            return Ok(slot);
        }

        if limit < self.len {
            return Err(DbError::graph(
                DbErrorType::OutOfBounds,
                format!("Graph full ({limit} {kind}s), rebuild with 64-bit ids"),
            ));
        }

        if self.array.capacity() <= self.len {
            self.array.grow(view, alloc)?;
        }

        self.len += 1;
        Ok(self.len - 1)
    }

    fn release(&mut self, view: &mut impl ViewMut, slot: u64) -> Result<(), DbError> {
        let width = self.width as usize;
        let mut bytes = [0_u8; 16];
        bytes[..width].copy_from_slice(&self.vacant.to_le_bytes()[..width]);
        bytes[width..2 * width].copy_from_slice(&self.free.to_le_bytes()[..width]);
        self.write(view, slot, 0, &bytes[..2 * width])?;
        self.free = slot;
        self.frees += 1;
        Ok(())
    }
}

impl<V: View> Iterator for Elements<'_, V> {
    type Item = Result<DbId, DbError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let records = if self.edges {
                &self.graph.edges
            } else {
                &self.graph.nodes
            };

            if records.len <= self.slot {
                if self.edges {
                    return None;
                }

                self.edges = true;
                self.slot = 1;
                continue;
            }

            let slot = self.slot;
            self.slot += 1;

            match records.get(self.view, slot, 0) {
                Ok(first) if first == records.vacant => {}
                Ok(_) if self.edges => return Some(Ok(edge_id(slot))),
                Ok(_) => return Some(Ok(node_id(slot))),
                Err(error) => {
                    self.edges = true;
                    self.slot = u64::MAX;
                    return Some(Err(error));
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::collections::array::tests::allocations as array_allocations;
    use crate::collections::array::tests::sparse;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::with_view;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::DEFAULT_PAGE_SHIFT;
    use crate::storage::pages::tests::XorShift;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    /// The segments of both arrays.
    pub(crate) fn allocations(graph: &GraphHeader) -> Vec<(u64, u64)> {
        let mut live = array_allocations(&graph.nodes.array);
        live.extend(array_allocations(&graph.edges.array));
        live
    }

    fn layouts() -> [Layout; 2] {
        [IdWidth::U32, IdWidth::U64].map(|ids| Layout {
            page_shift: DEFAULT_PAGE_SHIFT,
            ids,
        })
    }

    /// The nodes and the edges (from, to) by id.
    #[derive(Default)]
    struct Model {
        nodes: BTreeSet<i64>,
        edges: BTreeMap<i64, (i64, i64)>,
    }

    fn list(graph: &GraphHeader, view: &TestView, first: DbId, from: bool) -> Vec<i64> {
        let mut ids = Vec::new();
        let mut id = first;

        while id.0 != 0 {
            ids.push(id.0);
            let edge = graph.edge(view, id).unwrap().unwrap();
            id = if from { edge.next_from } else { edge.next_to };
        }

        ids
    }

    fn free_slots(records: &Records, view: &TestView) -> BTreeSet<u64> {
        let mut free = BTreeSet::new();
        let mut slot = records.free;

        while slot != 0 {
            assert!(free.insert(slot));
            slot = records.get(view, slot, records.width).unwrap();
        }

        assert_eq!(free.len() as u64, records.frees);
        free
    }

    /// Asserts that the graph holds the elements of the `model` with
    /// their edge lists, that the previous links mirror the next ones and
    /// that every other slot is free.
    fn assert_graph(graph: &GraphHeader, view: &TestView, model: &Model) {
        let ids = graph
            .elements(view)
            .map(|id| id.unwrap().0)
            .collect::<Vec<_>>();
        let nodes = model.nodes.iter().copied();
        let edges = model.edges.keys().rev().copied();
        assert_eq!(ids, nodes.chain(edges).collect::<Vec<_>>());
        assert_eq!(graph.node_count(), model.nodes.len() as u64);
        assert_eq!(graph.edge_count(), model.edges.len() as u64);

        for node in &model.nodes {
            let record = graph.node(view, DbId(*node)).unwrap().unwrap();
            let from = list(graph, view, record.first_from, true);
            let to = list(graph, view, record.first_to, false);
            assert_eq!(record.from_count, from.len() as u64);
            assert_eq!(record.to_count, to.len() as u64);

            for (from, ids) in [(true, &from), (false, &to)] {
                let mut previous = DbId(0);

                for id in ids {
                    let ends = model.edges[id];
                    assert_eq!(if from { ends.0 } else { ends.1 }, *node);
                    let edge = graph.edge(view, DbId(*id)).unwrap().unwrap();
                    let link = if from {
                        edge.previous_from
                    } else {
                        edge.previous_to
                    };
                    assert_eq!(link, previous);
                    previous = DbId(*id);
                }
            }
        }

        for (id, (from, to)) in &model.edges {
            let edge = graph.edge(view, DbId(*id)).unwrap().unwrap();
            assert_eq!((edge.from.0, edge.to.0), (*from, *to));
        }

        let free = free_slots(&graph.nodes, view);
        assert_eq!(1 + (model.nodes.len() + free.len()) as u64, graph.nodes.len);
        let free = free_slots(&graph.edges, view);
        assert_eq!(1 + (model.edges.len() + free.len()) as u64, graph.edges.len);
    }

    fn random_op(
        graph: &mut GraphHeader,
        view: &mut TestView,
        alloc: &mut Allocator,
        model: &mut Model,
        random: &mut XorShift,
    ) {
        let nodes = model.nodes.iter().copied().collect::<Vec<_>>();
        let pick = |random: &mut XorShift| nodes[random.next(nodes.len() as u64) as usize];

        match random.next(10) {
            _ if nodes.is_empty() => {}
            0..=3 => {
                let edges = model.edges.keys().copied().collect::<Vec<_>>();

                if !edges.is_empty() {
                    let id = edges[random.next(edges.len() as u64) as usize];
                    graph.remove_edge(view, DbId(id)).unwrap();
                    model.edges.remove(&id);
                }

                return;
            }
            4..=7 => {
                let (from, to) = (pick(random), pick(random));
                let id = graph
                    .insert_edge(view, alloc, DbId(from), DbId(to))
                    .unwrap();
                assert!(model.edges.insert(id.0, (from, to)).is_none());
                return;
            }
            8 => {
                let node = pick(random);
                graph.remove_node(view, DbId(node)).unwrap();
                model.nodes.remove(&node);
                model
                    .edges
                    .retain(|_, edge| edge.0 != node && edge.1 != node);
                return;
            }
            _ => {}
        }

        let id = graph.insert_node(view, alloc).unwrap();
        assert!(model.nodes.insert(id.0));
    }

    mod graph_header {
        use super::*;

        #[test]
        fn graph_header_should_hold_elements_over_random_operations() {
            for layout in layouts() {
                for seed in 1..=5 {
                    with_view(|view, alloc| {
                        let mut random = XorShift(seed);
                        let mut graph = GraphHeader::new(layout);
                        let mut model = Model::default();

                        for _ in 0..600 {
                            random_op(&mut graph, view, alloc, &mut model, &mut random);
                            assert_graph(&graph, view, &model);
                        }

                        verify(alloc, view, &allocations(&graph));
                    });
                }
            }
        }

        #[test]
        fn graph_header_failure_should_fail() {
            for layout in layouts() {
                assert_failures(
                    |view, alloc| {
                        // WHY: the first allocation of the graph then reads the header of the free run
                        let pos = alloc.alloc(view, 3 * 1024).unwrap();
                        alloc.alloc(view, 1024).unwrap();
                        alloc.free(view, pos, 3 * 1024).unwrap();
                        GraphHeader::new(layout)
                    },
                    |view, alloc, mut graph| {
                        for _ in 0..17 {
                            graph.insert_node(view, alloc)?;
                        }

                        let edges = [(1, 2), (1, 3), (2, 1), (1, 1), (3, 1), (2, 3)];

                        for (from, to) in edges {
                            graph.insert_edge(view, alloc, DbId(from), DbId(to))?;
                        }

                        graph.insert_node(view, alloc)?;
                        graph.remove_edge(view, DbId(-2))?;
                        graph.remove_node(view, DbId(1))?;
                        graph.insert_node(view, alloc)?;
                        graph.insert_edge(view, alloc, DbId(2), DbId(2))?;
                        graph.set_data(view, DbId(2), 2)?;
                        graph.set_data(view, DbId(-3), 7)?;
                        graph.node(view, DbId(2))?;
                        graph.edge(view, DbId(-3))?;
                        graph.elements(view).collect::<Result<Vec<_>, _>>()?;
                        Ok(())
                    },
                );
            }
        }

        mod decode {
            use super::*;

            #[test]
            fn decode_should_read_encoded_header() {
                for layout in layouts() {
                    with_view(|view, alloc| {
                        let mut graph = GraphHeader::new(layout);

                        for _ in 0..20 {
                            graph.insert_node(view, alloc).unwrap();
                        }

                        graph.insert_edge(view, alloc, DbId(1), DbId(2)).unwrap();
                        graph.remove_node(view, DbId(3)).unwrap();
                        assert_eq!(GraphHeader::decode(&graph.encode(), layout), graph);
                    });
                }
            }
        }

        mod elements {
            use super::*;

            #[test]
            fn elements_should_list_nodes_then_edges() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    let first = graph.insert_node(view, alloc).unwrap();
                    let second = graph.insert_node(view, alloc).unwrap();
                    graph.insert_edge(view, alloc, first, second).unwrap();
                    graph.insert_edge(view, alloc, second, first).unwrap();
                    graph.remove_edge(view, DbId(-1)).unwrap();
                    let ids = graph.elements(view).collect::<Result<Vec<_>, _>>().unwrap();
                    assert_eq!(ids, [DbId(1), DbId(2), DbId(-2)]);
                });
            }
        }

        mod set_data {
            use super::*;

            #[test]
            fn set_data_should_be_kept_by_element() {
                for layout in layouts() {
                    with_view(|view, alloc| {
                        let mut graph = GraphHeader::new(layout);
                        let node = graph.insert_node(view, alloc).unwrap();
                        let edge = graph.insert_edge(view, alloc, node, node).unwrap();
                        graph.set_data(view, node, u64::MAX - 1).unwrap();
                        graph.set_data(view, edge, u64::MAX).unwrap();
                        let node = graph.node(view, node).unwrap().unwrap();
                        assert_eq!((node.data, node.from_count), (u64::MAX - 1, 1));
                        let edge = graph.edge(view, edge).unwrap().unwrap();
                        assert_eq!((edge.data, edge.from), (u64::MAX, DbId(1)));
                    });
                }
            }

            #[test]
            fn set_data_of_missing_element_should_fail() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    graph.insert_node(view, alloc).unwrap();

                    for id in [-1, 0, 2] {
                        let error = graph.set_data(view, DbId(id), 1).unwrap_err();
                        assert_eq!(error.description, format!("'{id}' is invalid index"));
                    }
                });
            }
        }

        mod insert_node {
            use super::*;

            #[test]
            fn insert_node_at_last_u32_slot_should_take_it_with_any_id_width() {
                for layout in layouts() {
                    let mut alloc = Allocator::new(DEFAULT_PAGE_SHIFT);
                    let mut view = sparse(&alloc);
                    let mut graph = GraphHeader::new(layout);
                    let node = graph.insert_node(&mut view, &mut alloc).unwrap();

                    while graph.nodes.array.capacity() <= u64::from(u32::MAX) {
                        graph.nodes.array.grow(&mut view, &mut alloc).unwrap();
                    }

                    graph.nodes.len = u64::from(u32::MAX);
                    let last = graph.insert_node(&mut view, &mut alloc).unwrap();
                    assert_eq!(last, DbId(u32::MAX.into()));
                    let edge = graph
                        .insert_edge(&mut view, &mut alloc, last, node)
                        .unwrap();
                    let edge = graph.edge(&view, edge).unwrap().unwrap();
                    assert_eq!((edge.from, edge.to), (last, node));

                    if layout.ids == IdWidth::U64 {
                        let next = graph.insert_node(&mut view, &mut alloc).unwrap();
                        assert_eq!(next, DbId(1 << 32));
                    }
                }
            }

            #[test]
            fn insert_node_past_u32_slots_with_u32_ids_should_fail() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    graph.nodes.len = 1 << 32;
                    let error = graph.insert_node(view, alloc).unwrap_err();
                    assert_eq!(
                        error.description,
                        "Graph full (4294967295 nodes), rebuild with 64-bit ids"
                    );
                    assert_eq!(graph.nodes.array.capacity(), 0);
                });
            }
        }

        mod insert_edge {
            use super::*;

            #[test]
            fn insert_edge_from_missing_node_should_fail() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    let node = graph.insert_node(view, alloc).unwrap();
                    let error = graph.insert_edge(view, alloc, DbId(2), node).unwrap_err();
                    assert_eq!(error.description, "'2' is invalid index");
                    let error = graph.insert_edge(view, alloc, node, DbId(0)).unwrap_err();
                    assert_eq!(error.description, "'0' is invalid index");
                    let error = graph.insert_edge(view, alloc, node, DbId(-1)).unwrap_err();
                    assert_eq!(error.description, "'-1' is invalid index");
                    assert_eq!(graph.edge_count(), 0);
                });
            }

            #[test]
            fn insert_edge_past_edge_count_of_node_should_fail() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    let node = graph.insert_node(view, alloc).unwrap();

                    for count in [NODE_COUNT, NODE_COUNT + 4] {
                        graph
                            .nodes
                            .set_u32(view, 1, count, u32::MAX.into())
                            .unwrap();
                        let error = graph.insert_edge(view, alloc, node, node).unwrap_err();
                        assert_eq!(
                            error.description,
                            "Node '1' has too many edges (4294967295)"
                        );
                        graph.nodes.set_u32(view, 1, count, 0).unwrap();
                    }

                    assert_eq!(graph.edge_count(), 0);
                });
            }

            #[test]
            fn insert_edge_at_last_u32_slot_should_take_it_with_any_id_width() {
                for layout in layouts() {
                    let mut alloc = Allocator::new(DEFAULT_PAGE_SHIFT);
                    let mut view = sparse(&alloc);
                    let mut graph = GraphHeader::new(layout);
                    let node = graph.insert_node(&mut view, &mut alloc).unwrap();

                    while graph.edges.array.capacity() <= u64::from(u32::MAX) {
                        graph.edges.array.grow(&mut view, &mut alloc).unwrap();
                    }

                    graph.edges.len = u64::from(u32::MAX);
                    let last = graph
                        .insert_edge(&mut view, &mut alloc, node, node)
                        .unwrap();
                    assert_eq!(last, DbId(-i64::from(u32::MAX)));
                    let record = graph.node(&view, node).unwrap().unwrap();
                    assert_eq!((record.first_from, record.first_to), (last, last));

                    if layout.ids == IdWidth::U64 {
                        let next = graph
                            .insert_edge(&mut view, &mut alloc, node, node)
                            .unwrap();
                        assert_eq!(next, DbId(-(1 << 32)));
                        let edge = graph.edge(&view, next).unwrap().unwrap();
                        assert_eq!((edge.next_from, edge.next_to), (last, last));
                    }
                }
            }

            #[test]
            fn insert_edge_past_u32_slots_with_u32_ids_should_fail() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    let node = graph.insert_node(view, alloc).unwrap();
                    graph.edges.len = 1 << 32;
                    let error = graph.insert_edge(view, alloc, node, node).unwrap_err();
                    assert_eq!(
                        error.description,
                        "Graph full (4294967295 edges), rebuild with 64-bit ids"
                    );
                    let node = graph.node(view, node).unwrap().unwrap();
                    assert_eq!((node.from_count, node.first_from), (0, DbId(0)));
                });
            }
        }

        mod remove_node {
            use super::*;

            #[test]
            fn remove_node_of_edge_should_do_nothing() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    let node = graph.insert_node(view, alloc).unwrap();
                    let edge = graph.insert_edge(view, alloc, node, node).unwrap();
                    graph.remove_node(view, edge).unwrap();
                    assert_eq!((graph.node_count(), graph.edge_count()), (1, 1));
                });
            }
        }

        mod remove_edge {
            use super::*;

            #[test]
            fn remove_edge_of_node_or_missing_element_should_do_nothing() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new(Layout::default());
                    let node = graph.insert_node(view, alloc).unwrap();
                    graph.insert_edge(view, alloc, node, node).unwrap();
                    graph.remove_edge(view, node).unwrap();
                    graph.remove_edge(view, DbId(-3)).unwrap();
                    assert_eq!((graph.node_count(), graph.edge_count()), (1, 1));
                });
            }
        }
    }
}
