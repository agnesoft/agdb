use super::search_impl::SearchIndex;
use super::search_impl::SearchIterator;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;

pub(crate) struct DepthFirstSearchReverse {
    index: Option<SearchIndex>,
}

impl<S> SearchIterator<S> for DepthFirstSearchReverse {
    fn expand_edge<Data: GraphData<S>>(
        index: GraphIndex,
        graph: &GraphImpl<S, Data>,
    ) -> GraphIndex {
        graph
            .edge(index)
            .expect("invalid index, expected a valid edge index")
            .index_from()
    }

    fn expand_node<Data: GraphData<S>>(
        index: GraphIndex,
        graph: &GraphImpl<S, Data>,
    ) -> Vec<GraphIndex> {
        graph
            .node(index)
            .expect("invalid index, expected a valid node index")
            .edge_iter_to()
            .map(|edge| edge.index())
            .collect()
    }

    fn new(stack: &mut Vec<SearchIndex>) -> Self {
        Self { index: stack.pop() }
    }

    fn next(&mut self) -> Option<SearchIndex> {
        self.index.take()
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
    fn empty_graph_reverse() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let graph = DbGraph::new(&mut storage).unwrap();

        let result = GraphSearch::from(&graph)
            .depth_first_search_reverse(GraphIndex::default(), Handler::default());

        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn search_in_reverse() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut graph = DbGraph::new(&mut storage).unwrap();

        let node1 = graph.insert_node(&mut storage).unwrap();
        let node2 = graph.insert_node(&mut storage).unwrap();
        let node3 = graph.insert_node(&mut storage).unwrap();
        let node4 = graph.insert_node(&mut storage).unwrap();

        let edge1 = graph.insert_edge(&mut storage, node1, node2).unwrap();
        let edge2 = graph.insert_edge(&mut storage, node2, node3).unwrap();
        let edge3 = graph.insert_edge(&mut storage, node3, node4).unwrap();

        let result =
            GraphSearch::from(&graph).depth_first_search_reverse(node4, Handler::default());
        let expected = Ok(vec![node4, edge3, node3, edge2, node2, edge1, node1]);

        assert_eq!(result, expected);
    }
}
