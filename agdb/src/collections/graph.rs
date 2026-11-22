use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::storage::alloc::Allocator;
use crate::storage::pages::DEFAULT_PAGE_SHIFT;
use crate::storage::pages::View;
use crate::storage::pages::ViewMut;
use crate::utilities::serialize::u64_at;

/// The bytes of an encoded [`GraphHeader`].
pub(crate) const GRAPH_SIZE: usize = 48;
const FIELDS: usize = 7;
const RECORD_SIZE: u64 = FIELDS as u64 * 8;
/// The records of a page, which a record never crosses.
/// Until the graph takes the page size of the storage (plan 4c b).
const PAGE_SIZE: u64 = 1 << DEFAULT_PAGE_SHIFT;
const RECORDS: u64 = PAGE_SIZE / RECORD_SIZE;

// Node: first edge from, first edge to, edge count from, edge count to,
// data (3 fields).
// Edge: -from, -to, next edge from, next edge to, previous edge from,
// previous edge to, data (1 field). The links are the positive slots.
// Free: 0, 0, -1 - the next free slot.
const FROM: usize = 0;
const TO: usize = 1;
const FROM_META: usize = 2;
const TO_META: usize = 3;
const PREVIOUS_FROM: usize = 4;
const PREVIOUS_TO: usize = 5;
const NODE_DATA: usize = 4;
const EDGE_DATA: usize = 6;

type Record = [i64; FIELDS];

/// The graph: one record of 7 fields per element in a run of pages. The
/// id of an element is the slot of its record, negative for edges. The
/// slot 0 is never used. Removed elements form a list of free records
/// reused last removed first.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphHeader {
    pos: u64,
    cap: u64,
    len: u64,
    free: u64,
    nodes: u64,
    frees: u64,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Node {
    pub(crate) first_from: DbId,
    pub(crate) first_to: DbId,
    pub(crate) from_count: u64,
    pub(crate) to_count: u64,
    /// Kept for the users of the graph, 0 in a new node.
    pub(crate) data: [u64; 3],
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

/// The ids of the elements in slot order. A failed read is the last
/// item.
pub(crate) struct Elements<'a, V> {
    graph: &'a GraphHeader,
    view: &'a V,
    slot: u64,
}

fn invalid(id: DbId) -> DbError {
    DbError::graph(
        DbErrorType::InvalidIndex,
        format!("'{}' is invalid index", id.0),
    )
}

fn edge_id(slot: i64) -> DbId {
    DbId(-slot)
}

impl GraphHeader {
    pub(crate) fn new() -> Self {
        Self {
            pos: 0,
            cap: 0,
            len: 1,
            free: 0,
            nodes: 0,
            frees: 0,
        }
    }

    pub(crate) fn decode(bytes: &[u8]) -> Self {
        Self {
            pos: u64_at(bytes, 0),
            cap: u64_at(bytes, 1),
            len: u64_at(bytes, 2),
            free: u64_at(bytes, 3),
            nodes: u64_at(bytes, 4),
            frees: u64_at(bytes, 5),
        }
    }

    pub(crate) fn encode(&self) -> [u8; GRAPH_SIZE] {
        let mut bytes = [0_u8; GRAPH_SIZE];
        let words = [
            self.pos, self.cap, self.len, self.free, self.nodes, self.frees,
        ];

        for (chunk, word) in bytes.as_chunks_mut::<8>().0.iter_mut().zip(words) {
            *chunk = word.to_le_bytes();
        }

        bytes
    }

    pub(crate) fn node_count(&self) -> u64 {
        self.nodes
    }

    pub(crate) fn edge_count(&self) -> u64 {
        self.len - 1 - self.nodes - self.frees
    }

    pub(crate) fn elements<'a, V: View>(&'a self, view: &'a V) -> Elements<'a, V> {
        Elements {
            graph: self,
            view,
            slot: 1,
        }
    }

    pub(crate) fn node(&self, view: &impl View, id: DbId) -> Result<Option<Node>, DbError> {
        Ok(self.node_record(view, id)?.map(|record| Node {
            first_from: edge_id(record[FROM]),
            first_to: edge_id(record[TO]),
            from_count: record[FROM_META] as u64,
            to_count: record[TO_META] as u64,
            data: std::array::from_fn(|i| record[NODE_DATA + i] as u64),
        }))
    }

    pub(crate) fn edge(&self, view: &impl View, id: DbId) -> Result<Option<Edge>, DbError> {
        Ok(self.edge_record(view, id)?.map(|record| Edge {
            from: DbId(-record[FROM]),
            to: DbId(-record[TO]),
            next_from: edge_id(record[FROM_META]),
            next_to: edge_id(record[TO_META]),
            previous_from: edge_id(record[PREVIOUS_FROM]),
            previous_to: edge_id(record[PREVIOUS_TO]),
            data: record[EDGE_DATA] as u64,
        }))
    }

    /// Writes the `data` of the element `id` from its data field `index`.
    pub(crate) fn set_data(
        &self,
        view: &mut impl ViewMut,
        id: DbId,
        index: usize,
        data: &[u64],
    ) -> Result<(), DbError> {
        self.element(view, id)?.ok_or_else(|| invalid(id))?;
        let field = index + if 0 < id.0 { NODE_DATA } else { EDGE_DATA };

        if FIELDS < field + data.len() {
            return Err(invalid(id));
        }

        let mut fields = [0; FIELDS];

        for (field, data) in fields.iter_mut().zip(data) {
            *field = *data as i64;
        }

        self.write(view, id.0.abs(), field, &fields[..data.len()])
    }

    pub(crate) fn insert_node(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
    ) -> Result<DbId, DbError> {
        let slot = self.allocate(view, alloc)?;
        self.write(view, slot, FROM, &[0; FIELDS])?;
        self.nodes += 1;
        Ok(DbId(slot))
    }

    pub(crate) fn insert_edge(
        &mut self,
        view: &mut impl ViewMut,
        alloc: &mut Allocator,
        from: DbId,
        to: DbId,
    ) -> Result<DbId, DbError> {
        let origin = self.node_record(view, from)?.ok_or_else(|| invalid(from))?;
        let destination = self.node_record(view, to)?.ok_or_else(|| invalid(to))?;
        let slot = self.allocate(view, alloc)?;
        let edge = [-from.0, -to.0, origin[FROM], destination[TO], 0, 0, 0];
        self.write(view, slot, FROM, &edge)?;

        if origin[FROM] != 0 {
            self.write(view, origin[FROM], PREVIOUS_FROM, &[slot])?;
        }

        if destination[TO] != 0 {
            self.write(view, destination[TO], PREVIOUS_TO, &[slot])?;
        }

        self.write(view, from.0, FROM, &[slot])?;
        self.write(view, from.0, FROM_META, &[origin[FROM_META] + 1])?;
        self.write(view, to.0, TO, &[slot])?;
        self.write(view, to.0, TO_META, &[destination[TO_META] + 1])?;
        Ok(edge_id(slot))
    }

    /// Removes the node `id` with its edges, if it exists.
    pub(crate) fn remove_node(&mut self, view: &mut impl ViewMut, id: DbId) -> Result<(), DbError> {
        if self.node_record(view, id)?.is_none() {
            return Ok(());
        }

        for head in [FROM, TO] {
            loop {
                let edge = self.record(view, id.0)?[head];

                if edge == 0 {
                    break;
                }

                self.remove_edge_slot(view, edge)?;
            }
        }

        self.nodes -= 1;
        self.free(view, id.0)
    }

    /// Removes the edge `id`, if it exists.
    pub(crate) fn remove_edge(&mut self, view: &mut impl ViewMut, id: DbId) -> Result<(), DbError> {
        if self.edge_record(view, id)?.is_none() {
            return Ok(());
        }

        self.remove_edge_slot(view, -id.0)
    }

    /// Takes the slot of the free list head or a new one at the end.
    fn allocate(&mut self, view: &mut impl ViewMut, alloc: &mut Allocator) -> Result<i64, DbError> {
        if self.free != 0 {
            let slot = self.free;
            self.free = (-1 - self.record(view, slot as i64)?[FROM_META]) as u64;
            self.frees -= 1;
            return Ok(slot as i64);
        }

        if self.cap <= self.len {
            let pages = self.cap / RECORDS;
            let size = (pages * 2).max(1) * PAGE_SIZE;
            self.pos = if pages == 0 {
                alloc.alloc(view, size)?
            } else {
                alloc.realloc(view, self.pos, pages * PAGE_SIZE, size)?
            };
            self.cap = size / PAGE_SIZE * RECORDS;
        }

        self.len += 1;
        Ok(self.len as i64 - 1)
    }

    fn free(&mut self, view: &mut impl ViewMut, slot: i64) -> Result<(), DbError> {
        let record = [0, 0, -1 - self.free as i64, 0, 0, 0, 0];
        self.write(view, slot, FROM, &record)?;
        self.free = slot as u64;
        self.frees += 1;
        Ok(())
    }

    fn remove_edge_slot(&mut self, view: &mut impl ViewMut, slot: i64) -> Result<(), DbError> {
        let edge = self.record(view, slot)?;
        self.unlink(
            view,
            -edge[FROM],
            FROM,
            edge[FROM_META],
            edge[PREVIOUS_FROM],
        )?;
        self.unlink(view, -edge[TO], TO, edge[TO_META], edge[PREVIOUS_TO])?;
        self.free(view, slot)
    }

    /// Removes an edge from the `head` (FROM or TO) list of the `node`
    /// given its `next` and `previous` edges in that list. The list links
    /// of an edge and the count of a node are 2 fields after the head and
    /// the previous links 4 fields after it.
    fn unlink(
        &self,
        view: &mut impl ViewMut,
        node: i64,
        head: usize,
        next: i64,
        previous: i64,
    ) -> Result<(), DbError> {
        let count = self.record(view, node)?[head + 2] - 1;

        if previous == 0 {
            self.write(view, node, head, &[next])?;
        } else {
            self.write(view, previous, head + 2, &[next])?;
        }

        self.write(view, node, head + 2, &[count])?;

        if next != 0 {
            self.write(view, next, head + 4, &[previous])?;
        }

        Ok(())
    }

    /// The record of an existing element, `None` for the slot 0, free and
    /// out of bounds slots.
    fn element(&self, view: &impl View, id: DbId) -> Result<Option<Record>, DbError> {
        if id.0 == 0 || self.len <= id.0.unsigned_abs() {
            return Ok(None);
        }

        let record = self.record(view, id.0.abs())?;
        let valid = 0 <= record[FROM_META] && (id.0 < 0) == (record[FROM] < 0);
        Ok(valid.then_some(record))
    }

    fn node_record(&self, view: &impl View, id: DbId) -> Result<Option<Record>, DbError> {
        if id.0 < 0 {
            return Ok(None);
        }

        self.element(view, id)
    }

    fn edge_record(&self, view: &impl View, id: DbId) -> Result<Option<Record>, DbError> {
        if 0 < id.0 {
            return Ok(None);
        }

        self.element(view, id)
    }

    fn pos(&self, slot: i64, field: usize) -> u64 {
        let slot = slot as u64;
        self.pos + slot / RECORDS * PAGE_SIZE + slot % RECORDS * RECORD_SIZE + field as u64 * 8
    }

    fn record(&self, view: &impl View, slot: i64) -> Result<Record, DbError> {
        let bytes = view.read(self.pos(slot, 0), RECORD_SIZE)?;
        Ok(std::array::from_fn(|field| u64_at(&bytes, field) as i64))
    }

    fn write(
        &self,
        view: &mut impl ViewMut,
        slot: i64,
        field: usize,
        values: &[i64],
    ) -> Result<(), DbError> {
        let mut bytes = [0_u8; RECORD_SIZE as usize];

        for (chunk, value) in bytes.as_chunks_mut::<8>().0.iter_mut().zip(values) {
            *chunk = value.to_le_bytes();
        }

        view.write(self.pos(slot, field), &bytes[..values.len() * 8])
    }
}

impl<V: View> Iterator for Elements<'_, V> {
    type Item = Result<DbId, DbError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.slot < self.graph.len {
            let slot = self.slot as i64;
            self.slot += 1;

            match self.graph.record(self.view, slot) {
                Ok(record) if 0 <= record[FROM_META] => {
                    return Some(Ok(DbId(if record[FROM] < 0 { -slot } else { slot })));
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::collections::value::tests::TestView;
    use crate::collections::value::tests::assert_failures;
    use crate::collections::value::tests::with_view;
    use crate::storage::alloc::tests::verify;
    use crate::storage::pages::tests::XorShift;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    /// The run of pages of the graph.
    pub(crate) fn allocation(graph: &GraphHeader) -> (u64, u64) {
        (graph.pos, graph.cap / RECORDS * PAGE_SIZE)
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

    /// Asserts that the graph holds the elements of the `model` with
    /// their edge lists, that the previous links mirror the next ones and
    /// that every other slot is free.
    fn assert_graph(graph: &GraphHeader, view: &TestView, model: &Model) {
        let ids = graph.elements(view).collect::<Result<Vec<_>, _>>().unwrap();
        let expected = model
            .nodes
            .iter()
            .copied()
            .chain(model.edges.keys().copied())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.iter().map(|id| id.0).collect::<BTreeSet<_>>(), expected);
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
            assert_eq!(graph.node(view, DbId(*id)).unwrap(), None);
        }

        let mut free = BTreeSet::new();
        let mut slot = graph.free as i64;

        while slot != 0 {
            assert!(free.insert(slot));
            slot = -1 - graph.record(view, slot).unwrap()[FROM_META];
        }

        assert_eq!(free.len() as u64, graph.frees);
        assert_eq!(1 + expected.len() as u64 + graph.frees, graph.len);
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
            for seed in 1..=5 {
                with_view(|view, alloc| {
                    let mut random = XorShift(seed);
                    let mut graph = GraphHeader::new();
                    let mut model = Model::default();

                    for _ in 0..600 {
                        random_op(&mut graph, view, alloc, &mut model, &mut random);
                        assert_graph(&graph, view, &model);
                    }

                    verify(alloc, view, &[allocation(&graph)]);
                });
            }
        }

        #[test]
        fn graph_header_failure_should_fail() {
            assert_failures(
                |view, alloc| {
                    // WHY: the first allocation of the graph then reads the header of the free run
                    let pos = alloc.alloc(view, 3 * PAGE_SIZE).unwrap();
                    alloc.alloc(view, PAGE_SIZE).unwrap();
                    alloc.free(view, pos, 3 * PAGE_SIZE).unwrap();
                    GraphHeader::new()
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
                    graph.remove_edge(view, DbId(-19))?;
                    graph.remove_node(view, DbId(1))?;
                    graph.insert_node(view, alloc)?;
                    graph.set_data(view, DbId(2), 1, &[1, 2])?;
                    graph.set_data(view, DbId(-23), 0, &[7])?;
                    graph.node(view, DbId(2))?;
                    graph.edge(view, DbId(-23))?;
                    graph.elements(view).collect::<Result<Vec<_>, _>>()?;
                    Ok(())
                },
            );
        }

        mod decode {
            use super::*;

            #[test]
            fn decode_should_read_encoded_header() {
                let graph = GraphHeader {
                    pos: 4096,
                    cap: 18,
                    len: 5,
                    free: 3,
                    nodes: 2,
                    frees: 1,
                };
                assert_eq!(GraphHeader::decode(&graph.encode()), graph);
            }
        }

        mod set_data {
            use super::*;

            #[test]
            fn set_data_should_be_kept_by_element() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new();
                    let node = graph.insert_node(view, alloc).unwrap();
                    let edge = graph.insert_edge(view, alloc, node, node).unwrap();
                    graph.set_data(view, node, 1, &[5, 6]).unwrap();
                    graph.set_data(view, edge, 0, &[7]).unwrap();
                    let node = graph.node(view, node).unwrap().unwrap();
                    assert_eq!((node.data, node.from_count), ([0, 5, 6], 1));
                    let edge = graph.edge(view, edge).unwrap().unwrap();
                    assert_eq!((edge.data, edge.from), (7, DbId(1)));
                });
            }

            #[test]
            fn set_data_past_data_of_element_should_fail() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new();
                    let node = graph.insert_node(view, alloc).unwrap();
                    let edge = graph.insert_edge(view, alloc, node, node).unwrap();
                    let error = graph.set_data(view, edge, 0, &[1, 2]).unwrap_err();
                    assert_eq!(error.description, "'-2' is invalid index");
                    let error = graph.set_data(view, node, 3, &[1]).unwrap_err();
                    assert_eq!(error.description, "'1' is invalid index");
                    assert_eq!(graph.node(view, node).unwrap().unwrap().data, [0; 3]);
                });
            }

            #[test]
            fn set_data_of_missing_element_should_fail() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new();
                    graph.insert_node(view, alloc).unwrap();
                    let error = graph.set_data(view, DbId(-1), 0, &[1]).unwrap_err();
                    assert_eq!(error.description, "'-1' is invalid index");
                });
            }
        }

        mod insert_edge {
            use super::*;

            #[test]
            fn insert_edge_from_missing_node_should_fail() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new();
                    let node = graph.insert_node(view, alloc).unwrap();
                    let error = graph.insert_edge(view, alloc, DbId(2), node).unwrap_err();
                    assert_eq!(error.description, "'2' is invalid index");
                    let error = graph.insert_edge(view, alloc, node, DbId(0)).unwrap_err();
                    assert_eq!(error.description, "'0' is invalid index");
                    assert_eq!(graph.edge_count(), 0);
                });
            }
        }

        mod remove_node {
            use super::*;

            #[test]
            fn remove_node_of_edge_should_do_nothing() {
                with_view(|view, alloc| {
                    let mut graph = GraphHeader::new();
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
                    let mut graph = GraphHeader::new();
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
