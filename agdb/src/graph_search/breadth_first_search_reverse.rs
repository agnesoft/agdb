use crate::DbError;
use crate::StorageData;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::graph_search::search_impl::SearchIndex;
use crate::graph_search::search_impl::SearchIterator;
use crate::storage::Storage;
use std::collections::VecDeque;

pub struct BreadthFirstSearchReverse {
    stack: VecDeque<SearchIndex>,
}

impl<D> SearchIterator<D> for BreadthFirstSearchReverse
where
    D: StorageData,
{
    fn new(index: GraphIndex) -> Self {
        Self {
            stack: VecDeque::from(vec![SearchIndex { index, distance: 0 }]),
        }
    }

    fn expand(
        &mut self,
        current_index: SearchIndex,
        graph: &DbGraph<D>,
        storage: &Storage<D>,
        follow: bool,
    ) -> Result<(), DbError> {
        if current_index.index.is_node() {
            if follow {
                let i = graph.first_edge_to(storage, current_index.index)?;

                if i.is_valid() {
                    self.stack.push_back(SearchIndex {
                        index: i,
                        distance: current_index.distance + 1,
                    });
                }
            }
        } else {
            if follow {
                self.stack.push_back(SearchIndex {
                    index: graph.edge_from(storage, current_index.index)?,
                    distance: current_index.distance + 1,
                });
            }

            let i = graph.next_edge_to(storage, current_index.index)?;

            if i.is_valid() {
                self.stack.push_front(SearchIndex {
                    index: i,
                    distance: current_index.distance,
                })
            }
        }

        Ok(())
    }

    fn next(&mut self) -> Option<SearchIndex> {
        self.stack.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::tests::storage;
    use crate::graph::GraphIndex;
    use crate::graph_search::GraphSearch;
    use crate::graph_search::tests::Handler;
    use crate::graph_search::tests::all;
    use crate::graph_search::tests::diamond;
    use crate::graph_search::tests::indexes;
    use crate::graph_search::tests::stop_at;

    mod breadth_first_search_reverse {
        use super::*;

        #[test]
        fn breadth_first_search_reverse_should_visit_the_incoming_edges_level_by_level_once() {
            let mut storage = storage();
            let graph = diamond(&mut storage);

            let result = GraphSearch::from((&graph, &storage))
                .breadth_first_search_reverse(GraphIndex(4), all());

            assert_eq!(result, indexes(&[4, -8, -7, 3, 2, -6, -5, 1, -9]));
        }

        #[test]
        fn breadth_first_search_reverse_stopped_should_not_follow_the_stopped_elements() {
            let mut storage = storage();
            let graph = diamond(&mut storage);
            let search = GraphSearch::from((&graph, &storage));

            for (handler, expected) in [
                (Handler(stop_at::<1>), indexes(&[4, -8, -7])),
                (Handler(stop_at::<2>), indexes(&[4, -8, -7, 3, 2])),
            ] {
                assert_eq!(
                    search.breadth_first_search_reverse(GraphIndex(4), handler),
                    expected
                );
            }
        }
    }
}
