use crate::collections::bit_set::BitSet;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;
use crate::storage::Storage;
use crate::DbError;
use crate::StorageData;
use std::cmp::Ordering;

pub trait PathSearchHandler {
    fn process(&self, index: GraphIndex, distance: u64) -> Result<(u64, bool), DbError>;
}

#[derive(Clone)]
struct Path {
    elements: Vec<(GraphIndex, bool)>,
    cost: u64,
}

pub struct PathSearch<'a, D, Data, Handler>
where
    Data: GraphData<D>,
    D: StorageData,
    Handler: PathSearchHandler,
{
    current_path: Path,
    destination: GraphIndex,
    graph: &'a GraphImpl<D, Data>,
    storage: &'a Storage<D>,
    handler: Handler,
    paths: Vec<Path>,
    result: Vec<(GraphIndex, bool)>,
    visited: BitSet,
}

impl<'a, D, Data, Handler> PathSearch<'a, D, Data, Handler>
where
    Data: GraphData<D>,
    D: StorageData,
    Handler: PathSearchHandler,
{
    pub fn new(
        graph: &'a GraphImpl<D, Data>,
        storage: &'a Storage<D>,
        from: GraphIndex,
        to: GraphIndex,
        handler: Handler,
    ) -> Self {
        let add = handler.process(from, 0).unwrap_or_default();

        Self {
            current_path: Path {
                elements: vec![],
                cost: 0,
            },
            destination: to,
            graph,
            storage,
            handler,
            paths: vec![Path {
                elements: vec![(from, add.1)],
                cost: 0,
            }],
            result: vec![],
            visited: BitSet::new(),
        }
    }

    pub fn search(&mut self) -> Result<Vec<GraphIndex>, DbError> {
        while !self.is_finished() {
            self.sort_paths();
            self.process_last_path()?;
        }

        Ok(self.result.iter().filter(|e| e.1).map(|e| e.0).collect())
    }

    fn expand_edge(
        &mut self,
        mut path: Path,
        index: GraphIndex,
        node_index: GraphIndex,
    ) -> Result<(), DbError> {
        let cost = self
            .handler
            .process(index, self.current_path.elements.len() as u64 + 1)?;

        if cost.0 != 0 && !self.visited.value(node_index.as_u64()) {
            path.elements.push((index, cost.1));
            path.cost += cost.0;
            self.expand_node(path, node_index)?;
        }

        Ok(())
    }

    fn expand_node(&mut self, mut path: Path, index: GraphIndex) -> Result<(), DbError> {
        let cost = self
            .handler
            .process(index, self.current_path.elements.len() as u64 + 1)?;

        if cost.0 != 0 {
            path.elements.push((index, cost.1));
            path.cost += cost.0;
            self.paths.push(path);
        }

        Ok(())
    }

    fn expand(&mut self, index: GraphIndex) -> Result<(), DbError> {
        let node = self
            .graph
            .node(self.storage, index)
            .expect("unexpected invalid node index");
        for edge in node.edge_iter_from() {
            self.expand_edge(self.current_path.clone(), edge.index(), edge.index_to())?;
        }

        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.paths.is_empty() || !self.result.is_empty()
    }

    fn process_path(&mut self) -> Result<(), DbError> {
        let index = self
            .current_path
            .elements
            .last()
            .map_or(GraphIndex::default(), |index| index.0);
        self.process_index(index)
    }

    fn process_index(&mut self, index: GraphIndex) -> Result<(), DbError> {
        if !self.visited.value(index.as_u64()) {
            if index.0 == self.destination.0 {
                std::mem::swap(&mut self.result, &mut self.current_path.elements);
            } else {
                self.visited.set(index.as_u64());
                self.expand(index)?;
            }
        }

        Ok(())
    }

    fn process_last_path(&mut self) -> Result<(), DbError> {
        self.current_path = self.paths.pop().unwrap_or(Path {
            elements: vec![],
            cost: 0,
        });
        self.process_path()
    }

    fn sort_paths(&mut self) {
        self.paths.sort_by(|left, right| {
            let ordering = left.cost.cmp(&right.cost);

            if ordering == Ordering::Equal {
                return left.elements.len().cmp(&right.elements.len()).reverse();
            }

            ordering.reverse()
        });
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
