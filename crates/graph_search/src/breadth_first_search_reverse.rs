use crate::search_index::SearchIndex;
use crate::search_iterator::SearchIterator;
use agdb_graph::GraphData;
use agdb_graph::GraphImpl;
use agdb_graph::GraphIndex;
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
    fn expand_edge<Data: agdb_graph::GraphData>(
        index: &GraphIndex,
        graph: &agdb_graph::GraphImpl<Data>,
    ) -> GraphIndex {
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
