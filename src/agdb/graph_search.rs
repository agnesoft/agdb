mod breadth_first_search;
mod breadth_first_search_reverse;
mod depth_first_search;
mod depth_first_search_reverse;
mod path_search;
mod search_impl;

pub use self::path_search::PathSearchHandler;

use self::breadth_first_search::BreadthFirstSearch;
use self::breadth_first_search_reverse::BreadthFirstSearchReverse;
use self::depth_first_search::DepthFirstSearch;
use self::depth_first_search_reverse::DepthFirstSearchReverse;
use self::path_search::PathSearch;
use self::search_impl::SearchImpl;
use crate::graph::GraphData;
use crate::graph::GraphImpl;
use crate::graph::GraphIndex;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchControl {
    Continue(bool),
    Finish(bool),
    Stop(bool),
}

pub trait SearchHandler {
    fn process(&mut self, index: GraphIndex, distance: u64) -> SearchControl;
}

pub struct GraphSearch<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
}

#[allow(dead_code)]
impl<'a, Data> GraphSearch<'a, Data>
where
    Data: GraphData,
{
    pub fn breadth_first_search<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, BreadthFirstSearch>::new(self.graph, index).search(handler)
        } else {
            vec![]
        }
    }

    pub fn breadth_first_search_reverse<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, BreadthFirstSearchReverse>::new(self.graph, index)
                .search(handler)
        } else {
            vec![]
        }
    }

    pub fn depth_first_search<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, DepthFirstSearch>::new(self.graph, index).search(handler)
        } else {
            vec![]
        }
    }

    pub fn depth_first_search_reverse<Handler: SearchHandler>(
        &self,
        index: GraphIndex,
        handler: Handler,
    ) -> Vec<GraphIndex> {
        if self.is_valid_index(index) {
            SearchImpl::<'a, Data, DepthFirstSearchReverse>::new(self.graph, index).search(handler)
        } else {
            vec![]
        }
    }

    pub fn path<Handler: PathSearchHandler>(
        &self,
        from: GraphIndex,
        to: GraphIndex,
        handler: Handler,
    ) -> Vec<GraphIndex> {
        if from != to && self.is_valid_node(from) && self.is_valid_node(to) {
            PathSearch::<Data, Handler>::new(self.graph, from, to, handler).search()
        } else {
            vec![]
        }
    }

    fn is_valid_index(&self, index: GraphIndex) -> bool {
        self.is_valid_node(index) || self.graph.edge(index).is_some()
    }

    fn is_valid_node(&self, index: GraphIndex) -> bool {
        self.graph.node(index).is_some()
    }
}

impl<'a, Data> From<&'a GraphImpl<Data>> for GraphSearch<'a, Data>
where
    Data: GraphData,
{
    fn from(graph: &'a GraphImpl<Data>) -> Self {
        GraphSearch { graph }
    }
}

#[allow(dead_code)]
impl SearchControl {
    pub(crate) fn and(self, other: SearchControl) -> SearchControl {
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

    pub(crate) fn or(self, other: SearchControl) -> SearchControl {
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

    pub(crate) fn flip(&mut self) {
        match self {
            SearchControl::Continue(v) | SearchControl::Finish(v) | SearchControl::Stop(v) => {
                *v = !*v;
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", SearchControl::Continue(false));
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
}
