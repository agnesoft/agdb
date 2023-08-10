use super::search_impl::SearchIndex;
use super::search_impl::SearchIterator;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;
use std::mem::swap;
use std::vec::IntoIter;

pub(crate) struct BreadthFirstSearch {
    stack_iterator: IntoIter<SearchIndex>,
}

impl BreadthFirstSearch {
    fn take_stack(stack: &mut Vec<SearchIndex>) -> Vec<SearchIndex> {
        let mut res = Vec::<SearchIndex>::new();
        swap(&mut res, stack);

        res
    }
}

impl<S> SearchIterator<S> for BreadthFirstSearch {
    fn expand_edge<Data: GraphData<S>>(
        index: GraphIndex,
        graph: &GraphImpl<S, Data>,
    ) -> GraphIndex {
        graph.edge(index).unwrap().index_to()
    }

    fn expand_node<Data: GraphData<S>>(
        index: GraphIndex,
        graph: &GraphImpl<S, Data>,
    ) -> Vec<GraphIndex> {
        graph
            .node(index)
            .expect("invalid index, expected a valid node index")
            .edge_iter_from()
            .map(|edge| edge.index())
            .collect()
    }

    fn new(stack: &mut Vec<SearchIndex>) -> Self {
        Self {
            stack_iterator: Self::take_stack(stack).into_iter(),
        }
    }

    fn next(&mut self) -> Option<SearchIndex> {
        self.stack_iterator.next()
    }
}

#[cfg(test)]
mod tests {
    use super::super::SearchControl;
    use super::super::SearchHandler;
    use super::*;
    use crate::db::db_error::DbError;
    use crate::graph::DbGraph;
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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let graph = DbGraph::new(&mut storage).unwrap();

        let result = GraphSearch::from(&graph)
            .breadth_first_search(GraphIndex::default(), Handler::default());

        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn cyclic_graph() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
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

        let result = GraphSearch::from(&graph).breadth_first_search(node1, Handler::default());

        assert_eq!(
            result,
            Ok(vec![
                node1, edge2, edge1, node2, edge4, edge3, node3, edge6, edge5
            ])
        );
    }

    #[test]
    fn full_search() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();
        let node4 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node3).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node1, node4).unwrap();

        let result = GraphSearch::from(&graph).breadth_first_search(node1, Handler::default());

        assert_eq!(
            result,
            Ok(vec![node1, edge3, edge2, edge1, node4, node3, node2])
        );
    }

    #[test]
    fn filter_edges() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();
        let node4 = graph.insert_node(&mut storage).unwrap();

        graph.insert_edge(&mut storage, node1, node2).unwrap();
        graph.insert_edge(&mut storage, node1, node3).unwrap();
        graph.insert_edge(&mut storage, node1, node4).unwrap();

        let result = GraphSearch::from(&graph).breadth_first_search(
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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
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

        let result = GraphSearch::from(&graph).breadth_first_search(
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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();
        let node4 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node3).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node1, node4).unwrap();

        let mut result = GraphSearch::from(&graph).breadth_first_search(node1, Handler::default());
        let expected = Ok(vec![node1, edge3, edge2, edge1, node4, node3, node2]);

        assert_eq!(result, expected);

        result = GraphSearch::from(&graph).breadth_first_search(node1, Handler::default());

        assert_eq!(result, expected);
    }

    #[test]
    fn stop_at_distance() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let _edge3 = graph.insert_edge(&mut storage, node2, node3).unwrap();
        let _edge4 = graph.insert_edge(&mut storage, node3, node1).unwrap();

        let result = GraphSearch::from(&graph).breadth_first_search(
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

        assert_eq!(result, Ok(vec![node1, edge2, edge1, node2]));
    }
}
