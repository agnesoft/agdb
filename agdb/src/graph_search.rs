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
use crate::DbId;
use crate::collections::graph::GraphHeader;
use crate::graph_search::element_search::ElementSearch;
use crate::storage::pages::View;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchControl {
    Continue(bool),
    Finish(bool),
    Stop(bool),
}

pub trait SearchHandler {
    fn process(&mut self, id: DbId, distance: u64) -> Result<SearchControl, DbError>;
}

/// The key of the element `id` in a set of visited elements, nodes and
/// edges having the same slots.
pub(crate) fn visit_key(id: DbId) -> u64 {
    (id.0.unsigned_abs() << 1) | u64::from(id.0 < 0)
}

pub struct GraphSearch<'a, V: View> {
    graph: &'a GraphHeader,
    view: &'a V,
}

impl<'a, V: View> GraphSearch<'a, V> {
    pub fn new(graph: &'a GraphHeader, view: &'a V) -> Self {
        Self { graph, view }
    }

    pub fn breadth_first_search<Handler: SearchHandler>(
        &self,
        id: DbId,
        handler: Handler,
    ) -> Result<Vec<DbId>, DbError> {
        if self.is_valid(id)? {
            SearchImpl::<V, BreadthFirstSearch>::new(self.graph, self.view, id).search(handler)
        } else {
            Ok(vec![])
        }
    }

    pub fn breadth_first_search_reverse<Handler: SearchHandler>(
        &self,
        id: DbId,
        handler: Handler,
    ) -> Result<Vec<DbId>, DbError> {
        if self.is_valid(id)? {
            SearchImpl::<V, BreadthFirstSearchReverse>::new(self.graph, self.view, id)
                .search(handler)
        } else {
            Ok(vec![])
        }
    }

    pub fn depth_first_search<Handler: SearchHandler>(
        &self,
        id: DbId,
        handler: Handler,
    ) -> Result<Vec<DbId>, DbError> {
        if self.is_valid(id)? {
            SearchImpl::<V, DepthFirstSearch>::new(self.graph, self.view, id).search(handler)
        } else {
            Ok(vec![])
        }
    }

    pub fn depth_first_search_reverse<Handler: SearchHandler>(
        &self,
        id: DbId,
        handler: Handler,
    ) -> Result<Vec<DbId>, DbError> {
        if self.is_valid(id)? {
            SearchImpl::<V, DepthFirstSearchReverse>::new(self.graph, self.view, id).search(handler)
        } else {
            Ok(vec![])
        }
    }

    pub fn path<Handler: PathSearchHandler>(
        &self,
        from: DbId,
        to: DbId,
        handler: Handler,
    ) -> Result<Vec<DbId>, DbError> {
        if from != to && self.is_node(from)? && self.is_node(to)? {
            PathSearch::<V, Handler>::new(self.graph, self.view, from, to, handler)?.search()
        } else {
            Ok(vec![])
        }
    }

    pub fn path_reverse<Handler: PathSearchHandler>(
        &self,
        from: DbId,
        to: DbId,
        handler: Handler,
    ) -> Result<Vec<DbId>, DbError> {
        if from != to && self.is_node(from)? && self.is_node(to)? {
            PathSearch::<V, Handler>::new_reverse(self.graph, self.view, from, to, handler)?
                .search()
        } else {
            Ok(vec![])
        }
    }

    pub fn elements<Handler: SearchHandler>(&self, handler: Handler) -> Result<Vec<DbId>, DbError> {
        ElementSearch::<V, Handler>::new(self.graph, self.view, handler).search()
    }

    fn is_valid(&self, id: DbId) -> Result<bool, DbError> {
        if id.0 < 0 {
            Ok(self.graph.edge(self.view, id)?.is_some())
        } else {
            self.is_node(id)
        }
    }

    fn is_node(&self, id: DbId) -> Result<bool, DbError> {
        Ok(self.graph.node(self.view, id)?.is_some())
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
    use crate::collections::value::tests::TestView;
    use crate::storage::alloc::Allocator;
    use crate::storage::root::Layout;

    pub(crate) struct Handler(pub fn(DbId, u64) -> SearchControl);

    impl SearchHandler for Handler {
        fn process(&mut self, id: DbId, distance: u64) -> Result<SearchControl, DbError> {
            Ok((self.0)(id, distance))
        }
    }

    pub(crate) fn stop_at<const DISTANCE: u64>(_id: DbId, distance: u64) -> SearchControl {
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
    /// ids start at -1.
    pub(crate) fn graph(
        view: &mut TestView,
        alloc: &mut Allocator,
        nodes: u64,
        edges: &[(i64, i64)],
    ) -> GraphHeader {
        let mut graph = GraphHeader::new(Layout::default());

        for _ in 0..nodes {
            graph.insert_node(view, alloc).unwrap();
        }

        for (from, to) in edges {
            graph
                .insert_edge(view, alloc, DbId(*from), DbId(*to))
                .unwrap();
        }

        graph
    }

    /// 1 -1-> 2 -3-> 4 -5-> 1
    /// 1 -2-> 3 -4-> 4
    pub(crate) fn diamond(view: &mut TestView, alloc: &mut Allocator) -> GraphHeader {
        graph(view, alloc, 4, &[(1, 2), (1, 3), (2, 4), (3, 4), (4, 1)])
    }

    pub(crate) fn ids(ids: &[i64]) -> Result<Vec<DbId>, DbError> {
        Ok(ids.iter().map(|id| DbId(*id)).collect())
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
        use crate::collections::value::tests::assert_failures;
        use crate::collections::value::tests::with_view;
        use std::cell::RefCell;

        #[derive(Default)]
        struct Recorder {
            visits: RefCell<Vec<(i64, u64)>>,
        }

        impl SearchHandler for &Recorder {
            fn process(&mut self, id: DbId, distance: u64) -> Result<SearchControl, DbError> {
                self.visits.borrow_mut().push((id.0, distance));
                Ok(SearchControl::Continue(true))
            }
        }

        impl PathSearchHandler for &Recorder {
            fn process(&self, id: DbId, distance: u64) -> Result<(u64, bool), DbError> {
                self.visits.borrow_mut().push((id.0, distance));
                Ok((1, true))
            }
        }

        /// 1 -1-> 2 -2-> 3
        fn chain(view: &mut TestView, alloc: &mut Allocator) -> GraphHeader {
            graph(view, alloc, 3, &[(1, 2), (2, 3)])
        }

        mod breadth_first_search {
            use super::*;

            #[test]
            fn breadth_first_search_missing_index_should_return_nothing() {
                with_view(|view, alloc| {
                    let graph = chain(view, alloc);

                    for id in [0, 10, -10] {
                        assert_eq!(
                            GraphSearch::new(&graph, &*view)
                                .breadth_first_search(DbId(id), &Recorder::default()),
                            Ok(vec![]),
                            "{id}"
                        );
                    }
                });
            }

            #[test]
            fn breadth_first_search_read_failure_should_return_error() {
                assert_failures(chain, |view, _, graph| {
                    GraphSearch::new(&graph, &*view)
                        .breadth_first_search(DbId(1), &Recorder::default())
                        .map(|_| ())
                });
            }

            #[test]
            fn breadth_first_search_edge_should_start_at_the_edge() {
                with_view(|view, alloc| {
                    // GIVEN
                    let graph = chain(view, alloc);

                    // WHEN
                    let result = GraphSearch::new(&graph, &*view)
                        .breadth_first_search(DbId(-1), &Recorder::default());

                    // THEN
                    assert_eq!(result, ids(&[-1, 2, -2, 3]));
                });
            }

            #[test]
            fn breadth_first_search_edge_read_failure_should_return_error() {
                assert_failures(chain, |view, _, graph| {
                    GraphSearch::new(&graph, &*view)
                        .breadth_first_search(DbId(-1), &Recorder::default())
                        .map(|_| ())
                });
            }
        }

        mod breadth_first_search_reverse {
            use super::*;

            #[test]
            fn breadth_first_search_reverse_missing_index_should_return_nothing() {
                with_view(|view, alloc| {
                    let graph = chain(view, alloc);

                    for id in [0, 10, -10] {
                        assert_eq!(
                            GraphSearch::new(&graph, &*view)
                                .breadth_first_search_reverse(DbId(id), &Recorder::default()),
                            Ok(vec![]),
                            "{id}"
                        );
                    }
                });
            }

            #[test]
            fn breadth_first_search_reverse_read_failure_should_return_error() {
                assert_failures(chain, |view, _, graph| {
                    GraphSearch::new(&graph, &*view)
                        .breadth_first_search_reverse(DbId(3), &Recorder::default())
                        .map(|_| ())
                });
            }
        }

        mod depth_first_search {
            use super::*;

            #[test]
            fn depth_first_search_missing_index_should_return_nothing() {
                with_view(|view, alloc| {
                    let graph = chain(view, alloc);

                    for id in [0, 10, -10] {
                        assert_eq!(
                            GraphSearch::new(&graph, &*view)
                                .depth_first_search(DbId(id), &Recorder::default()),
                            Ok(vec![]),
                            "{id}"
                        );
                    }
                });
            }

            #[test]
            fn depth_first_search_read_failure_should_return_error() {
                assert_failures(chain, |view, _, graph| {
                    GraphSearch::new(&graph, &*view)
                        .depth_first_search(DbId(1), &Recorder::default())
                        .map(|_| ())
                });
            }
        }

        mod depth_first_search_reverse {
            use super::*;

            #[test]
            fn depth_first_search_reverse_missing_index_should_return_nothing() {
                with_view(|view, alloc| {
                    let graph = chain(view, alloc);

                    for id in [0, 10, -10] {
                        assert_eq!(
                            GraphSearch::new(&graph, &*view)
                                .depth_first_search_reverse(DbId(id), &Recorder::default()),
                            Ok(vec![]),
                            "{id}"
                        );
                    }
                });
            }

            #[test]
            fn depth_first_search_reverse_read_failure_should_return_error() {
                assert_failures(chain, |view, _, graph| {
                    GraphSearch::new(&graph, &*view)
                        .depth_first_search_reverse(DbId(3), &Recorder::default())
                        .map(|_| ())
                });
            }
        }

        mod elements {
            use super::*;

            #[test]
            fn elements_read_failure_should_return_error() {
                assert_failures(chain, |view, _, graph| {
                    GraphSearch::new(&graph, &*view)
                        .elements(&Recorder::default())
                        .map(|_| ())
                });
            }
        }

        mod path {
            use super::*;

            #[test]
            fn path_same_or_missing_nodes_should_return_nothing() {
                with_view(|view, alloc| {
                    let graph = graph(view, alloc, 2, &[(1, 1), (1, 2)]);

                    for (from, to) in [(1, 1), (1, 10), (10, 2), (1, -2), (-2, 2)] {
                        assert_eq!(
                            GraphSearch::new(&graph, &*view).path(
                                DbId(from),
                                DbId(to),
                                &Recorder::default()
                            ),
                            Ok(vec![]),
                            "{from} -> {to}"
                        );
                    }
                });
            }

            #[test]
            fn path_should_pass_breadth_first_distances() {
                with_view(|view, alloc| {
                    let graph = chain(view, alloc);
                    let search = GraphSearch::new(&graph, &*view);
                    let path = Recorder::default();
                    let breadth_first = Recorder::default();

                    search.path(DbId(1), DbId(3), &path).unwrap();
                    search
                        .breadth_first_search(DbId(1), &breadth_first)
                        .unwrap();

                    assert_eq!(
                        path.visits.take(),
                        vec![(1, 0), (-1, 1), (2, 2), (-2, 3), (3, 4)]
                    );
                    assert_eq!(
                        breadth_first.visits.take(),
                        vec![(1, 0), (-1, 1), (2, 2), (-2, 3), (3, 4)]
                    );
                });
            }

            #[test]
            fn path_read_failure_should_return_error() {
                assert_failures(chain, |view, _, graph| {
                    GraphSearch::new(&graph, &*view)
                        .path(DbId(1), DbId(3), &Recorder::default())
                        .map(|_| ())
                });
            }
        }

        mod path_reverse {
            use super::*;

            #[test]
            fn path_reverse_same_or_missing_nodes_should_return_nothing() {
                with_view(|view, alloc| {
                    let graph = graph(view, alloc, 2, &[(1, 1), (1, 2)]);

                    for (from, to) in [(1, 1), (1, 10), (10, 2), (1, -2), (-2, 2)] {
                        assert_eq!(
                            GraphSearch::new(&graph, &*view).path_reverse(
                                DbId(from),
                                DbId(to),
                                &Recorder::default()
                            ),
                            Ok(vec![]),
                            "{from} -> {to}"
                        );
                    }
                });
            }

            #[test]
            fn path_reverse_should_pass_breadth_first_reverse_distances() {
                with_view(|view, alloc| {
                    let graph = chain(view, alloc);
                    let search = GraphSearch::new(&graph, &*view);
                    let path = Recorder::default();
                    let breadth_first = Recorder::default();

                    search.path_reverse(DbId(1), DbId(3), &path).unwrap();
                    search
                        .breadth_first_search_reverse(DbId(3), &breadth_first)
                        .unwrap();

                    assert_eq!(
                        path.visits.take(),
                        vec![(3, 0), (-2, 1), (2, 2), (-1, 3), (1, 4)]
                    );
                    assert_eq!(
                        breadth_first.visits.take(),
                        vec![(3, 0), (-2, 1), (2, 2), (-1, 3), (1, 4)]
                    );
                });
            }

            #[test]
            fn path_reverse_read_failure_should_return_error() {
                assert_failures(chain, |view, _, graph| {
                    GraphSearch::new(&graph, &*view)
                        .path_reverse(DbId(1), DbId(3), &Recorder::default())
                        .map(|_| ())
                });
            }
        }
    }
}
