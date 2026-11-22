use super::visit_key;
use crate::DbError;
use crate::DbId;
use crate::collections::bit_set::BitSet;
use crate::collections::graph::GraphHeader;
use crate::storage::pages::View;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub trait PathSearchHandler {
    fn process(&self, index: DbId, distance: u64) -> Result<(u64, bool), DbError>;
}

/// An element of a path linked to the previous element of the path.
struct Step {
    index: DbId,
    add: bool,
    previous: Option<usize>,
}

/// The cost, length and last step of a path. The heap pops the cheapest
/// path first, then the shortest and then the one added last.
type Path = (Reverse<u64>, Reverse<u64>, usize);

pub struct PathSearch<'a, V: View, Handler: PathSearchHandler> {
    destination: DbId,
    graph: &'a GraphHeader,
    view: &'a V,
    handler: Handler,
    paths: BinaryHeap<Path>,
    steps: Vec<Step>,
    visited: BitSet,
    reverse: bool,
}

impl<'a, V: View, Handler: PathSearchHandler> PathSearch<'a, V, Handler> {
    pub fn new(
        graph: &'a GraphHeader,
        view: &'a V,
        from: DbId,
        to: DbId,
        handler: Handler,
    ) -> Result<Self, DbError> {
        Self::with_direction(graph, view, from, to, handler, false)
    }

    pub fn new_reverse(
        graph: &'a GraphHeader,
        view: &'a V,
        from: DbId,
        to: DbId,
        handler: Handler,
    ) -> Result<Self, DbError> {
        Self::with_direction(graph, view, to, from, handler, true)
    }

    pub fn search(&mut self) -> Result<Vec<DbId>, DbError> {
        while let Some((Reverse(cost), Reverse(len), last)) = self.paths.pop() {
            let index = self.steps[last].index;

            if !self.visited.value(visit_key(index)) {
                if index.0 == self.destination.0 {
                    return Ok(self.elements(last));
                }

                self.visited.set(visit_key(index));
                self.expand(cost, len, last, index)?;
            }
        }

        Ok(vec![])
    }

    fn with_direction(
        graph: &'a GraphHeader,
        view: &'a V,
        origin: DbId,
        destination: DbId,
        handler: Handler,
        reverse: bool,
    ) -> Result<Self, DbError> {
        let add = handler.process(origin, 0)?;

        Ok(Self {
            destination,
            graph,
            view,
            handler,
            paths: BinaryHeap::from([(Reverse(0), Reverse(1), 0)]),
            steps: vec![Step {
                index: origin,
                add: add.1,
                previous: None,
            }],
            visited: BitSet::new(),
            reverse,
        })
    }

    fn elements(&self, last: usize) -> Vec<DbId> {
        let mut elements = vec![];
        let mut current = Some(last);

        while let Some(step) = current {
            if self.steps[step].add {
                elements.push(self.steps[step].index);
            }

            current = self.steps[step].previous;
        }

        if !self.reverse {
            elements.reverse();
        }

        elements
    }

    fn expand(&mut self, cost: u64, len: u64, last: usize, index: DbId) -> Result<(), DbError> {
        let reverse = self.reverse;
        let mut edge = self.graph.node(self.view, index)?.map_or(DbId(0), |node| {
            if reverse {
                node.first_to
            } else {
                node.first_from
            }
        });

        while let Some(record) = self.graph.edge(self.view, edge)? {
            let (next, next_edge) = if self.reverse {
                (record.from, record.next_to)
            } else {
                (record.to, record.next_from)
            };
            let edge_cost = self.handler.process(edge, len)?;

            if edge_cost.0 != 0 && !self.visited.value(visit_key(next)) {
                let next_cost = self.handler.process(next, len + 1)?;

                if next_cost.0 != 0 {
                    self.steps.push(Step {
                        index: edge,
                        add: edge_cost.1,
                        previous: Some(last),
                    });
                    self.steps.push(Step {
                        index: next,
                        add: next_cost.1,
                        previous: Some(self.steps.len() - 1),
                    });
                    self.paths.push((
                        Reverse(cost + edge_cost.0 + next_cost.0),
                        Reverse(len + 2),
                        self.steps.len() - 1,
                    ));
                }
            }

            edge = next_edge;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::value::tests::with_view;
    use crate::graph_search::tests::graph;
    use crate::graph_search::tests::ids;

    struct Costs(fn(DbId, u64) -> (u64, bool));

    impl PathSearchHandler for Costs {
        fn process(&self, index: DbId, distance: u64) -> Result<(u64, bool), DbError> {
            Ok((self.0)(index, distance))
        }
    }

    const ONE: Costs = Costs(|_, _| (1, true));

    /// 1 -1-> 3, 1 -2-> 2 -3-> 3
    const SHORTCUT: &[(i64, i64)] = &[(1, 3), (1, 2), (2, 3)];

    mod path_search {
        use super::*;

        #[test]
        fn path_search_should_return_the_cheapest_then_the_shortest_path() {
            with_view(|view, alloc| {
                let graph = graph(view, alloc, 3, SHORTCUT);

                for (costs, expected) in [
                    (ONE, ids(&[1, -1, 3])),
                    (
                        Costs(|index, _| (if index.0 == -1 { 10 } else { 1 }, true)),
                        ids(&[1, -2, 2, -3, 3]),
                    ),
                    (
                        Costs(|index, _| (if index.0 == -1 { 3 } else { 1 }, true)),
                        ids(&[1, -1, 3]),
                    ),
                ] {
                    let result = PathSearch::new(&graph, &*view, DbId(1), DbId(3), costs)
                        .unwrap()
                        .search();

                    assert_eq!(result, expected);
                }
            });
        }

        #[test]
        fn path_search_zero_cost_should_skip_the_element() {
            with_view(|view, alloc| {
                let graph = graph(view, alloc, 3, SHORTCUT);
                let costs = Costs(|index, _| (u64::from(index.0 != -1), true));

                let result = PathSearch::new(&graph, &*view, DbId(1), DbId(3), costs)
                    .unwrap()
                    .search();

                assert_eq!(result, ids(&[1, -2, 2, -3, 3]));
            });
        }

        #[test]
        fn path_search_parallel_edges_should_take_the_first_inserted_edge() {
            with_view(|view, alloc| {
                let graph = graph(view, alloc, 3, &[(1, 2), (1, 2), (2, 3), (2, 3)]);

                let result = PathSearch::new(&graph, &*view, DbId(1), DbId(3), ONE)
                    .unwrap()
                    .search();

                assert_eq!(result, ids(&[1, -1, 2, -3, 3]));
            });
        }

        #[test]
        fn path_search_not_added_should_follow_but_not_return_the_element() {
            with_view(|view, alloc| {
                let graph = graph(view, alloc, 3, &[(1, 2), (2, 2), (2, 3)]);

                for (costs, expected) in [
                    (Costs(|index, _| (1, 0 < index.0)), ids(&[1, 2, 3])),
                    (Costs(|index, _| (1, index.0 < 0)), ids(&[-1, -3])),
                ] {
                    let result = PathSearch::new(&graph, &*view, DbId(1), DbId(3), costs)
                        .unwrap()
                        .search();

                    assert_eq!(result, expected);
                }
            });
        }

        #[test]
        fn path_search_unreachable_destination_should_return_nothing() {
            with_view(|view, alloc| {
                let graph = graph(view, alloc, 3, &[(1, 2)]);

                let result = PathSearch::new(&graph, &*view, DbId(1), DbId(3), ONE)
                    .unwrap()
                    .search();

                assert_eq!(result, Ok(vec![]));
            });
        }

        #[test]
        fn path_search_reverse_should_follow_the_incoming_edges_and_return_the_path_from_the_origin()
         {
            with_view(|view, alloc| {
                let graph = graph(view, alloc, 3, SHORTCUT);
                let costs = Costs(|index, _| (u64::from(index.0 != -1), true));

                let result = PathSearch::new_reverse(&graph, &*view, DbId(1), DbId(3), costs)
                    .unwrap()
                    .search();

                assert_eq!(result, ids(&[1, -2, 2, -3, 3]));
            });
        }
    }
}
