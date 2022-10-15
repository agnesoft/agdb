use std::mem::swap;

use crate::graph_search_index::GraphSearchIndex;
use crate::search_control::SearchControl;
use crate::search_handler::SearchHandler;
use agdb_bit_set::BitSet;
use agdb_graph::GraphData;
use agdb_graph::GraphEdgeIterator;
use agdb_graph::GraphImpl;
use agdb_graph::GraphIndex;

pub struct GraphSearch<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) stack: Vec<GraphSearchIndex>,
    pub(crate) visited: BitSet,
    pub(crate) result: Vec<GraphIndex>,
}

impl<'a, Data> GraphSearch<'a, Data>
where
    Data: GraphData,
{
    pub fn breadth_first_search<Handler: SearchHandler>(
        &mut self,
        index: &GraphIndex,
        handler: &Handler,
    ) -> Vec<GraphIndex> {
        self.clear();
        self.add_index_to_stack(index.clone(), 0);
        self.search(handler);

        self.take_result()
    }

    fn add_edges_to_stack(&mut self, edges: GraphEdgeIterator<Data>, distance: u64) {
        for edge in edges {
            self.stack.push(GraphSearchIndex {
                index: edge.index(),
                distance,
            });
        }
    }

    fn add_index_to_stack(&mut self, index: GraphIndex, distance: u64) {
        if self.validate_index(&index) {
            self.stack.push(GraphSearchIndex { index, distance });
        }
    }

    fn clear(&mut self) {
        self.result.clear();
        self.stack.clear();
        self.visited.clear();
    }

    fn expand_index(&mut self, index: &GraphSearchIndex) {
        if let Some(node) = self.graph.node(&index.index) {
            self.add_edges_to_stack(node.edge_from_iter(), index.distance + 1);
        } else if let Some(edge) = self.graph.edge(&index.index) {
            self.add_index_to_stack(edge.to_index(), index.distance + 1);
        }
    }

    fn process_index<Handler: SearchHandler>(
        &mut self,
        index: GraphSearchIndex,
        handler: &Handler,
    ) -> bool {
        if !self.visit_index(&index) {
            self.process_unvisited_index(index, handler)
        } else {
            true
        }
    }

    fn process_unvisited_index<Handler: SearchHandler>(
        &mut self,
        index: GraphSearchIndex,
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

    fn process_stack<Handler: SearchHandler>(&mut self, handler: &Handler) -> bool {
        for i in self.take_stack() {
            if !self.process_index(i, handler) {
                return false;
            }
        }

        true
    }

    fn search<T: SearchHandler>(&mut self, handler: &T) {
        while !self.stack.is_empty() && self.process_stack(handler) {}
    }

    fn take_result(&mut self) -> Vec<GraphIndex> {
        let mut res = Vec::<GraphIndex>::new();
        swap(&mut res, &mut self.result);

        res
    }

    fn take_stack(&mut self) -> Vec<GraphSearchIndex> {
        let mut res = Vec::<GraphSearchIndex>::new();
        swap(&mut res, &mut self.stack);

        res
    }

    fn validate_index(&self, index: &GraphIndex) -> bool {
        self.graph.node(index).is_some() || self.graph.edge(index).is_some()
    }

    fn visit_index(&mut self, index: &GraphSearchIndex) -> bool {
        let visited = self.visited.value(index.index.as_u64());
        self.visited.insert(index.index.as_u64());

        visited
    }
}
