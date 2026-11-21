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
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", SearchControl::Continue(false));
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn derived_from_clone() {
        let left = SearchControl::Continue(true);
        let other = left.clone();
        assert_eq!(left, other);
    }

    #[test]
    fn search_control_and() {
        use SearchControl::Continue;
        use SearchControl::Finish;
        use SearchControl::Stop;

        assert_eq!(Continue(true).and(Continue(true)), Continue(true));
        assert_eq!(Continue(true).and(Continue(false)), Continue(false));
        assert_eq!(Continue(false).and(Continue(true)), Continue(false));
        assert_eq!(Continue(false).and(Continue(false)), Continue(false));

        assert_eq!(Stop(true).and(Stop(true)), Stop(true));
        assert_eq!(Stop(true).and(Stop(false)), Stop(false));
        assert_eq!(Stop(false).and(Stop(true)), Stop(false));
        assert_eq!(Stop(false).and(Stop(false)), Stop(false));

        assert_eq!(Finish(true).and(Finish(true)), Finish(true));
        assert_eq!(Finish(true).and(Finish(false)), Finish(false));
        assert_eq!(Finish(false).and(Finish(true)), Finish(false));
        assert_eq!(Finish(false).and(Finish(false)), Finish(false));

        assert_eq!(Continue(true).and(Stop(true)), Stop(true));
        assert_eq!(Continue(false).and(Stop(true)), Stop(false));
        assert_eq!(Continue(true).and(Stop(false)), Stop(false));
        assert_eq!(Continue(false).and(Stop(false)), Stop(false));

        assert_eq!(Continue(true).and(Finish(true)), Finish(true));
        assert_eq!(Continue(false).and(Finish(true)), Finish(false));
        assert_eq!(Continue(true).and(Finish(false)), Finish(false));
        assert_eq!(Continue(false).and(Finish(false)), Finish(false));

        assert_eq!(Stop(true).and(Finish(true)), Finish(true));
        assert_eq!(Stop(true).and(Finish(false)), Finish(false));
        assert_eq!(Stop(false).and(Finish(false)), Finish(false));
        assert_eq!(Stop(false).and(Finish(true)), Finish(false));

        assert_eq!(Stop(true).and(Continue(true)), Stop(true));
        assert_eq!(Stop(true).and(Continue(false)), Stop(false));
        assert_eq!(Stop(false).and(Continue(true)), Stop(false));
        assert_eq!(Stop(false).and(Continue(false)), Stop(false));

        assert_eq!(Finish(true).and(Continue(true)), Finish(true));
        assert_eq!(Finish(true).and(Continue(false)), Finish(false));
        assert_eq!(Finish(false).and(Continue(true)), Finish(false));
        assert_eq!(Finish(false).and(Continue(false)), Finish(false));

        assert_eq!(Finish(true).and(Stop(true)), Finish(true));
        assert_eq!(Finish(true).and(Stop(false)), Finish(false));
        assert_eq!(Finish(false).and(Stop(false)), Finish(false));
        assert_eq!(Finish(false).and(Stop(true)), Finish(false));
    }

    #[test]
    fn search_control_or() {
        use SearchControl::Continue;
        use SearchControl::Finish;
        use SearchControl::Stop;

        assert_eq!(Continue(true).or(Continue(true)), Continue(true));
        assert_eq!(Continue(true).or(Continue(false)), Continue(true));
        assert_eq!(Continue(false).or(Continue(true)), Continue(true));
        assert_eq!(Continue(false).or(Continue(false)), Continue(false));

        assert_eq!(Stop(true).or(Stop(true)), Stop(true));
        assert_eq!(Stop(true).or(Stop(false)), Stop(true));
        assert_eq!(Stop(false).or(Stop(true)), Stop(true));
        assert_eq!(Stop(false).or(Stop(false)), Stop(false));

        assert_eq!(Finish(true).or(Finish(true)), Finish(true));
        assert_eq!(Finish(true).or(Finish(false)), Finish(true));
        assert_eq!(Finish(false).or(Finish(true)), Finish(true));
        assert_eq!(Finish(false).or(Finish(false)), Finish(false));

        assert_eq!(Continue(true).or(Stop(true)), Continue(true));
        assert_eq!(Continue(false).or(Stop(true)), Continue(true));
        assert_eq!(Continue(true).or(Stop(false)), Continue(true));
        assert_eq!(Continue(false).or(Stop(false)), Continue(false));

        assert_eq!(Continue(true).or(Finish(true)), Continue(true));
        assert_eq!(Continue(false).or(Finish(true)), Continue(true));
        assert_eq!(Continue(true).or(Finish(false)), Continue(true));
        assert_eq!(Continue(false).or(Finish(false)), Continue(false));

        assert_eq!(Stop(true).or(Finish(true)), Stop(true));
        assert_eq!(Stop(true).or(Finish(false)), Stop(true));
        assert_eq!(Stop(false).or(Finish(true)), Stop(true));
        assert_eq!(Stop(false).or(Finish(false)), Stop(false));

        assert_eq!(Stop(true).or(Continue(true)), Continue(true));
        assert_eq!(Stop(true).or(Continue(false)), Continue(true));
        assert_eq!(Stop(false).or(Continue(true)), Continue(true));
        assert_eq!(Stop(false).or(Continue(false)), Continue(false));

        assert_eq!(Finish(true).or(Continue(true)), Continue(true));
        assert_eq!(Finish(true).or(Continue(false)), Continue(true));
        assert_eq!(Finish(false).or(Continue(true)), Continue(true));
        assert_eq!(Finish(false).or(Continue(false)), Continue(false));

        assert_eq!(Finish(true).or(Stop(true)), Stop(true));
        assert_eq!(Finish(true).or(Stop(false)), Stop(true));
        assert_eq!(Finish(false).or(Stop(true)), Stop(true));
        assert_eq!(Finish(false).or(Stop(false)), Stop(false));
    }

    #[test]
    fn flip() {
        let mut control = SearchControl::Continue(true);
        control.flip();
        assert_eq!(control, SearchControl::Continue(false));
        control.flip();
        assert_eq!(control, SearchControl::Continue(true));

        control = SearchControl::Stop(true);
        control.flip();
        assert_eq!(control, SearchControl::Stop(false));
        control.flip();
        assert_eq!(control, SearchControl::Stop(true));

        control = SearchControl::Finish(true);
        control.flip();
        assert_eq!(control, SearchControl::Finish(false));
        control.flip();
        assert_eq!(control, SearchControl::Finish(true));
    }

    mod graph_search {
        use super::*;
        use crate::collections::tests::TestStorage;
        use crate::collections::tests::assert_read_failures;
        use crate::collections::tests::storage;
        use crate::storage::file_storage::FileStorageImpl;
        use agplatform::test_platform::TestPlatform;
        use std::cell::RefCell;

        type TestGraph = DbGraph<FileStorageImpl<TestPlatform>>;

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
