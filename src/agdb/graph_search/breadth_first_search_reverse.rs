use super::search_index::SearchIndex;
use super::search_iterator::SearchIterator;
use crate::graph::graph_data::GraphData;
use crate::graph::graph_impl::GraphImpl;
use crate::graph::graph_index::GraphIndex;
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
    fn expand_edge<Data: GraphData>(index: &GraphIndex, graph: &GraphImpl<Data>) -> GraphIndex {
        graph.edge(index).unwrap().index_from()
    }

    fn expand_node<Data: GraphData>(
        index: &GraphIndex,
        graph: &GraphImpl<Data>,
    ) -> Vec<GraphIndex> {
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
    use super::super::search_control::SearchControl;
    use super::super::search_handler::SearchHandler;
    use super::*;
    use crate::graph::Graph;
    use crate::graph_search::GraphSearch;

    struct Handler {
        pub processor: fn(&GraphIndex, &u64) -> SearchControl,
    }

    impl Default for Handler {
        fn default() -> Self {
            Self {
                processor: |_index: &GraphIndex, _distance: &u64| SearchControl::Continue(true),
            }
        }
    }

    impl SearchHandler for Handler {
        fn process(&self, index: &GraphIndex, distance: &u64) -> SearchControl {
            (self.processor)(index, distance)
        }
    }

    #[test]
    fn empty_graph_reverse() {
        let graph = Graph::new();

        let result = GraphSearch::from(&graph)
            .breadth_first_search_reverse(&GraphIndex::default(), &Handler::default());

        assert_eq!(result, vec![]);
    }

    #[test]
    fn search_in_reverse() {
        let mut graph = Graph::new();

        let node1 = graph.insert_node().unwrap();
        let node2 = graph.insert_node().unwrap();
        let node3 = graph.insert_node().unwrap();
        let node4 = graph.insert_node().unwrap();

        let edge1 = graph.insert_edge(&node1, &node2).unwrap();
        let edge2 = graph.insert_edge(&node2, &node3).unwrap();
        let edge3 = graph.insert_edge(&node3, &node4).unwrap();

        let result =
            GraphSearch::from(&graph).breadth_first_search_reverse(&node4, &Handler::default());
        let expected = vec![node4.clone(), edge3, node3, edge2, node2, edge1, node1];

        assert_eq!(result, expected);
    }
}
