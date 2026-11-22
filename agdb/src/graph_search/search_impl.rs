use super::SearchControl;
use super::SearchHandler;
use super::visit_key;
use crate::DbError;
use crate::DbId;
use crate::collections::bit_set::BitSet;
use crate::collections::graph::GraphHeader;
use crate::storage::pages::View;
use std::mem::swap;

#[derive(Clone, Copy)]
pub struct SearchIndex {
    pub id: DbId,
    pub distance: u64,
}

pub trait SearchIterator {
    fn new(id: DbId) -> Self;
    fn expand(
        &mut self,
        current: SearchIndex,
        graph: &GraphHeader,
        view: &impl View,
        follow: bool,
    ) -> Result<(), DbError>;
    fn next(&mut self) -> Option<SearchIndex>;
}

pub struct SearchImpl<'a, V: View, SearchIt: SearchIterator> {
    algorithm: SearchIt,
    graph: &'a GraphHeader,
    view: &'a V,
    result: Vec<DbId>,
    visited: BitSet,
}

impl<'a, V: View, SearchIt: SearchIterator> SearchImpl<'a, V, SearchIt> {
    pub fn new(graph: &'a GraphHeader, view: &'a V, id: DbId) -> Self {
        Self {
            algorithm: SearchIt::new(id),
            graph,
            view,
            result: vec![],
            visited: BitSet::new(),
        }
    }

    pub fn search<Handler: SearchHandler>(
        &mut self,
        mut handler: Handler,
    ) -> Result<Vec<DbId>, DbError> {
        while let Some(current) = self.algorithm.next() {
            if !self.process_index(current, &mut handler)? {
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
        let (add_index, result) = match handler.process(index.id, index.distance)? {
            SearchControl::Continue(add) => {
                self.algorithm.expand(index, self.graph, self.view, true)?;
                (add, true)
            }
            SearchControl::Finish(add) => (add, false),
            SearchControl::Stop(add) => {
                self.algorithm.expand(index, self.graph, self.view, false)?;
                (add, true)
            }
        };

        if add_index {
            self.result.push(index.id);
        }

        Ok(result)
    }

    fn take_result(&mut self) -> Vec<DbId> {
        let mut res = Vec::<DbId>::new();
        swap(&mut res, &mut self.result);

        res
    }

    fn visit_index(&mut self, index: &SearchIndex) -> bool {
        let key = visit_key(index.id);
        let visited = self.visited.value(key);
        self.visited.set(key);

        visited
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::value::tests::with_view;
    use crate::graph_search::breadth_first_search::BreadthFirstSearch;
    use crate::graph_search::tests::Handler;
    use crate::graph_search::tests::diamond;
    use crate::graph_search::tests::ids;

    mod search_impl {
        use super::*;

        mod search {
            use super::*;

            #[test]
            fn search_continue_false_should_follow_but_not_return_the_element() {
                with_view(|view, alloc| {
                    let graph = diamond(view, alloc);

                    let result = SearchImpl::<_, BreadthFirstSearch>::new(&graph, &*view, DbId(1))
                        .search(Handler(|index, _| SearchControl::Continue(0 < index.0)));

                    assert_eq!(result, ids(&[1, 3, 2, 4]));
                });
            }

            #[test]
            fn search_finish_should_end_the_search() {
                with_view(|view, alloc| {
                    let graph = diamond(view, alloc);

                    let result = SearchImpl::<_, BreadthFirstSearch>::new(&graph, &*view, DbId(1))
                        .search(Handler(|index, _| {
                            if index.0 == 3 {
                                SearchControl::Finish(true)
                            } else {
                                SearchControl::Continue(true)
                            }
                        }));

                    assert_eq!(result, ids(&[1, -2, -1, 3]));
                });
            }
        }
    }
}
