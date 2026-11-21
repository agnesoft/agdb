use crate::DbError;
use crate::StorageData;
use crate::collections::bit_set::BitSet;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::storage::Storage;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub trait PathSearchHandler {
    fn process(&self, index: GraphIndex, distance: u64) -> Result<(u64, bool), DbError>;
}

/// An element of a path linked to the previous element of the path.
struct Step {
    index: GraphIndex,
    add: bool,
    previous: Option<usize>,
}

/// The cost, length and last step of a path. The heap pops the cheapest
/// path first, then the shortest and then the one added last.
type Path = (Reverse<u64>, Reverse<u64>, usize);

pub struct PathSearch<'a, D, Handler>
where
    D: StorageData,
    Handler: PathSearchHandler,
{
    destination: GraphIndex,
    graph: &'a DbGraph<D>,
    storage: &'a Storage<D>,
    handler: Handler,
    paths: BinaryHeap<Path>,
    steps: Vec<Step>,
    visited: BitSet,
    reverse: bool,
}

impl<'a, D, Handler> PathSearch<'a, D, Handler>
where
    D: StorageData,
    Handler: PathSearchHandler,
{
    pub fn new(
        graph: &'a DbGraph<D>,
        storage: &'a Storage<D>,
        from: GraphIndex,
        to: GraphIndex,
        handler: Handler,
    ) -> Result<Self, DbError> {
        Self::with_direction(graph, storage, from, to, handler, false)
    }

    pub fn new_reverse(
        graph: &'a DbGraph<D>,
        storage: &'a Storage<D>,
        from: GraphIndex,
        to: GraphIndex,
        handler: Handler,
    ) -> Result<Self, DbError> {
        Self::with_direction(graph, storage, to, from, handler, true)
    }

    pub fn search(&mut self) -> Result<Vec<GraphIndex>, DbError> {
        while let Some((Reverse(cost), Reverse(len), last)) = self.paths.pop() {
            let index = self.steps[last].index;

            if !self.visited.value(index.as_u64()) {
                if index.0 == self.destination.0 {
                    return Ok(self.elements(last));
                }

                self.visited.set(index.as_u64());
                self.expand(cost, len, last, index)?;
            }
        }

        Ok(vec![])
    }

    fn with_direction(
        graph: &'a DbGraph<D>,
        storage: &'a Storage<D>,
        origin: GraphIndex,
        destination: GraphIndex,
        handler: Handler,
        reverse: bool,
    ) -> Result<Self, DbError> {
        let add = handler.process(origin, 0)?;

        Ok(Self {
            destination,
            graph,
            storage,
            handler,
            paths: BinaryHeap::from([(Reverse(0), Reverse(1), 0)]),
            steps: vec![Step {
                index: origin,
                add: add.1,
                previous: None,
            }],
            visited: BitSet::new(),
            reverse,
        })
    }

    fn elements(&self, last: usize) -> Vec<GraphIndex> {
        let mut elements = vec![];
        let mut current = Some(last);

        while let Some(step) = current {
            if self.steps[step].add {
                elements.push(self.steps[step].index);
            }

            current = self.steps[step].previous;
        }

        if !self.reverse {
            elements.reverse();
        }

        elements
    }

    fn expand(
        &mut self,
        cost: u64,
        len: u64,
        last: usize,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        let edges = if self.reverse {
            self.graph.edge_iter_to(self.storage, index)?
        } else {
            self.graph.edge_iter_from(self.storage, index)?
        };

        for edge in edges {
            let edge = edge?;
            let next = if self.reverse {
                edge.index_from()
            } else {
                edge.index_to()
            };
            let edge_cost = self.handler.process(edge.index(), len)?;

            if edge_cost.0 != 0 && !self.visited.value(next.as_u64()) {
                let next_cost = self.handler.process(next, len + 1)?;

                if next_cost.0 != 0 {
                    self.steps.push(Step {
                        index: edge.index(),
                        add: edge_cost.1,
                        previous: Some(last),
                    });
                    self.steps.push(Step {
                        index: next,
                        add: next_cost.1,
                        previous: Some(self.steps.len() - 1),
                    });
                    self.paths.push((
                        Reverse(cost + edge_cost.0 + next_cost.0),
                        Reverse(len + 2),
                        self.steps.len() - 1,
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DbGraph;
    use crate::graph_search::GraphSearch;
    use crate::storage::file_storage::FileStorage;
    use crate::test_utilities::test_file::TestFile;

    struct Handler {
        pub processor: fn(GraphIndex, u64) -> (u64, bool),
    }

    impl Default for Handler {
        fn default() -> Self {
            Self {
                processor: |_index: GraphIndex, _distance: u64| (1_u64, true),
            }
        }
    }

    impl PathSearchHandler for Handler {
        fn process(&self, index: GraphIndex, distance: u64) -> Result<(u64, bool), DbError> {
            Ok((self.processor)(index, distance))
        }
    }

    #[test]
    fn circular_path() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let node = graph.insert_node(&mut storage).unwrap();
        let _edge = graph.insert_edge(&mut storage, node, node).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(node, node, Handler::default());

        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn empty_graph() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let graph = DbGraph::new(&mut storage).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(
            GraphIndex::default(),
            GraphIndex::default(),
            Handler::default(),
        );

        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn multi_edge_path() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let _edge2 = graph.insert_edge(&mut storage, node1, node2).unwrap();

        let edge3 = graph.insert_edge(&mut storage, node2, node3).unwrap();
        let _edge4 = graph.insert_edge(&mut storage, node2, node3).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(node1, node3, Handler::default());

        assert_eq!(result, Ok(vec![node1, edge1, node2, edge3, node3]));
    }

    #[test]
    fn origin_is_destination() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();
        let node = graph.insert_node(&mut storage).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(node, node, Handler::default());

        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn short_circuit_path() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node3).unwrap();
        let _edge2 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let _edge3 = graph.insert_edge(&mut storage, node2, node3).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(node1, node3, Handler::default());

        assert_eq!(result, Ok(vec![node1, edge1, node3]));
    }

    #[test]
    fn single_path() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node2, node3).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(node1, node3, Handler::default());

        assert_eq!(result, Ok(vec![node1, edge1, node2, edge2, node3]));
    }

    #[test]
    fn skip_short_circuit_path() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let _edge1 = graph.insert_edge(&mut storage, node1, node3).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node2, node3).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(
            node1,
            node3,
            Handler {
                processor: |index: GraphIndex, _distance: u64| {
                    if index.0 == -4 {
                        return (0, true);
                    }

                    (1, true)
                },
            },
        );

        assert_eq!(result, Ok(vec![node1, edge2, node2, edge3, node3]));
    }

    #[test]
    fn unconnected() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let _edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(node1, node3, Handler::default());

        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn filtered_nodes() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let _edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let _edge2 = graph.insert_edge(&mut storage, node2, node2).unwrap();
        let _edge3 = graph.insert_edge(&mut storage, node2, node3).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(
            node1,
            node3,
            Handler {
                processor: |index: GraphIndex, _distance: u64| (1, index.is_node()),
            },
        );

        assert_eq!(result, Ok(vec![node1, node2, node3]));
    }

    #[test]
    fn filtered_edges() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let _edge2 = graph.insert_edge(&mut storage, node2, node2).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node2, node3).unwrap();

        let result = GraphSearch::from((&graph, &storage)).path(
            node1,
            node3,
            Handler {
                processor: |index: GraphIndex, _distance: u64| (1, index.is_edge()),
            },
        );

        assert_eq!(result, Ok(vec![edge1, edge3]));
    }
}
