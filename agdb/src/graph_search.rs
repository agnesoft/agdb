pub mod path_search;

mod breadth_first_search;
mod breadth_first_search_reverse;
mod depth_first_search;
mod depth_first_search_reverse;
mod element_search;
mod search_impl;

use self::breadth_first_search::BreadthFirstSearch;
use self::breadth_first_search_reverse::BreadthFirstSearchReverse;
use self::depth_first_search::DepthFirstSearch;
use self::depth_first_search_reverse::DepthFirstSearchReverse;
use self::path_search::PathSearch;
use self::path_search::PathSearchHandler;
use self::search_impl::SearchImpl;
use crate::DbError;
use crate::StorageData;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::graph_search::element_search::ElementSearch;
use crate::storage::Storage;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchControl {
    Continue(bool),
    Finish(bool),
    Stop(bool),
}

pub trait SearchHandler {
    fn process(&mut self, index: GraphIndex, distance: u64) -> Result<SearchControl, DbError>;
}

pub struct GraphSearch<'a, D>
where
    D: StorageData,
{
    graph: &'a DbGraph<D>,
    storage: &'a Storage<D>,
}

impl<'a, D> GraphSearch<'a, D>
where
    D: StorageData,
{
    pub fn breadth_first_search<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: Handler,
    ) -> Result<Vec<GraphIndex>, DbError> {
        if self.is_valid_index(index)? {
            SearchImpl::<'a, D, BreadthFirstSearch>::new(self.graph, self.storage, index)
                .search(handler)
        } else {
            Ok(vec![])
        }
    }

    pub fn breadth_first_search_reverse<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: Handler,
    ) -> Result<Vec<GraphIndex>, DbError> {
        if self.is_valid_index(index)? {
            SearchImpl::<'a, D, BreadthFirstSearchReverse>::new(self.graph, self.storage, index)
                .search(handler)
        } else {
            Ok(vec![])
        }
    }

    pub fn depth_first_search<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: Handler,
    ) -> Result<Vec<GraphIndex>, DbError> {
        if self.is_valid_index(index)? {
            SearchImpl::<'a, D, DepthFirstSearch>::new(self.graph, self.storage, index)
                .search(handler)
        } else {
            Ok(vec![])
        }
    }

    pub fn depth_first_search_reverse<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: Handler,
    ) -> Result<Vec<GraphIndex>, DbError> {
        if self.is_valid_index(index)? {
            SearchImpl::<'a, D, DepthFirstSearchReverse>::new(self.graph, self.storage, index)
                .search(handler)
        } else {
            Ok(vec![])
        }
    }

    pub fn path<Handler: PathSearchHandler>(
        &self,
        from: GraphIndex,
        to: GraphIndex,
        handler: Handler,
    ) -> Result<Vec<GraphIndex>, DbError> {
        if from != to && self.is_valid_node(from)? && self.is_valid_node(to)? {
            PathSearch::<D, Handler>::new(self.graph, self.storage, from, to, handler)?.search()
        } else {
            Ok(vec![])
        }
    }

    pub fn path_reverse<Handler: PathSearchHandler>(
        &self,
        from: GraphIndex,
        to: GraphIndex,
        handler: Handler,
    ) -> Result<Vec<GraphIndex>, DbError> {
        if from != to && self.is_valid_node(from)? && self.is_valid_node(to)? {
            PathSearch::<D, Handler>::new_reverse(self.graph, self.storage, from, to, handler)?
                .search()
        } else {
            Ok(vec![])
        }
    }

    pub fn elements<Handler: SearchHandler>(
        &self,
        handler: Handler,
    ) -> Result<Vec<GraphIndex>, DbError> {
        ElementSearch::<D, Handler>::new(self.graph, self.storage, handler).search()
    }

    fn is_valid_index(&self, index: GraphIndex) -> Result<bool, DbError> {
        if index.is_edge() {
            Ok(self.graph.edge(self.storage, index)?.is_some())
        } else {
            self.is_valid_node(index)
        }
    }

    fn is_valid_node(&self, index: GraphIndex) -> Result<bool, DbError> {
        Ok(self.graph.node(self.storage, index)?.is_some())
    }
}

impl<'a, D> From<(&'a DbGraph<D>, &'a Storage<D>)> for GraphSearch<'a, D>
where
    D: StorageData,
{
    fn from(graph_storage: (&'a DbGraph<D>, &'a Storage<D>)) -> Self {
        GraphSearch {
            graph: graph_storage.0,
            storage: graph_storage.1,
        }
    }
}

impl SearchControl {
    pub fn and(self, other: SearchControl) -> SearchControl {
        use SearchControl::Continue;
        use SearchControl::Finish;
        use SearchControl::Stop;

        match (self, other) {
            (Continue(left), Continue(right)) => Continue(left && right),
            (Continue(left), Finish(right)) => Finish(left && right),
            (Continue(left), Stop(right)) => Stop(left && right),
            (Finish(left), Continue(right)) => Finish(left && right),
            (Finish(left), Finish(right)) => Finish(left && right),
            (Finish(left), Stop(right)) => Finish(left && right),
            (Stop(left), Continue(right)) => Stop(left && right),
            (Stop(left), Finish(right)) => Finish(left && right),
            (Stop(left), Stop(right)) => Stop(left && right),
        }
    }

    pub fn or(self, other: SearchControl) -> SearchControl {
        use SearchControl::Continue;
        use SearchControl::Finish;
        use SearchControl::Stop;

        match (self, other) {
            (Continue(left), Continue(right)) => Continue(left || right),
            (Continue(left), Finish(right)) => Continue(left || right),
            (Continue(left), Stop(right)) => Continue(left || right),
            (Finish(left), Continue(right)) => Continue(left || right),
            (Finish(left), Finish(right)) => Finish(left || right),
            (Finish(left), Stop(right)) => Stop(left || right),
            (Stop(left), Continue(right)) => Continue(left || right),
            (Stop(left), Finish(right)) => Stop(left || right),
            (Stop(left), Stop(right)) => Stop(left || right),
        }
    }

    pub fn flip(&mut self) {
        match self {
            SearchControl::Continue(v) | SearchControl::Finish(v) | SearchControl::Stop(v) => {
                *v = !*v;
            }
        };
    }

    pub fn is_true(&self) -> bool {
        match self {
            SearchControl::Continue(v) | SearchControl::Finish(v) | SearchControl::Stop(v) => *v,
        }
    }

    pub fn set_value(&mut self, value: bool) {
        match self {
            SearchControl::Continue(v) | SearchControl::Finish(v) | SearchControl::Stop(v) => {
                *v = value;
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::collections::tests::TestStorage;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::test_platform::TestPlatform;

    pub(crate) type TestGraph = DbGraph<FileStorageImpl<TestPlatform>>;

    pub(crate) struct Handler(pub fn(GraphIndex, u64) -> SearchControl);

    impl SearchHandler for Handler {
        fn process(&mut self, index: GraphIndex, distance: u64) -> Result<SearchControl, DbError> {
            Ok((self.0)(index, distance))
        }
    }

    pub(crate) fn stop_at<const DISTANCE: u64>(_index: GraphIndex, distance: u64) -> SearchControl {
        if distance == DISTANCE {
            SearchControl::Stop(true)
        } else {
            SearchControl::Continue(true)
        }
    }

    pub(crate) fn all() -> Handler {
        Handler(|_, _| SearchControl::Continue(true))
    }

    /// Inserts nodes 1..=`nodes` and then the `edges` in order, so edge
    /// ids start at `-(nodes + 1)`.
    pub(crate) fn graph(storage: &mut TestStorage, nodes: u64, edges: &[(i64, i64)]) -> TestGraph {
        let mut graph = DbGraph::new(storage).unwrap();

        for _ in 0..nodes {
            graph.insert_node(storage).unwrap();
        }

        for (from, to) in edges {
            graph
                .insert_edge(storage, GraphIndex(*from), GraphIndex(*to))
                .unwrap();
        }

        graph
    }

    /// 1 -5-> 2 -7-> 4 -9-> 1
    /// 1 -6-> 3 -8-> 4
    pub(crate) fn diamond(storage: &mut TestStorage) -> TestGraph {
        graph(storage, 4, &[(1, 2), (1, 3), (2, 4), (3, 4), (4, 1)])
    }

    pub(crate) fn indexes(ids: &[i64]) -> Result<Vec<GraphIndex>, DbError> {
        Ok(ids.iter().map(|id| GraphIndex(*id)).collect())
    }

    type Variant = fn(bool) -> SearchControl;

    const CONTINUE: Variant = SearchControl::Continue;
    const FINISH: Variant = SearchControl::Finish;
    const STOP: Variant = SearchControl::Stop;
    const VARIANTS: [Variant; 3] = [CONTINUE, FINISH, STOP];

    mod search_control {
        use super::*;

        mod and {
            use super::*;

            #[test]
            fn and_should_keep_finish_over_stop_over_continue_and_both_values() {
                for (left, right, expected) in [
                    (CONTINUE, CONTINUE, CONTINUE),
                    (CONTINUE, FINISH, FINISH),
                    (CONTINUE, STOP, STOP),
                    (FINISH, CONTINUE, FINISH),
                    (FINISH, FINISH, FINISH),
                    (FINISH, STOP, FINISH),
                    (STOP, CONTINUE, STOP),
                    (STOP, FINISH, FINISH),
                    (STOP, STOP, STOP),
                ] {
                    for (l, r) in [(false, false), (false, true), (true, false), (true, true)] {
                        assert_eq!(
                            left(l).and(right(r)),
                            expected(l && r),
                            "{:?} and {:?}",
                            left(l),
                            right(r)
                        );
                    }
                }
            }
        }

        mod or {
            use super::*;

            #[test]
            fn or_should_keep_continue_over_stop_over_finish_and_either_value() {
                for (left, right, expected) in [
                    (CONTINUE, CONTINUE, CONTINUE),
                    (CONTINUE, FINISH, CONTINUE),
                    (CONTINUE, STOP, CONTINUE),
                    (FINISH, CONTINUE, CONTINUE),
                    (FINISH, FINISH, FINISH),
                    (FINISH, STOP, STOP),
                    (STOP, CONTINUE, CONTINUE),
                    (STOP, FINISH, STOP),
                    (STOP, STOP, STOP),
                ] {
                    for (l, r) in [(false, false), (false, true), (true, false), (true, true)] {
                        assert_eq!(
                            left(l).or(right(r)),
                            expected(l || r),
                            "{:?} or {:?}",
                            left(l),
                            right(r)
                        );
                    }
                }
            }
        }

        mod flip {
            use super::*;

            #[test]
            fn flip_should_negate_the_value_and_keep_the_variant() {
                for variant in VARIANTS {
                    for value in [false, true] {
                        let mut control = variant(value);

                        control.flip();

                        assert_eq!(control, variant(!value), "{:?}", variant(value));
                    }
                }
            }
        }

        mod is_true {
            use super::*;

            #[test]
            fn is_true_should_return_the_value() {
                for variant in VARIANTS {
                    for value in [false, true] {
                        assert_eq!(variant(value).is_true(), value, "{:?}", variant(value));
                    }
                }
            }
        }

        mod set_value {
            use super::*;

            #[test]
            fn set_value_should_replace_the_value_and_keep_the_variant() {
                for variant in VARIANTS {
                    for (old, new) in [(false, false), (false, true), (true, false), (true, true)] {
                        let mut control = variant(old);

                        control.set_value(new);

                        assert_eq!(control, variant(new), "{:?}", variant(old));
                    }
                }
            }
        }
    }

    mod graph_search {
        use super::*;
        use crate::collections::tests::assert_read_failures;
        use crate::collections::tests::storage;
        use std::cell::RefCell;

        #[derive(Default)]
        struct Recorder {
            visits: RefCell<Vec<(i64, u64)>>,
        }

        impl SearchHandler for &Recorder {
            fn process(
                &mut self,
                index: GraphIndex,
                distance: u64,
            ) -> Result<SearchControl, DbError> {
                self.visits.borrow_mut().push((index.0, distance));
                Ok(SearchControl::Continue(true))
            }
        }

        impl PathSearchHandler for &Recorder {
            fn process(&self, index: GraphIndex, distance: u64) -> Result<(u64, bool), DbError> {
                self.visits.borrow_mut().push((index.0, distance));
                Ok((1, true))
            }
        }

        /// 1 -4-> 2 -5-> 3
        fn chain(storage: &mut TestStorage) -> TestGraph {
            let mut graph = DbGraph::new(storage).unwrap();

            for _ in 0..3 {
                graph.insert_node(storage).unwrap();
            }

            graph
                .insert_edge(storage, GraphIndex(1), GraphIndex(2))
                .unwrap();
            graph
                .insert_edge(storage, GraphIndex(2), GraphIndex(3))
                .unwrap();
            graph
        }

        mod breadth_first_search {
            use super::*;

            #[test]
            fn breadth_first_search_missing_index_should_return_nothing() {
                let mut storage = storage();
                let graph = chain(&mut storage);

                for index in [0, 10, -10] {
                    assert_eq!(
                        GraphSearch::from((&graph, &storage))
                            .breadth_first_search(GraphIndex(index), &Recorder::default()),
                        Ok(vec![]),
                        "{index}"
                    );
                }
            }

            #[test]
            fn breadth_first_search_read_failure_should_return_error() {
                assert_read_failures(chain, |storage, graph| {
                    GraphSearch::from((&*graph, &*storage))
                        .breadth_first_search(GraphIndex(1), &Recorder::default())
                });
            }

            #[test]
            fn breadth_first_search_edge_should_start_at_the_edge() {
                // GIVEN
                let mut storage = storage();
                let graph = chain(&mut storage);

                // WHEN
                let result = GraphSearch::from((&graph, &storage))
                    .breadth_first_search(GraphIndex(-4), &Recorder::default());

                // THEN
                assert_eq!(
                    result,
                    Ok(vec![
                        GraphIndex(-4),
                        GraphIndex(2),
                        GraphIndex(-5),
                        GraphIndex(3)
                    ])
                );
            }

            #[test]
            fn breadth_first_search_edge_read_failure_should_return_error() {
                assert_read_failures(chain, |storage, graph| {
                    GraphSearch::from((&*graph, &*storage))
                        .breadth_first_search(GraphIndex(-4), &Recorder::default())
                });
            }
        }

        mod breadth_first_search_reverse {
            use super::*;

            #[test]
            fn breadth_first_search_reverse_missing_index_should_return_nothing() {
                let mut storage = storage();
                let graph = chain(&mut storage);

                for index in [0, 10, -10] {
                    assert_eq!(
                        GraphSearch::from((&graph, &storage))
                            .breadth_first_search_reverse(GraphIndex(index), &Recorder::default()),
                        Ok(vec![]),
                        "{index}"
                    );
                }
            }

            #[test]
            fn breadth_first_search_reverse_read_failure_should_return_error() {
                assert_read_failures(chain, |storage, graph| {
                    GraphSearch::from((&*graph, &*storage))
                        .breadth_first_search_reverse(GraphIndex(3), &Recorder::default())
                });
            }
        }

        mod depth_first_search {
            use super::*;

            #[test]
            fn depth_first_search_missing_index_should_return_nothing() {
                let mut storage = storage();
                let graph = chain(&mut storage);

                for index in [0, 10, -10] {
                    assert_eq!(
                        GraphSearch::from((&graph, &storage))
                            .depth_first_search(GraphIndex(index), &Recorder::default()),
                        Ok(vec![]),
                        "{index}"
                    );
                }
            }

            #[test]
            fn depth_first_search_read_failure_should_return_error() {
                assert_read_failures(chain, |storage, graph| {
                    GraphSearch::from((&*graph, &*storage))
                        .depth_first_search(GraphIndex(1), &Recorder::default())
                });
            }
        }

        mod depth_first_search_reverse {
            use super::*;

            #[test]
            fn depth_first_search_reverse_missing_index_should_return_nothing() {
                let mut storage = storage();
                let graph = chain(&mut storage);

                for index in [0, 10, -10] {
                    assert_eq!(
                        GraphSearch::from((&graph, &storage))
                            .depth_first_search_reverse(GraphIndex(index), &Recorder::default()),
                        Ok(vec![]),
                        "{index}"
                    );
                }
            }

            #[test]
            fn depth_first_search_reverse_read_failure_should_return_error() {
                assert_read_failures(chain, |storage, graph| {
                    GraphSearch::from((&*graph, &*storage))
                        .depth_first_search_reverse(GraphIndex(3), &Recorder::default())
                });
            }
        }

        mod elements {
            use super::*;

            #[test]
            fn elements_read_failure_should_return_error() {
                assert_read_failures(chain, |storage, graph| {
                    GraphSearch::from((&*graph, &*storage)).elements(&Recorder::default())
                });
            }
        }

        mod path {
            use super::*;

            #[test]
            fn path_same_or_missing_nodes_should_return_nothing() {
                let mut storage = storage();
                let graph = graph(&mut storage, 2, &[(1, 1), (1, 2)]);

                for (from, to) in [(1, 1), (1, 10), (10, 2), (1, -4), (-4, 2)] {
                    assert_eq!(
                        GraphSearch::from((&graph, &storage)).path(
                            GraphIndex(from),
                            GraphIndex(to),
                            &Recorder::default()
                        ),
                        Ok(vec![]),
                        "{from} -> {to}"
                    );
                }
            }

            #[test]
            fn path_should_pass_breadth_first_distances() {
                let mut storage = storage();
                let graph = chain(&mut storage);
                let search = GraphSearch::from((&graph, &storage));
                let path = Recorder::default();
                let breadth_first = Recorder::default();

                search.path(GraphIndex(1), GraphIndex(3), &path).unwrap();
                search
                    .breadth_first_search(GraphIndex(1), &breadth_first)
                    .unwrap();

                assert_eq!(
                    path.visits.take(),
                    vec![(1, 0), (-4, 1), (2, 2), (-5, 3), (3, 4)]
                );
                assert_eq!(
                    breadth_first.visits.take(),
                    vec![(1, 0), (-4, 1), (2, 2), (-5, 3), (3, 4)]
                );
            }

            #[test]
            fn path_read_failure_should_return_error() {
                assert_read_failures(chain, |storage, graph| {
                    GraphSearch::from((&*graph, &*storage)).path(
                        GraphIndex(1),
                        GraphIndex(3),
                        &Recorder::default(),
                    )
                });
            }
        }

        mod path_reverse {
            use super::*;

            #[test]
            fn path_reverse_same_or_missing_nodes_should_return_nothing() {
                let mut storage = storage();
                let graph = graph(&mut storage, 2, &[(1, 1), (1, 2)]);

                for (from, to) in [(1, 1), (1, 10), (10, 2), (1, -4), (-4, 2)] {
                    assert_eq!(
                        GraphSearch::from((&graph, &storage)).path_reverse(
                            GraphIndex(from),
                            GraphIndex(to),
                            &Recorder::default()
                        ),
                        Ok(vec![]),
                        "{from} -> {to}"
                    );
                }
            }

            #[test]
            fn path_reverse_should_pass_breadth_first_reverse_distances() {
                let mut storage = storage();
                let graph = chain(&mut storage);
                let search = GraphSearch::from((&graph, &storage));
                let path = Recorder::default();
                let breadth_first = Recorder::default();

                search
                    .path_reverse(GraphIndex(1), GraphIndex(3), &path)
                    .unwrap();
                search
                    .breadth_first_search_reverse(GraphIndex(3), &breadth_first)
                    .unwrap();

                assert_eq!(
                    path.visits.take(),
                    vec![(3, 0), (-5, 1), (2, 2), (-4, 3), (1, 4)]
                );
                assert_eq!(
                    breadth_first.visits.take(),
                    vec![(3, 0), (-5, 1), (2, 2), (-4, 3), (1, 4)]
                );
            }

            #[test]
            fn path_reverse_read_failure_should_return_error() {
                assert_read_failures(chain, |storage, graph| {
                    GraphSearch::from((&*graph, &*storage)).path_reverse(
                        GraphIndex(1),
                        GraphIndex(3),
                        &Recorder::default(),
                    )
                });
            }
        }
    }
}
