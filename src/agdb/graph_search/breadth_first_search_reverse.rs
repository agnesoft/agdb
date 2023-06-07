use super::search_impl::SearchIndex;
use super::search_impl::SearchIterator;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;
use std::mem::swap;
use std::vec::IntoIter;

pub(crate) struct BreadthFirstSearchReverse {
    stack_iterator: IntoIter<SearchIndex>,
}

impl BreadthFirstSearchReverse {
    fn take_stack(stack: &mut Vec<SearchIndex>) -> Vec<SearchIndex> {
        let mut res = Vec::<SearchIndex>::new();
        swap(&mut res, stack);

        res
    }
}

impl SearchIterator for BreadthFirstSearchReverse {
    fn expand_edge<Data: GraphData>(index: GraphIndex, graph: &GraphImpl<Data>) -> GraphIndex {
        graph.edge(index).unwrap().index_from()
    }

    fn expand_node<Data: GraphData>(index: GraphIndex, graph: &GraphImpl<Data>) -> Vec<GraphIndex> {
        graph
            .node(index)
            .expect("invalid index, expected a valid node index")
            .edge_iter_to()
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
    use crate::graph::DbGraph;
    use crate::graph_search::GraphSearch;
    use crate::storage::file_storage::FileStorage;
    use crate::test_utilities::test_file::TestFile;
    use std::cell::RefCell;
    use std::rc::Rc;

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
        fn process(&self, index: GraphIndex, distance: u64) -> SearchControl {
            (self.processor)(index, distance)
        }
    }

    #[test]
    fn empty_graph_reverse() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let graph = DbGraph::new(storage).unwrap();

        let result = GraphSearch::from(&graph)
            .breadth_first_search_reverse(GraphIndex::default(), &Handler::default());

        assert_eq!(result, vec![]);
    }

    #[test]
    fn search_in_reverse() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut graph = DbGraph::new(storage).unwrap();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();
        let node4 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(node1, node2).unwrap();
        let edge2 = graph.insert_edge(node2, node3).unwrap();
        let edge3 = graph.insert_edge(node3, node4).unwrap();

        let result =
            GraphSearch::from(&graph).breadth_first_search_reverse(node4, &Handler::default());
        let expected = vec![node4, edge3, node3, edge2, node2, edge1, node1];

        assert_eq!(result, expected);
    }
}
