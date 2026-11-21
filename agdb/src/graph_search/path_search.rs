use crate::DbError;
use crate::StorageData;
use crate::collections::bit_set::BitSet;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::storage::Storage;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub trait PathSearchHandler {
    fn process(&self, index: GraphIndex, distance: u64) -> Result<(u64, bool), DbError>;
}

/// An element of a path linked to the previous element of the path.
struct Step {
    index: GraphIndex,
    add: bool,
    previous: Option<usize>,
}

/// The cost, length and last step of a path. The heap pops the cheapest
/// path first, then the shortest and then the one added last.
type Path = (Reverse<u64>, Reverse<u64>, usize);

pub struct PathSearch<'a, D, Handler>
where
    D: StorageData,
    Handler: PathSearchHandler,
{
    destination: GraphIndex,
    graph: &'a DbGraph<D>,
    storage: &'a Storage<D>,
    handler: Handler,
    paths: BinaryHeap<Path>,
    steps: Vec<Step>,
    visited: BitSet,
    reverse: bool,
}

impl<'a, D, Handler> PathSearch<'a, D, Handler>
where
    D: StorageData,
    Handler: PathSearchHandler,
{
    pub fn new(
        graph: &'a DbGraph<D>,
        storage: &'a Storage<D>,
        from: GraphIndex,
        to: GraphIndex,
        handler: Handler,
    ) -> Result<Self, DbError> {
        Self::with_direction(graph, storage, from, to, handler, false)
    }

    pub fn new_reverse(
        graph: &'a DbGraph<D>,
        storage: &'a Storage<D>,
        from: GraphIndex,
        to: GraphIndex,
        handler: Handler,
    ) -> Result<Self, DbError> {
        Self::with_direction(graph, storage, to, from, handler, true)
    }

    pub fn search(&mut self) -> Result<Vec<GraphIndex>, DbError> {
        while let Some((Reverse(cost), Reverse(len), last)) = self.paths.pop() {
            let index = self.steps[last].index;

            if !self.visited.value(index.as_u64()) {
                if index.0 == self.destination.0 {
                    return Ok(self.elements(last));
                }

                self.visited.set(index.as_u64());
                self.expand(cost, len, last, index)?;
            }
        }

        Ok(vec![])
    }

    fn with_direction(
        graph: &'a DbGraph<D>,
        storage: &'a Storage<D>,
        origin: GraphIndex,
        destination: GraphIndex,
        handler: Handler,
        reverse: bool,
    ) -> Result<Self, DbError> {
        let add = handler.process(origin, 0)?;

        Ok(Self {
            destination,
            graph,
            storage,
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

    fn elements(&self, last: usize) -> Vec<GraphIndex> {
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

    fn expand(
        &mut self,
        cost: u64,
        len: u64,
        last: usize,
        index: GraphIndex,
    ) -> Result<(), DbError> {
        let edges = if self.reverse {
            self.graph.edge_iter_to(self.storage, index)?
        } else {
            self.graph.edge_iter_from(self.storage, index)?
        };

        for edge in edges {
            let edge = edge?;
            let next = if self.reverse {
                edge.index_from()
            } else {
                edge.index_to()
            };
            let edge_cost = self.handler.process(edge.index(), len)?;

            if edge_cost.0 != 0 && !self.visited.value(next.as_u64()) {
                let next_cost = self.handler.process(next, len + 1)?;

                if next_cost.0 != 0 {
                    self.steps.push(Step {
                        index: edge.index(),
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
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::tests::storage;
    use crate::graph_search::tests::graph;
    use crate::graph_search::tests::indexes;

    struct Costs(fn(GraphIndex, u64) -> (u64, bool));

    impl PathSearchHandler for Costs {
        fn process(&self, index: GraphIndex, distance: u64) -> Result<(u64, bool), DbError> {
            Ok((self.0)(index, distance))
        }
    }

    const ONE: Costs = Costs(|_, _| (1, true));

    /// 1 -4-> 3, 1 -5-> 2 -6-> 3
    const SHORTCUT: &[(i64, i64)] = &[(1, 3), (1, 2), (2, 3)];

    mod path_search {
        use super::*;

        #[test]
        fn path_search_should_return_the_cheapest_then_the_shortest_path() {
            let mut storage = storage();
            let graph = graph(&mut storage, 3, SHORTCUT);

            for (costs, expected) in [
                (ONE, indexes(&[1, -4, 3])),
                (
                    Costs(|index, _| (if index.0 == -4 { 10 } else { 1 }, true)),
                    indexes(&[1, -5, 2, -6, 3]),
                ),
                (
                    Costs(|index, _| (if index.0 == -4 { 3 } else { 1 }, true)),
                    indexes(&[1, -4, 3]),
                ),
            ] {
                let result = PathSearch::new(&graph, &storage, GraphIndex(1), GraphIndex(3), costs)
                    .unwrap()
                    .search();

                assert_eq!(result, expected);
            }
        }

        #[test]
        fn path_search_zero_cost_should_skip_the_element() {
            let mut storage = storage();
            let graph = graph(&mut storage, 3, SHORTCUT);
            let costs = Costs(|index, _| (u64::from(index.0 != -4), true));

            let result = PathSearch::new(&graph, &storage, GraphIndex(1), GraphIndex(3), costs)
                .unwrap()
                .search();

            assert_eq!(result, indexes(&[1, -5, 2, -6, 3]));
        }

        #[test]
        fn path_search_parallel_edges_should_take_the_first_inserted_edge() {
            let mut storage = storage();
            let graph = graph(&mut storage, 3, &[(1, 2), (1, 2), (2, 3), (2, 3)]);

            let result = PathSearch::new(&graph, &storage, GraphIndex(1), GraphIndex(3), ONE)
                .unwrap()
                .search();

            assert_eq!(result, indexes(&[1, -4, 2, -6, 3]));
        }

        #[test]
        fn path_search_not_added_should_follow_but_not_return_the_element() {
            let mut storage = storage();
            let graph = graph(&mut storage, 3, &[(1, 2), (2, 2), (2, 3)]);

            for (costs, expected) in [
                (Costs(|index, _| (1, index.is_node())), indexes(&[1, 2, 3])),
                (Costs(|index, _| (1, index.is_edge())), indexes(&[-4, -6])),
            ] {
                let result = PathSearch::new(&graph, &storage, GraphIndex(1), GraphIndex(3), costs)
                    .unwrap()
                    .search();

                assert_eq!(result, expected);
            }
        }

        #[test]
        fn path_search_unreachable_destination_should_return_nothing() {
            let mut storage = storage();
            let graph = graph(&mut storage, 3, &[(1, 2)]);

            let result = PathSearch::new(&graph, &storage, GraphIndex(1), GraphIndex(3), ONE)
                .unwrap()
                .search();

            assert_eq!(result, Ok(vec![]));
        }

        #[test]
        fn path_search_reverse_should_follow_the_incoming_edges_and_return_the_path_from_the_origin()
         {
            let mut storage = storage();
            let graph = graph(&mut storage, 3, SHORTCUT);
            let costs = Costs(|index, _| (u64::from(index.0 != -4), true));

            let result =
                PathSearch::new_reverse(&graph, &storage, GraphIndex(1), GraphIndex(3), costs)
                    .unwrap()
                    .search();

            assert_eq!(result, indexes(&[1, -5, 2, -6, 3]));
        }
    }
}
