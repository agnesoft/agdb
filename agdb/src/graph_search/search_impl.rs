use super::SearchControl;
use super::SearchHandler;
use crate::DbError;
use crate::StorageData;
use crate::collections::bit_set::BitSet;
use crate::graph::DbGraph;
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
    fn expand(
        &mut self,
        current_index: SearchIndex,
        graph: &DbGraph<D>,
        storage: &Storage<D>,
        follow: bool,
    ) -> Result<(), DbError>;
    fn next(&mut self) -> Option<SearchIndex>;
}

pub struct SearchImpl<'a, D, SearchIt>
where
    D: StorageData,
    SearchIt: SearchIterator<D>,
{
    algorithm: SearchIt,
    graph: &'a DbGraph<D>,
    storage: &'a Storage<D>,
    result: Vec<GraphIndex>,
    visited: BitSet,
}

impl<'a, D, SearchIt> SearchImpl<'a, D, SearchIt>
where
    D: StorageData,
    SearchIt: SearchIterator<D>,
{
    pub fn new(graph: &'a DbGraph<D>, storage: &'a Storage<D>, index: GraphIndex) -> Self {
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
        let (add_index, result) = match handler.process(index.index, index.distance)? {
            SearchControl::Continue(add) => {
                self.algorithm
                    .expand(index, self.graph, self.storage, true)?;
                (add, true)
            }
            SearchControl::Finish(add) => (add, false),
            SearchControl::Stop(add) => {
                self.algorithm
                    .expand(index, self.graph, self.storage, false)?;
                (add, true)
            }
        };

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
    use crate::collections::tests::storage;
    use crate::graph_search::breadth_first_search::BreadthFirstSearch;
    use crate::graph_search::tests::Handler;
    use crate::graph_search::tests::diamond;
    use crate::graph_search::tests::indexes;

    mod search_impl {
        use super::*;

        mod search {
            use super::*;

            #[test]
            fn search_continue_false_should_follow_but_not_return_the_element() {
                let mut storage = storage();
                let graph = diamond(&mut storage);

                let result =
                    SearchImpl::<_, BreadthFirstSearch>::new(&graph, &storage, GraphIndex(1))
                        .search(Handler(|index, _| SearchControl::Continue(index.is_node())));

                assert_eq!(result, indexes(&[1, 3, 2, 4]));
            }

            #[test]
            fn search_finish_should_end_the_search() {
                let mut storage = storage();
                let graph = diamond(&mut storage);

                let result =
                    SearchImpl::<_, BreadthFirstSearch>::new(&graph, &storage, GraphIndex(1))
                        .search(Handler(|index, _| {
                            if index.0 == 3 {
                                SearchControl::Finish(true)
                            } else {
                                SearchControl::Continue(true)
                            }
                        }));

                assert_eq!(result, indexes(&[1, -6, -5, 3]));
            }
        }
    }
}
