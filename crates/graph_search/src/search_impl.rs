use crate::search_index::SearchIndex;
use crate::search_iterator::SearchIterator;
use crate::SearchControl;
use crate::SearchHandler;
use agdb_bit_set::BitSet;
use agdb_graph::GraphData;
use agdb_graph::GraphImpl;
use agdb_graph::GraphIndex;
use std::marker::PhantomData;
use std::mem::swap;

pub(crate) struct SearchImpl<'a, Data, SearchIt>
where
    Data: GraphData,
    SearchIt: SearchIterator,
{
    pub(crate) algorithm: PhantomData<SearchIt>,
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) result: Vec<GraphIndex>,
    pub(crate) stack: Vec<SearchIndex>,
    pub(crate) visited: BitSet,
}

impl<'a, Data, SearchIt> SearchImpl<'a, Data, SearchIt>
where
    Data: GraphData,
    SearchIt: SearchIterator,
{
    pub(crate) fn new(graph: &'a GraphImpl<Data>, index: GraphIndex) -> Self {
        Self {
            algorithm: PhantomData,
            graph,
            result: vec![],
            stack: vec![SearchIndex { index, distance: 0 }],
            visited: BitSet::new(),
        }
    }

    pub(crate) fn search<Handler: SearchHandler>(&mut self, handler: &Handler) -> Vec<GraphIndex> {
        while !self.stack.is_empty() && self.process_stack(handler) {}

        self.take_result()
    }

    fn add_edges_to_stack(&mut self, edge_indexes: Vec<GraphIndex>, distance: u64) {
        for index in edge_indexes {
            self.stack.push(SearchIndex { index, distance });
        }
    }

    fn add_index_to_stack(&mut self, index: GraphIndex, distance: u64) {
        self.stack.push(SearchIndex { index, distance });
    }

    fn expand_index(&mut self, index: &SearchIndex) {
        if index.index.is_node() {
            self.add_edges_to_stack(
                SearchIt::expand_node(&index.index, self.graph),
                index.distance + 1,
            );
        } else {
            self.add_index_to_stack(
                SearchIt::expand_edge(&index.index, self.graph),
                index.distance + 1,
            );
        }
    }

    fn process_index<Handler: SearchHandler>(
        &mut self,
        index: SearchIndex,
        handler: &Handler,
    ) -> bool {
        if !self.visit_index(&index) {
            self.process_unvisited_index(index, handler)
        } else {
            true
        }
    }

    fn process_stack<Handler: SearchHandler>(&mut self, handler: &Handler) -> bool {
        let mut it = SearchIt::new(&mut self.stack);

        while let Some(i) = it.next() {
            if !self.process_index(i, handler) {
                return false;
            }
        }

        true
    }

    fn process_unvisited_index<Handler: SearchHandler>(
        &mut self,
        index: SearchIndex,
        handler: &Handler,
    ) -> bool {
        let add_index;
        let result;

        match handler.process(&index.index, &index.distance) {
            SearchControl::Continue(add) => {
                self.expand_index(&index);
                add_index = add;
                result = true;
            }
            SearchControl::Finish(add) => {
                add_index = add;
                result = false;
            }
            SearchControl::Stop(add) => {
                add_index = add;
                result = true;
            }
        }

        if add_index {
            self.result.push(index.index);
        }

        result
    }

    fn take_result(&mut self) -> Vec<GraphIndex> {
        let mut res = Vec::<GraphIndex>::new();
        swap(&mut res, &mut self.result);

        res
    }

    fn visit_index(&mut self, index: &SearchIndex) -> bool {
        let visited = self.visited.value(index.index.as_u64());
        self.visited.insert(index.index.as_u64());

        visited
    }
}
