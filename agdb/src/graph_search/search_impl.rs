use super::SearchControl;
use super::SearchHandler;
use crate::DbError;
use crate::StorageData;
use crate::collections::bit_set::BitSet;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;
use crate::storage::Storage;
use std::mem::swap;

#[derive(Clone, Copy)]
pub struct SearchIndex {
    pub index: GraphIndex,
    pub distance: u64,
}

pub trait SearchIterator<D: StorageData> {
    fn new(index: GraphIndex) -> Self;
    fn expand<Data: GraphData<D>>(
        &mut self,
        current_index: SearchIndex,
        graph: &GraphImpl<D, Data>,
        storage: &Storage<D>,
        follow: bool,
    );
    fn next(&mut self) -> Option<SearchIndex>;
}

pub struct SearchImpl<'a, D, Data, SearchIt>
where
    Data: GraphData<D>,
    D: StorageData,
    SearchIt: SearchIterator<D>,
{
    algorithm: SearchIt,
    graph: &'a GraphImpl<D, Data>,
    storage: &'a Storage<D>,
    result: Vec<GraphIndex>,
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
            algorithm: SearchIt::new(index),
            graph,
            storage,
            result: vec![],
            visited: BitSet::new(),
        }
    }

    pub fn search<Handler: SearchHandler>(
        &mut self,
        mut handler: Handler,
    ) -> Result<Vec<GraphIndex>, DbError> {
        while let Some(current_index) = self.algorithm.next() {
            if !self.process_index(current_index, &mut handler)? {
                break;
            }
        }

        Ok(self.take_result())
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

    fn process_unvisited_index<Handler: SearchHandler>(
        &mut self,
        index: SearchIndex,
        handler: &mut Handler,
    ) -> Result<bool, DbError> {
        let add_index;
        let result;

        match handler.process(index.index, index.distance)? {
            SearchControl::Continue(add) => {
                self.algorithm.expand(index, self.graph, self.storage, true);
                add_index = add;
                result = true;
            }
            SearchControl::Finish(add) => {
                add_index = add;
                result = false;
            }
            SearchControl::Stop(add) => {
                self.algorithm
                    .expand(index, self.graph, self.storage, false);
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
