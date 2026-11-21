use super::search_impl::SearchIndex;
use super::search_impl::SearchIterator;
use crate::DbError;
use crate::StorageData;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::storage::Storage;

pub struct DepthFirstSearch {
    stack: Vec<SearchIndex>,
}

impl<D> SearchIterator<D> for DepthFirstSearch
where
    D: StorageData,
{
    fn new(index: GraphIndex) -> Self {
        Self {
            stack: vec![SearchIndex { index, distance: 0 }],
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
                let i = graph.first_edge_from(storage, current_index.index)?;

                if i.is_valid() {
                    self.stack.push(SearchIndex {
                        index: i,
                        distance: current_index.distance + 1,
                    });
                }
            }
        } else {
            let i = graph.next_edge_from(storage, current_index.index)?;

            if i.is_valid() {
                self.stack.push(SearchIndex {
                    index: i,
                    distance: current_index.distance,
                })
            }

            if follow {
                self.stack.push(SearchIndex {
                    index: graph.edge_to(storage, current_index.index)?,
                    distance: current_index.distance + 1,
                });
            }
        }

        Ok(())
    }

    fn next(&mut self) -> Option<SearchIndex> {
        self.stack.pop()
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

    mod depth_first_search {
        use super::*;

        #[test]
        fn depth_first_search_should_visit_the_outgoing_edges_branch_by_branch_once() {
            let mut storage = storage();
            let graph = diamond(&mut storage);

            let result =
                GraphSearch::from((&graph, &storage)).depth_first_search(GraphIndex(1), all());

            assert_eq!(result, indexes(&[1, -6, 3, -8, 4, -9, -5, 2, -7]));
        }

        #[test]
        fn depth_first_search_stopped_should_not_follow_the_stopped_elements() {
            let mut storage = storage();
            let graph = diamond(&mut storage);
            let search = GraphSearch::from((&graph, &storage));

            for (handler, expected) in [
                (Handler(stop_at::<1>), indexes(&[1, -6, -5])),
                (Handler(stop_at::<2>), indexes(&[1, -6, 3, -5, 2])),
            ] {
                assert_eq!(search.depth_first_search(GraphIndex(1), handler), expected);
            }
        }
    }
}
