use super::SearchControl;
use super::SearchHandler;
use crate::collections::bit_set::BitSet;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;
use crate::storage::Storage;
use crate::DbError;
use crate::StorageData;
use std::marker::PhantomData;
use std::mem::swap;

#[derive(Clone, Copy)]
pub struct SearchIndex {
    index: GraphIndex,
    distance: u64,
}

pub trait SearchIterator<D: StorageData> {
    fn expand_edge<Data: GraphData<D>>(
        index: GraphIndex,
        graph: &GraphImpl<D, Data>,
        storage: &Storage<D>,
    ) -> GraphIndex;
    fn expand_node<Data: GraphData<D>>(
        index: GraphIndex,
        graph: &GraphImpl<D, Data>,
        storage: &Storage<D>,
    ) -> Vec<GraphIndex>;
    fn new(stack: &mut Vec<SearchIndex>) -> Self;
    fn next(&mut self) -> Option<SearchIndex>;
}

pub struct SearchImpl<'a, D, Data, SearchIt>
where
    Data: GraphData<D>,
    D: StorageData,
    SearchIt: SearchIterator<D>,
{
    algorithm: PhantomData<SearchIt>,
    graph: &'a GraphImpl<D, Data>,
    storage: &'a Storage<D>,
    result: Vec<GraphIndex>,
    stack: Vec<SearchIndex>,
    visited: BitSet,
}

impl<'a, D, Data, SearchIt> SearchImpl<'a, D, Data, SearchIt>
where
    Data: GraphData<D>,
    D: StorageData,
    SearchIt: SearchIterator<D>,
{
    pub fn new(graph: &'a GraphImpl<D, Data>, storage: &'a Storage<D>, index: GraphIndex) -> Self {
        Self {
            algorithm: PhantomData,
            graph,
            storage,
            result: vec![],
            stack: vec![SearchIndex { index, distance: 0 }],
            visited: BitSet::new(),
        }
    }

    pub fn search<Handler: SearchHandler>(
        &mut self,
        mut handler: Handler,
    ) -> Result<Vec<GraphIndex>, DbError> {
        while !self.stack.is_empty() && self.process_stack(&mut handler)? {}

        Ok(self.take_result())
    }

    fn add_edges_to_stack(&mut self, edge_indexes: Vec<GraphIndex>, distance: u64) {
        for index in edge_indexes {
            self.stack.push(SearchIndex { index, distance });
        }
    }

    fn add_index_to_stack(&mut self, index: GraphIndex, distance: u64) {
        self.stack.push(SearchIndex { index, distance });
    }

    fn expand_index(&mut self, index: SearchIndex) {
        if index.index.is_node() {
            self.add_edges_to_stack(
                SearchIt::expand_node(index.index, self.graph, self.storage),
                index.distance + 1,
            );
        } else {
            self.add_index_to_stack(
                SearchIt::expand_edge(index.index, self.graph, self.storage),
                index.distance + 1,
            );
        }
    }

    fn process_index<Handler: SearchHandler>(
        &mut self,
        index: SearchIndex,
        handler: &mut Handler,
    ) -> Result<bool, DbError> {
        if !self.visit_index(&index) {
            self.process_unvisited_index(index, handler)
        } else {
            Ok(true)
        }
    }

    fn process_stack<Handler: SearchHandler>(
        &mut self,
        handler: &mut Handler,
    ) -> Result<bool, DbError> {
        let mut it = SearchIt::new(&mut self.stack);

        while let Some(i) = it.next() {
            if !self.process_index(i, handler)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn process_unvisited_index<Handler: SearchHandler>(
        &mut self,
        index: SearchIndex,
        handler: &mut Handler,
    ) -> Result<bool, DbError> {
        let add_index;
        let result;

        match handler.process(index.index, index.distance)? {
            SearchControl::Continue(add) => {
                self.expand_index(index);
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

        Ok(result)
    }

    fn take_result(&mut self) -> Vec<GraphIndex> {
        let mut res = Vec::<GraphIndex>::new();
        swap(&mut res, &mut self.result);

        res
    }

    fn visit_index(&mut self, index: &SearchIndex) -> bool {
        let visited = self.visited.value(index.index.as_u64());
        self.visited.set(index.index.as_u64());

        visited
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn derived_from_clone() {
        let index = SearchIndex {
            index: GraphIndex(1),
            distance: 10,
        };
        let other = index.clone();

        assert_eq!(index.index, other.index);
        assert_eq!(index.distance, other.distance);
    }

    #[test]
    fn derived_from_copy() {
        let index = &SearchIndex {
            index: GraphIndex(1),
            distance: 10,
        };
        let other = *index;

        assert_eq!(index.index, other.index);
        assert_eq!(index.distance, other.distance);
    }
}
