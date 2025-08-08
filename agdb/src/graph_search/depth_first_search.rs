use super::search_impl::SearchIndex;
use super::search_impl::SearchIterator;
use crate::StorageData;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;
use crate::storage::Storage;

pub struct DepthFirstSearch {
    stack: Vec<SearchIndex>,
}

impl<D> SearchIterator<D> for DepthFirstSearch
where
    D: StorageData,
{
    fn new(index: GraphIndex) -> Self {
        Self {
            stack: vec![SearchIndex { index, distance: 0 }],
        }
    }

    fn expand<Data: GraphData<D>>(
        &mut self,
        current_index: SearchIndex,
        graph: &GraphImpl<D, Data>,
        storage: &Storage<D>,
        follow: bool,
    ) {
        if current_index.index.is_node() {
            if follow
                && let Some(i) = graph
                    .first_edge_from(storage, current_index.index)
                    .ok()
                    .filter(|i| i.is_valid())
                {
                    self.stack.push(SearchIndex {
                        index: i,
                        distance: current_index.distance + 1,
                    });
                }
        } else {
            if let Some(i) = graph
                .next_edge_from(storage, current_index.index)
                .ok()
                .filter(|i| i.is_valid())
            {
                self.stack.push(SearchIndex {
                    index: i,
                    distance: current_index.distance,
                })
            }

            if follow {
                self.stack.push(SearchIndex {
                    index: graph.edge_to(storage, current_index.index),
                    distance: current_index.distance + 1,
                });
            }
        }
    }

    fn next(&mut self) -> Option<SearchIndex> {
        self.stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::super::SearchControl;
    use super::super::SearchHandler;
    use super::*;
    use crate::DbError;
    use crate::graph::DbGraph;
    use crate::graph::GraphIndex;
    use crate::graph_search::GraphSearch;
    use crate::storage::file_storage::FileStorage;
    use crate::test_utilities::test_file::TestFile;

    struct Handler {
        pub processor: fn(GraphIndex, u64) -> SearchControl,
    }

    impl Default for Handler {
        fn default() -> Self {
            Self {
                processor: |_index: GraphIndex, _distance: u64| SearchControl::Continue(true),
            }
        }
    }

    impl SearchHandler for Handler {
        fn process(&mut self, index: GraphIndex, distance: u64) -> Result<SearchControl, DbError> {
            Ok((self.processor)(index, distance))
        }
    }

    #[test]
    fn empty_graph() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let graph = DbGraph::new(&mut storage).unwrap();

        let result = GraphSearch::from((&graph, &storage))
            .depth_first_search(GraphIndex::default(), Handler::default());

        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn cyclic_graph() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node2, node3).unwrap();
        let edge4 = graph.insert_edge(&mut storage, node2, node3).unwrap();
        let edge5 = graph.insert_edge(&mut storage, node3, node1).unwrap();
        let edge6 = graph.insert_edge(&mut storage, node3, node1).unwrap();

        let result =
            GraphSearch::from((&graph, &storage)).depth_first_search(node1, Handler::default());

        assert_eq!(
            result,
            Ok(vec![
                node1, edge2, node2, edge4, node3, edge6, edge5, edge3, edge1
            ])
        );
    }

    #[test]
    fn full_search() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();
        let node4 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node3).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node1, node4).unwrap();

        let result =
            GraphSearch::from((&graph, &storage)).depth_first_search(node1, Handler::default());

        assert_eq!(
            result,
            Ok(vec![node1, edge3, node4, edge2, node3, edge1, node2])
        );
    }

    #[test]
    fn filter_edges() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();
        let node4 = graph.insert_node(&mut storage).unwrap();

        graph.insert_edge(&mut storage, node1, node2).unwrap();
        graph.insert_edge(&mut storage, node1, node3).unwrap();
        graph.insert_edge(&mut storage, node1, node4).unwrap();

        let result = GraphSearch::from((&graph, &storage)).depth_first_search(
            node1,
            Handler {
                processor: |index: GraphIndex, _distance: u64| {
                    SearchControl::Continue(index.is_node())
                },
            },
        );

        assert_eq!(result, Ok(vec![node1, node4, node3, node2]));
    }

    #[test]
    fn finish_search() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        graph.insert_edge(&mut storage, node1, node2).unwrap();
        graph.insert_edge(&mut storage, node1, node2).unwrap();
        graph.insert_edge(&mut storage, node2, node3).unwrap();
        graph.insert_edge(&mut storage, node2, node3).unwrap();
        graph.insert_edge(&mut storage, node3, node1).unwrap();
        graph.insert_edge(&mut storage, node3, node1).unwrap();

        let result = GraphSearch::from((&graph, &storage)).depth_first_search(
            node1,
            Handler {
                processor: |index: GraphIndex, _distance: u64| {
                    if index.0 == 2 {
                        SearchControl::Finish(true)
                    } else {
                        SearchControl::Continue(false)
                    }
                },
            },
        );

        assert_eq!(result, Ok(vec![node2]));
    }

    #[test]
    fn search_twice() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();
        let node4 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node3).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node1, node4).unwrap();

        let mut result =
            GraphSearch::from((&graph, &storage)).depth_first_search(node1, Handler::default());
        let expected = Ok(vec![node1, edge3, node4, edge2, node3, edge1, node2]);

        assert_eq!(result, expected);

        result =
            GraphSearch::from((&graph, &storage)).depth_first_search(node1, Handler::default());

        assert_eq!(result, expected);
    }

    #[test]
    fn stop_at_distance() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let _edge3 = graph.insert_edge(&mut storage, node2, node3).unwrap();
        let _edge4 = graph.insert_edge(&mut storage, node3, node1).unwrap();

        let result = GraphSearch::from((&graph, &storage)).depth_first_search(
            node1,
            Handler {
                processor: |_index: GraphIndex, distance: u64| {
                    if distance == 2 {
                        SearchControl::Stop(true)
                    } else {
                        SearchControl::Continue(true)
                    }
                },
            },
        );

        assert_eq!(result, Ok(vec![node1, edge2, node2, edge1]));
    }
}
