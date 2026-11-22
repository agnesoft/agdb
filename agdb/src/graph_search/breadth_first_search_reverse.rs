use super::search_impl::SearchIndex;
use super::search_impl::SearchIterator;
use crate::DbError;
use crate::DbId;
use crate::collections::graph::GraphHeader;
use crate::storage::pages::View;
use std::collections::VecDeque;

pub struct BreadthFirstSearchReverse {
    stack: VecDeque<SearchIndex>,
}

impl SearchIterator for BreadthFirstSearchReverse {
    fn new(id: DbId) -> Self {
        Self {
            stack: VecDeque::from(vec![SearchIndex { id, distance: 0 }]),
        }
    }

    fn expand(
        &mut self,
        current: SearchIndex,
        graph: &GraphHeader,
        view: &impl View,
        follow: bool,
    ) -> Result<(), DbError> {
        if 0 < current.id.0 {
            if follow
                && let Some(node) = graph.node(view, current.id)?
                && node.first_to.0 != 0
            {
                self.stack.push_back(SearchIndex {
                    id: node.first_to,
                    distance: current.distance + 1,
                });
            }
        } else if let Some(edge) = graph.edge(view, current.id)? {
            if follow {
                self.stack.push_back(SearchIndex {
                    id: edge.from,
                    distance: current.distance + 1,
                });
            }

            if edge.next_to.0 != 0 {
                self.stack.push_front(SearchIndex {
                    id: edge.next_to,
                    distance: current.distance,
                });
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
    use crate::DbId;
    use crate::collections::value::tests::with_view;
    use crate::graph_search::GraphSearch;
    use crate::graph_search::tests::Handler;
    use crate::graph_search::tests::all;
    use crate::graph_search::tests::diamond;
    use crate::graph_search::tests::ids;
    use crate::graph_search::tests::stop_at;

    mod breadth_first_search_reverse {
        use super::*;

        #[test]
        fn breadth_first_search_reverse_should_visit_the_incoming_edges_level_by_level_once() {
            with_view(|view, alloc| {
                let graph = diamond(view, alloc);

                let result =
                    GraphSearch::new(&graph, &*view).breadth_first_search_reverse(DbId(4), all());

                assert_eq!(result, ids(&[4, -4, -3, 3, 2, -2, -1, 1, -5]));
            });
        }

        #[test]
        fn breadth_first_search_reverse_stopped_should_not_follow_the_stopped_elements() {
            with_view(|view, alloc| {
                let graph = diamond(view, alloc);
                let search = GraphSearch::new(&graph, &*view);

                for (handler, expected) in [
                    (Handler(stop_at::<1>), ids(&[4, -4, -3])),
                    (Handler(stop_at::<2>), ids(&[4, -4, -3, 3, 2])),
                ] {
                    assert_eq!(
                        search.breadth_first_search_reverse(DbId(4), handler),
                        expected
                    );
                }
            });
        }
    }
}
