use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;
use crate::graph_search::search_impl::SearchIndex;
use crate::graph_search::search_impl::SearchIterator;
use crate::storage::Storage;
use crate::StorageData;
use std::collections::VecDeque;

pub struct BreadthFirstSearchReverse {
    stack: VecDeque<SearchIndex>,
}

impl<D> SearchIterator<D> for BreadthFirstSearchReverse
where
    D: StorageData,
{
    fn new(index: GraphIndex) -> Self {
        Self {
            stack: VecDeque::from(vec![SearchIndex { index, distance: 0 }]),
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
            if follow {
                if let Some(i) = graph
                    .first_edge_to(storage, current_index.index)
                    .ok()
                    .filter(|i| i.is_valid())
                {
                    self.stack.push_back(SearchIndex {
                        index: i,
                        distance: current_index.distance + 1,
                    });
                }
            }
        } else {
            if follow {
                self.stack.push_back(SearchIndex {
                    index: graph.edge_from(storage, current_index.index),
                    distance: current_index.distance + 1,
                });
            }

            if let Some(i) = graph
                .next_edge_to(storage, current_index.index)
                .ok()
                .filter(|i| i.is_valid())
            {
                self.stack.push_front(SearchIndex {
                    index: i,
                    distance: current_index.distance,
                })
            }
        }
    }

    fn next(&mut self) -> Option<SearchIndex> {
        self.stack.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::super::SearchControl;
    use super::super::SearchHandler;
    use super::*;
    use crate::graph::DbGraph;
    use crate::graph::GraphIndex;
    use crate::graph_search::GraphSearch;
    use crate::storage::file_storage::FileStorage;
    use crate::test_utilities::test_file::TestFile;
    use crate::DbError;

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
    fn empty_graph_reverse() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let graph = DbGraph::new(&mut storage).unwrap();

        let result = GraphSearch::from((&graph, &storage))
            .breadth_first_search_reverse(GraphIndex::default(), Handler::default());

        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn search_in_reverse() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();
        let node4 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node2, node3).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node3, node4).unwrap();

        let result = GraphSearch::from((&graph, &storage))
            .breadth_first_search_reverse(node4, Handler::default());
        let expected = Ok(vec![node4, edge3, node3, edge2, node2, edge1, node1]);

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

        let _edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let _edge2 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node2, node3).unwrap();
        let _edge4 = graph.insert_edge(&mut storage, node3, node1).unwrap();

        let result = GraphSearch::from((&graph, &storage)).breadth_first_search_reverse(
            node3,
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

        assert_eq!(result, Ok(vec![node3, edge3, node2]));
    }
}
